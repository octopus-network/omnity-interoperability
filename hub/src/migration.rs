use crate::lifecycle::init::InitArgs;
use crate::memory::{self, Memory};

use crate::state::HubState;
use crate::types::{Amount, ChainMeta, ChainTokenFactor, Subscribers, TokenKey, TokenMeta};
use candid::Principal;

use ic_stable_structures::StableBTreeMap;

use omnity_types::{ChainId, Directive, Seq, SeqKey, Ticket, TicketId, TokenId, Topic};
use serde::{Deserialize, Serialize};

use std::collections::HashMap;

#[derive(Deserialize, Serialize)]
pub struct PreHubState {
    #[serde(skip, default = "memory::init_chain")]
    pub chains: StableBTreeMap<ChainId, ChainMeta, Memory>,
    #[serde(skip, default = "memory::init_token")]
    pub tokens: StableBTreeMap<TokenId, TokenMeta, Memory>,
    #[serde(skip, default = "memory::init_chain_factor")]
    pub target_chain_factors: StableBTreeMap<ChainId, u128, Memory>,
    #[serde(skip, default = "memory::init_token_factor")]
    pub fee_token_factors: StableBTreeMap<TokenKey, ChainTokenFactor, Memory>,
    #[serde(skip, default = "memory::init_directive")]
    pub directives: StableBTreeMap<String, Directive, Memory>,
    #[serde(skip, default = "memory::init_dire_queue")]
    pub dire_queue: StableBTreeMap<SeqKey, Directive, Memory>,
    #[serde(skip, default = "memory::init_subs")]
    pub topic_subscribers: StableBTreeMap<Topic, Subscribers, Memory>,
    #[serde(skip, default = "memory::init_ticket_queue")]
    pub ticket_queue: StableBTreeMap<SeqKey, Ticket, Memory>,
    #[serde(skip, default = "memory::init_token_position")]
    pub token_position: StableBTreeMap<TokenKey, Amount, Memory>,
    #[serde(skip, default = "memory::init_ledger")]
    pub cross_ledger: StableBTreeMap<TicketId, Ticket, Memory>,
    pub directive_seq: HashMap<String, Seq>,
    pub ticket_seq: HashMap<String, Seq>,
    pub admin: Principal,
    pub authorized_caller: HashMap<String, ChainId>,
    pub last_resubmit_ticket_time: u64,
}

// migrate pre state to current state
pub fn migrate(pre_state: PreHubState) -> HubState {
    let init_args = InitArgs {
        admin: pre_state.admin,
    };
    let mut cur_state = HubState::from(init_args);
    cur_state.caller_chain_map = pre_state.authorized_caller;
    cur_state.last_resubmit_ticket_time = pre_state.last_resubmit_ticket_time;
    cur_state.directive_seq = pre_state.directive_seq;
    cur_state.ticket_seq = pre_state.ticket_seq;
    cur_state
}