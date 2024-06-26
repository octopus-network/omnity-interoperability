use std::collections::BTreeMap;

pub use ic_btc_interface::{Address, OutPoint, Utxo};
use serde::{Deserialize, Serialize};

#[derive(candid::CandidType, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PushUtxosToAddress {
    pub utxos: BTreeMap<Address, Vec<Utxo>>,
}
