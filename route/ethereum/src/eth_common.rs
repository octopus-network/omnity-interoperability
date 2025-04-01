use std::future::Future;
use std::str::FromStr;

use candid::Nat;
use const_hex::ToHexExt;
use evm_rpc_types::RpcApi;
use evm_rpc_types::RpcConfig;
use evm_rpc_types::RpcService;
use evm_rpc_types::{
    BlockTag, GetTransactionCountArgs, MultiRpcResult, RpcServices, SendRawTransactionStatus,
};
use evm_rpc_types::{Hex20, Hex32, RpcError};

use ethers_core::types::U256;

use ethereum_common::error::Error;
use ethereum_common::error::Error::EvmRpcError;
use ethereum_common::tx_types::{EvmJsonRpcRequest, EvmRpcResponse};
use ic_canister_log::log;
use ic_cdk::api::management_canister::http_request::{
    http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod, TransformContext,
    TransformFunc,
};
use num_traits::ToPrimitive;
use serde_derive::{Deserialize, Serialize};

use crate::const_args::{BROADCAST_TX_CYCLES, GET_ACCOUNT_NONCE_CYCLES};
use crate::lightclient::rpc_types::receipt::TransactionReceipt;
use crate::state::{evm_transfer_gas_factor, read_state, rpc_providers};
use crate::{const_args, state};
use omnity_types::ic_log::{CRITICAL, INFO, WARNING};

pub async fn broadcast(tx: Vec<u8>, rpc: RpcApi) -> Result<String, Error> {
    let raw = tx.encode_hex_with_prefix();
    log!(INFO, "[evm route] preparing to send tx: {}", raw);
    let (r,): (MultiRpcResult<SendRawTransactionStatus>,) =
        ic_cdk::api::call::call_with_payment128(
            crate::state::rpc_addr(),
            "eth_sendRawTransaction",
            (
                RpcServices::Custom {
                    chain_id: crate::state::evm_chain_id(),
                    services: vec![rpc],
                },
                None::<RpcConfig>,
                raw,
            ),
            BROADCAST_TX_CYCLES,
        )
        .await
        .map_err(|(_, e)| Error::EvmRpcError(e))?;
    log!(INFO, "broadcast result:{:?}", r.clone());
    match r {
        MultiRpcResult::Consistent(res) => {
            match res {
                Ok(s) => match s {
                    SendRawTransactionStatus::Ok(hash) => {
                        let r = hex::encode(hash.unwrap_or(Hex32([0u8; 32]))).to_lowercase();
                        Ok(r)
                    }
                    SendRawTransactionStatus::InsufficientFunds => {
                        Err(Error::Fatal("InsufficientFunds".to_string()))
                    }
                    SendRawTransactionStatus::NonceTooLow => {
                        Err(Error::Custom("NonceTooLow".to_string()))
                    }
                    SendRawTransactionStatus::NonceTooHigh => {
                        Err(Error::Custom("NonceToohigh".to_string()))
                    }
                },
                Err(r) => {
                    if let RpcError::JsonRpcError(ref jerr) = r {
                        if (jerr.code == -32603 && (jerr.message == "already known" || jerr.message == "failed to send tx"))
                            || (jerr.code == -32010 && jerr.message == "pending transaction with same hash already exists")
                        || (jerr.code == -32015 && jerr.message == "transaction pool error transaction already exists in the pool") {
                        return Ok(hex::encode([1u8; 32]));
                    }
                        if jerr.code == -32015
                            && jerr
                                .message
                                .contains("transaction pool error invalid transaction nonce")
                        {
                            return Err(Error::Temporary);
                        }
                    }
                    Err(Error::EvmRpcError(format!("{:?}", r)))
                }
            }
        }
        MultiRpcResult::Inconsistent(_r) => {
            Err(Error::EvmRpcError("Inconsistent result".to_string()))
        }
    }
}

