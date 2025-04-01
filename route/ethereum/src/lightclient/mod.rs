use crate::lightclient::ic_execution_rpc::IcExecutionRpc;
use crate::lightclient::rpc_types::receipt::encode_receipt;
use crate::state::{mutate_state, read_state};
use candid::utils::ArgumentEncoder;
use candid::{CandidType, Principal};
use const_hex::ToHexExt;
use ic_canister_log::log;
use omnity_types::call_error::{CallError, Reason};
use omnity_types::ic_log::WARNING;
use serde_derive::{Deserialize, Serialize};
use tree_hash::fixed_bytes::B256;
use triehash_ethereum::ordered_trie_root;

mod http;
mod ic_execution_rpc;
pub mod rpc_types;

pub const LIGHTCLIENT_CANISTER: &str = "2jzxy-6qaaa-aaaai-atfya-cai";

pub async fn query_finality_height() -> Result<Option<u64>, CallError> {
    let principal = Principal::from_text(LIGHTCLIENT_CANISTER).unwrap();
    call(principal, "query_finality_height".into(), ()).await
}
pub async fn verify(
    block_height: u64,
    receipt_root: B256,
) -> Result<Result<(), String>, CallError> {
    let principal = Principal::from_text(LIGHTCLIENT_CANISTER).unwrap();
    call(
        principal,
        "verify".into(),
        (block_height, receipt_root.encode_hex_with_prefix()),
    )
    .await
}

pub async fn calculate_receipt_root(
    block_hash: B256,
    tx_receipt: &Vec<u8>,
) -> Result<Option<B256>, String> {
    let rpcs = read_state(|s| s.rpc_providers.clone());
    for r in rpcs {
        let Ok(ic_execution_rpc) = IcExecutionRpc::new(r.url.as_ref()) else {
            continue;
        };
        let Ok(receipts) = ic_execution_rpc.get_block_receipts(block_hash).await else {
            continue;
        };
        let v: Vec<Vec<u8>> = receipts.iter().map(encode_receipt).collect();
        if !v.contains(tx_receipt) {
            return Err("verfyFailed: the tx not in block".to_string());
        }
        let root = ordered_trie_root(v);
        return Ok(Some(B256::from_slice(root.as_ref())));
    }
    Ok(None)
}

pub async fn verify_task() {
    let verify_requests = read_state(|s| s.lightclient_verify_requests.clone());
    if verify_requests.is_empty() {
        return;
    }
    let Ok(Some(finality)) = query_finality_height().await else {
        log!(WARNING, "query lightclient finality height failed");
        return;
    };
    let check_time = ic_cdk::api::time() / 1000000000 - 7200;
    for (k, v) in verify_requests {
        if v.time < check_time {
            mutate_state(|s| {
                s.lightclient_verify_result
                    .insert(k.clone(), Err("check time out".to_string()));
                s.lightclient_verify_requests.remove(&k);
            });
            continue;
        }
        if v.block_number > finality {
            continue;
        }
        match calculate_receipt_root(v.block_hash, &v.receipt).await {
            Ok(vv) => match vv {
                None => {
                    continue;
                }
                Some(root) => {
                    let Ok(r) = verify(v.block_number, root).await else {
                        continue;
                    };
                    mutate_state(|s| {
                        s.lightclient_verify_result.insert(k.clone(), r);
                        s.lightclient_verify_requests.remove(&k);
                    })
                }
            },
            Err(r) => mutate_state(|s| {
                s.lightclient_verify_result.insert(k.clone(), Err(r));
                s.lightclient_verify_requests.remove(&k);
            }),
        }
    }
}

#[derive(Serialize, Clone, Deserialize)]
pub struct TicketVerifyRecord {
    pub receipt: Vec<u8>,
    pub block_number: u64,
    pub block_hash: B256,
    pub tx_hash: B256,
    pub time: u64,
}

async fn call<T: ArgumentEncoder, R>(
    principal: Principal,
    method: String,
    args: T,
) -> Result<R, CallError>
where
    R: for<'a> candid::Deserialize<'a> + CandidType,
{
    let resp: (R,) = ic_cdk::api::call::call(principal, &method, args)
        .await
        .map_err(|(code, message)| CallError {
            method: method.to_string(),
            reason: Reason::from_reject(code, message),
        })?;
    Ok(resp.0)
}
