// pub const NODES_IN_STANDARD_SUBNET: u32 = 13;

use std::time::Duration;

pub const NODES_IN_FIDUCIARY_SUBNET: u32 = 28;

// https://github.com/domwoe/schnorr_canister/blob/502a263c01902a1154ef354aefa161795a669de1/src/lib.rs#L54
pub const SCHNORR_KEY_NAME: &str = "key_1";
// https://internetcomputer.org/docs/current/references/t-sigs-how-it-works/#fees-for-the-t-schnorr-production-key
// pub const EDDSA_SIGN_COST: u128 = 26_153_846_153;
// pub const EDDSA_SIGN_COST: u128 = 26_200_000_000;

pub const FEE_ACCOUNT: &str = "3gghk7mHWtFsJcg6EZGK7sbHj3qW6ExUdZLs9q8GRjia";

// 1 lamport = 0.000_000_001 sol.
// 1 sol =1_000_000_000
pub const FEE_TOKEN: &str = "SOL";

// redeem fee = gas fee + service fee
// the service fee,there is 3 solutions
// s2e: free; e2s: 2$; e2e: 1$
// TODO: get SOL price from oracle ,and convert into 2$ valued lamports(SOL price/2$ * 10^9)
// SERVICE_FEE:u64= 0.015*10^9

pub const DIRECTIVE_LIMIT_SIZE: u64 = 20;
pub const TICKET_LIMIT_SIZE: u64 = 20;
pub const COUNTER_SIZE: u64 = 5;
pub const QUERY_DERECTIVE_INTERVAL: Duration = Duration::from_secs(30);
pub const CREATE_MINT_INTERVAL: Duration = Duration::from_secs(15);
pub const UPDATE_TOKEN_INTERVAL: Duration = Duration::from_secs(30);
pub const CREATE_ATA_INTERVAL: Duration = Duration::from_secs(15);
pub const QUERY_TICKET_INTERVAL: Duration = Duration::from_secs(10);
pub const MINT_TOKEN_INTERVAL: Duration = Duration::from_secs(15);
pub const RETRY_LIMIT_SIZE: u64 = 10;
