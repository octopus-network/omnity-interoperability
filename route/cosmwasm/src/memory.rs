use cosmrs::AccountId;
use ic_cdk_timers::TimerId;
use ic_stable_structures::{
    memory_manager::{MemoryId, MemoryManager, VirtualMemory},
    DefaultMemoryImpl,
};
use ic_stable_structures::{Cell, StableBTreeMap};
use omnity_types::TicketId;
use std::{cell::RefCell, collections::{HashMap, HashSet}};

use crate::{cosmwasm::TxHash, get_chain_time_seconds, JobName};
use crate::RouteState;

const LOG_MEMORY_ID: MemoryId = MemoryId::new(2);
const REDEEM_TICKETS_MEMORY_ID: MemoryId = MemoryId::new(3);
const ROUTE_STATE_MEMORY_ID: MemoryId = MemoryId::new(4);

type InnerMemory = DefaultMemoryImpl;

pub type Memory = VirtualMemory<InnerMemory>;



thread_local! {
    // static MEMORY: RefCell<Option<InnerMemory>> = RefCell::new(Some(InnerMemory::default()));

    pub static PERIODIC_JOB_MANAGER_MAP: RefCell<HashMap<JobName, PeriodicJobManager>> = RefCell::default();

    pub static GUARD_RUNNING_TASK: RefCell<HashSet<String>> = RefCell::default();

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

pub fn insert_periodic_job_manager(job_name: JobName, periodic_job_manager: PeriodicJobManager) {
    PERIODIC_JOB_MANAGER_MAP.with(|p| p.borrow_mut().insert(job_name, periodic_job_manager));
}

pub fn get_periodic_job_manager(job_name: &str) -> Option<PeriodicJobManager> {
    PERIODIC_JOB_MANAGER_MAP.with(|p| p.borrow().get(job_name).map(|e| e.clone()))
}

pub fn mutate_periodic_job_manager<F, R>(job_name: &str, f: F) -> R
where
    F: FnOnce(&mut PeriodicJobManager) -> R,
{
    let mut periodic_job_manager = get_periodic_job_manager(job_name).expect("Failed to get periodic job manager");
    let r = f(&mut periodic_job_manager);
    PERIODIC_JOB_MANAGER_MAP.with(|p| p.borrow_mut().insert(job_name.to_string(), periodic_job_manager));
    r
}

#[derive(Debug, Clone)]
pub struct PeriodicJobManager {
    pub job_name: JobName,
    pub timer_id: TimerId,
    pub is_running: bool,
    pub create_time: u64,
    pub last_execute_time: u64,
    pub failed_times: u32,
    pub next_execute_time: u64, 
    pub job_interval: u64,
}

impl PeriodicJobManager {
    pub fn new(
        job_name: JobName, 
        timer_id: TimerId,
        job_interval: u64, 
    )->PeriodicJobManager{
        Self {
            job_name,
            timer_id: timer_id,
            is_running: false,
            create_time: get_chain_time_seconds(),
            last_execute_time: 0,
            failed_times: 0,
            next_execute_time: get_chain_time_seconds() + job_interval,
            job_interval: job_interval,
        }
    }

    pub fn job_execute_success(&mut self) {
        self.failed_times = 0;
        self.last_execute_time = get_chain_time_seconds();
        self.next_execute_time = get_chain_time_seconds() + self.job_interval;
        self.is_running = false;
    }
    pub fn job_execute_failed(&mut self) {
        self.failed_times += 1;
        self.last_execute_time = get_chain_time_seconds();
        self.next_execute_time = get_chain_time_seconds() + 2_u64.pow(self.failed_times) * self.job_interval;
        self.is_running = false;
    }

    pub fn should_execute(&self) -> bool {
        self.is_running && get_chain_time_seconds() >= self.next_execute_time
    }
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

pub fn mutate_guard_running_task<F, R>(f: F) -> R
where
    F: FnOnce(&mut HashSet<String>) -> R,
{
    GUARD_RUNNING_TASK.with(|g| f(&mut *g.borrow_mut()))
}

pub fn get_contract_id() -> AccountId {
    read_state(|state| state.cw_port_contract_address.clone())
        .parse()
        .unwrap()
}
