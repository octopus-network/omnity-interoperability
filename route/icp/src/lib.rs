use candid::Principal;
use log::{self};
use omnity_types::Directive;
use state::{audit, mutate_state, read_state, MintTokenStatus};
use std::str::FromStr;
use updates::mint_token::MintTokenRequest;

pub mod call_error;
pub mod hub;
pub mod lifecycle;
pub mod log_util;
pub mod state;
pub mod updates;

pub const PERIODIC_TASK_INTERVAL: u64 = 5;
pub const BATCH_QUERY_LIMIT: u64 = 20;
pub const ICRC2_WASM: &[u8] = include_bytes!("../../../ic-icrc1-ledger.wasm");

async fn process_tickets() {
    let (hub_principal, offset) = read_state(|s| (s.hub_principal, s.next_ticket_seq));
    match hub::query_tickets(hub_principal, offset, BATCH_QUERY_LIMIT).await {
        Ok(tickets) => {
            let mut next_seq = offset;
            for (seq, ticket) in tickets {
                let receiver = if let Ok(receiver) = Principal::from_str(&ticket.receiver) {
                    receiver
                } else {
                    next_seq = seq + 1;
                    log::error!(
                        "[process tickets] failed to parse ticket receiver: {}",
                        ticket.receiver
                    );
                    continue;
                };
                let amount: u128 = if let Ok(amount) = ticket.amount.parse() {
                    amount
                } else {
                    next_seq = seq + 1;
                    log::error!(
                        "[process tickets] failed to parse ticket amount: {}",
                        ticket.amount
                    );
                    continue;
                };
                match updates::mint_token(&mut MintTokenRequest {
                    ticket_id: ticket.ticket_id.clone(),
                    token_id: ticket.token,
                    receiver,
                    amount,
                    status: MintTokenStatus::Finalized,
                })
                .await
                {
                    Ok(_) => {
                        log::info!(
                            "[process tickets] process successful for ticket id: {}",
                            ticket.ticket_id
                        );
                    }
                    Err(err) => {
                        log::error!(
                            "[process tickets] process failure for ticket id: {}, err: {:?}",
                            ticket.ticket_id,
                            err
                        );
                    }
                }
                next_seq = seq + 1;
            }
            mutate_state(|s| s.next_ticket_seq = next_seq)
        }
        Err(err) => {
            log::error!("[process tickets] failed to query tickets, err: {}", err);
        }
    }
}

async fn process_directives() {
    let (hub_principal, offset) = read_state(|s| (s.hub_principal, s.next_directive_seq));
    match hub::query_dires(hub_principal, offset, BATCH_QUERY_LIMIT).await {
        Ok(directives) => {
            for (_, directive) in &directives {
                match directive {
                    Directive::AddChain(chain) => {
                        mutate_state(|s| audit::add_chain(s, chain.clone()));
                    }
                    Directive::AddToken(token) => {
                        match updates::add_new_token(token.clone()).await {
                            Ok(_) => {
                                log::info!(
                                    "[process directives] add token successful, token id: {}",
                                    token.token_id
                                );
                            }
                            Err(err) => {
                                log::error!(
                                    "[process directives] failed to add token: token id: {}, err: {:?}",
                                    token.token_id,
                                    err
                                );
                            }
                        }
                    }
                    Directive::ToggleChainState(toggle) => {
                        mutate_state(|s| audit::toggle_chain_state(s, toggle.clone()));
                    }
                    Directive::UpdateFee(fee) => {
                        // todo update fee
                    }
                }
            }
            let next_seq = directives.last().map_or(offset, |(seq, _)| seq + 1);
            mutate_state(|s| {
                s.next_directive_seq = next_seq;
            });
        }
        Err(err) => {
            log::error!(
                "[process directives] failed to query directives, err: {:?}",
                err
            );
        }
    };
}

pub fn periodic_task() {
    ic_cdk::spawn(async {
        process_tickets().await;
        process_directives().await;
    });
}
