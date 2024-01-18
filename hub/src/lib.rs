mod auth;
mod errors;
mod memory;
mod signer;
mod utils;

use candid::types::principal::Principal;
use candid::CandidType;

use auth::auth;
use ic_cdk::{init, post_upgrade, pre_upgrade, query, update};
use ic_stable_structures::writer::Writer;
use ic_stable_structures::Memory;
use log::debug;
use omnity_types::{
    Action, BoardingPass, ChainInfo, Directive, Error, Fee, LandingPass, TokenInfo,
};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
// use utils::init_log;
use crate::signer::PublicKeyReply;
use crate::utils::Network;

pub type Timestamp = u64;
pub type ChainId = String;
pub type TokenId = String;
pub type TransId = String;

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State::default());
}

#[derive(CandidType, Deserialize, Serialize, Debug)]
struct Transaction {
    pub trans_id: String,
    pub timestamp: u64,
    pub nonce: u64,
    pub src_chain_id: String,
    pub dst_chain_id: String,
    pub action: Action,
    pub token: String,
    pub memo: Option<Vec<u8>>,
    pub receiver: String,
    pub amount: u64,
    pub signature: Option<Vec<u8>>,
}

#[derive(CandidType, Deserialize, Serialize, Default, Debug)]
struct CrossLedger {
    pub transfers: HashMap<TransId, Transaction>,
    pub redeems: HashMap<TransId, Transaction>,
}

#[derive(CandidType, Deserialize, Serialize, Default, Debug)]
struct State {
    pub chains: HashMap<ChainId, ChainInfo>,
    pub tokens: HashMap<(ChainId, TokenId), TokenInfo>,
    pub fees: HashMap<ChainId, Fee>,
    pub directives: Vec<Directive>,
    pub cross_ledger: CrossLedger,
    pub landing_passes: Vec<LandingPass>,
    pub owner: Option<Principal>,
    pub whitelist: HashSet<Principal>,
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
fn init() {
    // init_log()
}

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
    let mut memory = memory::get_upgrades_memory();
    let mut writer = Writer::new(&mut memory, 0);
    writer
        .write(&len.to_le_bytes())
        .expect("failed to save config len");
    writer.write(&state_bytes).expect("failed to save config");
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
/// input diretive without signature and sign it
#[update(guard = "auth")]
pub fn signe_directive(_directive: Directive) -> Result<(), Error> {
    Ok(())
}

/// input fee without signature ,sign it and build directive
#[update(guard = "auth")]
pub fn update_fee(_fee: Fee) -> Result<(), Error> {
    // signe and build update fee directive
    // signe_directive(directive)
    Ok(())
}

#[query]
pub fn get_directives() -> Result<Vec<Directive>, Error> {
    let directives = Vec::new();
    Ok(directives)
}

/// input a boarding pass and create a landing pass with signature
#[update(guard = "auth")]
pub fn generate_landing_pass(_boarding_pass: BoardingPass) -> Result<(), Error> {
    Ok(())
}

#[query]
pub fn get_landing_passes() -> Result<Vec<LandingPass>, Error> {
    let landing_passes = Vec::new();
    Ok(landing_passes)
}

ic_cdk::export_candid!();
