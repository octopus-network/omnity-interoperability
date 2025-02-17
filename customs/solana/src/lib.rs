use address::main_address_path;
use guard::{TaskType, TimerGuard};
use ic_canister_log::log;
use ic_solana::ic_log::ERROR;
use solana_rpc::{get_signature_status, init_solana_client};
use state::{mutate_state, read_state, ReleaseTokenReq, TxStatus};
use std::{collections::BTreeMap, time::Duration};
use transaction::{TransactionConfirmationStatus, TransactionStatus};
use types::omnity_types::{ChainState, Directive};
use updates::generate_ticket::send_collection_tx;

pub mod address;
pub mod call_error;
pub mod guard;
pub mod hub;
pub mod lifecycle;
pub mod memory;
pub mod service;
pub mod solana_rpc;
pub mod state;
pub mod transaction;
pub mod types;
pub mod updates;

pub const BATCH_QUERY_LIMIT: u64 = 20;
pub const FINALIZE_TIME: Duration = Duration::from_secs(15);
pub const INTERVAL_PROCESSING: Duration = Duration::from_secs(5);
pub const INTERVAL_QUERY_DIRECTIVES: Duration = Duration::from_secs(60);
pub const RETRY_COLLECTION_TX_INTERVAL: Duration = Duration::from_secs(3600);

pub fn process_release_token_task() {
    ic_cdk::spawn(async {
        let _guard = match TimerGuard::new(TaskType::ReleaseToken) {
            Ok(guard) => guard,
            Err(_) => return,
        };
        submit_txs().await;
        finalize_txs().await;
        finalize_collection_txs().await;
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
                if let Err(err) = updates::add_release_token_req(ticket.clone()) {
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

async fn submit_txs() {
    let mut requests = read_state(|s| s.pending_requests());
    let client = init_solana_client().await;
    let main_path = main_address_path();
    let main_address = solana_rpc::ecdsa_public_key(main_path.clone()).await;
    for req in &mut requests {
        // TODO construct multiple requests into a batch transaction to speed up and reduce gas
        match client
            .transfer(main_address, main_path.clone(), req.address, req.amount)
            .await
        {
            Err(err) => {
                log!(
                    ERROR,
                    "[submit_txs] failed to transfer token for ticket_id:{}, err: {:?}",
                    req.ticket_id,
                    err
                );
                mutate_state(|s| {
                    s.update_release_token_status(&req.ticket_id, TxStatus::Failed(err.to_string()))
                });
                continue;
            }
            Ok(signature) => {
                req.status = TxStatus::Submitted;
                req.signature = Some(signature);
                req.submitted_at = Some(ic_cdk::api::time());
                mutate_state(|s| {
                    s.release_token_requests
                        .insert(req.ticket_id.clone(), req.clone())
                });
            }
        }
    }
}

async fn finalize_txs() {
    let now = ic_cdk::api::time();
    let submitted_reqs: BTreeMap<String, ReleaseTokenReq> = read_state(|s| {
        s.release_token_requests
            .iter()
            .filter_map(|(_, req)| {
                (req.status == TxStatus::Submitted
                    && req.submitted_at.is_some_and(|submitted_at| {
                        submitted_at + FINALIZE_TIME.as_nanos() as u64 <= now
                    }))
                .then_some((req.signature.clone().unwrap(), req.clone()))
            })
            .collect()
    });

    let signatures: Vec<String> = submitted_reqs
        .iter()
        .map(|(signature, _)| signature.clone())
        .collect();

    match get_signature_status(signatures.clone()).await {
        Ok(status) => {
            for (i, sig) in signatures.iter().enumerate() {
                let mut request = submitted_reqs.get(sig).unwrap().clone();
                match status[i].clone() {
                    None => {
                        request.status = TxStatus::Failed("transaiton is not on chain".into());
                        mutate_state(|s| {
                            s.release_token_requests
                                .insert(request.ticket_id.clone(), request)
                        });
                    }
                    Some(TransactionStatus { err: Some(err), .. }) => {
                        request.status = TxStatus::Failed(err.to_string());
                        mutate_state(|s| {
                            s.release_token_requests
                                .insert(request.ticket_id.clone(), request)
                        });
                    }
                    Some(TransactionStatus {
                        confirmation_status: Some(TransactionConfirmationStatus::Finalized),
                        ..
                    }) => {
                        request.status = TxStatus::Finalized;
                        mutate_state(|s| {
                            s.release_token_requests.remove(&request.ticket_id);
                            s.finalized_requests
                                .insert(request.ticket_id.clone(), request.clone());
                        });

                        let hub_principal = read_state(|s| s.hub_principal);
                        if let Err(err) =
                            hub::update_tx_hash(hub_principal, request.ticket_id, sig.clone()).await
                        {
                            log!(ERROR, "fail to update tx hash to hub:{:?}", err);
                        }
                    }
                    _ => {}
                }
            }
        }
        Err(err) => {
            log!(ERROR, "failed to get signature status, err:{:?}", err);
        }
    }
}

async fn finalize_collection_txs() {
    let now = ic_cdk::api::time();
    let mut waiting_finalized = vec![];
    for (sig, mut tx) in read_state(|s| s.submitted_collection_txs.clone()) {
        if tx.signature.is_none()
            && tx.last_sent_at + RETRY_COLLECTION_TX_INTERVAL.as_nanos() as u64 <= now
        {
            if let Err(err) = send_collection_tx(&mut tx).await {
                log!(ERROR, "fail to resend collection tx:{}", err);
            }
            mutate_state(|s| s.submitted_collection_txs.insert(sig, tx.clone()));
        }
        if tx.signature.is_some() {
            waiting_finalized.push(tx.clone());
        }
    }

    let signatures: Vec<String> = waiting_finalized
        .iter()
        .map(|tx| tx.signature.clone().unwrap())
        .collect();

    match get_signature_status(signatures).await {
        Ok(status) => {
            for (i, tx) in waiting_finalized.iter_mut().enumerate() {
                match status[i].clone() {
                    Some(TransactionStatus {
                        confirmation_status: Some(TransactionConfirmationStatus::Finalized),
                        ..
                    }) => {
                        mutate_state(|s| s.submitted_collection_txs.remove(&tx.source_signature));
                    }
                    _ => {
                        if tx.last_sent_at + RETRY_COLLECTION_TX_INTERVAL.as_nanos() as u64 <= now {
                            if let Err(err) = send_collection_tx(tx).await {
                                log!(ERROR, "fail to resend collection tx:{}", err);
                            }
                            mutate_state(|s| {
                                s.submitted_collection_txs
                                    .insert(tx.source_signature.clone(), tx.clone())
                            });
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
