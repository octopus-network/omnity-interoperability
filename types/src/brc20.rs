use candid::CandidType;
use serde_derive::{Deserialize, Serialize};

#[derive(CandidType, Serialize, Deserialize, Default, Debug)]
pub struct Brc20TransferEvent {
    pub amout: u128,
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
