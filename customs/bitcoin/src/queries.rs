use crate::state::RunesId;
use candid::CandidType;
use ic_btc_interface::Txid;
use omnity_types::TicketId;
use serde::Deserialize;

#[derive(CandidType, Deserialize)]
pub struct ReleaseTokenStatusRequest {
    pub ticket_id: TicketId,
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
    pub bitcoin_fee: u64,
}