pub async fn get_account_nonce(addr: String, rpc: RpcApi) -> Result<u64, Error> {
    let (r,): (MultiRpcResult<Nat>,) = ic_cdk::api::call::call_with_payment128(
        crate::state::rpc_addr(),
        "eth_getTransactionCount",
        (
            RpcServices::Custom {
                chain_id: crate::state::evm_chain_id(),
                services: vec![rpc],
            },
            None::<RpcConfig>,
            GetTransactionCountArgs {
                address: Hex20::from_str(addr.as_str()).unwrap(),
                block: BlockTag::Pending,
            },
        ),
        GET_ACCOUNT_NONCE_CYCLES,
    )
    .await
    .map_err(|(_, e)| Error::EvmRpcError(e))?;
    match r {
        MultiRpcResult::Consistent(r) => match r {
            Ok(c) => Ok(c.0.to_u64().unwrap()),
            Err(r) => Err(Error::EvmRpcError(format!("{:?}", r))),
        },
        MultiRpcResult::Inconsistent(_) => {
            Err(Error::EvmRpcError("Inconsistent result".to_string()))
        }
    }
}

pub async fn get_gasprice(_v: (), rpc: RpcApi) -> Result<U256, Error> {
    // Define request parameters
    let params = (
        RpcService::Custom(rpc.clone()), // Ethereum mainnet
        serde_json::to_string(&EvmJsonRpcRequest {
            method: "eth_gasPrice".to_string(),
            params: vec![],
            id: 1,
            jsonrpc: "2.0".to_string(),
        })
        .unwrap(),
        1000u64,
    );
    // Get cycles cost
    let (cycles_result,): (std::result::Result<u128, RpcError>,) =
        ic_cdk::api::call::call(state::rpc_addr(), "requestCost", params.clone())
            .await
            .unwrap();
    let cycles = cycles_result.map_err(|e| {
        log!(WARNING, "[evm route] evm request error: {:?}", e);
        Error::Custom(format!("error in `request_cost`: {:?}", e))
    })?;
    // Call with expected number of cycles
    let (result,): (std::result::Result<String, RpcError>,) =
        ic_cdk::api::call::call_with_payment128(state::rpc_addr(), "request", params, cycles)
            .await
            .map_err(|err| Error::IcCallError(err.0, err.1))?;
    #[derive(Serialize, Deserialize, Debug)]
    struct BlockNumberResult {
        pub id: u32,
        pub jsonrpc: String,
        pub result: String,
    }
    let r = result.map_err(|e| {
        log!(WARNING, "[evm route]query gas price error: {:?}", &e);
        Error::Custom(format!("[evm route]query gas price error: {:?}", &e))
    })?;
    let r: BlockNumberResult =
        serde_json::from_str(r.as_str()).map_err(|e| Error::Fatal(e.to_string()))?;
    let r = r.result.strip_prefix("0x").unwrap_or(r.result.as_str());
    let r = u64::from_str_radix(r, 16).map_err(|e| Error::Fatal(e.to_string()))?;
    Ok(U256::from(r * evm_transfer_gas_factor() / 100))
}

pub async fn get_balance(addr: String, rpc: RpcApi) -> Result<U256, Error> {
    let params = (
        RpcService::Custom(rpc.clone()), // Ethereum mainnet
        serde_json::to_string(&EvmJsonRpcRequest {
            method: "eth_getBalance".to_string(),
            params: vec![addr, "latest".to_string()],
            id: 1,
            jsonrpc: "2.0".to_string(),
        })
        .unwrap(),
        1000u64,
    );
    // Get cycles cost
    let (cycles_result,): (std::result::Result<u128, RpcError>,) =
        ic_cdk::api::call::call(state::rpc_addr(), "requestCost", params.clone())
            .await
            .unwrap();
    let cycles = cycles_result.map_err(|e| {
        log!(WARNING, "[evm route] evm request error: {:?}", e);
        Error::Custom(format!("error in `request_cost`: {:?}", e))
    })?;
    // Call with expected number of cycles
    let (result,): (std::result::Result<String, RpcError>,) =
        ic_cdk::api::call::call_with_payment128(state::rpc_addr(), "request", params, cycles)
            .await
            .map_err(|err| Error::IcCallError(err.0, err.1))?;
    #[derive(Serialize, Deserialize, Debug)]
    struct BalanceResult {
        pub id: u32,
        pub jsonrpc: String,
        pub result: String,
    }
    let r = result.map_err(|e| {
        log!(
            WARNING,
            "[evm route]query chainkey address evm balance error: {:?}",
            &e
        );
        Error::Custom(format!(
            "[evm route]query chainkey address evm balance error: {:?}",
            &e
        ))
    })?;
    let r: BalanceResult =
        serde_json::from_str(r.as_str()).map_err(|e| Error::Fatal(e.to_string()))?;
    let r = r.result.strip_prefix("0x").unwrap_or(r.result.as_str());
    let r = u128::from_str_radix(r, 16).map_err(|e| Error::Fatal(e.to_string()))?;
    Ok(U256::from(r))
}

