use ethers_core::types::U256;
use ethers_core::utils::keccak256;
use ic_canister_log::log;

use crate::{Error, get_time_secs, hub};
use crate::const_args::{ADD_TOKEN_EVM_TX_FEE, DEFAULT_EVM_TX_FEE, SEND_EVM_TASK_NAME};
use crate::contracts::{gen_evm_tx, gen_execute_directive_data, gen_mint_token_data};
use crate::Error::Custom;
use crate::eth_common::{
    broadcast, call_rpc_with_retry, get_account_nonce, get_gasprice, sign_transaction,
};
use crate::ic_log::{INFO, WARNING};
use crate::state::{minter_addr, mutate_state, read_state};
use crate::types::{Directive, PendingDirectiveStatus, PendingTicketStatus, Seq};

pub fn to_evm_task() {
    ic_cdk::spawn(async {
        let _guard = match crate::guard::TimerLogicGuard::new(SEND_EVM_TASK_NAME.to_string()) {
            Some(guard) => guard,
            None => return,
        };
        send_directives_to_evm().await;
        send_tickets_to_evm().await;
    });
}

pub async fn send_directives_to_evm() {
    let from = read_state(|s| s.next_consume_directive_seq);
    let to = read_state(|s| s.next_directive_seq);
    for seq in from..to {
        let ret = send_directive(seq).await;
        match ret {
            Ok(_) => {}
            Err(e) => match e {
                Error::Temporary => {
                    return;
                }
                _ => {
                    log!(WARNING, "[evm_route] send directive to evm error: {}", e.to_string());
                }
            },
        }
        mutate_state(|s| s.next_consume_directive_seq = seq + 1);
    }
}

pub async fn send_tickets_to_evm() {
    let from = read_state(|s| s.next_consume_ticket_seq);
    let to = read_state(|s| s.next_ticket_seq);
    for seq in from..to {
        match send_ticket(seq).await {
            Ok(h) => match h {
                None => {}
                Some(tx_hash) => {
                    let hub_principal = read_state(|s| s.hub_principal);
                    let ticket_id = read_state(|s| s.tickets_queue.get(&seq).unwrap().ticket_id);
                    match hub::update_tx_hash(hub_principal, ticket_id, tx_hash.clone()).await {
                        Err(err) => {
                            log!(INFO,
                                "[rewrite tx_hash] failed to write mint tx hash, reason: {}",
                                err
                            );
                        }
                        _ => {}
                    }
                }
            },
            Err(e) => match e {
                Error::Temporary => {
                    return;
                }
                _ => {
                    log!(WARNING, "[evm_route] send ticket to evm error: {}", e.to_string());
                }
            },
        }
        mutate_state(|s| s.next_consume_ticket_seq = seq + 1);
    }
}

pub async fn send_ticket(seq: Seq) -> Result<Option<String>, Error> {
    match read_state(|s| s.tickets_queue.get(&seq)) {
        None => Ok(None),
        Some(t) => {
            if read_state(|s| s.finalized_mint_token_requests.contains_key(&t.ticket_id)) {
                return Ok(None);
            }
            let data_result = gen_mint_token_data(&t);
            let nonce = call_rpc_with_retry(minter_addr(), get_account_nonce)
                .await
                .unwrap_or_default();
            let tx = gen_evm_tx(
                data_result,
                call_rpc_with_retry((), get_gasprice).await.ok(),
                nonce,
                DEFAULT_EVM_TX_FEE,
            );
            log!(INFO,
                "[evm route] send ticket tx content: {:?}",
                serde_json::to_string(&tx)
            );
            let mut pending_ticket = PendingTicketStatus {
                evm_tx_hash: None,
                ticket_id: t.ticket_id.clone(),
                seq,
                error: None,
            };
            match sign_transaction(tx).await {
                Ok(data) => {
                    let hash = call_rpc_with_retry(data.clone(), broadcast).await;
                    match hash {
                        Ok(_h) => {
                            let tx_hash = format!("0x{}", hex::encode(keccak256(data)));
                            pending_ticket.evm_tx_hash = Some(tx_hash.clone());
                            mutate_state(|s| {
                                s.pending_tickets_map.insert(t.ticket_id, pending_ticket)
                            });
                            mutate_state(|s| {
                                s.pending_events_on_chain
                                    .insert(tx_hash.clone(), get_time_secs())
                            });
                            Ok(Some(tx_hash))
                        }
                        Err(e) => match e {
                            Error::Temporary => Err(e),
                            _ => {
                                pending_ticket.error = Some(e.to_string());
                                mutate_state(|s| {
                                    s.pending_tickets_map.insert(t.ticket_id, pending_ticket)
                                });
                                Err(e)
                            }
                        },
                    }
                }
                Err(e) => Err(Custom(e.to_string())),
            }
        }
    }
}

pub async fn send_directive(seq: Seq) -> Result<Option<String>, Error> {
    match read_state(|s| s.directives_queue.get(&seq)) {
        None => Ok(None),
        Some(d) => {
            let data = gen_execute_directive_data(&d, U256::from(seq));
            if data.is_empty() {
                //the directive needn't send to evm.
                return Ok(None);
            }
            let nonce = call_rpc_with_retry(minter_addr(), get_account_nonce)
                .await
                .unwrap_or_default();
            let fee = match d {
                Directive::AddToken(_) => ADD_TOKEN_EVM_TX_FEE,
                _ => DEFAULT_EVM_TX_FEE,
            };
            let tx = gen_evm_tx(
                data,
                call_rpc_with_retry((), get_gasprice).await.ok(),
                nonce,
                fee,
            );
            log!(INFO,
                "[evm route] send directive tx content: {:?}",
                serde_json::to_string(&tx)
            );
            let mut pending_directive = PendingDirectiveStatus {
                evm_tx_hash: None,
                seq,
                error: None,
            };
            match sign_transaction(tx).await {
                Ok(data) => {
                    let hash = call_rpc_with_retry(data.clone(), broadcast).await;
                    match hash {
                        Ok(_h) => {
                            let tx_hash = format!("0x{}", hex::encode(keccak256(data)));
                            pending_directive.evm_tx_hash = Some(tx_hash.clone());
                            mutate_state(|s| {
                                s.pending_directive_map.insert(seq, pending_directive)
                            });
                            mutate_state(|s| {
                                s.pending_events_on_chain
                                    .insert(tx_hash.clone(), get_time_secs())
                            });
                            Ok(Some(tx_hash))
                        }
                        Err(e) => match e {
                            Error::Temporary => Err(e),
                            _ => {
                                pending_directive.error = Some(e.to_string());
                                mutate_state(|s| {
                                    s.pending_directive_map.insert(seq, pending_directive)
                                });
                                Err(e)
                            }
                        },
                    }
                }
                Err(e) => Err(Custom(e.to_string())),
            }
        }
    }
}
