use ic_cdk::api::call::RejectionCode;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Hub error: {0}")]
    HubError(String),
    #[error("Ton rpc canister error: {0}")]
    TonRpcCanisterError(String),
    #[error("Ton rpc error: {0}")]
    TonRpcError(String),
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
    pub const TON_ADDR_BYTES_LEN: usize = 20;
    pub const PERIODIC_TASK_INTERVAL: u64 = 5;
    pub const BATCH_QUERY_LIMIT: u64 = 20;
    pub const FETCH_HUB_TICKET_INTERVAL: u64 = 5;
    pub const FETCH_HUB_DIRECTIVE_INTERVAL: u64 = 60;
    pub const FETCH_HUB_TICKET_NAME: &str = "FETCH_HUB_TICKET";
    pub const FETCH_HUB_DIRECTIVE_NAME: &str = "FETCH_HUB_DIRECTIVE";
    pub const SEND_TON_TASK_INTERVAL: u64 = 20;
    pub const SEND_TON_TASK_NAME: &str = "SEND_TON";
    pub const SCAN_TON_TASK_INTERVAL: u64 = 20;
    pub const SCAN_TON_TASK_NAME: &str = "SCAN_TON";
    pub const EIP1559_TX_ID: u8 = 2;
    pub const TON_FINALIZED_CONFIRM_HEIGHT: u64 = 15;
    pub const DEFAULT_TON_TX_FEE: u32 = 200_000u32;
    pub const ADD_TOKEN_TON_TX_FEE: u32 = 3_000_000u32;
    pub const SCAN_TON_CYCLES: u128 = 10_000_000_000;
    pub const BROADCAST_TX_CYCLES: u128 = 3_000_000_000;
    pub const GET_ACCOUNT_NONCE_CYCLES: u128 = 1_000_000_000;
    pub const PENDING_TICKET_TIMEOUT_SECONDS: u64 = 600; //10 minutes
    pub const MONITOR_PRINCIPAL: &str =
        "3edln-ixjzp-oflch-uwhc7-xu5yt-s7t72-rp3rp-25j7a-tu254-h4w3x-jqe";
}

pub fn get_time_secs() -> u64 {
    ic_cdk::api::time() / 1_000_000_000
}
