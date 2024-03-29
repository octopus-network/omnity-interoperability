use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
#[cfg(not(feature = "file_memory"))]
use ic_stable_structures::DefaultMemoryImpl;
#[cfg(feature = "file_memory")]
use ic_stable_structures::FileMemory;
use std::cell::RefCell;

const UPGRADES: MemoryId = MemoryId::new(0);
const CHAIN: MemoryId = MemoryId::new(1);
const TOKEN: MemoryId = MemoryId::new(2);
const FEE: MemoryId = MemoryId::new(3);
const TOKEN_POSITION: MemoryId = MemoryId::new(4);
const LEDGER: MemoryId = MemoryId::new(5);
const DIRE_QUEUE: MemoryId = MemoryId::new(6);
const TICKET_QUEUE: MemoryId = MemoryId::new(7);

#[cfg(feature = "file_memory")]
type InnerMemory = FileMemory;

#[cfg(not(feature = "file_memory"))]
type InnerMemory = DefaultMemoryImpl;

pub type Memory = VirtualMemory<InnerMemory>;

#[cfg(feature = "file_memory")]
thread_local! {
    static MEMORY: RefCell<Option<InnerMemory>> = RefCell::new(None);

    static MEMORY_MANAGER: RefCell<Option<MemoryManager<InnerMemory>>> = RefCell::new(None);
}

#[cfg(not(feature = "file_memory"))]
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

// upgrade stable memory
pub fn get_upgrades_memory() -> Memory {
    with_memory_manager(|m| m.get(UPGRADES))
}

// chain stable memory
pub fn get_chain_memory() -> Memory {
    with_memory_manager(|m| m.get(CHAIN))
}

// token stable memory
pub fn get_token_memory() -> Memory {
    with_memory_manager(|m| m.get(TOKEN))
}

// fee stable memory
pub fn get_fee_memory() -> Memory {
    with_memory_manager(|m| m.get(FEE))
}

// token postion stable memory
pub fn get_token_position_memory() -> Memory {
    with_memory_manager(|m| m.get(TOKEN_POSITION))
}

// ledger stable memory
pub fn get_ledger_memory() -> Memory {
    with_memory_manager(|m| m.get(LEDGER))
}

// dire stable memory
pub fn get_dire_queue_memory() -> Memory {
    with_memory_manager(|m| m.get(DIRE_QUEUE))
}

// ticket stable memory
pub fn get_ticket_queue_memory() -> Memory {
    with_memory_manager(|m| m.get(TICKET_QUEUE))
}
