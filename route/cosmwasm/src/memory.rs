use cosmrs::AccountId;
use ic_stable_structures::{
    memory_manager::{MemoryId, MemoryManager, VirtualMemory},
    DefaultMemoryImpl,
};
use ic_stable_structures::{Cell, StableBTreeMap};
use omnity_types::TicketId;
use std::{cell::RefCell, collections::HashMap};

use crate::cosmwasm::TxHash;
use crate::RouteState;

const LOG_MEMORY_ID: MemoryId = MemoryId::new(2);
const REDEEM_TICKETS_MEMORY_ID: MemoryId = MemoryId::new(3);
const ROUTE_STATE_MEMORY_ID: MemoryId = MemoryId::new(4);

type InnerMemory = DefaultMemoryImpl;

pub type Memory = VirtualMemory<InnerMemory>;

thread_local! {
    // static MEMORY: RefCell<Option<InnerMemory>> = RefCell::new(Some(InnerMemory::default()));

    static MEMORY_MANAGER: RefCell<MemoryManager<InnerMemory>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    static REDEEM_TICKETS: RefCell<StableBTreeMap<TxHash, TicketId, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(REDEEM_TICKETS_MEMORY_ID)),
        )
    );

    static ROUTE_STATE: RefCell<Cell<Option<RouteState>, Memory>> = RefCell::new(
        Cell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(ROUTE_STATE_MEMORY_ID)),
            None
        ).expect("Failed to init route state")
    );

}

pub fn init_stable_log() -> StableBTreeMap<Vec<u8>, Vec<u8>, Memory> {
    StableBTreeMap::init(MEMORY_MANAGER.with(|m| m.borrow().get(LOG_MEMORY_ID)))
}

pub fn insert_redeem_ticket(tx_hash: TxHash, ticket_id: TicketId) {
    REDEEM_TICKETS.with(|r| r.borrow_mut().insert(tx_hash, ticket_id));
}

pub fn get_redeem_tickets() -> HashMap<TxHash, TicketId> {
    REDEEM_TICKETS
    .with_borrow(|r| 
        r.iter().collect()
    )
}

pub fn get_redeem_ticket(tx_hash: &TxHash) -> Option<TicketId> {
    REDEEM_TICKETS.with(|r| r.borrow().get(tx_hash).clone())
}

pub fn set_route_state(state: RouteState) {
    ROUTE_STATE
        .with(|r| r.borrow_mut().set(Some(state)))
        .expect("Failed to set route state");
}

pub fn take_state() -> RouteState {
    ROUTE_STATE.with(|r| {
        r.borrow_mut()
            .get()
            .clone()
            .expect("Failed to take route state")
    })
}

pub fn read_state<F, R>(f: F) -> R
where
    F: FnOnce(&RouteState) -> R,
{
    ROUTE_STATE.with(|r| {
        f(r.borrow()
            .get()
            .as_ref()
            .expect("Failed to read route state"))
    })
}

pub fn mutate_state<F, R>(f: F) -> R
where
    F: FnOnce(&mut RouteState) -> R,
{
    let mut route_state = ROUTE_STATE.with(|r| {
        r.borrow_mut()
            .get()
            .clone()
            .expect("Failed to mutate route state")
    });
    let r = f(&mut route_state);
    ROUTE_STATE
        .with(|r| r.borrow_mut().set(Some(route_state)))
        .expect("Failed to set route state");
    r
}

pub fn get_contract_id() -> AccountId {
    read_state(|state| state.cw_port_contract_address.clone())
        .parse()
        .unwrap()
}
