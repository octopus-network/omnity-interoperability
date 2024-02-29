use candid::{CandidType, Deserialize};
use serde::Serialize;

pub type Chain = String;
pub type Timestamp = u64;
pub type TokenId = String;
pub type Account = String;

#[derive(CandidType, Deserialize, Serialize, Default, Clone, Debug)]
pub struct Ticket {
    pub ticket_id: String,
    pub created_time: Timestamp,
    pub src_chain: Chain,
    pub dst_chain: Chain,
    pub action: Action,
    pub token: TokenId,
    pub amount: String,
    pub sender: Account,
    pub receiver: Account,
    pub memo: Option<Vec<u8>>,
}

#[derive(CandidType, Deserialize, Serialize, Default, Clone, Debug)]
pub enum Action {
    #[default]
    Transfer,
    Redeem,
}
