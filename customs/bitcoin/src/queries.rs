use candid::CandidType;
use ic_btc_interface::Txid;
use serde::Deserialize;

use crate::state::{ReleaseId, RunesId};

#[derive(CandidType, Deserialize)]
pub struct ReleaseTokenStatusRequest {
    pub release_id: ReleaseId,
}

#[derive(CandidType, Deserialize)]
pub struct GenTicketStatusRequest {
    pub tx_id: Txid,
}

#[derive(CandidType, Deserialize)]
pub struct EstimateFeeArg {
    pub runes_id: RunesId,
    pub amount: Option<u128>,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct WithdrawalFee {
    pub minter_fee: u64,
    pub bitcoin_fee: u64,
}
