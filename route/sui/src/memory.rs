use ic_stable_structures::StableBTreeMap;
use ic_stable_structures::StableCell;
use ic_stable_structures::{
    memory_manager::{MemoryId, MemoryManager, VirtualMemory},
    DefaultMemoryImpl,
};
use std::cell::RefCell;

use crate::config::SuiRouteConfig;
use crate::handler::burn_token::BurnTx;
use crate::handler::clear_ticket::ClearTx;
use crate::handler::mint_token::MintTokenRequest;
use crate::ic_sui::ck_eddsa::KeyType;
use crate::state::{SuiToken, UpdateTokenStatus};
use crate::types::Ticket;
use crate::types::{Chain, Token};
use crate::{handler::gen_ticket::GenerateTicketReq, state::UpdateType};

const UPGRADES: MemoryId = MemoryId::new(0);
const CONFIG: MemoryId = MemoryId::new(1);
const TOKENS: MemoryId = MemoryId::new(2);
const SUI_TOKENS_INFO: MemoryId = MemoryId::new(3);
const UPDATE_TOKENS: MemoryId = MemoryId::new(4);
const TICKET_QUEUE: MemoryId = MemoryId::new(5);
const FAILED_TICKETS: MemoryId = MemoryId::new(6);
const COUNTERPARTIES: MemoryId = MemoryId::new(7);
const MINT_TOKEN_REQUESTS: MemoryId = MemoryId::new(8);
const GEN_TICKET_REQS: MemoryId = MemoryId::new(9);
const SEEDS: MemoryId = MemoryId::new(10);
const SUI_ADDRESSES: MemoryId = MemoryId::new(11);
const CLR_TICKET_QUEUE: MemoryId = MemoryId::new(12);
const BURN_TOKEN: MemoryId = MemoryId::new(13);

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

pub fn get_mint_token_requests_memory() -> Memory {
    with_memory_manager(|m| m.get(MINT_TOKEN_REQUESTS))
}

pub fn get_gen_ticket_req_memory() -> Memory {
    with_memory_manager(|m| m.get(GEN_TICKET_REQS))
}

pub fn get_seeds_memory() -> Memory {
    with_memory_manager(|m| m.get(SEEDS))
}

pub fn get_sui_tokens_memory() -> Memory {
    with_memory_manager(|m| m.get(SUI_TOKENS_INFO))
}

pub fn get_config_memory() -> Memory {
    with_memory_manager(|m| m.get(CONFIG))
}
pub fn get_sui_addresses_memory() -> Memory {
    with_memory_manager(|m| m.get(SUI_ADDRESSES))
}
pub fn get_clr_ticket_queue_memory() -> Memory {
    with_memory_manager(|m| m.get(CLR_TICKET_QUEUE))
}
pub fn get_burn_tokens_memory() -> Memory {
    with_memory_manager(|m| m.get(BURN_TOKEN))
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

pub fn init_update_tokens() -> StableBTreeMap<UpdateType, UpdateTokenStatus, Memory> {
    StableBTreeMap::init(get_update_tokens_memory())
}

pub fn init_mint_token_requests() -> StableBTreeMap<String, MintTokenRequest, Memory> {
    StableBTreeMap::init(get_mint_token_requests_memory())
}

pub fn init_gen_ticket_reqs() -> StableBTreeMap<String, GenerateTicketReq, Memory> {
    StableBTreeMap::init(get_gen_ticket_req_memory())
}

pub fn init_seed() -> StableBTreeMap<String, [u8; 64], Memory> {
    StableBTreeMap::init(get_seeds_memory())
}

pub fn init_sui_tokens() -> StableBTreeMap<String, SuiToken, Memory> {
    StableBTreeMap::init(get_sui_tokens_memory())
}

pub fn init_config() -> StableCell<SuiRouteConfig, Memory> {
    StableCell::init(get_config_memory(), SuiRouteConfig::default())
        .expect("failed to init sui route config")
}

pub fn init_sui_addresses() -> StableBTreeMap<KeyType, Vec<u8>, Memory> {
    StableBTreeMap::init(get_sui_addresses_memory())
}

pub fn init_clr_ticket_queue() -> StableBTreeMap<String, ClearTx, Memory> {
    StableBTreeMap::init(get_clr_ticket_queue_memory())
}

pub fn init_burn_tokens() -> StableBTreeMap<String, BurnTx, Memory> {
    StableBTreeMap::init(get_burn_tokens_memory())
}
