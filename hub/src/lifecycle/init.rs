use crate::state::{set_state, HubState};
use candid::{CandidType, Deserialize, Principal};
use serde::Serialize;

use super::upgrade::UpgradeArgs;

#[derive(CandidType, serde::Deserialize, Debug)]
pub enum HubArg {
    Init(InitArgs),
    Upgrade(Option<UpgradeArgs>),
}

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct InitArgs {
    pub admin: Principal,
}

pub fn init(args: InitArgs) {
    let state = HubState::from(args);
    set_state(state);
}
