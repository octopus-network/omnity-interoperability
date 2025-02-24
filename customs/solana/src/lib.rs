use guard::{TaskType, TimerGuard};
use ic_canister_log::log;
use ic_solana::ic_log::ERROR;
use solana_rpc::get_signature_status;
use state::{mutate_state, read_state, ReleaseTokenStatus};
use std::time::Duration;
use transaction::{TransactionConfirmationStatus, TransactionStatus};
use types::omnity_types::{ChainState, Directive};
use updates::submit_release_token_tx;

pub mod address;
pub mod call_error;
pub mod guard;
pub mod hub;
pub mod lifecycle;
pub mod memory;
pub mod port_native;
pub mod service;
pub mod solana_rpc;
pub mod state;
pub mod transaction;
pub mod types;
pub mod updates;

pub const BATCH_QUERY_LIMIT: u64 = 20;
pub const INTERVAL_PROCESSING: Duration = Duration::from_secs(5);
pub const INTERVAL_QUERY_DIRECTIVES: Duration = Duration::from_secs(60);
pub const RETRY_TX_INTERVAL: Duration = Duration::from_secs(600);

pub fn process_release_token_task() {
    ic_cdk::spawn(async {
        let _guard = match TimerGuard::new(TaskType::ProcessTx) {
            Ok(guard) => guard,
            Err(_) => return,
        };
        finalize_release_token_txs().await;
    });
}

pub fn process_ticket_msg_task() {
    ic_cdk::spawn(async {
        let _guard = match TimerGuard::new(TaskType::GetTickets) {
            Ok(guard) => guard,
            Err(_) => return,
        };
        process_tickets().await;
    });
}

pub fn process_directive_msg_task() {
    ic_cdk::spawn(async {
        let _guard = match TimerGuard::new(TaskType::GetDirectives) {
            Ok(guard) => guard,
            Err(_) => return,
        };
        process_directives().await;
    });
}

async fn process_tickets() {
    if read_state(|s| s.chain_state == ChainState::Deactive) {
        return;
    }

    let (hub_principal, offset) = read_state(|s| (s.hub_principal, s.next_ticket_seq));
    match hub::query_tickets(hub_principal, offset, BATCH_QUERY_LIMIT).await {
        Ok(tickets) => {
            for (_, ticket) in &tickets {
                if let Err(err) = updates::add_release_token_req(ticket.clone()).await {
                    log!(ERROR, "[process_tickets] err: {:?}", err);
                }
            }
            let next_seq = tickets.last().map_or(offset, |(seq, _)| seq + 1);
            mutate_state(|s| s.next_ticket_seq = next_seq);
        }
        Err(err) => {
            log!(ERROR, "[process_tickets] temporarily unavailable: {}", err);
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
                        mutate_state(|s| {
                            s.counterparties
                                .insert(chain.chain_id.clone(), chain.clone())
                        });
                    }
                    Directive::AddToken(token) | Directive::UpdateToken(token) => {
                        mutate_state(|s| s.tokens.insert(token.token_id.clone(), token.clone()));
                    }
                    Directive::ToggleChainState(toggle) => {
                        mutate_state(|s| s.toggle_chain_state(toggle.clone()))
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
            log!(
                ERROR,
                "[process directives] failed to query directives, err: {:?}",
                err
            );
        }
    };
}

async fn finalize_release_token_txs() {
    let mut waiting_finalized = vec![];
    let mut signatures = vec![];
    for (_, mut req) in read_state(|s| s.release_token_requests.clone()) {
        match &req.status {
            ReleaseTokenStatus::Pending => {
                submit_release_token_tx(&mut req).await;
            }
            ReleaseTokenStatus::Submitted(sig) => {
                waiting_finalized.push(req.clone());
                signatures.push(sig.clone());
            }
            _ => {}
        }
    }

    if waiting_finalized.is_empty() {
        return;
    }

    let now = ic_cdk::api::time();
    match get_signature_status(signatures.clone()).await {
        Ok(status) => {
            for (i, sig) in signatures.iter().enumerate() {
                let request = waiting_finalized.get_mut(i).unwrap();
                match status[i].clone() {
                    Some(TransactionStatus {
                        confirmation_status: Some(TransactionConfirmationStatus::Finalized),
                        ..
                    }) => {
                        request.status = ReleaseTokenStatus::Finalized(sig.clone());
                        mutate_state(|s| {
                            s.release_token_requests.remove(&request.ticket_id);
                            s.finalized_requests
                                .insert(request.ticket_id.clone(), request.clone());
                        });

                        let hub_principal = read_state(|s| s.hub_principal);
                        if let Err(err) = hub::update_tx_hash(
                            hub_principal,
                            request.ticket_id.clone(),
                            sig.clone(),
                        )
                        .await
                        {
                            log!(ERROR, "fail to update tx hash to hub:{:?}", err);
                        }
                    }
                    _ => {
                        if request.last_sent_at + RETRY_TX_INTERVAL.as_nanos() as u64 <= now {
                            submit_release_token_tx(request).await;
                        }
                    }
                }
            }
        }
        Err(err) => {
            log!(ERROR, "failed to get signature status, err:{:?}", err);
        }
    }
}
