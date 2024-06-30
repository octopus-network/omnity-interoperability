use candid::{CandidType, Principal};

use ic_cdk::api::call::RejectionCode;
use omnity_types::TicketId;
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, collections::BTreeMap, fmt};

pub use ic_btc_interface::{Address, OutPoint, Utxo};

#[derive(candid::CandidType, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PushUtxosToAddress {
    pub utxos: BTreeMap<Address, Vec<Utxo>>,
}

thread_local! {
    static __STATE: RefCell<RouteState> = RefCell::new(RouteState::default());
}

#[derive(candid::CandidType, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Args {
    pub hub_principal: Principal,
    pub directive_method: String,
    pub ticket_method: String,
}

#[derive(candid::CandidType, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MintTokenStatus {
    Finalized { block_index: u64 },
    Unknown,
}

#[derive(CandidType, Clone, Debug, Deserialize, Serialize)]
pub struct RouteState {
    pub finalized_mint_token_requests: BTreeMap<TicketId, u64>,

    #[serde(skip)]
    pub is_timer_running: BTreeMap<String, bool>,
    pub hub_principal: Principal,
    pub directive_method: String,
    pub ticket_method: String,
}

impl Default for RouteState {
    fn default() -> Self {
        Self {
            finalized_mint_token_requests: Default::default(),
            is_timer_running: BTreeMap::new(),
            hub_principal: Principal::anonymous(),
            directive_method: "query_directives".to_string(),
            ticket_method: "query_tickets".to_string(),
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
#[must_use]
pub struct TimerLogicGuard(String);

impl TimerLogicGuard {
    pub fn new(task_name: String) -> Option<Self> {
        mutate_state(|s| {
            let running = s
                .is_timer_running
                .get(&task_name)
                .cloned()
                .unwrap_or_default();
            if running {
                return None;
            }
            s.is_timer_running.insert(task_name.clone(), true);
            Some(TimerLogicGuard(task_name))
        })
    }
}

impl Drop for TimerLogicGuard {
    fn drop(&mut self) {
        mutate_state(|s| s.is_timer_running.remove(&self.0));
    }
}

/// Represents an error from a management canister call, such as
/// `sign_with_ecdsa` or `bitcoin_send_transaction`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallError {
    pub method: String,
    pub reason: Reason,
}

impl fmt::Display for CallError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            fmt,
            "management call '{}' failed: {}",
            self.method, self.reason
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// The reason for the management call failure.
pub enum Reason {
    /// Failed to send a signature request because the local output queue is
    /// full.
    QueueIsFull,
    /// The canister does not have enough cycles to submit the request.
    OutOfCycles,
    /// The call failed with an error.
    CanisterError(String),
    /// The management canister rejected the signature request (not enough
    /// cycles, the ECDSA subnet is overloaded, etc.).
    Rejected(String),
}

impl fmt::Display for Reason {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::QueueIsFull => write!(fmt, "the canister queue is full"),
            Self::OutOfCycles => write!(fmt, "the canister is out of cycles"),
            Self::CanisterError(msg) => write!(fmt, "canister error: {}", msg),
            Self::Rejected(msg) => {
                write!(fmt, "the management canister rejected the call: {}", msg)
            }
        }
    }
}

impl Reason {
    pub fn from_reject(reject_code: RejectionCode, reject_message: String) -> Self {
        match reject_code {
            RejectionCode::CanisterReject => Self::Rejected(reject_message),
            _ => Self::CanisterError(reject_message),
        }
    }
}
