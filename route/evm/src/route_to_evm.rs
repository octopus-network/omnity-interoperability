use anyhow::anyhow;
use ethers_core::types::U256;
use log::info;

use crate::const_args::{ADD_TOKEN_EVM_TX_FEE, DEFAULT_EVM_TX_FEE, SEND_EVM_TASK_NAME};
use crate::contracts::{gen_eip1559_tx, gen_execute_directive_data, gen_mint_token_data};
use crate::eth_common::{broadcast, get_account_nonce, get_gasprice, sign_transaction};
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
            Err(e) => {
                log::error!("[evm_route] send directive to evm error: {}", e.to_string());
            }
        }
    }
    mutate_state(|s| s.next_consume_directive_seq = to);
}

pub async fn send_tickets_to_evm() {
    let from = read_state(|s| s.next_consume_ticket_seq);
    let to = read_state(|s| s.next_ticket_seq);
    for seq in from..to {
        match send_ticket(seq).await {
            Ok(_) => {}
            Err(e) => {
                log::error!("[evm_route] send ticket to evm error: {}", e.to_string());
            }
        }
    }
    mutate_state(|s| s.next_consume_ticket_seq = to);
}

pub async fn send_ticket(seq: Seq) -> anyhow::Result<()> {
    match read_state(|s| s.tickets_queue.get(&seq)) {
        None => Ok(()),
        Some(t) => {
            if read_state(|s| s.finalized_mint_token_requests.contains_key(&t.ticket_id)) {
                return Ok(());
            }
            let data_result = gen_mint_token_data(&t);
            if data_result.is_err() {
                return Err(anyhow!(data_result.err().unwrap().to_string()));
            }
            let nonce = get_account_nonce(minter_addr()).await.unwrap_or_default();
            let tx = gen_eip1559_tx(
                data_result.unwrap(),
                get_gasprice().await.ok(),
                nonce,
                DEFAULT_EVM_TX_FEE,
            );
            info!(
                "[evm route] send directive eip1559tx content: {:?}",
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
                    let hash = broadcast(data).await;
                    match hash {
                        Ok(h) => {
                            pending_ticket.evm_tx_hash = Some(h);
                            mutate_state(|s| {
                                s.pending_tickets_map.insert(t.ticket_id, pending_ticket)
                            });
                            Ok(())
                        }
                        Err(e) => {
                            pending_ticket.error = Some(e.to_string());
                            mutate_state(|s| {
                                s.pending_tickets_map.insert(t.ticket_id, pending_ticket)
                            });
                            Err(anyhow!(e.to_string()))
                        }
                    }
                }
                Err(e) => Err(anyhow!(e.to_string())),
            }
        }
    }
}

pub async fn send_directive(seq: Seq) -> anyhow::Result<()> {
    match read_state(|s| s.directives_queue.get(&seq)) {
        None => Ok(()),
        Some(d) => {
            let data = gen_execute_directive_data(&d, U256::from(seq));
            if data.is_empty() {
                //the directive needn't send to evm.
                return Ok(());
            }
            let nonce = get_account_nonce(minter_addr()).await.unwrap_or_default();
            let fee = match d {
                Directive::AddToken(_) => ADD_TOKEN_EVM_TX_FEE,
                _ => DEFAULT_EVM_TX_FEE,
            };
            let tx = gen_eip1559_tx(data, get_gasprice().await.ok(), nonce, fee);
            info!(
                "[evm route] send directive eip1559tx content: {:?}",
                serde_json::to_string(&tx)
            );
            let mut pending_directive = PendingDirectiveStatus {
                evm_tx_hash: None,
                seq,
                error: None,
            };
            match sign_transaction(tx).await {
                Ok(data) => {
                    let hash = broadcast(data).await;
                    match hash {
                        Ok(h) => {
                            pending_directive.evm_tx_hash = Some(h);
                            mutate_state(|s| {
                                s.pending_directive_map.insert(seq, pending_directive)
                            });
                            Ok(())
                        }
                        Err(e) => {
                            pending_directive.error = Some(e.to_string());
                            mutate_state(|s| {
                                s.pending_directive_map.insert(seq, pending_directive)
                            });
                            Err(anyhow!(e.to_string()))
                        }
                    }
                }
                Err(e) => Err(anyhow!(e.to_string())),
            }
        }
    }
}