use candid::CandidType;
use serde::Deserialize;

#[derive(Deserialize, Debug, CandidType, Clone, Eq, PartialEq)]
pub struct Transaction {
    pub vout: Vec<TxOut>,
}

#[derive(Deserialize, Debug, CandidType, Clone, PartialEq, Eq)]
pub struct TxOut {
    pub scriptpubkey_address: Option<String>,
    pub value: u64,
}
