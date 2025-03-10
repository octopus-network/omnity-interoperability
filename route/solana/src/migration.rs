use crate::auth::Permission;
use crate::guard::TaskType;

use crate::handler::mint_token::MintTokenRequest;
// use crate::lifecycle::InitArgs;
use crate::state::{AccountInfo, AtaKey, MultiRpcConfig, SolanaRouteState, UpdateToken};
use crate::state::{Seqs, TxStatus};
use crate::types::{ChainId, ChainState, TicketId, Token};
use candid::{CandidType, Principal};

use ic_solana::compute_budget::compute_budget::Priority;
use ic_solana::eddsa::KeyType;

use crate::memory::{
    get_associated_accounts_memory, get_mint_token_requests_memory, get_token_mint_accounts_memory,
    get_update_tokens_memory, Memory,
};
use crate::types::TokenId;
use ic_stable_structures::storable::Bound;
use ic_stable_structures::{StableBTreeMap, Storable};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::HashMap;
use std::collections::{BTreeMap, HashSet};

#[derive(CandidType, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PreAccountInfo {
    pub account: String,
    pub retry: u64,
    pub signature: Option<String>,
    pub status: TxStatus,
}
impl Storable for PreAccountInfo {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        let mut bytes = vec![];
        let _ = ciborium::ser::into_writer(self, &mut bytes);
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        let tm = ciborium::de::from_reader(bytes.as_ref()).expect("failed to decode AccountInfo");
        tm
    }

    const BOUND: Bound = Bound::Unbounded;
}

#[derive(CandidType, Clone, Debug, Serialize, Deserialize)]
pub struct PreUpdateToken {
    pub token: Token,
    pub retry: u64,
}

impl Storable for PreUpdateToken {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        let mut bytes = vec![];
        let _ = ciborium::ser::into_writer(self, &mut bytes);
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        let tm = ciborium::de::from_reader(bytes.as_ref()).expect("failed to decode UpdateToken");
        tm
    }

    const BOUND: Bound = Bound::Unbounded;
}

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct PreMintTokenRequest {
    pub ticket_id: TicketId,
    pub associated_account: String,
    pub amount: u64,
    pub token_mint: String,
    pub status: TxStatus,
    pub signature: Option<String>,
    pub retry: u64,
}

impl Storable for PreMintTokenRequest {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        let mut bytes = vec![];
        let _ = ciborium::ser::into_writer(self, &mut bytes);
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        let cm = ciborium::de::from_reader(bytes.as_ref()).expect("failed to decode ChainMeta");
        cm
    }

    const BOUND: Bound = Bound::Unbounded;
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
}

pub fn migrate(pre_state: PreState) -> SolanaRouteState {
    // migrate old struct to new struct
    let old_update_token_queue: StableBTreeMap<TokenId, PreUpdateToken, Memory> =
        StableBTreeMap::init(get_update_tokens_memory());
    let mut new_update_token_queue: StableBTreeMap<TokenId, UpdateToken, Memory> =
        StableBTreeMap::init(crate::memory::get_update_tokens_v2_memory());

    let old_token_mint_accounts: StableBTreeMap<TokenId, PreAccountInfo, Memory> =
        StableBTreeMap::init(get_token_mint_accounts_memory());
    let mut new_token_mint_accounts: StableBTreeMap<TokenId, AccountInfo, Memory> =
        StableBTreeMap::init(crate::memory::get_token_mint_accounts_v2_memory());

    let old_associated_accounts: StableBTreeMap<AtaKey, PreAccountInfo, Memory> =
        StableBTreeMap::init(get_associated_accounts_memory());
    let mut new_associated_accounts: StableBTreeMap<AtaKey, AccountInfo, Memory> =
        StableBTreeMap::init(crate::memory::get_associated_accounts_v2_memory());

    let old_mint_token_requests: StableBTreeMap<TicketId, PreMintTokenRequest, Memory> =
        StableBTreeMap::init(get_mint_token_requests_memory());
    let mut new_mint_token_requests: StableBTreeMap<TicketId, MintTokenRequest, Memory> =
        StableBTreeMap::init(crate::memory::get_mint_token_requests_v2_memory());

    // migrate update_token_queue
    for (k, pre) in old_update_token_queue.iter() {
        let new_obj = UpdateToken {
            token: pre.token,
            retry_4_building: pre.retry,
            retry_4_status: 0,
            signature: None,
            status: TxStatus::Finalized,
        };
        new_update_token_queue.insert(k, new_obj);
    }

    // migrate token_mint_accounts
    for (k, pre) in old_token_mint_accounts.iter() {
        let new_obj = AccountInfo {
            account: pre.account,
            retry_4_building: pre.retry,
            retry_4_status: 0,
            signature: pre.signature,
            status: pre.status,
        };
        new_token_mint_accounts.insert(k, new_obj);
    }

    // migrate associated_accounts
    for (k, pre) in old_associated_accounts.iter() {
        let new_obj = AccountInfo {
            account: pre.account,
            retry_4_building: pre.retry,
            retry_4_status: 0,
            signature: pre.signature,
            status: pre.status,
        };
        new_associated_accounts.insert(k, new_obj);
    }

    // migrate mint_token_requests
    for (k, pre) in old_mint_token_requests.iter() {
        let new_obj = MintTokenRequest {
            ticket_id: pre.ticket_id,
            associated_account: pre.associated_account,
            amount: pre.amount,
            token_mint: pre.token_mint,
            status: pre.status,
            signature: pre.signature,
            retry_4_building: pre.retry,
            retry_4_status: 0,
        };
        new_mint_token_requests.insert(k, new_obj);
    }

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
        active_tasks: pre_state.active_tasks,
        admin: pre_state.admin,
        caller_perms: pre_state.caller_perms,
        multi_rpc_config: pre_state.multi_rpc_config,
        forward: pre_state.forward,
        enable_debug: pre_state.enable_debug,

        priority: Some(Priority::None),
        key_type: KeyType::ChainKey,
        tickets_queue: StableBTreeMap::init(crate::memory::get_ticket_queue_memory()),
        tickets_failed_to_hub: StableBTreeMap::init(crate::memory::get_failed_tickets_memory()),
        counterparties: StableBTreeMap::init(crate::memory::get_counterparties_memory()),
        tokens: StableBTreeMap::init(crate::memory::get_tokens_memory()),
        update_token_queue: new_update_token_queue,
        token_mint_accounts: new_token_mint_accounts,
        associated_accounts: new_associated_accounts,
        mint_token_requests: new_mint_token_requests,
        gen_ticket_reqs: StableBTreeMap::init(crate::memory::get_gen_ticket_req_memory()),
        seeds: StableBTreeMap::init(crate::memory::get_seeds_memory()),
        solana_client_cache: None,
    };

    new_state
}
