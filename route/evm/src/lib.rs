use ic_cdk::api::call::RejectionCode;
use thiserror::Error;

pub mod audit;
pub mod call_error;
pub mod contract_types;
pub mod contracts;
pub mod eth_common;
pub mod evm_scan;
pub mod guard;
pub mod hub;
pub mod hub_to_route;
pub mod route_to_evm;
pub mod service;
mod stable_log;
pub mod stable_memory;
pub mod state;
pub mod types;
pub mod updates;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Hub error: {0}")]
    HubError(String),
    #[error("Evm rpc canister error: {0}")]
    EvmRpcCanisterError(String),
    #[error("Evm rpc error: {0}")]
    EvmRpcError(String),
    #[error("Chain key error: {0}")]
    ChainKeyError(String),
    #[error("Parse event error: {0}")]
    ParseEventError(String),
    #[error("Route not initialized")]
    RouteNotInitialized,
    #[error("IC call error: {0:?}, {1}")]
    IcCallError(RejectionCode, String),
    #[error(transparent)]
    Custom(#[from] anyhow::Error),
}

pub mod const_args {
    pub const MAX_SCAN_BLOCKS: u64 = 200;
    pub const EVM_ADDR_BYTES_LEN: usize = 20;
    pub const PERIODIC_TASK_INTERVAL: u64 = 5;
    pub const BATCH_QUERY_LIMIT: u64 = 20;
    pub const FETCH_HUB_TASK_INTERVAL: u64 = 10;
    pub const FETCH_HUB_TASK_NAME: &str = "FETCH_HUB";
    pub const SEND_EVM_TASK_INTERVAL: u64 = 20;
    pub const SEND_EVM_TASK_NAME: &str = "SEND_EVM";
    pub const SCAN_EVM_TASK_INTERVAL: u64 = 30;
    pub const SCAN_EVM_TASK_NAME: &str = "SCAN_EVM";
    pub const EIP1559_TX_ID: u8 = 2;
    pub const EVM_FINALIZED_CONFIRM_HEIGHT: u64 = 12;
    pub const DEFAULT_EVM_TX_FEE: u32 = 200_000u32;
    pub const ADD_TOKEN_EVM_TX_FEE: u32 = 3_000_000u32;
    pub const SCAN_EVM_CYCLES: u128 = 3_000_000_000;
    pub const BROADCAST_TX_CYCLES: u128 = 3_000_000_000;
    pub const GET_ACCOUNT_NONCE_CYCLES: u128 = 1_000_000_000;
}