pub async fn checked_get_receipt(hash: &String) -> Result<Option<TransactionReceipt>, Error> {
    let (check_amt, total_amt, rpcs) = read_state(|s| {
        (
            s.minimum_response_count,
            s.total_required_count,
            s.rpc_providers.clone(),
        )
    });
    let mut fut = Vec::with_capacity(total_amt);
    for rpc in rpcs {
        fut.push(get_receipt(&hash, rpc));
    }
    let mut r = futures::future::join_all(fut)
        .await
        .into_iter()
        .filter_map(|s| s.ok())
        .filter(|s| s.is_some())
        .map(|s| s.unwrap())
        .collect::<Vec<TransactionReceipt>>();
    if r.len() < check_amt {
        return Err(Error::Custom(
            "checked length less than required".to_string(),
        ));
    }
    if r.len() == 1 {
        return Ok(Some(r.pop().unwrap()));
    }
    let mut count;
    for i in 0..r.len() - 1 {
        count = 0;
        for x in i + 1..r.len() {
            if r.get(x) == r.get(i) {
                count += 1;
                if count == check_amt {
                    return Ok(r.get(x).cloned());
                }
            }
        }
    }
    return Err(Error::Custom("have no enough check result".to_string()));
}

pub async fn get_receipt(hash: &str, api: RpcApi) -> Result<Option<TransactionReceipt>, Error> {
    let url = api.url.clone();
    const MAX_CYCLES: u128 = 1_100_000_000;
    let body = EvmJsonRpcRequest {
        method: "eth_getTransactionReceipt".to_string(),
        params: vec![hash.to_owned()],
        id: 1,
        jsonrpc: "2.0".to_string(),
    };
    let body = serde_json::to_string(&body).unwrap();
    let request = CanisterHttpRequestArgument {
        url,
        method: HttpMethod::POST,
        body: Some(body.as_bytes().to_vec()),
        max_response_bytes: Some(5000),
        transform: Some(TransformContext {
            function: TransformFunc(candid::Func {
                principal: ic_cdk::api::id(),
                method: "transform".to_string(),
            }),
            context: vec![],
        }),
        headers: vec![HttpHeader {
            name: "Content-Type".to_string(),
            value: "application/json".to_string(),
        }],
    };

    match http_request(request, MAX_CYCLES).await {
        Ok((response,)) => {
            log!(
                INFO,
                "get_receipt result: {}",
                serde_json::to_string(&response).unwrap()
            );
            let status = response.status;
            if status == 200_u32 {
                let body = String::from_utf8(response.body).map_err(|_| {
                    EvmRpcError("Transformed response is not UTF-8 encoded".to_string())
                })?;
                let tx: EvmRpcResponse<TransactionReceipt> =
                    serde_json::from_str(&body).map_err(|_| {
                        EvmRpcError("failed to decode transaction from json".to_string())
                    })?;
                Ok(tx.result)
            } else {
                Err(EvmRpcError("http response not 200".to_string()))
            }
        }
        Err((_, m)) => Err(EvmRpcError(m)),
    }
}

pub async fn call_rpc_with_retry<P: Clone, T, R: Future<Output = Result<T, Error>>>(
    params: P,
    call_rpc: fn(params: P, rpc_api: RpcApi) -> R,
) -> Result<T, Error> {
    let rpcs = rpc_providers();
    let mut rs = Err(Error::RouteNotInitialized);
    if rpcs.is_empty() {
        return rs;
    }
    for i in 0..const_args::RPC_RETRY_TIMES {
        let r = rpcs[i % rpcs.len()].clone();
        log!(INFO, "[evm route]request rpc request times: {}", i + 1);
        let call_res = call_rpc(params.clone(), r).await;
        if call_res.is_ok() {
            rs = call_res;
            break;
        } else {
            let err = call_res.err().unwrap();
            log!(
                WARNING,
                "[evm route]call  rpc error: {}",
                err.clone().to_string()
            );
            rs = Err(err);
        }
        if let Err(Error::Fatal(_)) = rs {
            break;
        }
    }
    match rs {
        Ok(t) => Ok(t),
        Err(e) => {
            log!(CRITICAL, "rpc error after retry {:?}", &e);
            Err(e)
        }
    }
}
