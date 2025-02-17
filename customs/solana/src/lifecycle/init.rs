use candid::{CandidType, Deserialize, Principal};
use serde::Serialize;

use crate::{
    state::{replace_state, CustomsState},
    types::omnity_types::ChainState,
};

use super::upgrade::UpgradeArgs;

#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum CustomArg {
    Init(InitArgs),
    Upgrade(Option<UpgradeArgs>),
}

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct InitArgs {
    pub chain_id: String,
    pub hub_principal: Principal,
    pub schnorr_key_name: String,
    pub sol_canister: Principal,
    pub chain_state: ChainState,
    pub rpc_list: Vec<String>,
    pub min_response_count: u32,
}

pub fn init(args: InitArgs) {
    let state = CustomsState::from(args);
    state.validate_config();
    replace_state(state);
}
