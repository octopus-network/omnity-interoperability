use ic_stable_structures::StableBTreeMap;
use ic_stable_structures::StableLog as IcLog;
use ic_stable_structures::{
    memory_manager::{MemoryId, MemoryManager, VirtualMemory},
    DefaultMemoryImpl,
};
use std::cell::RefCell;

use crate::types::Ticket;

const UPGRADES: MemoryId = MemoryId::new(0);
const EVENT_INDEX_MEMORY_ID: MemoryId = MemoryId::new(1);
const EVENT_DATA_MEMORY_ID: MemoryId = MemoryId::new(2);
const TICKET_QUEUE: MemoryId = MemoryId::new(3);

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

pub fn get_upgrades_memory() -> Memory {
    with_memory_manager(|m| m.get(UPGRADES))
}

pub fn get_ticket_queue_memory() -> Memory {
    with_memory_manager(|m| m.get(TICKET_QUEUE))
}

pub fn init_ticket_queue() -> StableBTreeMap<u64, Ticket, Memory> {
    StableBTreeMap::init(get_ticket_queue_memory())
}

pub fn init_event() -> IcLog<Vec<u8>, Memory, Memory> {
    IcLog::init(
        with_memory_manager(|m| m.get(EVENT_DATA_MEMORY_ID)),
        with_memory_manager(|m| m.get(EVENT_INDEX_MEMORY_ID)),
    )
    .expect("failed to initialize stable log")
}

