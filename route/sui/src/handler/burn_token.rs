use crate::ic_sui::rpc_client::RpcClient;
use crate::ic_sui::sui_json_rpc_types::sui_transaction::{
    SuiExecutionStatus, SuiTransactionBlockEffectsAPI,
};

use crate::config::read_config;
use crate::ic_sui::sui_types::base_types::ObjectID;
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
use std::str::FromStr;

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct BurnTx {
    pub token_id: String,
    pub digest: Option<String>,
    pub status: TxStatus,
    pub retry: u64,
}

impl BurnTx {
    pub fn new(token_id: String) -> Self {
        Self {
            token_id,
            digest: None,
            status: TxStatus::New,
            retry: 0,
        }
    }
}

impl Storable for BurnTx {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        let bytes = bincode::serialize(&self).expect("failed to serialize BurnTx");
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        bincode::deserialize(bytes.as_ref()).expect("failed to deserialize BurnTx")
    }

    const BOUND: Bound = Bound::Unbounded;
}

pub async fn burn_token() {
    let burn_tokens = read_state(|s| {
        s.burn_tokens
            .iter()
            .filter(|(_, burn_tx)| !matches!(burn_tx.status, TxStatus::Finalized))
            .take(TAKE_SIZE.try_into().unwrap())
            .map(|(burn_coin_id, burn_tx)| (burn_coin_id, burn_tx))
            .collect::<Vec<_>>()
    });

    for (obj_id, burn_tx) in burn_tokens.into_iter() {
        if burn_tx.retry < RETRY_NUM {
            burn_token_from_sui(obj_id.to_owned(), burn_tx.token_id.to_owned()).await;
        } else {
            log!(
                WARNING,
               "[burn_token::burn_token_from_sui] burn token for object: {}, error: {:?} , and reach to max retry,pls contact your administrator",
               obj_id,burn_tx.status
            );
        }
    }
}

/// send tx to sui to burn token
pub async fn burn_token_from_sui(obj_id: String, token_id: String) {
    let (provider, nodes, forward) = read_config(|s| {
        (
            s.get().rpc_provider.to_owned(),
            s.get().nodes_in_subnet,
            s.get().forward.to_owned(),
        )
    });
    let sui_token = read_state(|s| s.sui_tokens.get(&token_id)).expect("Sui token not found");
    let client = RpcClient::new(provider, Some(nodes));
    let obj = ObjectID::from_str(obj_id.as_ref()).expect("Can't Convert to object");
    let tx_resp = client
        .burn_token(sui_token, obj, None, forward.clone())
        .await;

    match tx_resp {
        Ok(resp) => {
            log!(
                DEBUG,
                "[burn_token::burn_token_from_sui] burned token was submited for ticket id: {} and tx_resp: {:?} ",
                obj_id,resp);
            //check tx status
            match resp.effects {
                None => {
                    log!(
                        WARNING,
                        "[burn_token::burn_token_from_sui] Not Found tx effects and retry ... ",
                    );

                    mutate_state(|s| {
                        if let Some(burn_tx) = s.burn_tokens.get(&obj_id).as_mut() {
                            burn_tx.retry += 1;
                            burn_tx.status = TxStatus::TxFailed {
                                e: " Not Found effects in tx response".to_string(),
                            };
                            // req.digest = None;
                            s.burn_tokens.insert(obj_id.to_string(), burn_tx.to_owned());
                        }
                    });
                }
                Some(effects) => match effects.status() {
                    SuiExecutionStatus::Success => {
                        log!(
                            DEBUG,
                            "[burn_token::burn_token_from_sui] burn token for obj id: {} successfully!",
                            obj_id
                        );
                        mutate_state(|s| {
                            if let Some(burn_tx) = s.burn_tokens.get(&obj_id).as_mut() {
                                burn_tx.status = TxStatus::Finalized;
                                burn_tx.digest = Some(resp.digest.to_string());
                                s.burn_tokens.insert(obj_id.to_string(), burn_tx.to_owned());
                            }
                        });
                    }
                    SuiExecutionStatus::Failure { error } => {
                        log!(
                            WARNING,
                            "[burn_token::burn_token_from_sui] sui tx execute failured: {} ",
                            error
                        );
                        mutate_state(|s| {
                            if let Some(burn_tx) = s.burn_tokens.get(&obj_id).as_mut() {
                                burn_tx.retry += 1;
                                burn_tx.status = TxStatus::TxFailed {
                                    e: error.to_owned(),
                                };
                                s.burn_tokens.insert(obj_id.to_string(), burn_tx.to_owned());
                            }
                        });
                    }
                },
            }
        }
        Err(e) => {
            let error = format!(
                "[burn_token::burn_token_from_sui] failed to burn token for obj id: {}, rpc error: {:?}",
                obj_id, e
            );
            log!(WARNING, "{}", error.to_string());

            // if err, update req status
            mutate_state(|s| {
                if let Some(burn_tx) = s.burn_tokens.get(&obj_id).as_mut() {
                    burn_tx.retry += 1;
                    burn_tx.status = TxStatus::TxFailed { e: error };
                    s.burn_tokens.insert(obj_id.to_string(), burn_tx.to_owned());
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
