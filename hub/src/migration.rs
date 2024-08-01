use crate::auth::Permission;
use crate::lifecycle::init::InitArgs;

use crate::state::HubState;

use candid::Principal;

use crate::self_help::AddRunesTokenReq;
use omnity_types::{ChainId, Directive, Seq, SeqKey};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap};
#[derive(Deserialize, Serialize, Debug)]
pub struct PreHubState {
    // memory variable
    pub directive_seq: HashMap<String, Seq>,
    pub ticket_seq: HashMap<String, Seq>,
    pub admin: Principal,
    pub caller_chain_map: HashMap<String, ChainId>,
    pub caller_perms: HashMap<String, Permission>,
    pub last_resubmit_ticket_time: u64,
    pub add_runes_token_requests: BTreeMap<String, AddRunesTokenReq>,
    pub runes_oracles: BTreeSet<Principal>,
    pub dire_map: BTreeMap<SeqKey, Directive>,
    pub ticket_map: BTreeMap<SeqKey, String>,
}

// migrate pre state to current state
pub fn migrate(pre_state: PreHubState) -> HubState {
    let init_args = InitArgs {
        admin: pre_state.admin,
    };
    let mut cur_state = HubState::from(init_args);
    cur_state.directive_seq = pre_state.directive_seq;
    cur_state.ticket_seq = pre_state.ticket_seq;
    cur_state.caller_chain_map = pre_state.caller_chain_map;
    cur_state.caller_perms = pre_state.caller_perms;
    cur_state.last_resubmit_ticket_time = pre_state.last_resubmit_ticket_time;
    cur_state.add_runes_token_requests = pre_state.add_runes_token_requests;
    cur_state.runes_oracles = pre_state.runes_oracles;
    cur_state.dire_map = pre_state.dire_map;
    cur_state.ticket_map = pre_state.ticket_map;
    cur_state
}
