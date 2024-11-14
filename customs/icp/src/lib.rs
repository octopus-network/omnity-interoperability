use candid::{Nat, Principal};
use icrc_ledger_types::icrc1::account::Account;
use omnity_types::{ic_log::WARNING, Directive, IcpChainKeyToken, Ticket};
use state::{insert_counterparty, is_icp, mutate_state, read_state};
use std::str::FromStr;
use ic_canister_log::log;
use omnity_types::ic_log::{ERROR, INFO};
use updates::mint_token::{retrieve_ckbtc, unlock_icp, MintTokenRequest};
use omnity_types::TxAction;

pub mod call_error;
pub mod hub;
pub mod lifecycle;
pub mod state;
pub mod updates;
pub mod utils;
pub mod service;

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
            log!(INFO,
                "[process tickets] failed to parse ticket receiver: {}, err: {}",
                ticket.receiver,
                err
            );
             None
        }
    }
}

pub async fn handle_redeem_ticket(ticket: &Ticket) -> Result<u64, String> {
    let block_index = match ticket.action {
        TxAction::Transfer | TxAction::Burn | TxAction::Mint => {
            return Err("Unsupported action".to_string())
        },
        TxAction::Redeem => {
            if let Some(receiver) = parse_receiver(&ticket) {
                let amount = ticket.amount.parse::<u128>().map_err(|e| e.to_string())?;
                if is_icp(&ticket.token) {
                    unlock_icp(& MintTokenRequest{
                        ticket_id: ticket.ticket_id.clone(),
                        token_id: ticket.token.clone(),
                        receiver,
                        amount,
                    }).await.map_err(|e| format!("{:?}",e).to_string())?
                } else {
                    updates::mint_token(&mut MintTokenRequest {
                        ticket_id: ticket.ticket_id.clone(),
                        token_id: ticket.token.clone(),
                        receiver,
                        amount,
                    }).await.map_err(|e| format!("{:?}", e).to_string())?
                }
            } else {
                // regard ticket receiver as chain key assets source chain receiver
                match ticket.token.as_str() {
                    "sICP-icrc-ckBTC" => {
                        retrieve_ckbtc(
                            ticket.receiver.clone(), 
                            Nat::from_str(ticket.amount.as_str()).unwrap(), ticket.ticket_id.clone()
                        )
                        .await
                        .map_err(|e| format!("Failed to retrieve_ckbtc, error: {:?}", e).to_string())?
                    }
                    _ => {
                        return Err("Unsupported token".to_string())
                    }
                } 
            }
        },
        TxAction::RedeemIcpChainKeyAssets(icp_chain_key_token) => {
            // use receiver address and token id to judge if the chain key token should retrieve
            // this branch will be deprecated in the future
            match icp_chain_key_token {
                IcpChainKeyToken::CKBTC => {
                    retrieve_ckbtc(
                        ticket.receiver.clone(), 
                        Nat::from_str(ticket.amount.as_str()).unwrap(), ticket.ticket_id.clone()
                    )
                    .await
                    .map_err(|e| format!("{:?}", e).to_string())?
                },
            }
        },
    };

    Ok(block_index)
}

async fn process_tickets() {
    let (hub_principal, offset) = read_state(|s| (s.hub_principal, s.next_ticket_seq));
    match hub::query_tickets(hub_principal, offset, BATCH_QUERY_LIMIT).await {
        Ok(tickets) => {
            for (seq, ticket) in &tickets {
                match handle_redeem_ticket(ticket).await {
                    Ok(block_index) => {
                        log!(INFO, "[process tickets] process successful for ticket{}, block_index: {}", ticket, block_index);
                        mutate_state(|s| s.next_ticket_seq = seq+1)
                    },
                    Err(e) => {
                        log!(ERROR, "[process tickets] failed to process ticket: {}, err: {}", ticket, e);
                        break;
                    },
                }
            }
        }
        Err(err) => {
            log!(ERROR, "[process tickets] failed to query tickets, err: {}", err);
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
                                log!(INFO,
                                    "[process directives] add token successful, token id: {}",
                                    token.token_id
                                );
                            }
                            Err(err) => {
                                log!(ERROR,
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
                                log!(INFO,
                                    "[process directives] update token successful, token id: {}",
                                    token.token_id
                                );
                            }
                            Err(err) => {
                                log!(ERROR,
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
            log!(ERROR,
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
