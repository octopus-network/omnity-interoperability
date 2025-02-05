use std::str::FromStr;

use ic_canister_log::log;
use ic_cdk::api::management_canister::http_request::{
    CanisterHttpRequestArgument, HttpHeader, HttpMethod, TransformContext, TransformFunc,
};
use omnity_types::ic_log::ERROR;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::{
    constants::{KB, KB10, KB100, MB},
    errors::CustomsError,
    types::{http_request_with_retry, serialize_hex, wrap_to_customs_error, RpcConfig},
};

use super::{
    header::{BlockHeaderJsonResult, BlockJsonResult},
    transaction::{DogeRpcResponse, RpcTxOut, Transaction, TransactionJsonResult, Txid},
};

pub const PROXY_URL: &str = "https://common-rpc-proxy-398338012986.us-central1.run.app";
pub const IDEMPOTENCY_KEY: &str = "X-Idempotency";
pub const FORWARD_RPC: &str = "X-Forward-Host";

#[derive(Clone, Debug)]
pub struct DogeRpc {
    pub url: String,
    pub api_key: Option<String>,
}

impl DogeRpc {
    async fn call_rpc<R>(
        &self,
        method: &str,
        params: Vec<Value>,
        max_response_bytes: u64,
    ) -> Result<R, CustomsError>
    where
        R: for<'de> Deserialize<'de>,
    {
        let mut headers = vec![HttpHeader {
            name: "Content-Type".to_string(),
            value: "application/json".to_string(),
        }];
        if self.api_key.is_some() {
            headers.push(HttpHeader {
                name: "x-api-key".to_string(),
                value: self.api_key.clone().unwrap(),
            });
        }
        let mut request = CanisterHttpRequestArgument {
            url: self.url.clone(),
            method: HttpMethod::POST,
            body: Some(
                json!({
                    "jsonrpc": "2.0",
                    "method": method,
                    "params": params,
                    "id": 1
                })
                .to_string()
                .into_bytes(),
            ),
            max_response_bytes: Some(max_response_bytes),
            transform: Some(TransformContext {
                function: TransformFunc(candid::Func {
                    principal: ic_cdk::api::id(),
                    method: "transform".to_string(),
                }),
                context: vec![],
            }),
            headers,
        };
        self.proxy_request(&mut request);
        let response = http_request_with_retry(request).await?;
        let rpc_result: DogeRpcResponse<R> =
            serde_json::from_slice(&response.body).map_err(|e| {
                log!(ERROR, "json error {:?}", e);
                CustomsError::RpcError("failed to decode transaction from json".to_string())
            })?;
        rpc_result.try_result()
    }

    pub async fn get_block_header_hex(&self, block_hash: &str) -> Result<String, CustomsError> {
        self.call_rpc(
            "getblockheader",
            vec![block_hash.into(), false.into()],
            KB10,
        )
        .await
    }

    pub async fn get_block_header(
        &self,
        block_hash: &str,
    ) -> Result<BlockHeaderJsonResult, CustomsError> {
        self.call_rpc("getblockheader", vec![block_hash.into()], KB)
            .await
    }

    pub async fn get_block_hash(&self, block_height: u64) -> Result<String, CustomsError> {
        self.call_rpc("getblockhash", vec![block_height.into()], KB)
            .await
    }

    pub async fn get_block(&self, block_hash: &str) -> Result<BlockJsonResult, CustomsError> {
        self.call_rpc("getblock", vec![block_hash.into()], MB).await
    }

    pub async fn get_raw_transaction(
        &self,
        txid: &str,
    ) -> Result<TransactionJsonResult, CustomsError> {
        self.call_rpc("getrawtransaction", vec![txid.into(), 1.into()], KB10)
            .await
        // let mut headers = vec![
        //     HttpHeader {
        //         name: "Content-Type".to_string(),
        //         value: "application/json".to_string(),
        //     },
        // ];
        // if self.api_key.is_some() {
        //     headers.push(HttpHeader {
        //         name: "x-api-key".to_string(),
        //         value: self.api_key.clone().unwrap(),
        //     });
        // }
        // let mut request = CanisterHttpRequestArgument {
        //     url: self.url.clone(),
        //     method: HttpMethod::POST,
        //     body: Some(json!({
        //         "jsonrpc": "2.0",
        //         "method": "getrawtransaction",
        //         "params": [txid],
        //         "id": 1
        //     }).to_string().into_bytes()),
        //     max_response_bytes: Some(KB100),
        //     transform: Some(TransformContext {
        //         function: TransformFunc(candid::Func {
        //             principal: ic_cdk::api::id(),
        //             method: "transform".to_string(),
        //         }),
        //         context: vec![],
        //     }),
        //     headers
        // };
        // self.proxy_request(&mut request);
        // let response = http_request_with_retry(request).await?;
        // let raw_tx: DogeRpcResponse<TransactionJsonResult>  = serde_json::from_slice(&response.body).map_err(|e| {
        //     log!(ERROR, "json error {:?}", e);
        //     CustomsError::RpcError(
        //         "failed to decode transaction from json".to_string(),
        //     )
        // })?;
        // let result = raw_tx.try_result()?;
        // Ok(result)
    }

