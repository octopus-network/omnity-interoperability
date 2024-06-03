use candid::CandidType;
use ic_cdk::{query, update};

use omnity_types::TicketId;
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, collections::BTreeMap};

thread_local! {
    static __STATE: RefCell<RouteState> = RefCell::new(RouteState::default());
}

#[derive(candid::CandidType, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MintTokenStatus {
    Finalized { block_index: u64 },
    Unknown,
}

#[derive(CandidType, Clone, Debug, Deserialize, Serialize)]
pub struct RouteState {
  
    pub finalized_mint_token_requests: BTreeMap<TicketId, u64>,

   
}

impl Default for RouteState {
    fn default() -> Self {
        Self {
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
    __STATE.with(|s| f(s.take()))
}

/// Mutates (part of) the current state using `f`.
///
/// Panics if there is no state.
pub fn mutate_state<F, R>(f: F) -> R
where
    F: FnOnce(&mut RouteState) -> R,
{
    __STATE.with(|s| f(&mut s.borrow_mut()))
}

/// Read (part of) the current state using `f`.
///
/// Panics if there is no state.
pub fn read_state<F, R>(f: F) -> R
where
    F: FnOnce(&RouteState) -> R,
{
    __STATE.with(|s| f(&s.borrow()))
}

/// Replaces the current state.
pub fn replace_state(state: RouteState) {
    __STATE.with(|s| {
        *s.borrow_mut() = state;
    });
}

#[update]
fn mock_finalized_mint_token(ticket_id: TicketId, block_idx: u64) {
    mutate_state(|s| {
        s.finalized_mint_token_requests.insert(ticket_id, block_idx);
    })
}

#[query]
fn mint_token_status(ticket_id: String) -> MintTokenStatus {
    read_state(|s| {
        s.finalized_mint_token_requests
            .get(&ticket_id)
            .map_or(MintTokenStatus::Unknown, |&block_index| {
                MintTokenStatus::Finalized { block_index }
            })
    })
}

fn main() {}
ic_cdk::export_candid!();
