use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};

use crate::state::{replace_state, RouteState};

#[derive(CandidType, serde::Deserialize)]
pub enum RouteArg {
    Init(InitArgs),
    // Upgrade(Option<UpgradeArgs>),
}

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct InitArgs {
    pub schnorr_canister_principal: Principal,
    pub cosmoswasm_port_contract_address: String,
    pub chain_id: String,
    pub cw_url: String,
    pub hub_principal: Principal,
}

pub fn init(args: InitArgs) {
    let state = RouteState::from(args);
    replace_state(state);
}
