use crate::{
    state::ReleaseTokenReq, types::omnity_types::TicketId,
    updates::generate_ticket::GenerateTicketArgs,
};
use ic_stable_structures::{
    memory_manager::{MemoryId, MemoryManager, VirtualMemory},
    DefaultMemoryImpl, StableBTreeMap,
};
use std::cell::RefCell;
use std::ops::Deref;

const UPGRADES: MemoryId = MemoryId::new(0);
const FINALIZED_REQUESTS_MEMORY_ID: MemoryId = MemoryId::new(1);
const FINALIZED_GEN_TICKETS_MEMORY_ID: MemoryId = MemoryId::new(2);

pub type VMem = VirtualMemory<DefaultMemoryImpl>;

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );
}

fn with_memory_manager<R>(f: impl FnOnce(&MemoryManager<DefaultMemoryImpl>) -> R) -> R {
    MEMORY_MANAGER.with(|cell| f(cell.borrow().deref()))
}

pub fn get_upgrades_memory() -> VMem {
    with_memory_manager(|m| m.get(UPGRADES))
}

pub fn init_finalized_requests() -> StableBTreeMap<TicketId, ReleaseTokenReq, VMem> {
    StableBTreeMap::init(with_memory_manager(|m| m.get(FINALIZED_REQUESTS_MEMORY_ID)))
}

pub fn init_finalized_gen_tickets() -> StableBTreeMap<TicketId, GenerateTicketArgs, VMem> {
    StableBTreeMap::init(with_memory_manager(|m| {
        m.get(FINALIZED_GEN_TICKETS_MEMORY_ID)
    }))
}
