use std::str::FromStr;
use sha2::Digest;

use crate::*;

pub fn nat_to_u64(nat: candid::Nat) -> Result<u64> {
    u64::from_str(&nat.0.to_string()).map_err(|_| Errors::NatConversionError(nat.0.to_string()))
}

pub fn nat_to_u128(nat: candid::Nat) -> Result<u128> {
    u128::from_str(&nat.0.to_string()).map_err(|_| Errors::NatConversionError(nat.0.to_string()))
}

pub fn get_chain_time_seconds() -> u64 {
    ic_cdk::api::time() / 1_000_000_000
}

pub fn sha256(input: &[u8]) -> [u8; 32] {
    let mut hasher = sha2::Sha256::new();
    hasher.update(input);
    hasher.finalize().into()
}