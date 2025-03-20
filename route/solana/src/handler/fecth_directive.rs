use crate::constants::DIRECTIVE_LIMIT_SIZE;

use crate::state::UpdateToken;
use crate::types::{ChainId, Directive, Error, Seq, Token, Topic};
use candid::Principal;

use crate::{
    call_error::{CallError, Reason},
    state::{mutate_state, read_state},
};
use ic_canister_log::log;
use ic_solana::logs::{DEBUG, ERROR};

/// query directives from hub and save to route state
pub async fn query_directives() {
    let (hub_principal, offset) = read_state(|s| (s.hub_principal, s.seqs.next_directive_seq));
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
                            None => mutate_state(|s| s.add_token(update_token.to_owned())),

                            Some(current_token) => {
                                log!(
                                    DEBUG,
                                    "[query_directives] \ncurrent token metadata :{:#?} \nupdate token metadata :{:#?} ",
                                    current_token,update_token
                                );

                                let new_token = Token {
                                    token_id: update_token.token_id.to_string(),
                                    name: update_token.name.to_owned(),
                                    symbol: update_token.symbol.to_owned(),
                                    decimals: update_token.decimals,
                                    metadata: update_token.metadata.to_owned(),
                                    //keep icon
                                    icon: current_token.icon.to_owned(),
                                };
                                // just support to update name and symbol, the uri need to update via cli
                                if (!current_token.name.eq(&new_token.name))
                                    || (!current_token.symbol.eq(&new_token.symbol))
                                // || (!current_token
                                //     .metadata
                                //     .get("uri")
                                //     .eq(&new_token.metadata.get("uri")))
                                {
                                    mutate_state(|s| {
                                        s.update_token_queue.insert(
                                            new_token.token_id.to_string(),
                                            UpdateToken::new(new_token.to_owned()),
                                        )
                                    });
                                } else {
                                    // just update token info in route
                                    mutate_state(|s| {
                                        s.tokens.insert(
                                            new_token.token_id.to_string(),
                                            new_token.to_owned(),
                                        )
                                    });
                                }
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
                s.seqs.next_directive_seq = next_seq;
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
