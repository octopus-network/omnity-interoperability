#![allow(unused)]
#![allow(unreachable_code)]

use crate::auth::Permission;

use crate::constants::DEFAULT_GAS_BUDGET;
use crate::guard::TaskType;

use crate::handler::gen_ticket::GenerateTicketReq;
use crate::handler::mint_token::MintTokenRequest;
use crate::ic_sui::ck_eddsa::KeyType;
use crate::ic_sui::sui_providers::Provider;
// use crate::lifecycle::InitArgs;
use crate::config::{MultiRpcConfig, Seqs, SuiPortAction, SuiRouteConfig};
use crate::state::{SuiRouteState, UpdateTokenStatus, UpdateType};
use crate::state::{SuiToken, TxStatus};
use crate::types::{Chain, ChainId, ChainState, Ticket, TicketId, Token, TokenId};
use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};
use tabled::settings::peaker;

use crate::memory::{get_mint_token_requests_memory, Memory};

use ic_stable_structures::storable::Bound;
use ic_stable_structures::{StableBTreeMap, Storable};

use std::borrow::Cow;
use std::collections::HashMap;
use std::collections::{BTreeMap, HashSet};
#[derive(Deserialize, Serialize)]
pub struct PreConfig {
    pub chain_id: String,
    pub hub_principal: Principal,
    pub seqs: Seqs,
    pub fee_token_factor: Option<u128>,
    pub target_chain_factor: BTreeMap<ChainId, u128>,
    pub chain_state: ChainState,
    pub schnorr_key_name: String,
    pub rpc_provider: Provider,
    pub nodes_in_subnet: u32,
    pub fee_account: String,
    pub gas_budget: u64,
    // Locks preventing concurrent execution timer tasks
    pub active_tasks: HashSet<TaskType>,
    pub admin: Principal,
    pub caller_perms: HashMap<String, Permission>,
    pub multi_rpc_config: MultiRpcConfig,
    pub forward: Option<String>,
    pub enable_debug: bool,
    pub key_type: KeyType,
    pub sui_port_action: SuiPortAction,
}

#[derive(Deserialize, Serialize)]
pub struct PreState {
    #[serde(skip, default = "crate::memory::init_ticket_queue")]
    pub tickets_queue: StableBTreeMap<u64, Ticket, Memory>,
    #[serde(skip, default = "crate::memory::init_failed_tickets")]
    pub tickets_failed_to_hub: StableBTreeMap<String, Ticket, Memory>,
    #[serde(skip, default = "crate::memory::init_counterparties")]
    pub counterparties: StableBTreeMap<ChainId, Chain, Memory>,
    #[serde(skip, default = "crate::memory::init_tokens")]
    pub tokens: StableBTreeMap<TokenId, Token, Memory>,
    #[serde(skip, default = "crate::memory::init_sui_tokens")]
    pub sui_tokens: StableBTreeMap<TokenId, SuiToken, Memory>,
    #[serde(skip, default = "crate::memory::init_update_tokens")]
    pub update_token_queue: StableBTreeMap<UpdateType, UpdateTokenStatus, Memory>,
    #[serde(skip, default = "crate::memory::init_mint_token_requests")]
    pub mint_token_requests: StableBTreeMap<TicketId, MintTokenRequest, Memory>,
    #[serde(skip, default = "crate::memory::init_gen_ticket_reqs")]
    pub gen_ticket_reqs: StableBTreeMap<TicketId, GenerateTicketReq, Memory>,
    #[serde(skip, default = "crate::memory::init_seed")]
    pub seeds: StableBTreeMap<String, [u8; 64], Memory>,
}

pub fn migrate_config(pre_config: PreConfig) -> SuiRouteConfig {
    todo!()
}

pub fn migrate_state(pre_state: PreConfig) -> SuiRouteState {
    todo!()
}
