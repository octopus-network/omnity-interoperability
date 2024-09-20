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
mod ic_log;
pub mod stable_memory;
pub mod state;
pub mod types;
pub mod updates;
mod upgrade;

#[derive(Error, Debug, Clone)]
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
    #[error("generate rpc request data error: {0}")]
    RequestDataError(String),
    #[error("custom error: {0}")]
    Custom(String),
    #[error("Temporay error")]
    Temporary,
    #[error("Fatal error: {0}")]
    Fatal(String),
}

pub mod const_args {
    pub const MAX_SCAN_BLOCKS: u64 = 200;
    pub const EVM_ADDR_BYTES_LEN: usize = 20;
    pub const PERIODIC_TASK_INTERVAL: u64 = 5;
    pub const BATCH_QUERY_LIMIT: u64 = 20;
    pub const FETCH_HUB_TICKET_INTERVAL: u64 = 5;
    pub const FETCH_HUB_DIRECTIVE_INTERVAL: u64 = 60;
    pub const FETCH_HUB_TICKET_NAME: &str = "FETCH_HUB_TICKET";
    pub const FETCH_HUB_DIRECTIVE_NAME: &str = "FETCH_HUB_DIRECTIVE";
    pub const SEND_EVM_TASK_INTERVAL: u64 = 20;
    pub const SEND_EVM_TASK_NAME: &str = "SEND_EVM";
    pub const SCAN_EVM_TASK_INTERVAL: u64 = 10;
    pub const SCAN_EVM_TASK_NAME: &str = "SCAN_EVM";
    pub const EIP1559_TX_ID: u8 = 2;
    pub const EVM_FINALIZED_CONFIRM_HEIGHT: u64 = 15;
    pub const DEFAULT_EVM_TX_FEE: u32 = 200_000u32;
    pub const ADD_TOKEN_EVM_TX_FEE: u32 = 3_000_000u32;
    pub const SCAN_EVM_CYCLES: u128 = 10_000_000_000;
    pub const BROADCAST_TX_CYCLES: u128 = 3_000_000_000;
    pub const GET_ACCOUNT_NONCE_CYCLES: u128 = 1_000_000_000;
    pub const PENDING_TICKET_TIMEOUT_SECONDS: u64 = 600; //10 minutes
    pub const MONITOR_PRINCIPAL: &str =
        "3edln-ixjzp-oflch-uwhc7-xu5yt-s7t72-rp3rp-25j7a-tu254-h4w3x-jqe";
    pub const RPC_RETRY_TIMES: usize = 4;
}

pub fn get_time_secs() -> u64 {
    ic_cdk::api::time() / 1_000_000_000
}
