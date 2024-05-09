use ethers_core::types::U256;
use crate::contracts::{broadcast, gen_eip1559_tx, gen_execute_directive_data, gen_mint_token_data, PortContractCommandIndex, sign_transaction};
use crate::state::{mutate_state, read_state};
use crate::types::{Directive, PendingTicketStatus};

pub fn to_cdk_tickets_task() {
    ic_cdk::spawn(async {
        let _guard = match crate::guard::TimerLogicGuard::new() {
            Some(guard) => guard,
            None => return,
        };
    });
}

pub async fn send_directives_to_cdk() {
    let from = read_state(|s| s.next_consume_directive_seq);
    let to  = read_state(|s|s.next_directive_seq);
    for seq in from..to {
        let  dire = read_state(|s|s.directives_queue.get(&seq));
        match dire {
            None => {
                continue;
            }
            Some(d) => {
                let data = gen_execute_directive_data(&d, U256::from(seq));
                let tx = gen_eip1559_tx(data);
                let raw = sign_transaction(tx).await;

            }
        }
    }
    mutate_state(|s|s.next_consume_ticket_seq = to);
}



pub async fn send_tickets_to_cdk() {
    let from = read_state(|s| s.next_consume_ticket_seq);
    let to = read_state(|s| s.next_ticket_seq);
    for seq in from..to {
        let ticket = read_state(|s|s.tickets_queue.get(&seq));
        match ticket {
            None => {
                continue;
            }
            Some(t) => {
                let data = gen_mint_token_data(&t);
                let tx = gen_eip1559_tx(data);
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