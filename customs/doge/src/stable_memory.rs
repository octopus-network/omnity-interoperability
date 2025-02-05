use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap};
use omnity_types::{Directive, Seq, Ticket};
use std::cell::RefCell;

use crate::custom_to_dogecoin::SendTicketResult;
use crate::doge::header::BlockHeaderJsonResult;
use crate::types::{LockTicketRequest, Txid};

pub type InnerMemory = DefaultMemoryImpl;
pub type Memory = VirtualMemory<InnerMemory>;
pub const UPGRADE_STASH_MEMORY_ID: MemoryId = MemoryId::new(0);
pub const UNLOCK_TICKETS_MEMORY_ID: MemoryId = MemoryId::new(1);
pub const DIRECTIVES_MEMORY_ID: MemoryId = MemoryId::new(2);
pub const DEPOSIT_TX_MEMORY_ID: MemoryId = MemoryId::new(3);
pub const UNLOCK_TICKETS_RESULTS_MEMORY_ID: MemoryId = MemoryId::new(4);
pub const LOCK_TICKETS_REQUESTS_MEMORY_ID: MemoryId = MemoryId::new(5);
pub const DOGE_BLOCK_HEADERS_MEMORY_ID: MemoryId = MemoryId::new(6);

thread_local! {
    static MEMORY: RefCell<Option<InnerMemory>> = RefCell::new(Some(InnerMemory::default()));

    static MEMORY_MANAGER: RefCell<Option<MemoryManager<InnerMemory>>> =
        RefCell::new(Some(MemoryManager::init(MEMORY.with(|m| m.borrow().clone().unwrap()))));
}

fn with_memory_manager<R>(f: impl FnOnce(&MemoryManager<InnerMemory>) -> R) -> R {
    MEMORY_MANAGER.with(|cell| {
        f(cell
            .borrow()
            .as_ref()
            .expect("memory manager not initialized"))
    })
}

pub fn get_unlock_tickets_memory() -> Memory {
    with_memory_manager(|m| m.get(UNLOCK_TICKETS_MEMORY_ID))
}

pub fn get_directives_memory() -> Memory {
    with_memory_manager(|m| m.get(DIRECTIVES_MEMORY_ID))
}

pub fn get_deposit_tx_memory() -> Memory {
    with_memory_manager(|m| m.get(DEPOSIT_TX_MEMORY_ID))
}

pub fn get_upgrade_stash_memory() -> Memory {
    with_memory_manager(|m| m.get(UPGRADE_STASH_MEMORY_ID))
}

pub fn get_unlock_ticket_results_memory() -> Memory {
    with_memory_manager(|m| m.get(UNLOCK_TICKETS_RESULTS_MEMORY_ID))
}

pub fn get_lock_ticket_requests_memory() -> Memory {
    with_memory_manager(|m| m.get(LOCK_TICKETS_REQUESTS_MEMORY_ID))
}

pub fn get_doge_block_headers_memory() -> Memory {
    with_memory_manager(|m| m.get(DOGE_BLOCK_HEADERS_MEMORY_ID))
}

pub fn init_unlock_tickets_queue() -> StableBTreeMap<u64, Ticket, Memory> {
    StableBTreeMap::init(get_unlock_tickets_memory())
}

pub fn init_directives_queue() -> StableBTreeMap<u64, Directive, Memory> {
    StableBTreeMap::init(get_directives_memory())
}

// pub fn init_deposit_fee_tx_set()
pub fn init_deposit_fee_tx_set() -> StableBTreeMap<String, (), Memory> {
    StableBTreeMap::init(get_deposit_tx_memory())
}

pub fn init_unlock_ticket_results() -> StableBTreeMap<Seq, SendTicketResult, Memory> {
    StableBTreeMap::init(get_unlock_ticket_results_memory())
}

pub fn init_lock_ticket_requests() -> StableBTreeMap<Txid, LockTicketRequest, Memory> {
    StableBTreeMap::init(get_lock_ticket_requests_memory())
}

pub fn init_doge_block_headers() -> StableBTreeMap<u64, BlockHeaderJsonResult, Memory> {
    StableBTreeMap::init(get_doge_block_headers_memory())
}
