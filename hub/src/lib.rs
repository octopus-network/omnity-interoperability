use candid::types::principal::Principal;
use candid::CandidType;

use ic_cdk::{init, post_upgrade, pre_upgrade, query, update};
use ic_stable_structures::Memory;
use log::{debug, error, info};
use omnity_types::{
    Action, BoardingPass, ChainInfo, ChainStatus, Directive, Error, Fee, LandingPass, TokenInfo,
};
mod memory;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap, HashSet};
pub type Timestamp = u64;
pub type ChainId = String;

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State::default());
}

#[derive(CandidType, Deserialize, Serialize, Debug)]
struct Transaction {
    pub timestamp: u64,
    pub source: String,
    pub target: String,
    pub action: Action,
    pub token: String,
    pub receiver: String,
    pub amount: u64,
    pub nonce: u64,
    pub signature: Option<Vec<u8>>,
}

#[derive(CandidType, Deserialize, Serialize, Default, Debug)]
struct CrossLedger {
    pub transfer: HashMap<Timestamp, Transaction>,
    pub redeem: HashMap<Timestamp, Transaction>,
}

#[derive(CandidType, Deserialize, Serialize, Default, Debug)]
struct State {
    pub directives: HashMap<Timestamp, Directive>,
    pub chains: HashMap<ChainId, ChainInfo>,
    pub tokens: HashMap<ChainId, TokenInfo>,
    pub fees: HashMap<ChainId, Fee>,
    pub cross_ledger: CrossLedger,
    pub authed_whitelist: HashSet<Principal>,
}

fn auth() -> Result<(), String> {
    Ok(())
}

/// A helper method to read the state.
///
/// Precondition: the state is already initialized.
fn with_state<R>(f: impl FnOnce(&State) -> R) -> R {
    STATE.with(|cell| f(&cell.borrow()))
}
/// A helper method to mutate the state.
///
/// Precondition: the state is already initialized.
fn with_state_mut<R>(f: impl FnOnce(&mut State) -> R) -> R {
    STATE.with(|cell| f(&mut cell.borrow_mut()))
}

// A helper method to set the state.
//
// Precondition: the state is _not_ initialized.
fn set_state(state: State) {
    STATE.with(|cell| *cell.borrow_mut() = state);
}
#[init]
fn init() {}

#[pre_upgrade]
fn pre_upgrade() {
    debug!("begin to handle pre_update state ...");

    // Serialize the state.
    let mut state_bytes = vec![];
    with_state(|state| ciborium::ser::into_writer(state, &mut state_bytes))
        .expect("failed to encode state");

    // Write the length of the serialized bytes to memory, followed by the
    // by the bytes themselves.
    let len = state_bytes.len() as u32;
    let memory = memory::get_upgrades_memory();
    crate::memory::write(&memory, 0, &len.to_le_bytes());
    crate::memory::write(&memory, 4, &state_bytes);
}

#[post_upgrade]
fn post_upgrade() {
    let memory = memory::get_upgrades_memory();

    // Read the length of the state bytes.
    let mut state_len_bytes = [0; 4];
    memory.read(0, &mut state_len_bytes);
    let state_len = u32::from_le_bytes(state_len_bytes) as usize;

    // Read the bytes
    let mut state_bytes = vec![0; state_len];
    memory.read(4, &mut state_bytes);

    // Deserialize and set the state.
    let state: State = ciborium::de::from_reader(&*state_bytes).expect("failed to decode state");

    set_state(state);
}

#[update(guard = "auth")]
pub fn add_chain(chain: ChainInfo) -> Result<(), Error> {
    Ok(())
}

#[update(guard = "auth")]
pub fn add_token(token: TokenInfo) -> Result<(), Error> {
    Ok(())
}

#[update(guard = "auth")]
pub fn set_chain_status(chain_id: String, status: ChainStatus) -> Result<(), Error> {
    Ok(())
}

fn signe_directive(directive: Directive) -> Result<(), Error> {
    Ok(())
}

#[query]
pub fn get_directive() -> Result<Directive, Error> {
    let directive = Directive::AddChain(ChainInfo::default());
    Ok(directive)
}

#[update(guard = "auth")]
pub fn update_fee(fee: Fee) -> Result<(), Error> {
    Ok(())
}

#[update(guard = "auth")]
pub fn generate_landing_pass(bp: BoardingPass) -> Result<(), Error> {
    Ok(())
}
#[query]
pub fn get_landing_pass() -> Result<LandingPass, Error> {
    let landing_pass = LandingPass::default();
    Ok(landing_pass)
}
