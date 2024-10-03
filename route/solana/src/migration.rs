use crate::auth::Permission;
use crate::guard::TaskType;
use crate::lifecycle::InitArgs;
use crate::state::Seqs;
use crate::state::{MultiRpcConfig, SolanaRouteState};
use crate::types::{ChainId, ChainState};
use candid::{Deserialize, Principal};

use serde::Serialize;
use std::collections::HashMap;
use std::collections::{BTreeMap, HashSet};

#[derive(Serialize, Deserialize)]
pub struct PreState {
    pub chain_id: String,
    pub hub_principal: Principal,
    pub seqs: Seqs,
    pub fee_token_factor: Option<u128>,
    pub target_chain_factor: BTreeMap<ChainId, u128>,
    pub chain_state: ChainState,
    pub schnorr_key_name: String,
    pub sol_canister: Principal,
    pub fee_account: String,
    // Locks preventing concurrent execution timer tasks
    pub active_tasks: HashSet<TaskType>,
    pub admin: Principal,
    pub caller_perms: HashMap<String, Permission>,
    pub multi_rpc_config: MultiRpcConfig,
    pub forward: Option<String>,
}

pub fn migrate(pre_state: PreState) -> SolanaRouteState {
    let init_args = InitArgs {
        admin: pre_state.admin,
        chain_id: pre_state.chain_id,
        hub_principal: pre_state.hub_principal,
        chain_state: pre_state.chain_state,
        schnorr_key_name: Some(pre_state.schnorr_key_name),
        sol_canister: pre_state.sol_canister,
        fee_account: Some(pre_state.fee_account),
        // multi_rpc_config: pre_state.multi_rpc_config,
        // forward: pre_state.forward,
    };
    let mut new_state = SolanaRouteState::from(init_args);

    new_state.fee_token_factor = pre_state.fee_token_factor;
    new_state.target_chain_factor = pre_state.target_chain_factor;
    new_state.caller_perms = pre_state.caller_perms;
    new_state.multi_rpc_config = pre_state.multi_rpc_config;
    new_state.forward = pre_state.forward;
    new_state.seqs = pre_state.seqs;
    new_state.enable_debug = false;

    new_state
}
