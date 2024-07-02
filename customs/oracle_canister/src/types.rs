use candid::CandidType;
use omnity_types::rune_id::RuneId;
use serde::{Deserialize, Serialize};

pub use bitcoin_customs::{
    state::{GenTicketRequestV2, RunesBalance},
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
