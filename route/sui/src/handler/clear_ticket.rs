use crate::ic_sui::rpc_client::RpcClient;
use crate::ic_sui::sui_json_rpc_types::sui_transaction::{
    SuiExecutionStatus, SuiTransactionBlockEffectsAPI,
};

use crate::config::read_config;
use crate::state::TxStatus;
use crate::state::{mutate_state, read_state};

use crate::constants::{RETRY_NUM, TAKE_SIZE};
use crate::ic_log::{DEBUG, WARNING};
use candid::CandidType;
use ic_canister_log::log;
use ic_stable_structures::storable::Bound;
use ic_stable_structures::Storable;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct ClearTx {
    pub digest: Option<String>,
    pub status: TxStatus,
    pub retry: u64,
}

impl ClearTx {
    pub fn new() -> Self {
        Self {
            digest: None,
            status: TxStatus::New,
            retry: 0,
        }
    }
}

impl Storable for ClearTx {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        let bytes = bincode::serialize(&self).expect("failed to serialize ClearTx");
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        bincode::deserialize(bytes.as_ref()).expect("failed to deserialize ClearTx")
    }

    const BOUND: Bound = Bound::Unbounded;
}

pub async fn clear_ticket_from_port() {
    let clr_tickets = read_state(|s| {
        s.clr_ticket_queue
            .iter()
            .filter(|(_, clear_tx)| !matches!(clear_tx.status, TxStatus::Finalized))
            .take(TAKE_SIZE.try_into().unwrap())
            .map(|(ticket_id, clear_tx)| (ticket_id, clear_tx))
            .collect::<Vec<_>>()
    });

    for (ticket_id, clear_tx) in clr_tickets.into_iter() {
        if clear_tx.retry < RETRY_NUM {
            remove_ticket_from_port(ticket_id.to_owned()).await;
        } else {
            log!(
                WARNING,
               "[clear_ticket::clear_ticket_from_port] failed to clear ticket for ticket id: {}, error: {:?} , and reach to max retry,pls contact your administrator",
               ticket_id,clear_tx.status
            );
        }
    }
}

/// send tx to sui port for clear ticket
pub async fn remove_ticket_from_port(ticket_id: String) {
    let (action, provider, nodes, gas_budget, forward) = read_config(|s| {
        (
            s.get().sui_port_action.to_owned(),
            s.get().rpc_provider.to_owned(),
            s.get().nodes_in_subnet,
            s.get().gas_budget,
            s.get().forward.to_owned(),
        )
    });
    let client = RpcClient::new(provider, Some(nodes));
    let tx_resp = client
        .remove_ticket(action, ticket_id.to_owned(), Some(gas_budget), forward)
        .await;

    match tx_resp {
        Ok(resp) => {
            log!(
                DEBUG,
                "[clear_ticket::remove_ticket_from_port] clear ticket was submited for ticket id: {} and tx_resp: {:?} ",
                ticket_id,resp);
            //check tx status
            match resp.effects {
                None => {
                    log!(
                        WARNING,
                        "[clear_ticket::remove_ticket_from_port] Not Found tx effects and retry ... ",
                    );

                    mutate_state(|s| {
                        if let Some(clr_tx) = s.clr_ticket_queue.get(&ticket_id).as_mut() {
                            clr_tx.retry += 1;
                            clr_tx.status = TxStatus::TxFailed {
                                e: " Not Found effects in tx response".to_string(),
                            };
                            // req.digest = None;
                            s.clr_ticket_queue
                                .insert(ticket_id.to_string(), clr_tx.to_owned());
                        }
                    });
                }
                Some(effects) => match effects.status() {
                    SuiExecutionStatus::Success => {
                        log!(
                            DEBUG,
                            "[clear_ticket::remove_ticket_from_port] clear ticket for ticket id: {} successfully!",
                            ticket_id
                        );
                        mutate_state(|s| {
                            if let Some(clr_tx) = s.clr_ticket_queue.get(&ticket_id).as_mut() {
                                clr_tx.status = TxStatus::Finalized;
                                clr_tx.digest = Some(resp.digest.to_string());
                                s.clr_ticket_queue
                                    .insert(ticket_id.to_string(), clr_tx.to_owned());
                            }
                        });
                    }
                    SuiExecutionStatus::Failure { error } => {
                        log!(
                            WARNING,
                            "[clear_ticket::remove_ticket_from_port] sui tx execute failured: {} ",
                            error
                        );
                        mutate_state(|s| {
                            if let Some(clr_tx) = s.clr_ticket_queue.get(&ticket_id).as_mut() {
                                clr_tx.retry += 1;
                                clr_tx.status = TxStatus::TxFailed {
                                    e: error.to_owned(),
                                };
                                s.clr_ticket_queue
                                    .insert(ticket_id.to_string(), clr_tx.to_owned());
                            }
                        });
                    }
                },
            }
        }
        Err(e) => {
            let error = format!(
                "[clear_ticket::remove_ticket_from_port] failed to clear ticket for ticket id: {}, rpc error: {:?}",
                ticket_id, e
            );
            log!(WARNING, "{}", error.to_string());

            // if err, update req status
            mutate_state(|s| {
                if let Some(clr_tx) = s.clr_ticket_queue.get(&ticket_id).as_mut() {
                    clr_tx.retry += 1;
                    clr_tx.status = TxStatus::TxFailed { e: error };
                    s.clr_ticket_queue
                        .insert(ticket_id.to_string(), clr_tx.to_owned());
                }
            });
        }
    }
}

#[cfg(test)]
mod test {

    #[test]
    fn test_match_tx_error() {
        let log_message = r#"
            TxFailed { e: \"management call '[solana_rpc::create_mint_account] create_mint_with_metaplex' failed: canister error: TxError: block_hash=B9p4ZCrQuWqbWFdhTx3ZseunFiV1sNQ5ZyjEZvuKNjbJ, signature=5o1BYJ76Yx65U3brvkuFwkJ4LkZVev28337mq8u4eg2Vi8S2DBjvSn9LuNuuNp5Gqi1D3BDexmRRHjYM6NdhWAVW, error=[solana_client::send_raw_transaction] rpc error: RpcResponseError { code: -32002, message: \\\"Transactionsimulationfailed: Blockhashnotfound\\\", data: None }\" } 
            "#;
        if log_message.contains("Transactionsimulationfailed: Blockhashnotfound") {
            println!("{}", log_message);
        } else {
            println!("not found");
        }

        if log_message.contains("Transactionsimulationfailed") {
            println!("{}", log_message);
        } else {
            println!("not found");
        }
    }

    #[test]
    fn test_match_status_error() {
        let log_message = r#"
          TxFailed { e: \"management call 'sol_getSignatureStatuses' failed: canister error: parse error: expected invalid type: null, expected struct TransactionStatus at line 1 column 91\" }
            "#;
        if log_message.contains("expected invalid type: null") {
            println!("{}", log_message);
        } else {
            println!("not found");
        }

        if log_message.contains("expected struct TransactionStatus") {
            println!("{}", log_message);
        } else {
            println!("not found");
        }
    }
}
