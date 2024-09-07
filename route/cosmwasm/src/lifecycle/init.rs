use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};

use crate::{memory::set_route_state, state::RouteState};

#[derive(CandidType, serde::Deserialize)]
pub enum RouteArg {
    Init(InitArgs),
}

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct InitArgs {
    pub cosmwasm_port_contract_address: String,
    pub chain_id: String,
    pub cw_rpc_url: String,
    pub cw_rest_url: String,
    pub hub_principal: Principal,
}

pub fn init(args: InitArgs) {
    let state = RouteState::from(args);
    set_route_state(state);
}
