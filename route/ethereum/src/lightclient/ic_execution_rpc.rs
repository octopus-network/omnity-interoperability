use crate::lightclient::http::post;
use crate::lightclient::rpc_types::errors::RpcError;
use crate::lightclient::rpc_types::receipt::TransactionReceipt;
use const_hex::ToHexExt;
use serde::{Deserialize, Serialize};
use tree_hash::fixed_bytes::B256;

#[derive(Debug, Clone)]
pub struct IcExecutionRpc {
    rpc: String,
}

impl IcExecutionRpc {
    pub fn new(rpcx: &str) -> eyre::Result<Self>
    where
        Self: Sized,
    {
        Ok(Self {
            rpc: rpcx.to_string(),
        })
    }
    pub(crate) async fn get_block_receipts(
        &self,
        block_hash: B256,
    ) -> eyre::Result<Vec<TransactionReceipt>> {
        let real_hex = block_hash.0.as_slice().encode_hex_with_prefix();
        let params = r#"{"id":1, "jsonrpc":"2.0", "method": "eth_getBlockReceipts", "params":["block_hash"]}"#;
        let params = params.replace("block_hash", &real_hex);
        post_request(
            "eth_getBlockReceipts",
            params,
            self.rpc.clone(),
            1500000,
            100_000_000_000,
        )
        .await
    }
}

async fn post_request<T>(
    name: impl AsRef<str>,
    body: String,
    url: impl AsRef<str>,
    max_response_size: u64,
    max_cost_cycles: u128,
) -> eyre::Result<T>
where
    T: serde::de::DeserializeOwned,
{
    let name = name.as_ref();
    let url = url.as_ref();
    let resp = post(
        url,
        body.as_bytes().to_vec(),
        max_response_size,
        max_cost_cycles,
    )
    .await
    .map_err(|e| RpcError::new(name, e))?;
    if resp.status != 200 {
        let e = format!("http response with status {}", resp.status);
        Err(RpcError::new(name, e))?;
    }
    let value: EvmRpcResponse<T> =
        serde_json::from_slice(&resp.body).map_err(|e| RpcError::new(name, e))?;
    if value.result.is_some() {
        Ok(value.result.unwrap())
    } else {
        Err(RpcError::new(name, "result is null".to_string()))?
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct EvmJsonRpcRequest {
    pub method: String,
    pub params: Vec<String>,
    pub id: u64,
    pub jsonrpc: String,
}

#[derive(Deserialize, Clone, Debug, Default)]
pub struct EvmRpcResponse<T> {
    pub result: Option<T>,
}
