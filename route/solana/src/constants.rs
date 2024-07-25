// pub const NODES_IN_STANDARD_SUBNET: u32 = 13;

use std::time::Duration;

pub const NODES_IN_FIDUCIARY_SUBNET: u32 = 28;

// https://github.com/domwoe/schnorr_canister/blob/502a263c01902a1154ef354aefa161795a669de1/src/lib.rs#L54
pub const SCHNORR_KEY_NAME: &str = "test_key_1";

// 1 lamport = 0.000_000_001 sol.
// 1 sol =1_000_000_000
pub const FEE_TOKEN: &str = "SOL";

pub const QUERY_DERECTIVE_INTERVAL: Duration = Duration::from_secs(60);
pub const QUERY_TICKET_INTERVAL: Duration = Duration::from_secs(5);
pub const HANDLE_TICKET_INTERVAL: Duration = Duration::from_secs(10);
