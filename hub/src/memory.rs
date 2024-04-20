use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::StableLog as IcLog;

#[cfg(not(feature = "file_memory"))]
use ic_stable_structures::DefaultMemoryImpl;
#[cfg(feature = "file_memory")]
use ic_stable_structures::FileMemory;
use ic_stable_structures::StableBTreeMap;
use std::cell::RefCell;

use omnity_types::{ChainId, Directive, SeqKey, Ticket, TicketId, TokenId, Topic};

use crate::types::{Amount, ChainTokenFactor, ChainWithSeq, Subscribers, TokenKey, TokenMeta};

const UPGRADES: MemoryId = MemoryId::new(0);
const CHAIN: MemoryId = MemoryId::new(1);
const TOKEN: MemoryId = MemoryId::new(2);
const CHAIN_FACTOR: MemoryId = MemoryId::new(3);
const TOKEN_FACTOR: MemoryId = MemoryId::new(4);
const TOKEN_POSITION: MemoryId = MemoryId::new(5);
const LEDGER: MemoryId = MemoryId::new(6);
const DIRE_QUEUE: MemoryId = MemoryId::new(7);
const TICKET_QUEUE: MemoryId = MemoryId::new(8);
const LOG_MEMORY_ID: MemoryId = MemoryId::new(9);
const SUBCRIBER: MemoryId = MemoryId::new(10);

const EVENT_INDEX_MEMORY_ID: MemoryId = MemoryId::new(11);
const EVENT_DATA_MEMORY_ID: MemoryId = MemoryId::new(12);

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

// chain factor stable memory
pub fn get_chain_factor_memory() -> Memory {
    with_memory_manager(|m| m.get(CHAIN_FACTOR))
}

// token factor stable memory
pub fn get_token_factor_memory() -> Memory {
    with_memory_manager(|m| m.get(TOKEN_FACTOR))
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

pub fn get_subs_memory() -> Memory {
    with_memory_manager(|m| m.get(SUBCRIBER))
}

// ticket stable memory
pub fn get_ticket_queue_memory() -> Memory {
    with_memory_manager(|m| m.get(TICKET_QUEUE))
}

pub fn init_chain() -> StableBTreeMap<ChainId, ChainWithSeq, Memory> {
    StableBTreeMap::init(get_chain_memory())
}
pub fn init_token() -> StableBTreeMap<TokenId, TokenMeta, Memory> {
    StableBTreeMap::init(get_token_memory())
}
pub fn init_chain_factor() -> StableBTreeMap<ChainId, u128, Memory> {
    StableBTreeMap::init(get_chain_factor_memory())
}

pub fn init_token_factor() -> StableBTreeMap<TokenKey, ChainTokenFactor, Memory> {
    StableBTreeMap::init(get_token_factor_memory())
}

pub fn init_token_position() -> StableBTreeMap<TokenKey, Amount, Memory> {
    StableBTreeMap::init(get_token_position_memory())
}
pub fn init_ledger() -> StableBTreeMap<TicketId, Ticket, Memory> {
    StableBTreeMap::init(get_ledger_memory())
}
pub fn init_dire_queue() -> StableBTreeMap<SeqKey, Directive, Memory> {
    StableBTreeMap::init(get_dire_queue_memory())
}
pub fn init_subs() -> StableBTreeMap<Topic, Subscribers, Memory> {
    StableBTreeMap::init(get_subs_memory())
}

pub fn init_ticket_queue() -> StableBTreeMap<SeqKey, Ticket, Memory> {
    StableBTreeMap::init(get_ticket_queue_memory())
}

pub fn init_stable_log() -> StableBTreeMap<Vec<u8>, Vec<u8>, Memory> {
    StableBTreeMap::init(with_memory_manager(|m| m.get(LOG_MEMORY_ID)))
}

pub fn init_event_log() -> IcLog<Vec<u8>, Memory, Memory> {
    IcLog::init(
        with_memory_manager(|m| m.get(EVENT_DATA_MEMORY_ID)),
        with_memory_manager(|m| m.get(EVENT_INDEX_MEMORY_ID)),
    )
    .expect("failed to initialize stable log")
}
