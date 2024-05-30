use ethers_core::types::U256;
use log::{error};

use crate::contracts::{gen_eip1559_tx, gen_execute_directive_data, gen_mint_token_data};
use crate::eth_common::{broadcast, get_account_nonce, get_gasprice, sign_transaction};
use crate::state::{minter_addr, mutate_state, read_state};
use crate::types::{Directive, PendingDirectiveStatus, PendingTicketStatus, Seq};

pub fn to_evm_task() {
    ic_cdk::spawn(async {
        let _guard = match crate::guard::TimerLogicGuard::new() {
            Some(guard) => guard,
            None => return,
        };
        send_directives_to_evm().await;
        send_tickets_to_evm().await;
    });
}

pub async fn send_one_directive(seq: Seq) {
    let dire = read_state(|s| s.directives_queue.get(&seq));
    match dire {
        None => {}
        Some(d) => {
            let data = gen_execute_directive_data(&d, U256::from(seq));
            if data.is_empty() {
                return;
            }
            let nonce = get_account_nonce(minter_addr()).await.unwrap_or_default();
            let fee = match d {
                Directive::AddToken(_) => Some(3000000u32),
                _ => None,
            };
            let tx = gen_eip1559_tx(data, get_gasprice().await.ok(), nonce, fee);
            let raw = sign_transaction(tx).await;
            let mut pending_directive = PendingDirectiveStatus {
                evm_tx_hash: None,
                seq,
                error: None,
            };
            match raw {
                Ok(data) => {
                    let hash = broadcast(data).await;
                    match hash {
                        Ok(h) => {
                            pending_directive.evm_tx_hash = Some(h);
                        }
                        Err(e) => {
                            pending_directive.error = Some(e.to_string());
                        }
                    }
                }
                Err(e) => {
                    pending_directive.error = Some(e.to_string());
                }
            }
            mutate_state(|s| s.pending_directive_map.insert(seq, pending_directive));
        }
    }
}

pub async fn send_directives_to_evm() {
    let from = read_state(|s| s.next_consume_directive_seq);
    let to = read_state(|s| s.next_directive_seq);
    for seq in from..to {
        send_one_directive(seq).await;
    }
    mutate_state(|s| s.next_consume_directive_seq = to);
}

pub async fn send_tickets_to_evm() {
    let from = read_state(|s| s.next_consume_ticket_seq);
    let to = read_state(|s| s.next_ticket_seq);
    for seq in from..to {
        let ticket = read_state(|s| s.tickets_queue.get(&seq));
        match ticket {
            None => {
                continue;
            }
            Some(t) => {
                let data = gen_mint_token_data(&t);
                if data.is_err() {
                    error!("{}", data.err().unwrap().to_string());
                    continue;
                }
                let nonce = get_account_nonce(minter_addr()).await.unwrap_or_default();
                let tx = gen_eip1559_tx(data.unwrap(), get_gasprice().await.ok(), nonce, None);

                let raw = sign_transaction(tx).await;
                let mut pending_ticket = PendingTicketStatus {
                    evm_tx_hash: None,
                    ticket_id: t.ticket_id.clone(),
                    seq,
                    error: None,
                };
                match raw {
                    Ok(data) => {
                        let hash = broadcast(data).await;
                        match hash {
                            Ok(h) => {
                                pending_ticket.evm_tx_hash = Some(h);
                            }
                            Err(e) => {
                                pending_ticket.error = Some(e.to_string());
                            }
                        }
                    }
                    Err(e) => {
                        pending_ticket.error = Some(e.to_string());
                    }
                }
                mutate_state(|s| s.pending_tickets_map.insert(t.ticket_id, pending_ticket));
            }
        }
    }
    mutate_state(|s| s.next_consume_ticket_seq = to);
}