    pub async fn get_tx_out(&self, txid: &str) -> Result<RpcTxOut, CustomsError> {
        let mut headers = vec![HttpHeader {
            name: "Content-Type".to_string(),
            value: "application/json".to_string(),
        }];
        if self.api_key.is_some() {
            headers.push(HttpHeader {
                name: "x-api-key".to_string(),
                value: self.api_key.clone().unwrap(),
            });
        }
        let mut request = CanisterHttpRequestArgument {
            url: self.url.clone(),
            method: HttpMethod::POST,
            body: Some(
                json!({
                    "jsonrpc": "2.0",
                    "method": "gettxout",
                    "params": [txid, 0],
                    "id": 1
                })
                .to_string()
                .into_bytes(),
            ),
            max_response_bytes: Some(KB100),
            transform: Some(TransformContext {
                function: TransformFunc(candid::Func {
                    principal: ic_cdk::api::id(),
                    method: "transform".to_string(),
                }),
                context: vec![],
            }),
            headers,
        };
        self.proxy_request(&mut request);
        let response = http_request_with_retry(request).await?;
        let tx_out_response: DogeRpcResponse<RpcTxOut> = serde_json::from_slice(&response.body)
            .map_err(|e| {
                log!(ERROR, "json error {:?}", e);
                CustomsError::RpcError("failed to decode transaction from json".to_string())
            })?;

        let result = tx_out_response.try_result()?;

        Ok(result)
    }

    pub async fn send_transaction(&self, tx: &Transaction) -> Result<Txid, CustomsError> {
        let tx_hex = serialize_hex(tx);
        let mut headers = vec![HttpHeader {
            name: "Content-Type".to_string(),
            value: "application/json".to_string(),
        }];
        if self.api_key.is_some() {
            headers.push(HttpHeader {
                name: "x-api-key".to_string(),
                value: self.api_key.clone().unwrap(),
            });
        }
        let mut request = CanisterHttpRequestArgument {
            url: self.url.clone(),
            method: HttpMethod::POST,
            body: Some(
                json!({
                    "jsonrpc": "2.0",
                    "method": "sendrawtransaction",
                    "params": [tx_hex],
                    "id": 1
                })
                .to_string()
                .into_bytes(),
            ),
            max_response_bytes: Some(KB),
            transform: Some(TransformContext {
                function: TransformFunc(candid::Func {
                    principal: ic_cdk::api::id(),
                    method: "transform".to_string(),
                }),
                context: vec![],
            }),
            headers,
        };

        self.proxy_request(&mut request);
        let response = http_request_with_retry(request.clone()).await?;
        let rpc_response: DogeRpcResponse<String> = serde_json::from_slice(&response.body)
            .map_err(|e| {
                log!(ERROR, "json error {:?}", e);
                CustomsError::RpcError("failed to desc result from json".to_string())
            })?;

        let txid_str = rpc_response.try_result()?;

        let txid = Txid::from_str(&txid_str).map_err(wrap_to_customs_error)?;
        Ok(txid)
    }

    fn proxy_request(&self, request: &mut CanisterHttpRequestArgument) {
        request.url = PROXY_URL.to_string();
        let idempotency_key = format!("doge_customs-{}", ic_cdk::api::time());
        request.headers.push(HttpHeader {
            name: IDEMPOTENCY_KEY.to_string(),
            value: idempotency_key,
        });
        request.headers.push(HttpHeader {
            name: FORWARD_RPC.to_string(),
            value: self.url.to_string(),
        });
    }
}

pub async fn get_raw_transaction_by_rpc(
    txid: &str,
    rpc_config: RpcConfig,
) -> Result<TransactionJsonResult, CustomsError> {
    let doge_rpc = DogeRpc::from(rpc_config);
    doge_rpc.get_raw_transaction(txid).await
}
