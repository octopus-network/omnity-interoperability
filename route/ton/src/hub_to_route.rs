use crate::base::const_args::{BATCH_QUERY_LIMIT, FETCH_HUB_DIRECTIVE_NAME, FETCH_HUB_TICKET_NAME};
use crate::state::{mutate_state, read_state, TON_NATIVE_TOKEN};
use crate::{audit, hub};
use ic_canister_log::log;
use log::{self};
use omnity_types::ic_log::ERROR;
use omnity_types::{Directive, Factor};

pub async fn process_tickets() {
    let (hub_principal, offset) = read_state(|s| (s.hub_principal, s.next_ticket_seq));
    match hub::query_tickets(hub_principal, offset, BATCH_QUERY_LIMIT).await {
        Ok(tickets) => {
            for (seq, ticket) in tickets {
                mutate_state(|s| {
                    s.tickets_queue.insert(seq, ticket);
                    s.next_ticket_seq = seq + 1
                });
            }
        }
        Err(err) => {
            log::error!("[process tickets] failed to query tickets, err: {}", err);
        }
    }
}

async fn process_directives() {
    let (hub_principal, offset) = read_state(|s| (s.hub_principal, s.next_directive_seq));
    match hub::query_directives(hub_principal, offset, BATCH_QUERY_LIMIT).await {
        Ok(directives) => {
            for (seq, directive) in &directives {
                let final_directive = directive.clone();
                match directive.clone() {
                    Directive::AddChain(chain) | Directive::UpdateChain(chain) => {
                        mutate_state(|s| audit::add_chain(s, chain.clone()));
                    }
                    Directive::ToggleChainState(_t) => {}
                    Directive::UpdateToken(token) => {
                        mutate_state(|s| audit::add_token(s, token.clone()));
                    }
                    Directive::AddToken(token) => {
                        mutate_state(|s| audit::add_token(s, token));
                    }
                    Directive::UpdateFee(fee) => match fee {
                        Factor::UpdateTargetChainFactor(factor) => {
                            mutate_state(|s| {
                                s.target_chain_factor.insert(
                                    factor.target_chain_id.clone(),
                                    factor.target_chain_factor,
                                );
                            });
                        }
                        Factor::UpdateFeeTokenFactor(token_factor) => {
                            mutate_state(|s| {
                                if token_factor.fee_token == TON_NATIVE_TOKEN {
                                    s.fee_token_factor = Some(token_factor.fee_token_factor);
                                }
                            });
                        }
                    },
                }
                mutate_state(|s| s.directives_queue.insert(*seq, final_directive));
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

pub fn fetch_hub_ticket_task() {
    ic_cdk::spawn(async {
        let _guard = match crate::guard::TimerLogicGuard::new(FETCH_HUB_TICKET_NAME.to_string()) {
            Some(guard) => guard,
            None => return,
        };
        process_tickets().await;
    });
}

pub fn fetch_hub_directive_task() {
    ic_cdk::spawn(async {
        let _guard = match crate::guard::TimerLogicGuard::new(FETCH_HUB_DIRECTIVE_NAME.to_string())
        {
            Some(guard) => guard,
            None => return,
        };
        process_directives().await;
    });
}
