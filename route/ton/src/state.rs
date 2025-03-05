use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, HashSet};

use candid::{CandidType, Principal};
use ic_stable_structures::StableBTreeMap;
use ic_stable_structures::writer::Writer;
use serde::{Deserialize, Serialize};

use omnity_types::{Chain, Token, TokenId};
use omnity_types::{ChainId, Directive, Seq, Ticket, TicketId};

use crate::InitArgs;
use crate::stable_memory;
use crate::stable_memory::Memory;
use crate::types::*;

thread_local! {
    static STATE: RefCell<Option<TonRouteState>> =  const {RefCell::new(None)};
}

pub const TON_NATIVE_TOKEN: &str = "TON";
pub const TON_CHAIN_ID: &str = "Ton";
impl TonRouteState {
    pub fn init(args: InitArgs) -> anyhow::Result<Self> {
        let ret = TonRouteState {
            admins: args.admins,
            hub_principal: args.hub_principal,
            tokens: Default::default(),
            token_jetton_master_map: Default::default(),
            counterparties: Default::default(),
            pubkey: vec![],
            fee_token_factor: None,
            target_chain_factor: Default::default(),
            next_ticket_seq: 0,
            next_directive_seq: 0,
            next_consume_ticket_seq: 0,
            next_consume_directive_seq: 0,
            handled_ton_event: Default::default(),
            finalized_mint_requests: StableBTreeMap::init(crate::stable_memory::get_finalized_mint_requests_memory()),
            tickets_queue: StableBTreeMap::init(crate::stable_memory::get_to_ton_tickets_memory()),
            directives_queue: StableBTreeMap::init(
                crate::stable_memory::get_to_ton_directives_memory(),
            ),
            pending_tickets_map: StableBTreeMap::init(
                crate::stable_memory::get_pending_ticket_map_memory(),
            ),
            pending_directive_map: StableBTreeMap::init(
                crate::stable_memory::get_pending_directive_map_memory(),
            ),
            is_timer_running: Default::default(),
            pending_events_on_chain: Default::default(),
            last_success_seqno: 0,
            generating_ticketid: Default::default(),
        };
        Ok(ret)
    }

    pub fn pre_upgrade(&self) {
        let mut state_bytes = vec![];
        let _ = ciborium::ser::into_writer(self, &mut state_bytes);
        let len = state_bytes.len() as u32;
        let mut memory = crate::stable_memory::get_upgrade_stash_memory();
        let mut writer = Writer::new(&mut memory, 0);
        writer
            .write(&len.to_le_bytes())
            .expect("failed to save hub state len");
        writer
            .write(&state_bytes)
            .expect("failed to save hub state");
    }

    pub fn post_upgrade() {
        use ic_stable_structures::Memory;
        let memory = stable_memory::get_upgrade_stash_memory();
        // Read the length of the state bytes.
        let mut state_len_bytes = [0; 4];
        memory.read(0, &mut state_len_bytes);
        let state_len = u32::from_le_bytes(state_len_bytes) as usize;
        let mut state_bytes = vec![0; state_len];
        memory.read(4, &mut state_bytes);
        let state: TonRouteState =
            ciborium::de::from_reader(&*state_bytes).expect("failed to decode state");
        replace_state(state);
    }

    pub fn pull_tickets(&self, from: usize, limit: usize) -> Vec<(Seq, Ticket)> {
        self.tickets_queue
            .iter()
            .skip(from)
            .take(limit)
            .map(|(seq, t)| (seq, t.clone()))
            .collect()
    }

    pub fn pull_directives(&self, from: usize, limit: usize) -> Vec<(Seq, Directive)> {
        self.directives_queue
            .iter()
            .skip(from)
            .take(limit)
            .map(|(seq, d)| (seq, d.clone()))
            .collect()
    }
}

