use crate::auth::Permission;
use crate::guard::TaskType;

// use crate::lifecycle::InitArgs;
use crate::state::Seqs;
use crate::state::SolanaRouteState;
use crate::types::{ChainId, ChainState};
use candid::CandidType;
use candid::Principal;

use crate::eddsa::KeyType;
use ic_spl::compute_budget::compute_budget::Priority;

use ic_stable_structures::StableBTreeMap;
use serde::{Deserialize, Serialize};

use std::collections::HashMap;
use std::collections::{BTreeMap, HashSet};

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, Default, PartialEq, Eq)]
pub struct MultiRpcConfig {
    pub rpc_list: Vec<String>,
    pub minimum_response_count: u32,
}

#[derive(Deserialize, Serialize)]
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
    pub enable_debug: bool,
    pub priority: Option<Priority>,
    pub key_type: KeyType,
}

pub fn migrate(pre_state: PreState) -> SolanaRouteState {
    // migrate old struct to new struct

    let new_state = SolanaRouteState {
        chain_id: pre_state.chain_id,
        hub_principal: pre_state.hub_principal,
        seqs: pre_state.seqs,
        fee_token_factor: pre_state.fee_token_factor,
        target_chain_factor: pre_state.target_chain_factor,
        chain_state: pre_state.chain_state,
        schnorr_key_name: pre_state.schnorr_key_name,
        sol_canister: pre_state.sol_canister,
        fee_account: pre_state.fee_account,
        //new fields
        solana_client_cache: None,
        active_tasks: pre_state.active_tasks,
        admin: pre_state.admin,
        caller_perms: pre_state.caller_perms,

        enable_debug: pre_state.enable_debug,
        priority: pre_state.priority,
        key_type: pre_state.key_type,

        //new fields
        providers: vec![],
        proxy: String::default(),
        minimum_response_count: 1,

        tickets_queue: StableBTreeMap::init(crate::memory::get_ticket_queue_memory()),
        tickets_failed_to_hub: StableBTreeMap::init(crate::memory::get_failed_tickets_memory()),
        counterparties: StableBTreeMap::init(crate::memory::get_counterparties_memory()),
        tokens: StableBTreeMap::init(crate::memory::get_tokens_memory()),
        update_token_queue: StableBTreeMap::init(crate::memory::get_update_tokens_v2_memory()),
        token_mint_accounts: StableBTreeMap::init(
            crate::memory::get_token_mint_accounts_v2_memory(),
        ),
        associated_accounts: StableBTreeMap::init(
            crate::memory::get_associated_accounts_v2_memory(),
        ),
        mint_token_requests: StableBTreeMap::init(
            crate::memory::get_mint_token_requests_v2_memory(),
        ),
        gen_ticket_reqs: StableBTreeMap::init(crate::memory::get_gen_ticket_req_memory()),
        seeds: StableBTreeMap::init(crate::memory::get_seeds_memory()),
    };

    new_state
}
