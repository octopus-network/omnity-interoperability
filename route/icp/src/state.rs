pub mod audit;
use candid::Principal;
use omnity_types::{Chain, ChainId, TicketId, Token, TokenId};
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, collections::BTreeMap};

use crate::{
    lifecycle::init::InitArgs,
    updates::mint_token::{MintTokenError, MintTokenRequest},
};

thread_local! {
    static __STATE: RefCell<Option<RouteState>> = RefCell::default();
}

#[derive(candid::CandidType, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MintTokenStatus {
    Finalized,
    Failure(MintTokenError),
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
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

    pub finalized_mint_token_requests: BTreeMap<TicketId, MintTokenRequest>,
}

impl RouteState {
    pub fn validate_config(&self) {}
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
