use candid::{CandidType, Deserialize, Principal};
use ic_canister_log::log;
use ic_solana::ic_log::DEBUG;

use crate::auth::Permission;
use crate::guard::TaskType;

use crate::lifecycle::InitArgs;
use crate::state::{MultiRpcConfig, SolanaRouteState};
use crate::types::{ChainId, ChainState};

use serde::Serialize;
use std::collections::HashMap;
use std::collections::{BTreeMap, HashSet};

#[derive(CandidType, Serialize, Deserialize, Debug, Hash, Copy, Clone, PartialEq, Eq)]
pub enum PreTaskType {
    GetDirectives,
    GetTickets,
    CreateMint,
    CreateAssoicatedAccount,
    UpdateToken,
    MintToken,
}

#[derive(Serialize, Deserialize)]
pub struct PreState {
    pub chain_id: String,
    pub hub_principal: Principal,
    // Next index of query tickets from hub
    pub next_ticket_seq: u64,
    pub next_consume_ticket_seq: u64,
    // Next index of query directives from hub
    pub next_directive_seq: u64,
    pub fee_token_factor: Option<u128>,
    pub target_chain_factor: BTreeMap<ChainId, u128>,
    pub chain_state: ChainState,
    pub schnorr_key_name: String,
    pub sol_canister: Principal,
    pub fee_account: String,
    // Locks preventing concurrent execution timer tasks
    pub active_tasks: HashSet<PreTaskType>,
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
        multi_rpc_config: pre_state.multi_rpc_config,
        // forward: pre_state.forward,
    };
    let mut new_state = SolanaRouteState::from(init_args);

    new_state.next_ticket_seq = pre_state.next_ticket_seq;
    new_state.next_consume_ticket_seq = pre_state.next_consume_ticket_seq;
    new_state.next_directive_seq = pre_state.next_directive_seq;
    new_state.fee_token_factor = pre_state.fee_token_factor;
    new_state.target_chain_factor = pre_state.target_chain_factor;
    new_state.caller_perms = pre_state.caller_perms;
    new_state.forward = pre_state.forward;

    log!(DEBUG, "migrate active_tasks ...");
    // new_state.active_tasks = pre_state.active_tasks;
    for t in pre_state.active_tasks {
        match t {
            PreTaskType::GetDirectives => {
                new_state.active_tasks.insert(TaskType::GetDirectives);
            }
            PreTaskType::GetTickets => {
                new_state.active_tasks.insert(TaskType::GetTickets);
            }
            PreTaskType::CreateMint => {
                new_state.active_tasks.insert(TaskType::CreateMint);
            }
            PreTaskType::CreateAssoicatedAccount => {
                new_state.active_tasks.insert(TaskType::CreateATA);
            }
            PreTaskType::UpdateToken => {
                new_state.active_tasks.insert(TaskType::UpdateToken);
            }
            PreTaskType::MintToken => {
                new_state.active_tasks.insert(TaskType::MintToken);
            }
        }
    }

    new_state
}
