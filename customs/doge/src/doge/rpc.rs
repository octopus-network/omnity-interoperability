use std::str::FromStr;

use ic_cdk::api::management_canister::http_request::{CanisterHttpRequestArgument, HttpHeader, HttpMethod, TransformContext, TransformFunc};
use omnity_types::ic_log::ERROR;
use serde_json::json;
use ic_canister_log::log;

use crate::{constants::{KB, KB100}, errors::CustomsError, types::{deserialize_hex, http_request_with_retry, serialize_hex, wrap_to_customs_error}};

use super::transaction::{DogeRpcResponse, RpcTxOut, Transaction, Txid};

pub const PROXY_URL: &str = "https://common-rpc-proxy-398338012986.us-central1.run.app";
pub const IDEMPOTENCY_KEY: &str = "X-Idempotency";
pub const FORWARD_RPC: &str = "X-Forward-Host";

pub struct DogeRpc {
    pub url: String,
    pub api_key: Option<String>,
}

impl DogeRpc {
    // const HTTP_OUT_CALL_CYCLE: u128 = 60_000_000_000;
    // const RPC_RETRY_TIMES: u32 = 3;
    pub async fn get_raw_transaction(
        &self, 
        txid: &str
    ) -> Result<Transaction, CustomsError> {
        let mut headers = vec![
            HttpHeader {
                name: "Content-Type".to_string(),
                value: "application/json".to_string(),
            },
        ];
        if self.api_key.is_some() {
            headers.push(HttpHeader {
                name: "x-api-key".to_string(),
                value: self.api_key.clone().unwrap(),
            });
        }
        let request = CanisterHttpRequestArgument {
            url: self.url.clone(),
            method: HttpMethod::POST,
            body: Some(json!({
                "jsonrpc": "2.0",
                "method": "getrawtransaction",
                "params": [txid],
                "id": 1
            }).to_string().into_bytes()),
            max_response_bytes: Some(KB100),
            transform: Some(TransformContext {
                function: TransformFunc(candid::Func {
                    principal: ic_cdk::api::id(),
                    method: "transform".to_string(),
                }),
                context: vec![],
            }),
            headers
        }; 
        let response = http_request_with_retry(request).await?;
        let raw_tx: DogeRpcResponse<String>  = serde_json::from_slice(&response.body).map_err(|e| {
            log!(ERROR, "json error {:?}", e);
            CustomsError::RpcError(
                "failed to decode transaction from json".to_string(),
            )
        })?;
        let result = raw_tx.unwrap_result()?;
        
        let tx: Transaction = deserialize_hex(&result).map_err(wrap_to_customs_error)?;
        Ok(tx)
    }

    pub async fn get_tx_out(
        &self,
        txid: &str,
    ) -> Result<RpcTxOut, CustomsError> {
        let mut headers = vec![
            HttpHeader {
                name: "Content-Type".to_string(),
                value: "application/json".to_string(),
            },
            
        ];
        if self.api_key.is_some() {
            headers.push(HttpHeader {
                name: "x-api-key".to_string(),
                value: self.api_key.clone().unwrap(),
            });
        }
        let request = CanisterHttpRequestArgument {
            url: self.url.clone(),
            method: HttpMethod::POST,
            body: Some(json!({
                "jsonrpc": "2.0",
                "method": "gettxout",
                "params": [txid, 0],
                "id": 1
            }).to_string().into_bytes()),
            max_response_bytes: Some(KB100),
            transform: Some(TransformContext {
                function: TransformFunc(candid::Func {
                    principal: ic_cdk::api::id(),
                    method: "transform".to_string(),
                }),
                context: vec![],
            }),
            headers
        }; 
        let response = http_request_with_retry(request).await?;
        let tx_out_response: DogeRpcResponse<RpcTxOut> = serde_json::from_slice(&response.body).map_err(|e| {
            log!(ERROR, "json error {:?}", e);
            CustomsError::RpcError(
                "failed to decode transaction from json".to_string(),
            )
        })?;

        let result = tx_out_response.unwrap_result()?;
    
        Ok(result)
    }

    pub async fn send_transaction(
        &self,
        tx: &Transaction
    )-> Result<Txid, CustomsError>{
        let tx_hex = serialize_hex(tx);
        let mut headers = vec![
            HttpHeader {
                name: "Content-Type".to_string(),
                value: "application/json".to_string(),
            },
        ];
        if self.api_key.is_some() {
            headers.push(HttpHeader {
                name: "x-api-key".to_string(),
                value: self.api_key.clone().unwrap(),
            });
        }
        let mut request = CanisterHttpRequestArgument {
            url: self.url.clone(),
            method: HttpMethod::POST,
            body: Some(json!({
                "jsonrpc": "2.0",
                "method": "sendrawtransaction",
                "params": [tx_hex],
                "id": 1
            }).to_string().into_bytes()),
            max_response_bytes: Some(KB),
            transform: Some(TransformContext {
                function: TransformFunc(candid::Func {
                    principal: ic_cdk::api::id(),
                    method: "transform".to_string(),
                }),
                context: vec![],
            }),
            headers
        }; 

        self.proxy_request(&mut request);
        let response = http_request_with_retry(request.clone()).await?;
        // log!(INFO, "send transaction, request: {:?},  response: {:?}",request, response);
        let rpc_response: DogeRpcResponse<String> = serde_json::from_slice(&response.body).map_err(|e| {
            log!(ERROR, "json error {:?}", e);
            CustomsError::RpcError(
                "failed to desc result from json".to_string(),
            )
        })?;

        let txid_str = rpc_response.unwrap_result()?;

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
 