#[derive(Deserialize, Serialize)]
pub struct TonRouteState {
    pub admins: Vec<Principal>,
    pub hub_principal: Principal,
    pub tokens: BTreeMap<TokenId, Token>,
    pub token_jetton_master_map: BTreeMap<TokenId, String>,
    pub counterparties: BTreeMap<ChainId, Chain>,
    pub pubkey: Vec<u8>,
    pub fee_token_factor: Option<u128>,
    pub target_chain_factor: BTreeMap<ChainId, u128>,
    pub next_ticket_seq: u64,
    pub next_directive_seq: u64,
    pub next_consume_ticket_seq: u64,
    pub next_consume_directive_seq: u64,
    pub handled_ton_event: BTreeSet<String>,
    #[serde(skip, default = "crate::stable_memory::init_finalized_mint_requests")]
    pub finalized_mint_requests: StableBTreeMap<TicketId, String, Memory>,
    #[serde(skip, default = "crate::stable_memory::init_to_ton_tickets_queue")]
    pub tickets_queue: StableBTreeMap<u64, Ticket, Memory>,
    #[serde(skip, default = "crate::stable_memory::init_to_ton_directives_queue")]
    pub directives_queue: StableBTreeMap<u64, Directive, Memory>,
    #[serde(skip, default = "crate::stable_memory::init_pending_ticket_map")]
    pub pending_tickets_map: StableBTreeMap<Seq, PendingTicketStatus, Memory>,
    #[serde(skip, default = "crate::stable_memory::init_pending_directive_map")]
    pub pending_directive_map: StableBTreeMap<Seq, PendingDirectiveStatus, Memory>,
    #[serde(skip)]
    pub is_timer_running: BTreeMap<String, bool>,
    pub pending_events_on_chain: BTreeMap<String, u64>,
    #[serde(default)]
    pub last_success_seqno: i32,
    #[serde(default)]
    pub generating_ticketid: HashSet<String>,
}

impl From<&TonRouteState> for StateProfile {
    fn from(v: &TonRouteState) -> Self {
        StateProfile {
            admins: v.admins.clone(),
            hub_principal: v.hub_principal,
            omnity_chain_id: TON_CHAIN_ID.to_string(),
            tokens: v.tokens.clone(),
            token_contracts: v.token_jetton_master_map.clone(),
            counterparties: v.counterparties.clone(),
            pubkey: v.pubkey.clone(),
            next_ticket_seq: v.next_ticket_seq,
            next_directive_seq: v.next_directive_seq,
            next_consume_ticket_seq: v.next_consume_ticket_seq,
            next_consume_directive_seq: v.next_consume_directive_seq,
            fee_token_factor: v.fee_token_factor,
            target_chain_factor: v.target_chain_factor.clone(),
            last_success_seqno: v.last_success_seqno,
        }
    }
}

#[derive(Deserialize, Serialize, CandidType)]
pub struct StateProfile {
    pub admins: Vec<Principal>,
    pub hub_principal: Principal,
    pub omnity_chain_id: String,
    pub tokens: BTreeMap<TokenId, Token>,
    pub token_contracts: BTreeMap<TokenId, String>,
    pub counterparties: BTreeMap<ChainId, Chain>,
    pub pubkey: Vec<u8>,
    pub next_ticket_seq: u64,
    pub next_directive_seq: u64,
    pub next_consume_ticket_seq: u64,
    pub next_consume_directive_seq: u64,
    pub fee_token_factor: Option<u128>,
    pub target_chain_factor: BTreeMap<ChainId, u128>,
    pub last_success_seqno: i32,
}

pub fn hub_addr() -> Principal {
    read_state(|s| s.hub_principal)
}

pub fn public_key() -> Vec<u8> {
    read_state(|s| s.pubkey.clone())
}

pub fn bridge_fee(chain_id: &ChainId) -> Option<u64> {
    read_state(|s| {
        s.target_chain_factor
            .get(chain_id)
            .and_then(|target_chain_factor| {
                s.fee_token_factor
                    .map(|fee_token_factor| (target_chain_factor * fee_token_factor) as u64)
            })
    })
}

pub fn mutate_state<F, R>(f: F) -> R
where
    F: FnOnce(&mut TonRouteState) -> R,
{
    STATE.with(|s| f(s.borrow_mut().as_mut().expect("State not initialized!")))
}

pub fn read_state<F, R>(f: F) -> R
where
    F: FnOnce(&TonRouteState) -> R,
{
    STATE.with(|s| f(s.borrow().as_ref().expect("State not initialized!")))
}

/// Replaces the current state.
pub fn replace_state(state: TonRouteState) {
    STATE.with(|s| {
        *s.borrow_mut() = Some(state);
    });
}

pub fn take_state<F, R>(f: F) -> R
where
    F: FnOnce(TonRouteState) -> R,
{
    STATE.with(|s| f(s.take().expect("State not initialized!")))
}
