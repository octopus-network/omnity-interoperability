

pub mod audit;
pub mod auth;
pub mod call_error;
pub mod event;
pub mod guard;
pub mod lifecycle;
pub mod memory;
pub mod state;
// pub mod updates;
pub mod handler;

pub const PERIODIC_TASK_INTERVAL: u64 = 60;
pub const BATCH_QUERY_LIMIT: u64 = 20;
pub const ICP_TRANSFER_FEE: u64 = 10_000;
pub const BLOCK_HOLE_ADDRESS: &str = "e3mmv-5qaaa-aaaah-aadma-cai";
