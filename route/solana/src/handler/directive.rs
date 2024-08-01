use crate::types::{ChainId, Directive, Error, Seq, Topic};
use candid::Principal;

use crate::handler::sol_call::create_mint_account;

use crate::{
    call_error::{CallError, Reason},
    state::{mutate_state, read_state},
};
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
                        mutate_state(|s| s.add_chain(chain.clone()));
                    }

                    Directive::AddToken(token) => {
                        mutate_state(|s| s.add_token(token.clone()));
                    }
                    //TODO: if update_token, need to update solana token metadata
                    Directive::UpdateToken(_token) => {
                        todo!()
                    }
                    Directive::ToggleChainState(toggle) => {
                        mutate_state(|s| s.toggle_chain_state(toggle.clone()));
                    }
                    Directive::UpdateFee(fee) => {
                        mutate_state(|s| s.update_fee(fee.clone()));
                    }
                }
            }
            let next_seq = directives.last().map_or(offset, |(seq, _)| seq + 1);
            mutate_state(|s| {
                s.next_directive_seq = next_seq;
            });
        }
        Err(err) => {
            ic_cdk::eprintln!(
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
    // TODO: optmize
    let (tokens, token_mint_map) =
        read_state(|s| (s.tokens.to_owned(), s.token_mint_map.to_owned()));

    for (token_id, token) in tokens.iter() {
        if matches!(token_mint_map.get(token_id), None) {
            let token_create_info = TokenCreateInfo {
                name: token.name.to_owned(),
                symbol: token.symbol.to_owned(),
                decimals: token.decimals,
                uri: token.icon.to_owned().unwrap_or_default(),
            };
            match create_mint_account(token_create_info).await {
                Ok(token_mint) => {
                    ic_cdk::println!(
                        "[directive::create_token_mint] {:?} new mint token address on solana: {:?} ",
                        token_id.to_string(),
                        token_mint
                    );
                    // save the token mint
                    mutate_state(|s| {
                        s.token_mint_map
                            .insert(token_id.to_string(), token_mint.to_string())
                    });
                }
                Err(e) => {
                    ic_cdk::eprintln!(
                        "[directive::create_token_mint]  create token mint error: {:?}  ",
                        e
                    );
                    continue;
                }
            }
        }
    }
}

// # TODO: update token_medadata()
