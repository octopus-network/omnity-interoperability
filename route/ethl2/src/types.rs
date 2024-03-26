use candid::{CandidType, Deserialize};
use cketh_common::eth_rpc::LogEntry;
use serde::Serialize;

use crate::redeem::TokenBurned;

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

impl Ticket {
    pub fn from_event(log_entry: &LogEntry, token_burned: TokenBurned) -> Self {
        let ticket = Ticket {
            ticket_id: log_entry.transaction_hash.clone().unwrap().to_string()
                + log_entry.log_index.clone().unwrap().to_string().as_str(),
            created_time: log_entry.block_number.clone().unwrap().as_f64() as u64,
            src_chain: "eth".to_string(),
            dst_chain: "ic".to_string(),
            action: Action::Redeem,
            token: token_burned.tokenId.to_string(),
            amount: token_burned.amount.to_string(),
            sender: token_burned.receiver,
            receiver: token_burned.receiver,
            memo: None,
        };
        ticket
    }
}

#[derive(CandidType, Deserialize, Serialize, Default, Clone, Debug)]
pub enum Action {
    #[default]
    Transfer,
    Redeem,
}
