use serde::{Deserialize, Serialize};
// use std::fmt;
/// Wrapper for rpc return types of methods that provide responses both with and without context.
/// The Main purpose of this is to fix methods that lack context information in their return type,
/// without breaking backwards compatibility.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OptionalContext<T> {
    Context(Response<T>),
    NoContext(T),
}

impl<T> OptionalContext<T> {
    pub fn parse_value(self) -> T {
        match self {
            Self::Context(response) => response.value,
            Self::NoContext(value) => value,
        }
    }
}

/// The unit of time given to a leader for encoding a block.
///
/// It is some number of _ticks_ long.
pub type Slot = u64;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RpcResponseContext {
    pub slot: Slot,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Response<T> {
    pub context: RpcResponseContext,
    pub value: T,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RpcBlockCommitment<T> {
    pub commitment: Option<T>,
    pub total_stake: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RpcBlockhash {
    pub blockhash: String,
    pub last_valid_block_height: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RpcAccountBalance {
    pub address: String,
    pub lamports: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RpcSupply {
    pub total: u64,
    pub circulating: u64,
    pub non_circulating: u64,
    pub non_circulating_accounts: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum StakeActivationState {
    Activating,
    Active,
    Deactivating,
    Inactive,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RpcStakeActivation {
    pub state: StakeActivationState,
    pub active: u64,
    pub inactive: u64,
}

