use candid::CandidType;
use ic_btc_interface::Txid;
use omnity_types::TicketId;
use serde::Deserialize;

#[derive(CandidType, Deserialize)]
pub struct ReleaseTokenStatusArgs {
    pub ticket_id: TicketId,
}

#[derive(CandidType, Deserialize)]
pub struct GenTicketStatusArgs {
    pub txid: Txid,
}

#[derive(CandidType, Deserialize)]
pub struct EstimateFeeArg {
    pub rune_id: String,
    pub amount: Option<u128>,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct RedeemFee {
    pub bitcoin_fee: u64,
}
