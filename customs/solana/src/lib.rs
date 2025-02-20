use address::main_address_path;
use guard::{TaskType, TimerGuard};
use ic_canister_log::log;
use ic_solana::ic_log::ERROR;
use solana_rpc::{get_signature_status, init_solana_client};
use state::{mutate_state, read_state, CollectionTx, ReleaseTokenReq, ReleaseTokenStatus};
use std::{collections::BTreeMap, time::Duration};
use transaction::{TransactionConfirmationStatus, TransactionStatus};
use types::omnity_types::{ChainState, Directive};
use updates::submit_release_token_tx;

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
pub const INTERVAL_PROCESSING: Duration = Duration::from_secs(5);
pub const INTERVAL_QUERY_DIRECTIVES: Duration = Duration::from_secs(60);
pub const RETRY_COLLECTION_TX_INTERVAL: Duration = Duration::from_secs(3600);

pub fn process_release_token_task() {
    ic_cdk::spawn(async {
        let _guard = match TimerGuard::new(TaskType::ProcessTx) {
            Ok(guard) => guard,
            Err(_) => return,
        };
        finalize_collection_txs().await;
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
    for mut req in read_state(|s| {
        s.release_token_requests
            .iter()
            .filter(|(_, req)| req.status == ReleaseTokenStatus::Pending)
            .map(|(_, req)| req.clone())
            .collect::<Vec<ReleaseTokenReq>>()
    }) {
        submit_release_token_tx(&mut req).await;
    }
    let submitted_reqs: BTreeMap<String, ReleaseTokenReq> = read_state(|s| {
        s.release_token_requests
            .iter()
            .filter(|(_, req)| req.status == ReleaseTokenStatus::Submitted)
            .map(|(_, req)| (req.signature.clone().unwrap(), req.clone()))
            .collect()
    });
    if submitted_reqs.is_empty() {
        return;
    }

    let signatures: Vec<String> = submitted_reqs.keys().cloned().collect();
    match get_signature_status(signatures.clone()).await {
        Ok(status) => {
            for (i, sig) in signatures.iter().enumerate() {
                let mut request = submitted_reqs.get(sig).unwrap().clone();
                match status[i].clone() {
                    None => {
                        request.status =
                            ReleaseTokenStatus::Failed("transaiton is not on chain".into());
                        mutate_state(|s| {
                            s.release_token_requests
                                .insert(request.ticket_id.clone(), request)
                        });
                    }
                    Some(TransactionStatus { err: Some(err), .. }) => {
                        request.status = ReleaseTokenStatus::Failed(err.to_string());
                        mutate_state(|s| {
                            s.release_token_requests
                                .insert(request.ticket_id.clone(), request)
                        });
                    }
                    Some(TransactionStatus {
                        confirmation_status: Some(TransactionConfirmationStatus::Finalized),
                        ..
                    }) => {
                        request.status = ReleaseTokenStatus::Finalized;
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

async fn send_collection_tx(args: &mut CollectionTx) -> Result<(), String> {
    let sol_client = init_solana_client().await;
    let main_address = solana_rpc::ecdsa_public_key(main_address_path()).await;
    args.last_sent_at = ic_cdk::api::time();
    args.try_cnt += 1;

    match sol_client
        .transfer(args.from, args.from_path.clone(), main_address, args.amount)
        .await
    {
        Ok(signature) => {
            args.signature = Some(signature);
            Ok(())
        }
        Err(err) => Err(err.to_string()),
    }?;
    mutate_state(|s| {
        s.collection_tx_requests.remove(&args.source_signature);
        s.submitted_collection_txs
            .insert(args.source_signature.clone(), args.clone())
    });
    Ok(())
}

async fn finalize_collection_txs() {
    for (_, mut tx) in read_state(|s| s.collection_tx_requests.clone()) {
        if let Err(err) = send_collection_tx(&mut tx).await {
            log!(
                ERROR,
                "failed to send collection tx: {:?}, signature:{}",
                err,
                tx.source_signature
            );
        }
    }

    let now = ic_cdk::api::time();
    let mut waiting_finalized = vec![];
    for (_, mut tx) in read_state(|s| s.submitted_collection_txs.clone()) {
        match tx.signature {
            None => {
                if let Err(err) = send_collection_tx(&mut tx).await {
                    log!(ERROR, "fail to resend collection tx:{}", err);
                }
            }
            Some(_) => {
                waiting_finalized.push(tx.clone());
            }
        }
    }

    if waiting_finalized.is_empty() {
        return;
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
