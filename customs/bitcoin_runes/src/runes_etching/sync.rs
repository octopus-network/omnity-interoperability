use std::collections::{BTreeMap, HashMap};
use std::time::Duration;

use candid::CandidType;
use ic_btc_interface::Network;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use omnity_types::hub_types::{Proposal, TokenMeta};

use crate::{finalization_time_estimate, state, updates};
use crate::call_error::{CallError, Reason};
use crate::hub::execute_proposal;
use crate::management::{CallSource, get_bitcoin_balance};
use crate::runes_etching::InternalEtchingArgs;
use crate::runes_etching::transactions::{EtchingStatus, SendEtchingRequest};
use crate::runes_etching::transactions::EtchingStatus::{Final, SendCommitFailed, SendRevealFailed, SendRevealSuccess};
use crate::state::{mutate_state, read_state};
use crate::updates::generate_ticket::GenerateTicketArgs;

pub async fn get_etching(txid: &str) -> Result<Option<GetEtchingResponse>, CallError>{
    let method = "get_etching";
    let ord_principal = read_state(|s|s.ord_indexer_principal.clone().unwrap());
    let resp: (Result<Option<GetEtchingResponse>, OrdError>,) =
        ic_cdk::api::call::call(ord_principal, method, (txid,))
            .await
            .map_err(|(code, message)| CallError {
                method: method.to_string(),
                reason: Reason::from_reject(code, message),
            })?;
    let data = resp.0.map_err(|e: OrdError| CallError {
        method: method.to_string(),
        reason: Reason::CanisterError(e.to_string()),
    })?;
    Ok(data)
}



pub async fn send_add_token(args: InternalEtchingArgs, rune_id: String) -> Result<(), CallError> {
    let mut meta = HashMap::new();
    meta.insert("rune_id".to_string(), rune_id);
    let token_meta = TokenMeta {
        token_id: args.token_id,
        name: args.rune_name.clone(),
        symbol: args.rune_name.clone(),
        issue_chain: "Bitcoin".to_string(),
        decimals: args.divisibility.unwrap_or_default(),
        icon: Some(args.bridge_logo_url.clone()),
        metadata: meta,
        dst_chains: vec!["Bitcoin".to_string(), "eICP".to_string()],
    };
    let hub_principal = read_state(|s|s.hub_principal);
    execute_proposal(Proposal::AddToken(token_meta), hub_principal).await
}

#[derive(Deserialize, Serialize, CandidType, Clone, Debug, Default, Eq, PartialEq)]
pub struct GetEtchingResponse {
    pub confirmations: u32,
    pub rune_id: String,
}

#[derive(Debug, Eq, PartialEq, Error, CandidType, Deserialize)]
enum OrdError {
    #[error("params: {0}")]
    Params(String),
    #[error("overflow")]
    Overflow,
    #[error("wrong block hash: {0}")]
    WrongBlockHash(String),
    #[error("wrong block merkle root: {0}")]
    WrongBlockMerkleRoot(String),
    #[error("index error: {0}")]
    Index(#[from] MintError),
    #[error("rpc error: {0}")]
    Rpc(#[from] RpcError),
    #[error("recoverable reorg at height {height} with depth {depth}")]
    Recoverable { height: u32, depth: u32 },
    #[error("unrecoverable reorg")]
    Unrecoverable,
    #[error("outpoint not found")]
    OutPointNotFound,
    #[error("not enough confirmations")]
    NotEnoughConfirmations,
}

#[derive(Debug, Clone, Error, Eq, PartialEq, CandidType, Deserialize)]
pub enum RpcError {
    #[error("IO error occured while calling {0} onto {1} due to {2}.")]
    Io(String, String, String),
    #[error("Decoding response of {0} from {1} failed due to {2}.")]
    Decode(String, String, String),
    #[error("Received an error of endpoint {0} from {1}: {2}.")]
    Endpoint(String, String, String),
}

#[derive(Debug, Clone, Error, Eq, PartialEq, CandidType, Deserialize)]
pub enum MintError {
    #[error("limited to {0} mints")]
    Cap(u128),
    #[error("mint ended on block {0}")]
    End(u64),
    #[error("mint starts on block {0}")]
    Start(u64),
    #[error("not mintable")]
    Unmintable,
}

fn check_time(confirmation_blocks: u32, req_time: u64) -> bool  {
    let now = ic_cdk::api::time();
    let network = read_state(|s|s.btc_network);
    let wait_time = finalization_time_estimate(confirmation_blocks, network);
    let check_timeline = req_time + (wait_time.as_nanos() as u64);
    let check_time_window = Duration::from_secs(10800).as_nanos() as u64;
    check_timeline < now && now < check_timeline + check_time_window
}

pub async fn handle_etching_result_task() {
    if state::read_state(|s| s.pending_etching_requests.is_empty()) {
        return;
    }


    let kvs = read_state(|s|s.pending_etching_requests.iter().collect::<BTreeMap<String, SendEtchingRequest>>());
    for  (k,mut req) in kvs {
        match req.status.clone() {
            EtchingStatus::SendCommitSuccess => {
                if !check_time(6, req.time_at) {
                    continue;
                }
                let balance = get_bitcoin_balance(Network::Mainnet, &req.script_out_address, 6, CallSource::Custom).await.unwrap_or_default();
                if balance == 0 {
                    continue;
                }
                let r = crate::management::send_etching(&req.txs[1]).await;
                if r.is_err() {
                    req.status = SendRevealFailed;
                    req.err_info = r.err();
                }else {
                    req.status = SendRevealSuccess
                }
                req.time_at = ic_cdk::api::time();
                mutate_state(|s|s.pending_etching_requests.insert(k, req));
            },
            EtchingStatus::SendRevealSuccess => {
                if !check_time(1, req.time_at) {
                    continue;
                }
                //query etching,
                let tx = req.txs[1].txid().to_string();
                let rune = get_etching(tx.as_str()).await;
                if let Ok(Some(resp)) = rune {
                    if resp.confirmations >= 1 {
                        let r = send_add_token(req.etching_args.clone(), resp.rune_id.clone()).await;
                        match r {
                            Ok(_) => {
                                req.status = EtchingStatus::TokenAdded;
                                mutate_state(|s| s.pending_etching_requests.insert(k.clone(), req));
                            }
                            Err(_e) => {
                                // do nothing
                            }
                        }
                    }
                }
            }
            EtchingStatus::TokenAdded => {
                //generate_ticket
                if let Some(t) = read_state(|s|s.tokens.get(&req.etching_args.token_id).cloned()) {
                    match req.etching_args.premine {
                        None => {
                            mutate_state(|s| {
                                req.status = Final;
                                s.finalized_etching_requests.insert(k.clone(), req);
                            });
                        }
                        Some(premine) => {
                            let generate_ticket_args = GenerateTicketArgs {
                                target_chain_id: req.etching_args.target_chain_id.clone(),
                                receiver: req.etching_args.premine_receiver_principal.clone(),
                                rune_id: format!("{}",t.0),
                                amount: premine,
                                txid: req.txs[1].txid().to_string(),
                            };
                            if let Ok(_) = updates::generate_ticket::generate_ticket(generate_ticket_args).await {
                                mutate_state(|s| {
                                    req.status = Final;
                                    s.finalized_etching_requests.insert(k.clone(), req);
                                });
                            }
                        }
                    }
                    mutate_state(|s|s.pending_etching_requests.remove(&k));
                }
            }
            EtchingStatus::Final |SendCommitFailed | SendRevealFailed => {}
        }
    }
}