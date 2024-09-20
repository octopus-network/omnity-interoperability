use std::str::FromStr;

use crate::*;
use subtle_encoding::bech32;

// https://github.com/tendermint/spec/blob/master/spec/core/encoding.md
pub fn account_id_to_address(account_id: &str) -> Result<(String, Vec<u8>)> {
    bech32::decode(account_id)
        .map_err(|err| Errors::AccountIdParseError(account_id.to_string(), err.to_string()))
}

pub fn nat_to_u64(nat: candid::Nat) -> u64 {
    u64::from_str(&nat.0.to_string()).unwrap()
}

pub fn nat_to_u128(nat: candid::Nat) -> u128 {
    u128::from_str(&nat.0.to_string()).unwrap()
}

pub fn get_chain_time_seconds() -> u64 {
    ic_cdk::api::time() / 1_000_000_000
}