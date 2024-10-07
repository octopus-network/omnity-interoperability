use std::ops::Mul;
use std::str::FromStr;
use bigdecimal::{BigDecimal, ToPrimitive};
use candid::CandidType;
use serde_derive::{Deserialize, Serialize};

#[derive(CandidType, Serialize, Deserialize, Default, Debug)]
pub struct Brc20TransferEvent {
    pub amout: String,
    pub from: String,
    pub to: String,
    pub valid: bool,

}

#[derive(CandidType, Serialize, Deserialize, Default, Debug)]
pub struct QueryBrc20TransferArgs {
    pub tx_id: String,
    pub ticker: String,
    pub to_addr: String,
    pub amt: String,
    pub decimals: u8,
}

impl QueryBrc20TransferArgs {
    pub fn get_amt_satoshi(&self) ->  u128 {
        BigDecimal::from_str(&self.amt).unwrap().mul(10u128.pow(self.decimals as u32)).to_u128().unwrap()
    }
}

#[test]
pub fn test() {
    let a = "100.22231";
    let r = BigDecimal::from_str(a).unwrap().mul(10u128.pow(18u32)).to_u128().unwrap();
    println!("{r}");
}