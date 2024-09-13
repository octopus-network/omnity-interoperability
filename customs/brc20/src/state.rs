use std::cell::RefCell;
use std::collections::{BTreeMap, VecDeque};

use candid::Principal;
use ic_btc_interface::Txid;
use ic_stable_structures::StableBTreeMap;
use serde::{Deserialize, Serialize};

use omnity_types::{Chain, ChainId, ChainState, Directive, Seq, Ticket, TicketId, Token, TokenId};

use crate::stable_memory::Memory;
use crate::types::{Brc20Ticker, GenTicketRequest, GenTicketStatus, PendingDirectiveStatus, PendingTicketStatus};

thread_local! {
    static STATE: RefCell<Option<Brc20State >> = RefCell::new(None);
}

#[derive(Deserialize, Serialize)]
pub struct Brc20State {
    pub admins: Vec<Principal>,
    pub deposit_addr: String,
    pub hub_principal: Principal,
    pub fee_token_id: String,
    pub chain_id: String,
    pub tokens: BTreeMap<TokenId, Token>,
    pub token_contracts: BTreeMap<TokenId, String>,
    pub counterparties: BTreeMap<ChainId, Chain>,
    pub finalized_mint_token_requests: BTreeMap<TicketId, String>,
    pub chain_state: ChainState,
    pub fee_token_factor: Option<u128>,
    pub target_chain_factor: BTreeMap<ChainId, u128>,
    pub next_ticket_seq: u64,
    pub next_directive_seq: u64,
    pub next_consume_ticket_seq: u64,
    pub next_consume_directive_seq: u64,
    #[serde(skip, default = "crate::stable_memory::init_to_evm_tickets_queue")]
    pub tickets_queue: StableBTreeMap<u64, Ticket, Memory>,
    #[serde(skip, default = "crate::stable_memory::init_to_evm_directives_queue")]
    pub directives_queue: StableBTreeMap<u64, Directive, Memory>,
    #[serde(skip, default = "crate::stable_memory::init_pending_ticket_map")]
    pub pending_tickets_map: StableBTreeMap<TicketId, PendingTicketStatus, Memory>,
    #[serde(skip, default = "crate::stable_memory::init_pending_directive_map")]
    pub pending_directive_map: StableBTreeMap<Seq, PendingDirectiveStatus, Memory>,
    #[serde(skip)]
    pub is_timer_running: BTreeMap<String, bool>,

    /// The transaction has just entered the memory pool
    /// or has not reached sufficient confirmation.
    pub pending_gen_ticket_requests: BTreeMap<Txid, GenTicketRequest>,

    /// The transaction needs to wait for runes oracle to update the runes balance.
    pub confirmed_gen_ticket_requests: BTreeMap<Txid, GenTicketRequest>,

    pub finalized_gen_ticket_requests: VecDeque<GenTicketRequest>,

}

impl Brc20State {

    pub fn generate_ticket_status(&self, tx_id: Txid) -> GenTicketStatus {
        if let Some(req) = self.pending_gen_ticket_requests.get(&tx_id) {
            return GenTicketStatus::Pending(req.clone());
        }
        if let Some(req) = self.confirmed_gen_ticket_requests.get(&tx_id) {
            return GenTicketStatus::Confirmed(req.clone());
        }
        match self
            .finalized_gen_ticket_requests
            .iter()
            .find(|req| req.txid == tx_id)
        {
            Some(req) => GenTicketStatus::Finalized(req.clone()),
            None => GenTicketStatus::Unknown,
        }
    }
}

pub fn deposit_addr() -> String {
    read_state(|s|s.deposit_addr.clone())
}

pub fn mutate_state<F, R>(f: F) -> R
    where
        F: FnOnce(&mut Brc20State) -> R,
{
    STATE.with(|s| f(s.borrow_mut().as_mut().expect("State not initialized!")))
}

pub fn read_state<F, R>(f: F) -> R
    where
        F: FnOnce(&Brc20State) -> R,
{
    STATE.with(|s| f(s.borrow().as_ref().expect("State not initialized!")))
}

/// Replaces the current state.
pub fn replace_state(state: Brc20State) {
    STATE.with(|s| {
        *s.borrow_mut() = Some(state);
    });
}

pub fn take_state<F, R>(f: F) -> R
    where
        F: FnOnce(Brc20State) -> R,
{
    STATE.with(|s| f(s.take().expect("State not initialized!")))
}
