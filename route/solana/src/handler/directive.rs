use crate::types::{ChainId, Directive, Error, Seq, Topic};
use candid::Principal;

use crate::handler::sol_call::create_mint_account;

use crate::{
    call_error::{CallError, Reason},
    state::{mutate_state, read_state},
};
use ic_canister_log::log;
use ic_solana::logs::{ERROR, INFO};
use ic_solana::token::TokenCreateInfo;

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
                    Directive::UpdateToken(token) => {
                        let t = read_state(|s| s.tokens.get(&token.token_id).cloned());
                        match t {
                            //TODO: if update_token, need to update solana token metadata
                            Some(_t) => todo!(),
                            None => mutate_state(|s| s.add_token(token.to_owned())),
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
        let token_create_info = TokenCreateInfo {
            name: token.name,
            symbol: token.symbol,
            decimals: token.decimals,
            uri: token.icon.unwrap_or_default(),
        };

        match create_mint_account(token_create_info).await {
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
                    "[directive::create_token_mint]  create token mint error: {:?}  ",
                    e
                );
                continue;
            }
        }
    }
}

// # TODO: update token_medadata()
