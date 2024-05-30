use candid::Principal;
use omnity_types::{Chain, ChainId, TicketId, Token, TokenId};
use serde::Serialize;
use std::cell::RefCell;
use std::collections::BTreeMap;

use crate::lifecycle::init::InitArgs;

thread_local! {
    static __STATE: RefCell<Option<CustomsState>> = RefCell::default();
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, Serialize)]
pub struct CustomsState {
    pub chain_id: String,

    pub hub_principal: Principal,

    // Next index of query tickets from hub
    pub next_ticket_seq: u64,

    // Next index of query directives from hub
    pub next_directive_seq: u64,

    pub tokens: BTreeMap<TokenId, (Token, Principal)>,

    pub counterparties: BTreeMap<ChainId, Chain>,

    pub finalized_mint_token_requests: BTreeMap<TicketId, u64>,

    #[serde(skip)]
    pub is_timer_running: bool,
}

impl From<InitArgs> for CustomsState {
    fn from(args: InitArgs) -> Self {
        Self {
            chain_id: args.chain_id,
            hub_principal: args.hub_principal,
            next_ticket_seq: 0,
            next_directive_seq: 0,
            tokens: Default::default(),
            counterparties: Default::default(),
            finalized_mint_token_requests: Default::default(),
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
    F: FnOnce(CustomsState) -> R,
{
    __STATE.with(|s| f(s.take().expect("State not initialized!")))
}

/// Mutates (part of) the current state using `f`.
///
/// Panics if there is no state.
pub fn mutate_state<F, R>(f: F) -> R
where
    F: FnOnce(&mut CustomsState) -> R,
{
    __STATE.with(|s| f(s.borrow_mut().as_mut().expect("State not initialized!")))
}

/// Read (part of) the current state using `f`.
///
/// Panics if there is no state.
pub fn read_state<F, R>(f: F) -> R
where
    F: FnOnce(&CustomsState) -> R,
{
    __STATE.with(|s| f(s.borrow().as_ref().expect("State not initialized!")))
}

/// Replaces the current state.
pub fn replace_state(state: CustomsState) {
    __STATE.with(|s| {
        *s.borrow_mut() = Some(state);
    });
}
