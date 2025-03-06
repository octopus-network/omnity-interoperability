use crate::{
    guard::TaskType,
    lifecycle::init::InitArgs,
    memory::{init_finalized_gen_tickets, init_finalized_requests, VMem},
    types::omnity_types::{
        Chain, ChainId, ChainState, TicketId, ToggleState, Token, TokenId, TxAction,
    },
    updates::generate_ticket::GenerateTicketArgs,
};
use candid::Principal;
use ic_solana::types::Pubkey;
use ic_stable_structures::{storable::Bound, StableBTreeMap, Storable};
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    cell::RefCell,
    collections::{BTreeMap, HashSet},
    str::FromStr,
};

thread_local! {
    static STATE: RefCell<Option<CustomsState>> = RefCell::default();
}

#[derive(candid::CandidType, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReleaseTokenStatus {
    Unknown,
    Pending,
    Submitted(String),
    Finalized(String),
}

#[derive(candid::CandidType, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GenTicketStatus {
    Unknown,
    Finalized(GenerateTicketArgs),
}

#[derive(candid::CandidType, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseTokenReq {
    pub ticket_id: TicketId,
    pub action: TxAction,
    pub token_id: TokenId,
    pub amount: u64,
    pub address: Pubkey,
    pub received_at: u64,
    pub last_sent_at: u64,
    pub try_cnt: u32,
    pub status: ReleaseTokenStatus,
}

impl Storable for ReleaseTokenReq {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        let mut bytes = vec![];
        let _ = ciborium::ser::into_writer(self, &mut bytes);
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        let cm =
            ciborium::de::from_reader(bytes.as_ref()).expect("failed to decode ReleaseTokenReq");
        cm
    }

    const BOUND: Bound = Bound::Unbounded;
}

#[derive(Deserialize, Serialize)]
pub struct CustomsState {
    pub chain_id: String,
    pub hub_principal: Principal,
    pub schnorr_key_name: String,
    pub sol_canister: Principal,
    pub port_program_id: Pubkey,
    pub chain_state: ChainState,
    pub counterparties: BTreeMap<ChainId, Chain>,
    pub tokens: BTreeMap<TokenId, Token>,
    pub release_token_requests: BTreeMap<TicketId, ReleaseTokenReq>,
    pub rpc_list: Vec<String>,
    pub proxy_rpc: String,
    pub min_response_count: u32,
    pub enable_debug: bool,

    // Next index of query tickets from hub
    pub next_ticket_seq: u64,

    // Next index of query directives from hub
    pub next_directive_seq: u64,

    pub active_tasks: HashSet<TaskType>,

    #[serde(skip, default = "crate::memory::init_finalized_requests")]
    pub finalized_requests: StableBTreeMap<TicketId, ReleaseTokenReq, VMem>,
    #[serde(skip, default = "crate::memory::init_finalized_gen_tickets")]
    pub finalized_gen_tickets: StableBTreeMap<TicketId, GenerateTicketArgs, VMem>,
}

impl From<InitArgs> for CustomsState {
    fn from(args: InitArgs) -> Self {
        Self {
            chain_id: args.chain_id,
            hub_principal: args.hub_principal,
            schnorr_key_name: args.schnorr_key_name,
            sol_canister: args.sol_canister,
            port_program_id: Pubkey::from_str(&args.port_program_id).unwrap(),
            chain_state: args.chain_state,
            counterparties: Default::default(),
            tokens: Default::default(),
            release_token_requests: Default::default(),
            rpc_list: args.rpc_list,
            proxy_rpc: args.proxy_rpc,
            min_response_count: args.min_response_count,
            enable_debug: false,
            next_ticket_seq: 0,
            next_directive_seq: 0,
            active_tasks: Default::default(),
            finalized_requests: init_finalized_requests(),
            finalized_gen_tickets: init_finalized_gen_tickets(),
        }
    }
}

impl CustomsState {
    pub fn validate_config(&self) {
        if self.schnorr_key_name.is_empty() {
            ic_cdk::trap("schnorr_key_name is not set");
        }
        if self.rpc_list.is_empty() {
            ic_cdk::trap("rpc_list is empty");
        }
        if self.min_response_count == 0 || self.min_response_count as usize > self.rpc_list.len() {
            ic_cdk::trap("invalid min_response_count");
        }
    }

    pub fn toggle_chain_state(&mut self, toggle: ToggleState) {
        if toggle.chain_id == self.chain_id {
            self.chain_state = toggle.action.into();
        } else if let Some(chain) = self.counterparties.get_mut(&toggle.chain_id) {
            chain.chain_state = toggle.action.into();
        }
    }
}

pub fn take_state<F, R>(f: F) -> R
where
    F: FnOnce(CustomsState) -> R,
{
    STATE.with(|s| f(s.take().expect("State not initialized!")))
}

pub fn mutate_state<F, R>(f: F) -> R
where
    F: FnOnce(&mut CustomsState) -> R,
{
    STATE.with(|s| f(s.borrow_mut().as_mut().expect("State not initialized!")))
}

pub fn read_state<F, R>(f: F) -> R
where
    F: FnOnce(&CustomsState) -> R,
{
    STATE.with(|s| f(s.borrow().as_ref().expect("State not initialized!")))
}

pub fn replace_state(state: CustomsState) {
    STATE.with(|s| {
        *s.borrow_mut() = Some(state);
    });
}
