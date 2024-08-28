use candid::{Nat, Principal};
use icrc_ledger_types::icrc1::account::Account;
use omnity_types::{Directive, Ticket};
use state::{insert_counterparty, is_ckbtc, is_icp, mutate_state, read_state};
use std::str::FromStr;
use ic_canister_log::log;
use ic_ledger_types::MAINNET_LEDGER_CANISTER_ID;
use omnity_types::ic_log::P0;
use updates::mint_token::{retrieve_ckbtc, unlock_icp, MintTokenError, MintTokenRequest};

pub mod call_error;
pub mod hub;
pub mod lifecycle;
pub mod state;
pub mod updates;
pub mod utils;
pub mod service;

type TempUnavalible = bool;
pub const PERIODIC_TASK_INTERVAL: u64 = 5;
pub const BATCH_QUERY_LIMIT: u64 = 20;
pub const ICP_TRANSFER_FEE: u64 = 10_000;

pub fn parse_receiver(ticket: &Ticket) -> Option<Account> {
    let receiver_parse_result = if ticket.receiver.contains(".") {
        Account::from_str(ticket.receiver.as_str()).map_err(|e| e.to_string())
    } else {
        Principal::from_str(ticket.receiver.as_str())
            .map(|owner| Account {
                owner,
                subaccount: None,
            })
            .map_err(|e| e.to_string())
    };

    match receiver_parse_result {
        Ok(receiver) => Some(receiver),
        Err(err) => {
            log!(P0,
                "[process tickets] failed to parse ticket receiver: {}, err: {}",
                ticket.receiver,
                err
            );
             None
        }
    }
}
pub async fn handle_ticket(ticket: &Ticket) -> TempUnavalible {

    if is_ckbtc(&ticket.token) {
        match retrieve_ckbtc(ticket.receiver.clone(), Nat::from_str(ticket.amount.as_str()).unwrap(), ticket.ticket_id.clone()).await {
            Ok(_) => {
                log!(P0, "[process tickets] process successful for ticket id: {}", ticket.ticket_id);
            },
            Err(e) => {
                log!(P0, "[process tickets] failed to retrieve ckbtc: {:?}", e);
            },
        }
        return false;
    }

    let receiver = parse_receiver(&ticket);
    if receiver.is_none() {
        return false;
    }
    let receiver = receiver.unwrap();

    let amount: u128 = if let Ok(amount) = ticket.amount.parse() {
        amount
    } else {
        log!(P0,
            "[process tickets] failed to parse ticket amount: {}",
            ticket.amount
        );
        return false;
    };

    if is_icp(&ticket.token) {
        match unlock_icp(& MintTokenRequest{
            ticket_id: ticket.ticket_id.clone(),
            token_id: ticket.token.clone(),
            receiver,
            amount,
        }).await {
            Ok(height) => {
                log!(P0, "[process tickets] process successful for ticket id: {}", &ticket.ticket_id);
                let hub_principal = read_state(|s| s.hub_principal);
                let tx_hash = format!("{}_{}", MAINNET_LEDGER_CANISTER_ID.to_string(), height.to_string());
                let _ = hub::update_tx_hash(hub_principal, ticket.ticket_id.clone(), tx_hash).await;
            },
            Err(e) => {
                log!(P0, "[process tickets] failed to unlock icp: {:?}", e);
            },
        }
        return false;
    }

    match updates::mint_token(&mut MintTokenRequest {
        ticket_id: ticket.ticket_id.clone(),
        token_id: ticket.token.clone(),
        receiver,
        amount,
    })
        .await
    {
        Ok(_) => {
            log!(P0,
                "[process tickets] process successful for ticket id: {}",
                ticket.ticket_id
            );
        }
        Err(MintTokenError::TemporarilyUnavailable(desc)) => {
            log!(P0,
                "[process tickets] failed to mint token for ticket id: {}, err: {}",
                ticket.ticket_id,
                desc
            );
            return true;
        }
        Err(err) => {
            log!(P0,
                "[process tickets] process failure for ticket id: {}, err: {:?}",
                ticket.ticket_id,
                err
            );
        }
    }
    false
}

async fn process_tickets() {
    let (hub_principal, offset) = read_state(|s| (s.hub_principal, s.next_ticket_seq));
    match hub::query_tickets(hub_principal, offset, BATCH_QUERY_LIMIT).await {
        Ok(tickets) => {
            for (seq, ticket) in &tickets {
                if handle_ticket(ticket).await {
                    break;
                }
                mutate_state(|s| s.next_ticket_seq = seq+1)
            }
        }
        Err(err) => {
            log!(P0, "[process tickets] failed to query tickets, err: {}", err);
        }
    }
}

async fn process_directives() {
    let (hub_principal, offset) = read_state(|s| (s.hub_principal, s.next_directive_seq));
    match hub::query_directives(hub_principal, offset, BATCH_QUERY_LIMIT).await {
        Ok(directives) => {
            for (_, directive) in &directives {
                match directive {
                    Directive::AddChain(chain) | Directive::UpdateChain(chain) => {
                        insert_counterparty(chain.clone());
                    }
                    Directive::AddToken(token) => {
                        match updates::add_new_token(token.clone()).await {
                            Ok(_) => {
                                log!(P0,
                                    "[process directives] add token successful, token id: {}",
                                    token.token_id
                                );
                            }
                            Err(err) => {
                                log!(P0,
                                    "[process directives] failed to add token: token id: {}, err: {:?}",
                                    token.token_id,
                                    err
                                );
                            }
                        }
                    }
                    Directive::UpdateToken(token) => {
                        match updates::update_token(token.clone()).await {
                            Ok(_) => {
                                log!(P0,
                                    "[process directives] update token successful, token id: {}",
                                    token.token_id
                                );
                            }
                            Err(err) => {
                                log!(P0,
                                    "[process directives] failed to update token: token id: {}, err: {:?}",
                                    token.token_id,
                                    err
                                );
                            }
                        }
                    }
                    _ => {}
                }
            }
            let next_seq = directives.last().map_or(offset, |(seq, _)| seq + 1);
            mutate_state(|s| {
                s.next_directive_seq = next_seq;
            });
        }
        Err(err) => {
            log!(P0,
                "[process directives] failed to query directives, err: {:?}",
                err
            );
        }
    };
}

#[must_use]
pub struct TimerLogicGuard(());

impl TimerLogicGuard {
    pub fn new() -> Option<Self> {
        mutate_state(|s| {
            if s.is_timer_running {
                return None;
            }
            s.is_timer_running = true;
            Some(TimerLogicGuard(()))
        })
    }
}

impl Drop for TimerLogicGuard {
    fn drop(&mut self) {
        mutate_state(|s| {
            s.is_timer_running = false;
        });
    }
}

pub fn periodic_task() {
    ic_cdk::spawn(async {
        let _guard = match TimerLogicGuard::new() {
            Some(guard) => guard,
            None => return,
        };
        process_directives().await;
        process_tickets().await;
    });
}
