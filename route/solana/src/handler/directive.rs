use crate::types::{ChainId, Directive, Error, Seq, Topic};
use candid::Principal;

use crate::handler::sol_call::{create_mint_account, update_token_metadata};

use crate::{
    call_error::{CallError, Reason},
    state::{mutate_state, read_state},
};
use ic_canister_log::log;
use ic_solana::logs::{ERROR, INFO};
use ic_solana::token::TokenInfo;

pub const DIRECTIVE_LIMIT_SIZE: u64 = 20;

/// query directives from hub and save to route state
pub async fn query_directives() {
    let (hub_principal, offset) = read_state(|s| (s.hub_principal, s.next_directive_seq));
    match inner_query_directives(hub_principal, offset, DIRECTIVE_LIMIT_SIZE).await {
        Ok(directives) => {
            for (_, directive) in &directives {
                match directive {
                    Directive::AddChain(chain) | Directive::UpdateChain(chain) => {
                        mutate_state(|s| s.add_chain(chain.to_owned()));
                    }

                    Directive::AddToken(token) => {
                        mutate_state(|s| s.add_token(token.to_owned()));
                    }
                    Directive::UpdateToken(update_token) => {
                        let t = read_state(|s| s.tokens.get(&update_token.token_id).cloned());
                        match t {
                            None => mutate_state(|s| s.add_token(update_token.to_owned())),
                            //if update_token, need to update solana token metadata
                            Some(current_token) => {
                                log!(
                                    INFO,
                                    "[Directive::UpdateToken] need to update token metadata for :{:?} ",
                                    current_token,
                                );
                                mutate_state(|s| {
                                    s.update_token_queue.insert(
                                        update_token.token_id.to_string(),
                                        update_token.to_owned(),
                                    )
                                });
                            }
                        }
                    }
                    Directive::ToggleChainState(toggle) => {
                        mutate_state(|s| s.toggle_chain_state(toggle.to_owned()));
                    }
                    Directive::UpdateFee(fee) => {
                        mutate_state(|s| s.update_fee(fee.to_owned()));
                    }
                }
            }
            let next_seq = directives.last().map_or(offset, |(seq, _)| seq + 1);
            mutate_state(|s| {
                s.next_directive_seq = next_seq;
            });
        }
        Err(err) => {
            log!(
                ERROR,
                "[process directives] failed to query directives, err: {:?}",
                err
            );
        }
    };
}

pub async fn inner_query_directives(
    hub_principal: Principal,
    offset: u64,
    limit: u64,
) -> Result<Vec<(Seq, Directive)>, CallError> {
    let resp: (Result<Vec<(Seq, Directive)>, Error>,) = ic_cdk::api::call::call(
        hub_principal,
        "query_directives",
        (
            None::<Option<ChainId>>,
            None::<Option<Topic>>,
            offset,
            limit,
        ),
    )
    .await
    .map_err(|(code, message)| CallError {
        method: "query_directives".to_string(),
        reason: Reason::from_reject(code, message),
    })?;
    let data = resp.0.map_err(|err| CallError {
        method: "query_directives".to_string(),
        reason: Reason::CanisterError(err.to_string()),
    })?;
    Ok(data)
}

pub async fn create_token_mint() {
    let creating_token_mint = read_state(|s| {
        let mut creating_token_mint = vec![];
        for (token_id, token) in s.tokens.iter() {
            if matches!(s.token_mint_map.get(token_id), None) {
                creating_token_mint.push(token.to_owned())
            }
        }
        creating_token_mint
    });

    for token in creating_token_mint.into_iter() {
        let token_reate_info = TokenInfo {
            name: token.name,
            symbol: token.symbol,
            decimals: token.decimals,
            uri: token.icon.unwrap_or_default(),
        };

        match create_mint_account(token_reate_info).await {
            Ok(token_mint) => {
                log!(
                    INFO,
                    "[directive::create_token_mint] {:?} new mint token address on solana: {:?} ",
                    token.token_id.to_string(),
                    token_mint
                );
                // save the token mint
                mutate_state(|s| {
                    s.token_mint_map
                        .insert(token.token_id.to_string(), token_mint.to_string())
                });
            }
            Err(e) => {
                log!(
                    ERROR,
                    "[directive::create_token_mint] create token mint error: {:?}  ",
                    e
                );
                continue;
            }
        }
    }
}

pub async fn update_token() {
    let update_tokens = read_state(|s| {
        s.update_token_queue
            .iter()
            .take(5)
            .map(|(token_id, token)| (token_id.to_owned(), token.to_owned()))
            .collect::<Vec<_>>()
    });

    for (token_id, token) in update_tokens.into_iter() {
        let token_mint = read_state(|s| s.token_mint_map.get(&token_id).cloned());
        if let Some(token_mint) = token_mint {
            let token_update_info = TokenInfo {
                name: token.name.to_owned(),
                symbol: token.symbol.to_owned(),
                decimals: token.decimals,
                uri: token.icon.to_owned().unwrap_or_default(),
            };

            match update_token_metadata(token_mint, token_update_info).await {
                Ok(signature) => {
                    log!(
                    INFO,
                    "[directive::update_token] {:?} update token metadata on solana sucessfully ! \n{:?} ",
                    token.token_id.to_string(),
                    signature
                );

                    mutate_state(|s| {
                        // update the token info
                        s.add_token(token.to_owned());
                        // remove the updated token from queue
                        s.update_token_queue.remove(&token_id)
                    });
                }
                Err(e) => {
                    log!(
                        ERROR,
                        "[directive::update_token] update token metadata error: {:?}  ",
                        e
                    );
                    continue;
                }
            }
        } else {
            log!(
                ERROR,
                "[directive::update_token] not found token mint : {:?}",
                token.token_id
            );
            continue;
        }
    }
}
