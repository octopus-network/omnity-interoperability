use serde::Deserialize;
use thiserror::Error;

use crate::lightclient::rpc_types::block_types::BlockTag;

#[derive(Debug, Error)]
#[error("block not available: {block}")]
pub struct BlockNotFoundError {
    block: BlockTag,
}

impl BlockNotFoundError {
    pub fn new(block: BlockTag) -> Self {
        Self { block }
    }
}

#[derive(Debug, Error)]
#[error("slot not found: {slot:?}")]
pub struct SlotNotFoundError {
    slot: String,
}

impl SlotNotFoundError {
    pub fn new(slot: String) -> Self {
        Self { slot }
    }
}

#[derive(Debug, Error)]
#[error("rpc error on method: {method}, message: {error}")]
pub struct RpcError<E: ToString> {
    method: String,
    error: E,
}

impl<E: ToString> RpcError<E> {
    pub fn new(method: &str, err: E) -> Self {
        Self {
            method: method.to_string(),
            error: err,
        }
    }
}

#[derive(Debug, Error)]
pub enum HttpError {
    #[error("http error: status: {0}, body: {1}")]
    Http(u16, String),

    #[error("an error occured: {0}")]
    Other(String),

    #[cfg(target_arch = "wasm32")]
    #[error("canister call error: rejection code: {0:?}, message: {1}")]
    CanisterCall(ic_cdk::api::call::RejectionCode, String),
}

#[cfg(target_arch = "wasm32")]
impl From<(ic_cdk::api::call::RejectionCode, String)> for HttpError {
    fn from(value: (ic_cdk::api::call::RejectionCode, String)) -> Self {
        HttpError::CanisterCall(value.0, value.1)
    }
}

/// A JSON-RPC 2.0 error
#[derive(Clone, Debug, Deserialize, Error)]
#[error("rpc error on method: {code}, message: {message}, data: {data:?}")]
pub struct JsonRpcError {
    /// The error code
    code: i64,
    /// The error message
    message: String,
    /// Additional data
    data: Option<serde_json::Value>,
}
