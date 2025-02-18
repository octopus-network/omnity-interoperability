use std::str::FromStr;

use ic_canister_log::log;
use ic_cdk::api::management_canister::http_request::{
    CanisterHttpRequestArgument, HttpHeader, HttpMethod, TransformContext, TransformFunc,
};
use omnity_types::ic_log::ERROR;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::{
    constants::{KB, KB10, KB100, MB}, errors::CustomsError, types::{http_request_with_retry, serialize_hex, wrap_to_customs_error, CanisterHttpRequestArgumentHasher, RpcConfig}
};

use super::{
    header::{BlockHeaderJsonResult, BlockJsonResult},
    transaction::{DogeRpcResponse, RpcTxOut, Transaction, TransactionJsonResult, Txid},
};

// pub const PROXY_URL: &str = "https://doge-idempotent-proxy-219952077564.us-central1.run.app";
// pub const IDEMPOTENCY_KEY: &str = "idempotency-key";
// pub const FORWARD_RPC: &str = "x-forwarded-host";

pub const PROXY_URL: &str = "https://doge-idempotent-proxy-219952077564.us-central1.run.app";
pub const IDEMPOTENCY_KEY: &str = "idempotency-key";
pub const FORWARD_RPC: &str = "x-forwarded-host";

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
        self.proxy_request(&mut request)?;
        let response = http_request_with_retry(request).await?;
        let rpc_result: DogeRpcResponse<R> =
            serde_json::from_slice(&response.body).map_err(|e| {
                CustomsError::RpcError(
                    format!("failed to decode transaction from json, error:{:?}, response: {:?}", e, response)
                )
            })?;
        rpc_result.try_result()
    }

    pub async fn get_block_header_hex(&self, block_hash: &str) -> Result<String, CustomsError> {
        self.call_rpc(
            "getblockheader",
            vec![block_hash.into(), false.into()],
            5 * KB,
        )
        .await
    }

    pub async fn get_block_header(
        &self,
        block_hash: &str,
    ) -> Result<BlockHeaderJsonResult, CustomsError> {
        self.call_rpc("getblockheader", vec![block_hash.into()], 2 * KB)
            .await
    }

    pub async fn get_block_hash(&self, block_height: u64) -> Result<String, CustomsError> {
        self.call_rpc("getblockhash", vec![block_height.into()], 2*KB)
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
        self.proxy_request(&mut request)?;
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

        self.proxy_request(&mut request)?;
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

    fn proxy_request(&self, request: &mut CanisterHttpRequestArgument)->Result<(), CustomsError> {
        
        let parsed_rpc_url = url::Url::parse(&self.url).map_err(
            |e| CustomsError::CustomError(
                format!("failed to parse rpc url: {}, error: {:?}", self.url, e)
            )
        )?;

        let host_str = parsed_rpc_url.host_str().ok_or(
            CustomsError::CustomError(
                format!("failed to get host from rpc url: {}", self.url)
            )
        )?;

        let path = if parsed_rpc_url.path().eq("/") {
            // if no path, it'll failed, so we add a default path
            "/api"
        } else {
            parsed_rpc_url.path()
        };

        request.url = format!("{}{}", PROXY_URL, path);

        let request_hasher: CanisterHttpRequestArgumentHasher = request.clone().into();
        
        let idempotency_key = format!("doge_customs-{}", request_hasher.calculate_hash());

        request.headers.push(HttpHeader {
            name: IDEMPOTENCY_KEY.to_string(),
            value: idempotency_key,
        });
        request.headers.push(HttpHeader {
            name: FORWARD_RPC.to_string(),
            value: host_str.to_string(),
        });

        Ok(())
    }
}

pub async fn get_raw_transaction_by_rpc(
    txid: &str, 
    rpc_config: RpcConfig,
) -> Result<TransactionJsonResult, CustomsError> {
    let doge_rpc = DogeRpc::from(rpc_config);
    doge_rpc.get_raw_transaction(txid).await
}

#[test]
pub fn test_rpc_response() {
    let body_bin = vec![123, 34, 106, 115, 111, 110, 114, 112, 99, 34, 58, 34, 50, 46, 48, 34, 44, 34, 101, 114, 114, 111, 114, 34, 58, 123, 34, 99, 111, 100, 101, 34, 58, 45, 51, 50, 48, 48, 48, 44, 34, 109, 101, 115, 115, 97, 103, 101, 34, 58, 34, 83, 101, 114, 118, 101, 114, 32, 101, 114, 114, 111, 114, 34, 125, 44, 34, 100, 97, 115, 104, 98, 111, 97, 114, 100, 76, 111, 103, 34, 58, 34, 104, 116, 116, 112, 115, 58, 47, 47, 100, 97, 115, 104, 98, 111, 97, 114, 100, 46, 116, 97, 116, 117, 109, 46, 105, 111, 47, 108, 111, 103, 115, 63, 105, 100, 61, 54, 55, 97, 57, 57, 99, 50, 49, 48, 56, 56, 53, 55, 100, 51, 57, 50, 53, 51, 56, 55, 102, 99, 97, 34, 125];
    let body_text = String::from_utf8_lossy(&body_bin).to_string();
    dbg!(&body_text);
    let doge_rpc_res: DogeRpcResponse<String> = serde_json::from_slice(&body_bin).unwrap();
    let res = doge_rpc_res.try_result().unwrap();
    dbg!(&res);
}

#[test]
pub fn test_url_parse() {
    let rpc_url = "https://doge-mainnet.gateway.tatum.io"; 
    let parsed_url = url::Url::parse(rpc_url).unwrap();
    let host_str = parsed_url.host_str().unwrap();
    let path = parsed_url.path();
    dbg!(&host_str);
    dbg!(&path);
}
