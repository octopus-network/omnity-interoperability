use candid_derive::CandidType;
use ethers_core::types::{Eip1559TransactionRequest, TransactionRequest};
use serde_derive::{Deserialize, Serialize};

#[derive(
    CandidType, Serialize, Deserialize, Default, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord,
)]
pub enum EvmTxType {
    Legacy,
    #[default]
    Eip1559,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum EvmTxRequest {
    Legacy(TransactionRequest),
    Eip1559(Eip1559TransactionRequest),
}

#[derive(Deserialize, Clone, Debug, Default)]
pub struct EvmRpcResponse<T> {
    pub id: u32,
    pub jsonrpc: String,
    pub result: Option<T>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct EvmJsonRpcRequest {
    pub method: String,
    pub params: Vec<String>,
    pub id: u64,
    pub jsonrpc: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct JsonRpcResponse<T> {
    pub jsonrpc: String,
    pub result: T,
    pub id: u32,
}
