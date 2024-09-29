use ic_stable_structures::StableBTreeMap;
use ic_stable_structures::{
    memory_manager::{MemoryId, MemoryManager, VirtualMemory},
    DefaultMemoryImpl,
};
use std::cell::RefCell;

use crate::handler::gen_ticket::GenerateTicketReq;
use crate::handler::mint_token::MintTokenRequest;
use crate::state::{AccountInfo, AtaKey, UpdateToken};
use crate::types::Ticket;
use crate::types::{Chain, Token};

const UPGRADES: MemoryId = MemoryId::new(0);
const TOKENS: MemoryId = MemoryId::new(1);
const UPDATE_TOKENS: MemoryId = MemoryId::new(2);
const TICKET_QUEUE: MemoryId = MemoryId::new(3);
const FAILED_TICKETS: MemoryId = MemoryId::new(4);
const COUNTERPARTIES: MemoryId = MemoryId::new(5);
const TOKEN_MINT_ACCOUNTS: MemoryId = MemoryId::new(6);
const ASSOCIATED_ACCOUNTS: MemoryId = MemoryId::new(7);
const MINT_TOKEN_REQUESTS: MemoryId = MemoryId::new(8);
const GEN_TICKET_REQS: MemoryId = MemoryId::new(9);

type InnerMemory = DefaultMemoryImpl;

pub type Memory = VirtualMemory<InnerMemory>;

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

pub fn get_failed_tickets_memory() -> Memory {
    with_memory_manager(|m| m.get(FAILED_TICKETS))
}

pub fn get_counterparties_memory() -> Memory {
    with_memory_manager(|m| m.get(COUNTERPARTIES))
}

pub fn get_tokens_memory() -> Memory {
    with_memory_manager(|m| m.get(TOKENS))
}

pub fn get_update_tokens_memory() -> Memory {
    with_memory_manager(|m| m.get(UPDATE_TOKENS))
}

pub fn get_token_mint_accounts_memory() -> Memory {
    with_memory_manager(|m| m.get(TOKEN_MINT_ACCOUNTS))
}

pub fn get_associated_accounts_memory() -> Memory {
    with_memory_manager(|m| m.get(ASSOCIATED_ACCOUNTS))
}

pub fn get_mint_token_requests_memory() -> Memory {
    with_memory_manager(|m| m.get(MINT_TOKEN_REQUESTS))
}

pub fn get_gen_ticket_req_memory() -> Memory {
    with_memory_manager(|m| m.get(GEN_TICKET_REQS))
}

pub fn init_ticket_queue() -> StableBTreeMap<u64, Ticket, Memory> {
    StableBTreeMap::init(get_ticket_queue_memory())
}

pub fn init_failed_tickets() -> StableBTreeMap<String, Ticket, Memory> {
    StableBTreeMap::init(get_failed_tickets_memory())
}

pub fn init_counterparties() -> StableBTreeMap<String, Chain, Memory> {
    StableBTreeMap::init(get_counterparties_memory())
}

pub fn init_tokens() -> StableBTreeMap<String, Token, Memory> {
    StableBTreeMap::init(get_tokens_memory())
}

pub fn init_update_tokens() -> StableBTreeMap<String, UpdateToken, Memory> {
    StableBTreeMap::init(get_update_tokens_memory())
}

pub fn init_token_mint_accounts() -> StableBTreeMap<String, AccountInfo, Memory> {
    StableBTreeMap::init(get_token_mint_accounts_memory())
}

pub fn init_associated_accounts() -> StableBTreeMap<AtaKey, AccountInfo, Memory> {
    StableBTreeMap::init(get_associated_accounts_memory())
}

pub fn init_mint_token_requests() -> StableBTreeMap<String, MintTokenRequest, Memory> {
    StableBTreeMap::init(get_mint_token_requests_memory())
}

pub fn init_gen_ticket_reqs() -> StableBTreeMap<String, GenerateTicketReq, Memory> {
    StableBTreeMap::init(get_gen_ticket_req_memory())
}
