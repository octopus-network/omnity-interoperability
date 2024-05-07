use cketh_common::eth_rpc_client::RpcConfig;
use evm_rpc::candid_types::SendRawTransactionStatus;
use evm_rpc::RpcServices;

use crate::contracts::{gen_eip1559_tx, gen_mint_token_data, sign_transaction};
use crate::state::{mutate_state, read_state};
use crate::types::PendingTicketStatus;

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

async fn broadcast(tx: Vec<u8>) -> Result<String, super::Error> {
    let raw = hex::encode(tx);
    let (r,): (SendRawTransactionStatus,) = ic_cdk::call(
        crate::state::rpc_addr(),
        "eth_sendRawTransaction",
        (
            RpcServices::Custom {
                chain_id: crate::state::target_chain_id(),
                services: crate::state::rpc_providers(),
            },
            None::<RpcConfig>,
            raw,
        ),
    )
        .await
        .map_err(|(_, e)| super::Error::EvmRpcError(e))?;
    match r {
        SendRawTransactionStatus::Ok(hash) => hash.map(|h| h.to_string()).ok_or(
            super::Error::EvmRpcError("A transaction hash is expected".to_string()),
        ),
        _ => Err(super::Error::EvmRpcError(format!("{:?}", r))),
    }
}