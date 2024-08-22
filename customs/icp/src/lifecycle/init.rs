use crate::state::{replace_state, CustomsState};
use candid::{CandidType, Deserialize, Principal};
use serde::Serialize;

#[derive(CandidType, serde::Deserialize)]
pub enum CustomArg {
    Init(InitArgs),
}

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct InitArgs {
    pub chain_id: String,
    pub hub_principal: Principal,
    pub ckbtc_ledger_principal: Principal,
}

pub fn init(args: InitArgs) {
    let state = CustomsState::from(args);
    replace_state(state);
}
