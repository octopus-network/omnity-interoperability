pub mod audit;
pub mod eth_common;
pub mod evm_scan;
pub mod guard;
pub mod hub_to_route;
mod log_converter;
pub mod route_to_evm;
pub mod service;
pub mod state;
mod state_provider;

pub mod const_args {
    pub const MAX_SCAN_BLOCKS: u64 = 200;
    pub const EVM_ADDR_BYTES_LEN: usize = 20;
    pub const PERIODIC_TASK_INTERVAL: u64 = 5;
    pub const BATCH_QUERY_LIMIT: u64 = 20;
    pub const SEND_EVM_TASK_INTERVAL: u64 = 5;
    pub const SEND_EVM_TASK_NAME: &str = "SEND_EVM";
    pub const SCAN_EVM_TASK_INTERVAL: u64 = 10;
    pub const SCAN_EVM_TASK_NAME: &str = "SCAN_EVM";
    pub const EVM_FINALIZED_CONFIRM_HEIGHT: u64 = 10;
    pub const DEFAULT_EVM_TX_FEE: u32 = 200_000u32;
    pub const ADD_TOKEN_EVM_TX_FEE: u32 = 3_000_000u32;
    pub const PENDING_TICKET_TIMEOUT_SECONDS: u64 = 600; //10 minutes
    pub const MONITOR_PRINCIPAL: &str =
        "3edln-ixjzp-oflch-uwhc7-xu5yt-s7t72-rp3rp-25j7a-tu254-h4w3x-jqe";
}

pub fn get_time_secs() -> u64 {
    ic_cdk::api::time() / 1_000_000_000
}
