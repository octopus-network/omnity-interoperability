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
}

pub fn init(args: InitArgs) {
    let state = RouteState::from(args);
    replace_state(state);
}