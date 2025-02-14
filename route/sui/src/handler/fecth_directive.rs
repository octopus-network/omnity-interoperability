#![allow(unused)]
use crate::constants::DIRECTIVE_LIMIT_SIZE;

use crate::config::{mutate_config, read_config};
use crate::state::{mutate_state, read_state, UpdateTokenStatus, UpdateType};
use crate::types::{ChainId, Directive, Error, Seq, Topic};
use candid::Principal;

use crate::call_error::{CallError, Reason};
use crate::ic_log::{DEBUG, ERROR};
use ic_canister_log::log;

/// query directives from hub and save to route state
pub async fn query_directives() {
    // log!(DEBUG, "[query_directives] running .... ");
    let (hub_principal, offset) =
        read_config(|s| (s.get().hub_principal, s.get().seqs.next_directive_seq));
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
                                    "[query_directives] \ncurrent token metadata :{:#?} \nupdate token metadata :{:#?} ",
                                    current_token,update_token
                                );

                                if !current_token.name.eq(&update_token.name) {
                                    let update_type =
                                        UpdateType::Name(update_token.name.to_owned());
                                    let update_status =
                                        UpdateTokenStatus::new(update_token.token_id.to_owned());
                                    mutate_state(|s| {
                                        s.update_token_queue.insert(update_type, update_status)
                                    });
                                } else if !current_token.symbol.eq(&update_token.symbol) {
                                    let update_type =
                                        UpdateType::Symbol(update_token.symbol.to_owned());
                                    let update_status =
                                        UpdateTokenStatus::new(update_token.token_id.to_owned());
                                    mutate_state(|s| {
                                        s.update_token_queue.insert(update_type, update_status)
                                    });
                                } else if !current_token.icon.eq(&update_token.icon) {
                                    let update_type =
                                        UpdateType::Icon(update_token.icon.to_owned().unwrap());
                                    let update_status =
                                        UpdateTokenStatus::new(update_token.token_id.to_owned());
                                    mutate_state(|s| {
                                        s.update_token_queue.insert(update_type, update_status)
                                    });
                                } else {
                                    mutate_state(|s| s.add_token(update_token.to_owned()));
                                }
                            }
                        }
                    }
                    Directive::ToggleChainState(toggle) => {
                        mutate_state(|s| s.toggle_chain_state(toggle.to_owned()));
                    }
                    Directive::UpdateFee(fee) => {
                        mutate_config(|s| {
                            let mut config = s.get().to_owned();
                            config.update_fee(fee.to_owned());
                            s.set(config);
                        });
                    }
                }
            }
            let next_seq = directives.last().map_or(offset, |(seq, _)| seq + 1);
            mutate_config(|s| {
                let mut config = s.get().to_owned();
                config.seqs.next_directive_seq = next_seq;
                s.set(config);
            });
        }
        Err(err) => {
            log!(
                ERROR,
                "[query_directives] failed to query directives, err: {:?}",
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
