use crate::constants::DIRECTIVE_LIMIT_SIZE;
use crate::state::UpdateToken;
use crate::types::{ChainId, Directive, Error, Seq, Topic};
use candid::Principal;

use crate::{
    call_error::{CallError, Reason},
    state::{mutate_state, read_state},
};
use ic_canister_log::log;
use ic_solana::ic_log::{DEBUG, ERROR};

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
                        let t = read_state(|s| s.tokens.get(&update_token.token_id));
                        match t {
                            // new token
                            None => mutate_state(|s| s.add_token(update_token.to_owned())),
                            //if update_token, need to update solana token metadata
                            Some(current_token) => {
                                log!(
                                    DEBUG,
                                    "[Directive::UpdateToken] \ncurrent token metadata :{:#?} \nupdate token metadata :{:#?} ",
                                    current_token,update_token
                                );
                                mutate_state(|s| {
                                    // update token info in route
                                    // s.tokens.insert(current_token.token_id, update_token.clone());
                                    // update token on solana chain
                                    s.update_token_queue.insert(
                                        update_token.token_id.to_string(),
                                        UpdateToken::new(update_token.to_owned(), 0),
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
