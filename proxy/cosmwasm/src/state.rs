use std::{borrow::Cow, collections::{HashSet, VecDeque}};

use ic_btc_interface::Utxo;
use ic_stable_structures::{
    memory_manager::{MemoryId, MemoryManager, VirtualMemory},
    storable::Bound,
    Cell, DefaultMemoryImpl, StableBTreeMap, Storable
};
use omnity_types::{ChainId, TokenId};

use crate::*;

type Memory = VirtualMemory<DefaultMemoryImpl>;

const LOG_MEMORY_ID: MemoryId = MemoryId::new(1);
const SETTINGS_MEMORY_ID: MemoryId = MemoryId::new(2);
const UTXO_RECORDS_MAP_MEMORY_ID: MemoryId = MemoryId::new(3);
const TICKET_RECORDS_MAP_MEMORY_ID: MemoryId = MemoryId::new(4);
const SCHDULED_OSMOSIS_ACCOUNT_QUEUE_MEMORY_ID: MemoryId = MemoryId::new(5);

thread_local! {

    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
    RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    static SETTINGS: RefCell<Cell<Settings, Memory>> = RefCell::new(
        Cell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(SETTINGS_MEMORY_ID)),
            Settings {
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

    static UTXO_RECORDS_MAP: RefCell<StableBTreeMap<String, UtxoRecordList, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(UTXO_RECORDS_MAP_MEMORY_ID)),
        )
    );

    static TICKET_RECORDS_MAP: RefCell<StableBTreeMap<TicketId, TicketRecordList, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(TICKET_RECORDS_MAP_MEMORY_ID)),
        )
    );

    static SCHDULED_OSMOSIS_ACCOUNT_QUEUE: RefCell<Cell<SchduledOsmosisAccountList, Memory>> = RefCell::new(
        Cell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(SCHDULED_OSMOSIS_ACCOUNT_QUEUE_MEMORY_ID)),
            SchduledOsmosisAccountList(VecDeque::default())
        ).expect("Failed to init cell for UPDATE_OSMOSIS_ACCOUNT_QUEUE.")
    );

}

pub fn init_stable_log() -> StableBTreeMap<Vec<u8>, Vec<u8>, Memory> {
    StableBTreeMap::init(MEMORY_MANAGER.with(|m| m.borrow().get(LOG_MEMORY_ID)))
}

pub fn get_utxo_records(osmosis_account_id: String) -> Vec<UtxoRecord> {
    UTXO_RECORDS_MAP.with(|c| {
        c.borrow()
            .get(&osmosis_account_id)
            .unwrap_or_else(|| UtxoRecordList(vec![]))
            .0
    })
}

pub fn get_ticket_records(osmosis_account_id: String) -> Vec<TicketRecord> {
    TICKET_RECORDS_MAP.with(|c| {
        c.borrow()
            .get(&osmosis_account_id)
            .unwrap_or_else(|| TicketRecordList(vec![]))
            .0
    })
}

pub fn insert_utxo_records(
    osmosis_account_id: String,
    utxo_records: Vec<UtxoRecord>,
) -> Option<UtxoRecordList> {
    UTXO_RECORDS_MAP.with(|c| {
        c.borrow_mut()
            .insert(osmosis_account_id, UtxoRecordList(utxo_records))
    })
}

pub fn insert_ticket_records(
    osmosis_account_id: TicketId,
    ticket_records: Vec<TicketRecord>,
) -> Option<TicketRecordList> {
    TICKET_RECORDS_MAP.with(|c| {
        c.borrow_mut()
            .insert(osmosis_account_id, TicketRecordList(ticket_records))
    })
}

pub fn extend_ticket_records(osmosis_account_id: TicketId, ticket_records: Vec<TicketRecord>) {
    TICKET_RECORDS_MAP.with(|c| {
        let mut records = c
            .borrow()
            .get(&osmosis_account_id)
            .unwrap_or_else(|| TicketRecordList(vec![]))
            .0;
        records.extend(ticket_records);
        c.borrow_mut()
            .insert(osmosis_account_id, TicketRecordList(records))
    });
}

pub fn push_scheduled_osmosis_account_id(osmosis_account_id: String)->Result<SchduledOsmosisAccountList> {
    SCHDULED_OSMOSIS_ACCOUNT_QUEUE.with(|c| {
        let mut queue = c.borrow().get().0.clone();
        queue.push_back(osmosis_account_id);
        c.borrow_mut()
            .set(SchduledOsmosisAccountList(queue))
            .map_err(|e| Errors::CustomError(format!("Failed to set SCHDULED_OSMOSIS_ACCOUNT_QUEUE. {:?}", e)))
            // .expect("Failed to set SCHDULED_OSMOSIS_ACCOUNT_QUEUE.")
    })
}

