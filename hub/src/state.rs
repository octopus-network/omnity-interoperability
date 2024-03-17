use candid::CandidType;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashMap;

use crate::types::{Amount, ChainWithSeq, DireQueue, TicketQueue, TokenMeta};
use omnity_types::{ChainId, Fee, Ticket, TicketId, TokenId};

thread_local! {
    static STATE: RefCell<HubState> = RefCell::new(HubState::default());
}

#[derive(CandidType, Deserialize, Serialize, Default, Debug)]
pub struct HubState {
    pub chains: HashMap<ChainId, ChainWithSeq>,
    pub tokens: HashMap<(ChainId, TokenId), TokenMeta>,
    pub fees: HashMap<(ChainId, TokenId), Fee>,
    pub cross_ledger: HashMap<TicketId, Ticket>,
    pub token_position: HashMap<(ChainId, TokenId), Amount>,
    pub dire_queue: DireQueue,
    pub ticket_queue: TicketQueue,
    pub owner: Option<String>,
    pub authorized_caller: HashMap<String, ChainId>,
}

/// A helper method to read the state.
///
/// Precondition: the state is already initialized.
pub fn with_state<R>(f: impl FnOnce(&HubState) -> R) -> R {
    STATE.with(|cell| f(&cell.borrow()))
}
/// A helper method to mutate the state.
///
/// Precondition: the state is already initialized.
pub fn with_state_mut<R>(f: impl FnOnce(&mut HubState) -> R) -> R {
    STATE.with(|cell| f(&mut cell.borrow_mut()))
}

// A helper method to set the state.
//
// Precondition: the state is _not_ initialized.
pub fn set_state(state: HubState) {
    STATE.with(|cell| *cell.borrow_mut() = state);
}
