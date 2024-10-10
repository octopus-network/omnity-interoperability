pub mod audit;
pub mod eventlog;

use crate::lifecycle::{init::InitArgs, upgrade::UpgradeArgs};
use candid::{CandidType, Principal};
use omnity_types::{Chain, ChainId, ChainState, Ticket, TicketId, Token, TokenId};
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, collections::BTreeMap};

thread_local! {
    static __STATE: RefCell<Option<RouteState>> = RefCell::default();
}

#[derive(candid::CandidType, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MintTokenStatus {
    Finalized { block_index: u64 },
    Unknown,
}

#[derive(CandidType, Clone, Debug, Deserialize, Serialize)]
pub struct RouteState {
    pub chain_id: String,

    pub hub_principal: Principal,

    // Next index of query tickets from hub
    pub next_ticket_seq: u64,

    // Next index of query directives from hub
    pub next_directive_seq: u64,

    pub counterparties: BTreeMap<ChainId, Chain>,

    pub tokens: BTreeMap<TokenId, Token>,

    pub token_ledgers: BTreeMap<TokenId, Principal>,

    pub finalized_mint_token_requests: BTreeMap<TicketId, u64>,

    pub fee_token_factor: Option<u128>,

    pub target_chain_factor: BTreeMap<ChainId, u128>,

    pub chain_state: ChainState,

    pub failed_tickets: Vec<Ticket>,

    #[serde(skip)]
    pub is_timer_running: bool,
}

impl RouteState {
    pub fn validate_config(&self) {}

    pub fn upgrade(
        &mut self,
        UpgradeArgs {
            chain_id,
            hub_principal,
            chain_state,
        }: UpgradeArgs,
    ) {
        if let Some(chain_id) = chain_id {
            self.chain_id = chain_id;
        }
        if let Some(hub_principal) = hub_principal {
            self.hub_principal = hub_principal;
        }
        if let Some(chain_state) = chain_state {
            self.chain_state = chain_state;
        }
    }
}

impl From<InitArgs> for RouteState {
    fn from(args: InitArgs) -> Self {
        Self {
            chain_id: args.chain_id,
            hub_principal: args.hub_principal,
            token_ledgers: Default::default(),
            next_ticket_seq: 0,
            next_directive_seq: 0,
            counterparties: Default::default(),
            tokens: Default::default(),
            finalized_mint_token_requests: Default::default(),
            fee_token_factor: None,
            target_chain_factor: Default::default(),
            chain_state: args.chain_state,
            failed_tickets: Default::default(),
            is_timer_running: false,
        }
    }
}

/// Take the current state.
///
/// After calling this function the state won't be initialized anymore.
/// Panics if there is no state.
pub fn take_state<F, R>(f: F) -> R
where
    F: FnOnce(RouteState) -> R,
{
    __STATE.with(|s| f(s.take().expect("State not initialized!")))
}

/// Mutates (part of) the current state using `f`.
///
/// Panics if there is no state.
pub fn mutate_state<F, R>(f: F) -> R
where
    F: FnOnce(&mut RouteState) -> R,
{
    __STATE.with(|s| f(s.borrow_mut().as_mut().expect("State not initialized!")))
}

/// Read (part of) the current state using `f`.
///
/// Panics if there is no state.
pub fn read_state<F, R>(f: F) -> R
where
    F: FnOnce(&RouteState) -> R,
{
    __STATE.with(|s| f(s.borrow().as_ref().expect("State not initialized!")))
}

/// Replaces the current state.
pub fn replace_state(state: RouteState) {
    __STATE.with(|s| {
        *s.borrow_mut() = Some(state);
    });
}