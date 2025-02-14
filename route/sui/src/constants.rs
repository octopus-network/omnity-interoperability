// pub const NODES_IN_STANDARD_SUBNET: u32 = 13;

use std::time::Duration;

pub const NODES_IN_FIDUCIARY_SUBNET: u32 = 28;

// https://github.com/domwoe/schnorr_canister/blob/502a263c01902a1154ef354aefa161795a669de1/src/lib.rs#L54
pub const SCHNORR_KEY_NAME: &str = "key_1";
// https://internetcomputer.org/docs/current/references/t-sigs-how-it-works/#fees-for-the-t-schnorr-production-key
// pub const EDDSA_SIGN_COST: u128 = 26_153_846_153;
// pub const EDDSA_SIGN_COST: u128 = 26_200_000_000;

// pub const FEE_ACCOUNT: &str = "0xc8d919cef59bf118454313315950f8a10ddb49f799fcceac7479721891051e45";
pub const FEE_TOKEN: &str = "SUI";

pub const COIN_PKG_ID: &str = "0x2";
pub const COIN_MODULE: &str = "coin";
pub const SUI_COIN: &str = "0x2::sui::SUI";
pub const DEFAULT_GAS_BUDGET: u64 = 5_000_000;
//funcs
pub const MINT_WITH_TICKET_FUNC: &str = "mint_with_ticket";
pub const MINT_FUNC: &str = "mint";
pub const BURN_FUNC: &str = "burn";
pub const UPDATE_NAME_FUNC: &str = "update_name";
pub const UPDATE_SYMBOL_FUNC: &str = "update_symbol";
pub const UPDATE_ICON_FUNC: &str = "update_icon_url";
pub const UPDATE_DESC_FUNC: &str = "update_description";
// 1  MIST = 0.000_000_001 SUI.
// 1 SUI =1_000_000_000 MIST

// redeem fee = gas fee + service fee
// the service fee,there is 3 solutions
// s2e: free; e2s: 2$; e2e: 1$

pub const DIRECTIVE_LIMIT_SIZE: u64 = 20;
pub const TICKET_LIMIT_SIZE: u64 = 20;
pub const TAKE_SIZE: u64 = 1;
pub const QUERY_DERECTIVE_INTERVAL: Duration = Duration::from_secs(30);
// pub const CREATE_MINT_INTERVAL: Duration = Duration::from_secs(50);
pub const UPDATE_TOKEN_INTERVAL: Duration = Duration::from_secs(30);
pub const QUERY_TICKET_INTERVAL: Duration = Duration::from_secs(10);
pub const MINT_TOKEN_INTERVAL: Duration = Duration::from_secs(20);
pub const CLEAR_INTERVAL: Duration = Duration::from_secs(30);
// pub const RETRY_4_BUILDING: u64 = 10;
pub const RETRY_NUM: u64 = 5;
