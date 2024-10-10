use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap};
use omnity_types::{Directive, Ticket};
use std::cell::RefCell;

pub type InnerMemory = DefaultMemoryImpl;
pub type Memory = VirtualMemory<InnerMemory>;
pub const UPGRADE_STASH_MEMORY_ID: MemoryId = MemoryId::new(0);
pub const UNLOCK_TICKETS_MEMORY_ID: MemoryId = MemoryId::new(1);
pub const DIRECTIVES_MEMORY_ID: MemoryId = MemoryId::new(2);

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

pub fn get_upgrade_stash_memory() -> Memory {
    with_memory_manager(|m| m.get(UPGRADE_STASH_MEMORY_ID))
}

pub fn init_unlock_tickets_queue() -> StableBTreeMap<u64, Ticket, Memory> {
    StableBTreeMap::init(get_unlock_tickets_memory())
}

pub fn init_directives_queue() -> StableBTreeMap<u64, Directive, Memory> {
    StableBTreeMap::init(get_directives_memory())
}
