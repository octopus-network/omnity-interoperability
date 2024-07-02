use candid::CandidType;
use serde::{Deserialize, Serialize};

pub use bitcoin_customs::{
    queries::GetGenTicketReqsArgs,
    state::{GenTicketRequestV2, RuneId, RunesBalance},
    updates::update_runes_balance::UpdateRunesBalanceArgs,
};
pub use ic_btc_interface::Txid;

#[derive(Copy, Eq, PartialEq, Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct Balance {
    pub id: RuneId,
    pub balance: u128,
}

impl Balance {
    pub fn into_runes_balance(self, vout: u32) -> RunesBalance {
        RunesBalance {
            rune_id: self.id,
            vout,
            amount: self.balance,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, CandidType)]
pub enum OrdError {
    Params(String),
    Overflow,
    BlockVerification(u32),
    Index(MintError),
    Rpc(RpcError),
}

#[derive(Debug, CandidType, Deserialize, Serialize)]
pub enum MintError {
    Cap(u128),
    End(u64),
    Start(u64),
    Unmintable,
}

#[derive(Debug, CandidType, Deserialize, Serialize)]
pub enum RpcError {
    Io(String, String, String),
    Decode(String, String, String),
    Endpoint(String, String, String),
}