pub fn pop_first_scheduled_osmosis_account_id() -> Result<Option<String>> {
    SCHDULED_OSMOSIS_ACCOUNT_QUEUE.with(|c| {
        let mut queue = c.borrow().get().0.clone();
        let osmosis_account_id = queue.pop_front();
        c.borrow_mut()
            .set(SchduledOsmosisAccountList(queue))
            .map_err(|e| Errors::CustomError(format!("Failed to set SCHDULED_OSMOSIS_ACCOUNT_QUEUE. {:?}", e)))
            .map(|_| osmosis_account_id)
    })
}

pub fn get_scheduled_osmosis_account_id_list() -> Vec<String> {
    SCHDULED_OSMOSIS_ACCOUNT_QUEUE.with(|c| c.borrow().get().0.clone().into())
}

#[derive(CandidType, Deserialize, Serialize, PartialEq, Eq, Clone, Debug)]
pub struct MintedUtxo {
    pub block_index: u64,
    pub minted_amount: u64,
    pub utxo: Utxo,
}

#[derive(Deserialize, Serialize, PartialEq, Eq, Clone, Debug)]
pub struct SchduledOsmosisAccountList(pub VecDeque<String>);

impl Storable for SchduledOsmosisAccountList {
    fn to_bytes(&self) -> Cow<[u8]> {
        let mut bytes = vec![];
        let _ = ciborium::ser::into_writer(self, &mut bytes);
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        let schduled_osmosis_account_list = ciborium::de::from_reader(bytes.as_ref())
            .expect("failed to decode SchduledOsmosisAccountList");
        schduled_osmosis_account_list
    }

    const BOUND: Bound = Bound::Unbounded;
}

#[derive(CandidType, Deserialize, Serialize, PartialEq, Eq, Clone, Debug)]
pub struct TicketRecordList(pub Vec<TicketRecord>);

#[derive(CandidType, Deserialize, Serialize, PartialEq, Eq, Clone, Debug)]
pub struct TicketRecord {
    pub ticket_id: TicketId,
    pub minted_utxos: Vec<MintedUtxo>,
}

impl Storable for TicketRecord {
    fn to_bytes(&self) -> Cow<[u8]> {
        let mut bytes = vec![];
        let _ = ciborium::ser::into_writer(self, &mut bytes);
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        let ticket_record =
            ciborium::de::from_reader(bytes.as_ref()).expect("failed to decode TicketRecord");
        ticket_record
    }

    const BOUND: Bound = Bound::Unbounded;
}

impl Storable for TicketRecordList {
    fn to_bytes(&self) -> Cow<[u8]> {
        let mut bytes = vec![];
        let _ = ciborium::ser::into_writer(self, &mut bytes);
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        let ticket_record_list =
            ciborium::de::from_reader(bytes.as_ref()).expect("failed to decode TicketRecordList");
        ticket_record_list
    }

    const BOUND: Bound = Bound::Unbounded;
}

#[derive(CandidType, Deserialize, Serialize, PartialEq, Eq, Clone, Debug)]
pub struct UtxoRecordList(pub Vec<UtxoRecord>);

#[derive(CandidType, Deserialize, Serialize, PartialEq, Eq, Clone, Debug)]
pub struct UtxoRecord {
    pub minted_utxo: MintedUtxo,
    pub ticket_id: Option<TicketId>,
}

impl Storable for UtxoRecord {
    fn to_bytes(&self) -> Cow<[u8]> {
        let mut bytes = vec![];
        let _ = ciborium::ser::into_writer(self, &mut bytes);
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        let ck_btc_minted_record =
            ciborium::de::from_reader(bytes.as_ref()).expect("failed to decode CkBtcMintedRecord");
        ck_btc_minted_record
    }

    const BOUND: Bound = Bound::Unbounded;
}

impl Storable for UtxoRecordList {
    fn to_bytes(&self) -> Cow<[u8]> {
        let mut bytes = vec![];
        let _ = ciborium::ser::into_writer(self, &mut bytes);
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        let ck_btc_minted_record_list = ciborium::de::from_reader(bytes.as_ref())
            .expect("failed to decode CkBtcMintedRecordList");
        ck_btc_minted_record_list
    }

    const BOUND: Bound = Bound::Unbounded;
}
#[derive(CandidType, Deserialize, Serialize, PartialEq, Eq, Clone, Debug)]
pub struct Settings {
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

impl Storable for Settings {
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

pub fn get_settings() -> Settings {
    SETTINGS.with(|c| c.borrow().get().clone())
}

pub fn set_settings(settings: Settings) {
    SETTINGS.with(|c| {
        c.borrow_mut()
            .set(settings.clone())
            .expect("Failed to set SETTINGS.")
    });
}

pub fn mutate_settings<F, R>(f: F) -> R
where 
    F: FnOnce(&mut Settings)->R
{
    let mut settings = get_settings();
    let r = f(&mut settings);
    set_settings(settings);
    r
}
