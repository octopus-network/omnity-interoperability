use candid::CandidType;
use serde::{Deserialize, Serialize};

pub use ic_btc_interface::{Txid, Utxo};

#[derive(candid::CandidType, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenTicketRequestV2 {
    pub address: String,
    pub target_chain_id: String,
    pub receiver: String,
    pub token_id: String,
    pub rune_id: RuneId,
    pub amount: u128,
    pub txid: Txid,
    pub new_utxos: Vec<Utxo>,
    pub received_at: u64,
}

#[derive(CandidType, Clone, Debug, PartialEq, Eq, Copy, Serialize, Deserialize)]
pub struct RuneId {
    pub block: u64,
    pub tx: u32,
}

#[derive(CandidType, Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct RunesBalance {
    pub rune_id: RuneId,
    pub vout: u32,
    pub amount: u128,
}

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct UpdateRunesBalanceArgs {
    pub txid: Txid,
    pub balances: Vec<RunesBalance>,
}

#[derive(Copy, Eq, PartialEq, Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct Balance {
    pub id: RuneId,
    pub balance: u128,
}

impl From<(u32, Balance)> for RunesBalance {
    fn from((vout, balance): (u32, Balance)) -> Self {
        RunesBalance {
            rune_id: balance.id,
            vout,
            amount: balance.balance,
        }
    }
}
