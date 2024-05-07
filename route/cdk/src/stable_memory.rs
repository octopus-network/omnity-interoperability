use std::cell::RefCell;
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use crate::types::{Directive, PendingTicketStatus, Ticket, TicketId};

pub type InnerMemory = DefaultMemoryImpl;
pub type Memory = VirtualMemory<InnerMemory>;
pub const TO_CDK_TICKETS_MEMORY_ID: MemoryId = MemoryId::new(0);
pub const TO_CDK_DIRECTIVES_MEMORY_ID: MemoryId = MemoryId::new(1);
pub const PENDING_TICKET_MAP_MEMORY_ID: MemoryId = MemoryId::new(2);
pub const PENDING_DIRECTIVE_MAP_MEMORY_ID: MemoryId = MemoryId::new(3);

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

pub fn get_to_cdk_tickets_memory() -> Memory {
    with_memory_manager(|m| m.get(TO_CDK_TICKETS_MEMORY_ID))
}

pub fn get_to_cdk_directives_memory() -> Memory {
    with_memory_manager(|m| m.get(TO_CDK_DIRECTIVES_MEMORY_ID))
}

pub fn get_pending_ticket_map_memory() -> Memory {
    with_memory_manager(|m| m.get(PENDING_TICKET_MAP_MEMORY_ID))
}

pub fn init_to_cdk_tickets_queue() -> StableBTreeMap<u64, Ticket, Memory> {
    StableBTreeMap::init(get_to_cdk_tickets_memory())
}
pub fn init_pending_ticket_map() -> StableBTreeMap<TicketId, PendingTicketStatus, Memory> {
    StableBTreeMap::init(get_pending_ticket_map_memory())
}

pub fn init_to_cdk_directives_queue() -> StableBTreeMap<u64, Directive, Memory> {
    StableBTreeMap::init(get_to_cdk_directives_memory())
}
