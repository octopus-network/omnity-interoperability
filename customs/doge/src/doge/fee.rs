#![allow(unused)]
pub const DOGE_AMOUNT: u64 = 100_000_000;
pub const CENT_AMOUNT: u64 = 1_000_000;
// https://github.com/dogecoin/dogecoin/blob/master/doc/fee-recommendation.md
// 0.01 DOGE per kilobyte transaction fee
// 0.01 DOGE dust limit (discard threshold)
// 0.001 DOGE replace-by-fee increments
pub const MIN_FEE: u64 = 1_000_000;
pub const MIN_FEE_RATE: u64 = 1_000; // units per vByte
pub const DUST_LIMIT: u64 = 1_000_000;

pub fn fee_by_size(bytes: u64, fee_rate: Option<u64>) -> u64 {
    let fee_rate = fee_rate.unwrap_or(MIN_FEE_RATE).max(MIN_FEE_RATE);
    (bytes * fee_rate).max(MIN_FEE)
}
