use anyhow::anyhow;
use log::info;

use crate::base::const_args::{ADD_TOKEN_EVM_TX_FEE, DEFAULT_EVM_TX_FEE, SEND_EVM_TASK_NAME};
use crate::state::{minter_addr, mutate_state, read_state};
use omnity_types::{Directive, Seq};
use crate::{base::get_time_secs, hub};

pub fn to_evm_task() {
    ic_cdk::spawn(async {
        let _guard = match crate::guard::TimerLogicGuard::new(SEND_EVM_TASK_NAME.to_string()) {
            Some(guard) => guard,
            None => return,
        };
        send_directives_to_ton().await;
        send_tickets_to_ton().await;
    });
}

pub async fn send_directives_to_ton() {
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

pub async fn send_tickets_to_ton() {
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
                            log::error!(
                                "[rewrite tx_hash] failed to write mint tx hash, reason: {}",
                                err
                            );
                        }
                        _ => {}
                    }
                }
            },
            Err(e) => {
                log::error!("[evm_route] send ticket to evm error: {}", e.to_string());
            }
        }
    }
    mutate_state(|s| s.next_consume_ticket_seq = to);
}

pub async fn send_ticket(seq: Seq) -> anyhow::Result<Option<String>> {
    match read_state(|s| s.tickets_queue.get(&seq)) {
        None => Ok(None),
        Some(t) => {
            if read_state(|s| s.finalized_mint_token_requests.contains_key(&t.ticket_id)) {
                return Ok(None);
            }
            //TODO
            Ok(None)
        }
    }
}

pub async fn send_directive(seq: Seq) -> anyhow::Result<Option<String>> {
    match read_state(|s| s.directives_queue.get(&seq)) {
        None => Ok(None),
        Some(d) => {
            //TODO
            Ok(None)
        }
    }
}
