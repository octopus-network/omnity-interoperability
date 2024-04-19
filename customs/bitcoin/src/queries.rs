use crate::state::RuneId;
use candid::CandidType;
use ic_btc_interface::Txid;
use serde::Deserialize;

#[derive(CandidType, Deserialize)]
pub struct EstimateFeeArgs {
    pub rune_id: RuneId,
    pub amount: Option<u128>,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct RedeemFee {
    pub bitcoin_fee: u64,
}

#[derive(CandidType, Deserialize)]
pub struct GetGenTicketReqsArgs {
    pub start_txid: Option<Txid>,
    pub max_count: u64,
}
