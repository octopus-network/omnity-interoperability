use candid::{CandidType, Deserialize};
use serde::Serialize;

#[derive(Serialize, CandidType, Deserialize, Clone, Debug)]
pub struct Destination {
    pub target_chain_id: String,
    pub receiver: String,
}
