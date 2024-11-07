use std::cell::RefCell;

use ic_stable_structures::BTreeMap as StableBTreeMap;
use types::{OmnityAccount, TicketRecord, TicketRecordList, UpdateBalanceJob, UtxoRecord, UtxoRecordList};
use std::borrow::Cow;

use crate::*;

type Memory = VirtualMemory<DefaultMemoryImpl>;
const SETTINGS_MEMORY_ID: MemoryId = MemoryId::new(2);
const UTXO_RECORDS_MAP_MEMORY_ID: MemoryId = MemoryId::new(3);
const TICKET_RECORDS_MAP_MEMORY_ID: MemoryId = MemoryId::new(4);

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
    RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    static STATE: RefCell<Cell<State, Memory>> = RefCell::new(
        Cell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(SETTINGS_MEMORY_ID)),
            State {
                ckbtc_ledger_principal: Principal::anonymous(),
                ckbtc_minter_principal: Principal::anonymous(),
                icp_customs_principal: Principal::anonymous(),
                update_balances_jobs: vec![],
                is_timer_running: HashSet::new(),
                token_id: "".to_string(),
                target_chain_id: "".to_string()
            },
        ).expect("Failed to init cell for SETTINGS.")
    );

    static UTXO_RECORDS_MAP: RefCell<StableBTreeMap<OmnityAccount, UtxoRecordList, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(UTXO_RECORDS_MAP_MEMORY_ID)),
        )
    );

    static TICKET_RECORDS_MAP: RefCell<StableBTreeMap<OmnityAccount, TicketRecordList, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(TICKET_RECORDS_MAP_MEMORY_ID)),
        )
    );
}

#[derive(CandidType, Deserialize, Serialize, PartialEq, Eq, Clone, Debug)]
pub struct State {
    pub ckbtc_ledger_principal: Principal,
    pub ckbtc_minter_principal: Principal,
    pub icp_customs_principal: Principal,
    #[serde[default]]
    pub update_balances_jobs: Vec<UpdateBalanceJob>,
    #[serde[default]]
    pub is_timer_running: HashSet<String>,
    #[serde[default]]
    pub token_id: TokenId,
    #[serde[default]]
    pub target_chain_id: ChainId
}

impl Storable for State {
    fn to_bytes(&self) -> Cow<[u8]> {
        let mut bytes = vec![];
        let _ = ciborium::ser::into_writer(self, &mut bytes);
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        let settings =
            ciborium::de::from_reader(bytes.as_ref()).expect("failed to decode Settings");
        settings
    }

    const BOUND: Bound = Bound::Unbounded;
}

pub fn get_utxo_records(omnity_account: OmnityAccount) -> Vec<UtxoRecord> {
    UTXO_RECORDS_MAP.with(|c| {
        c.borrow()
            .get(&omnity_account)
            .unwrap_or_else(|| UtxoRecordList(vec![]))
            .0
    })
}

pub fn get_ticket_records(omnity_account: OmnityAccount) -> Vec<TicketRecord> {
    TICKET_RECORDS_MAP.with(|c| {
        c.borrow()
            .get(&omnity_account)
            .unwrap_or_else(|| TicketRecordList(vec![]))
            .0
    })
}

pub fn get_state() -> State {
    STATE.with(|c| c.borrow().get().clone())
}

pub fn set_state(state: State) {
    STATE.with(|c| {
        c.borrow_mut()
            .set(state.clone())
            .expect("Failed to set SETTINGS.")
    });
}

pub fn mutate_state<F, R>(f: F) -> R
where 
    F: FnOnce(&mut State)->R
{
    let mut state = get_state();
    let r = f(&mut state);
    set_state(state);
    r
}

pub fn insert_utxo_records(
    omnity_account: OmnityAccount,
    utxo_records: Vec<UtxoRecord>,
) -> Option<UtxoRecordList> {
    UTXO_RECORDS_MAP.with(|c| {
        c.borrow_mut()
            .insert(omnity_account, UtxoRecordList(utxo_records))
    })
}

pub fn insert_ticket_records(
    omnity_account: OmnityAccount,
    ticket_records: Vec<TicketRecord>,
) -> Option<TicketRecordList> {
    TICKET_RECORDS_MAP.with(|c| {
        c.borrow_mut()
            .insert(omnity_account, TicketRecordList(ticket_records))
    })
}

pub fn extend_ticket_records(omnity_account: OmnityAccount, ticket_records: Vec<TicketRecord>) {
    TICKET_RECORDS_MAP.with(|c| {
        let mut records = c
            .borrow()
            .get(&omnity_account)
            .unwrap_or_else(|| TicketRecordList(vec![]))
            .0;
        records.extend(ticket_records);
        c.borrow_mut()
            .insert(omnity_account, TicketRecordList(records))
    });
}