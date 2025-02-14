use core::fmt;

use crate::config::read_config;
use crate::constants::RETRY_NUM;
use crate::ic_log::{DEBUG, ERROR, WARNING};
use crate::ic_sui::rpc_client::RpcClient;
use crate::ic_sui::sui_json_rpc_types::sui_transaction::{
    SuiExecutionStatus, SuiTransactionBlockEffectsAPI,
};
use crate::state::{TxStatus, UpdateTokenStatus};
use candid::CandidType;

use crate::state::{mutate_state, read_state};
use ic_canister_log::log;

use anyhow::Error;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, CandidType, Deserialize, Serialize)]
pub struct TxError {
    pub block_hash: String,
    pub signature: String,
    pub error: String,
}
impl fmt::Display for TxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "TxError: block_hash={}, signature={}, error={}",
            self.block_hash, self.signature, self.error
        )
    }
}
impl std::error::Error for TxError {}
impl TryFrom<Error> for TxError {
    type Error = Error;

    fn try_from(e: Error) -> Result<Self, Self::Error> {
        if let Some(tx_error) = e.downcast_ref::<TxError>() {
            Ok(TxError {
                block_hash: tx_error.block_hash.to_owned(),
                signature: tx_error.signature.to_owned(),
                error: tx_error.error.to_owned(),
            })
        } else {
            Err(e)
        }
    }
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub struct TokenInfo {
    pub token_id: String,
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub uri: String,
}

pub async fn update_token() {
    if read_state(|s| s.update_token_queue.is_empty()) {
        return;
    }

    if let Some((update_type, update_status)) = mutate_state(|s| s.update_token_queue.pop_first()) {
        if update_status.retry >= RETRY_NUM {
            return;
        }

        let (provider, nodes, forward) = read_config(|s| {
            (
                s.get().rpc_provider.to_owned(),
                s.get().nodes_in_subnet,
                s.get().forward.to_owned(),
            )
        });
        let client = RpcClient::new(provider, Some(nodes));

        let sui_token = read_state(|s| s.sui_tokens.get(&update_status.token_id))
            .expect("sui token should exists");
        match client
            .update_token_meta(sui_token, update_type.to_owned(), None, forward)
            .await
        {
            Ok(tx_resp) => {
                log!(
                    DEBUG,
                    "[sui_token::update_token] update token metadata for {} ,tx_resp: {:?}",
                    update_status.token_id.to_string(),
                    tx_resp
                );
                //check tx status
                match tx_resp.effects {
                    None => {
                        log!(
                            WARNING,
                            "[sui_token::update_token] Not Found effects and retry ... ",
                        );

                        let re_update = UpdateTokenStatus {
                            token_id: update_status.token_id,
                            retry: update_status.retry + 1,
                            degist: update_status.degist,
                            status: update_status.status,
                        };
                        mutate_state(|s| s.update_token_queue.insert(update_type, re_update));
                    }
                    Some(effects) => match effects.status() {
                        SuiExecutionStatus::Success => {
                            mutate_state(|s| {
                                // update the token info in route
                                if let Some(mut token) = s.tokens.get(&update_status.token_id) {
                                    match update_type {
                                        crate::state::UpdateType::Name(name) => {
                                            token.name = name;
                                            s.tokens.insert(update_status.token_id, token);
                                        }
                                        crate::state::UpdateType::Symbol(symbol) => {
                                            token.symbol = symbol;
                                            s.tokens.insert(update_status.token_id, token);
                                        }
                                        crate::state::UpdateType::Icon(icon) => {
                                            token.icon = Some(icon);
                                            s.tokens.insert(update_status.token_id, token);
                                        }
                                        crate::state::UpdateType::Description(desc) => {
                                            // nothing to do,because the token without description field
                                            log!(
                                                DEBUG,
                                                "[sui_token::update_token] update token description: {:?}  ",
                                                desc,
                                            );
                                        }
                                    };
                                }
                            });
                        }
                        SuiExecutionStatus::Failure { error } => {
                            let re_update = UpdateTokenStatus {
                                token_id: update_status.token_id,
                                retry: update_status.retry + 1,
                                degist: update_status.degist,
                                status: TxStatus::TxFailed {
                                    e: error.to_owned(),
                                },
                            };
                            mutate_state(|s| s.update_token_queue.insert(update_type, re_update));
                        }
                    },
                }
            }
            Err(e) => {
                log!(
                    ERROR,
                    "[sui_token::update_token] update token metadata rpc error: {:?}  ",
                    e
                );

                let re_update = UpdateTokenStatus {
                    token_id: update_status.token_id,
                    // update_type: update_token.update_type,
                    retry: update_status.retry + 1,
                    degist: update_status.degist,
                    status: TxStatus::TxFailed { e: e.to_string() },
                };
                mutate_state(|s| s.update_token_queue.insert(update_type, re_update));
            }
        }
    }
}
