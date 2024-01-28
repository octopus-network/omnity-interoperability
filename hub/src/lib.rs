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
use omnity_types::{Action, ChainId, ChainInfo, DeliverStatus, Directive, DirectiveId, Error, Fee, Ticket, TicketId, TokenId, TokenInfo};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap, HashSet};
// use utils::init_log;
use crate::signer::PublicKeyReply;
use crate::utils::Network;



thread_local! {
    static STATE: RefCell<HubState> = RefCell::new(HubState::default());
}

#[derive(CandidType, Deserialize, Serialize, Debug)]
struct Transaction {
    pub trans_id: String,
    pub timestamp: u64,
    pub seq: u64,
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
    pub transfers: HashMap<TicketId, Ticket>,
    pub redeems: HashMap<TicketId, Ticket>,
}

#[derive(CandidType, Deserialize, Serialize, Default, Debug)]
struct HubState {
    pub chains: HashMap<ChainId, ChainInfo>,
    pub tokens: HashMap<(ChainId, TokenId), TokenInfo>,
    pub fees: HashMap<ChainId, Fee>,
    pub directives: BTreeMap<DirectiveId, (Directive, DeliverStatus)>,
    pub cross_ledger: CrossLedger,
    pub tickets: Vec<Ticket>,
    pub owner: Option<Principal>,
    pub whitelist: HashSet<Principal>,
}

/// A helper method to read the state.
///
/// Precondition: the state is already initialized.
fn with_state<R>(f: impl FnOnce(&HubState) -> R) -> R {
    STATE.with(|cell| f(&cell.borrow()))
}
/// A helper method to mutate the state.
///
/// Precondition: the state is already initialized.
fn with_state_mut<R>(f: impl FnOnce(&mut HubState) -> R) -> R {
    STATE.with(|cell| f(&mut cell.borrow_mut()))
}

// A helper method to set the state.
//
// Precondition: the state is _not_ initialized.
fn set_state(state: HubState) {
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
    let state: HubState = ciborium::de::from_reader(&*state_bytes).expect("failed to decode state");
    set_state(state);
}

/// validate directive ,this method will be called by sns
#[update(guard = "auth")]
pub fn validate_directive(_d: Directive) -> Result<String, String> {
    Ok("".to_string())
}

/// input diretive without signature and sign it
#[update(guard = "auth")]
pub async fn handl_directive(_d: Directive) -> Result<(), Error> {
    Ok(())
}

/// input fee without signature ,sign it and build directive
#[update(guard = "auth")]
pub async fn update_fee(_fee: Fee) -> Result<(), Error> {
    // signe and build update fee directive
    // signe_directive(directive)
    Ok(())
}

/// input diretive without signature and sign it
#[update(guard = "auth")]
pub async fn send_directive(_d: Directive) -> Result<(), Error> {
    Ok(())
}

#[update(guard = "auth")]
pub async fn update_directive_status(_id: DirectiveId, _s: DeliverStatus) -> Result<(), Error> {
    Ok(())
}

/// check the ticket availability
#[update(guard = "auth")]
pub async fn check_ticket(_t: Ticket) -> Result<(), Error> {
    Ok(())
}

#[update(guard = "auth")]
pub async fn send_ticket(_t: Ticket) -> Result<(), Error> {
    Ok(())
}

#[update(guard = "auth")]
pub async fn update_ticket_status(_tid: TicketId, _s: DeliverStatus) -> Result<(), Error> {
    Ok(())
}

#[query(guard = "auth")]
pub async fn query_ticket(_tid: String) -> Result<Ticket, Error> {
    let ticket = Ticket::default();
    Ok(ticket)
}

#[query(guard = "auth")]
pub async fn query_tickets() -> Result<Vec<Ticket>, Error> {
    let tickets = Vec::new();
    Ok(tickets)
}

ic_cdk::export_candid!();

#[cfg(test)]
mod tests {
    // use super::*;
    use crypto::digest::Digest;
    use crypto::sha3::Sha3;
    #[test]
    fn hash() {
        let mut hasher = Sha3::keccak256();
        hasher.input_str("Hi,Boern");
        let hex = hasher.result_str();
        println!("{}", hex);
    }
}
