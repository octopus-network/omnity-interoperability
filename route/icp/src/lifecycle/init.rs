use crate::state::{replace_state, RouteState};
use candid::{CandidType, Deserialize, Principal};
use serde::Serialize;

#[derive(CandidType, serde::Deserialize)]
pub enum RouteArg {
    Init(InitArgs),
    Upgrade(),
}

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct InitArgs {
    pub chain_id: String,
    pub hub_principal: Principal,
    pub ledger_principal: Principal,
}

pub fn init(args: InitArgs) {
    let state = RouteState::from(args);
    state.validate_config();
    replace_state(state);
}
