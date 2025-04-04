pub mod audit;
pub mod eth_common;
pub mod evm_scan;
pub mod guard;
pub mod hub_to_route;
pub mod lightclient;
pub mod route_to_evm;
pub mod service;
pub mod state;
mod state_provider;
pub mod updates;

pub mod const_args {
    pub const MAX_SCAN_BLOCKS: u64 = 200;
    pub const EVM_ADDR_BYTES_LEN: usize = 20;
    pub const PERIODIC_TASK_INTERVAL: u64 = 5;
    pub const LIGHTCLIENT_CHECK_INVERVAL: u64 = 120;
    pub const BATCH_QUERY_LIMIT: u64 = 20;
    pub const SEND_EVM_TASK_INTERVAL: u64 = 20;
    pub const SEND_EVM_TASK_NAME: &str = "SEND_EVM";
    pub const LIGHTCLIENT_CHECK_TASK_NAME: &str = "LIGHTCLIENT_CHECK";
    pub const SCAN_EVM_TASK_INTERVAL: u64 = 10;
    pub const SCAN_EVM_TASK_NAME: &str = "SCAN_EVM";
    pub const EIP1559_TX_ID: u8 = 2;
    pub const EVM_FINALIZED_CONFIRM_HEIGHT: u64 = 12;
    pub const DEFAULT_EVM_TX_FEE: u32 = 200_000u32;
    pub const ADD_TOKEN_EVM_TX_FEE: u32 = 1_100_000u32;
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
