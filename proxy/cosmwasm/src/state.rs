use std::borrow::Cow;

use ic_btc_interface::Utxo;
use ic_stable_structures::{
    memory_manager::{MemoryId, MemoryManager, VirtualMemory},
    storable::Bound,
    Cell, DefaultMemoryImpl, StableBTreeMap, Storable,
};

use crate::*;

type Memory = VirtualMemory<DefaultMemoryImpl>;

thread_local! {

    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
    RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    static CKBTC_LEDGER_PRINCIPAL: RefCell<Cell<Option<Principal>, Memory>> = RefCell::new(
        Cell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))),
            None,
        ).expect("Failed to init cell for CKBTC_LEDGER_PRINCIPAL.")
    );

    static ICP_CUSTOMS_PRINCIPAL: RefCell<Cell<Option<Principal>, Memory>> = RefCell::new(
        Cell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1))),
            None,
        ).expect("Failed to init cell for ICP_CUSTOMS_PRINCIPAL.")
    );

    static EXECUTED_TRANSACTIONS_INDEXES: RefCell<StableBTreeMap<u64, TicketId, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2))),
        )
    );

    static TRIGGER_PRINCIPAL: RefCell<Cell<Principal, Memory>> = RefCell::new(
        Cell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(3))),
            Principal::anonymous()
        ).expect("Failed to init cell for TRIGGER_PRINCIPAL.")
    );

    static CKBTC_MINTED_RECORDS_MAP: RefCell<StableBTreeMap<String, BtcTransportRecordList, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(4))),
        )
    );

    static CKBTC_MINTER_PRINCIPAL: RefCell<Cell<Option<Principal>, Memory>> = RefCell::new(
        Cell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(5))),
            None,
        ).expect("Failed to init cell for CKBTC_MINTER_PRINCIPAL.")
    );

}


pub fn get_btc_transport_records(osmosis_account_id: String) -> Vec<BtcTransportRecord> {
    CKBTC_MINTED_RECORDS_MAP.with(|c| {
        c.borrow()
            .get(&osmosis_account_id)
            .unwrap_or_else(|| BtcTransportRecordList(vec![]))
            .0
    })
}

pub fn insert_btc_transport_records(
    osmosis_account_id: String,
    minted_records: Vec<BtcTransportRecord>,
) {
    CKBTC_MINTED_RECORDS_MAP.with(|c| {
        c.borrow_mut()
            .insert(osmosis_account_id, BtcTransportRecordList(minted_records))
            .expect("Failed to insert CKBTC minted records.");
    });
}

#[derive(CandidType, Deserialize, Serialize, PartialEq, Eq, Clone, Debug)]
pub struct BtcCrossChainInfoList(pub Vec<BtcTransportInfo>);

#[derive(CandidType, Deserialize, Serialize, PartialEq, Eq, Clone, Debug)]
pub struct BtcTransportInfo {
    tx_hash: Utxo,
    block_index: u64,
    ticket_id: Option<TicketId>,
}

#[derive(CandidType, Deserialize, Serialize, PartialEq, Eq, Clone, Debug)]
pub struct BtcTransportRecordList(pub Vec<BtcTransportRecord>);

#[derive(CandidType, Deserialize, Serialize, PartialEq, Eq, Clone, Debug)]
pub struct BtcTransportRecord {
    pub block_index: u64,
    pub minted_amount: u64,
    pub utxo: Utxo,
    pub ticket_id: Option<TicketId>,
}

impl Storable for BtcTransportRecord {
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

impl Storable for BtcTransportRecordList {
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

impl Storable for BtcCrossChainInfoList {
    fn to_bytes(&self) -> Cow<[u8]> {
        let mut bytes = vec![];
        let _ = ciborium::ser::into_writer(self, &mut bytes);
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        let btc_cross_chain_info =
            ciborium::de::from_reader(bytes.as_ref()).expect("failed to decode BtcCrossChainInfo");
        btc_cross_chain_info
    }

    const BOUND: Bound = Bound::Unbounded;
}

pub fn contains_executed_transaction_index(index: u64) -> bool {
    EXECUTED_TRANSACTIONS_INDEXES.with(|c| c.borrow().contains_key(&index))
}

pub fn get_ticket_id_of_executed_transaction(index: u64) -> Option<TicketId> {
    EXECUTED_TRANSACTIONS_INDEXES.with(|c| c.borrow().get(&index))
}

pub fn insert_executed_transaction_index(index: u64, ticket_id: TicketId) {
    EXECUTED_TRANSACTIONS_INDEXES.with(|c| {
        c.borrow_mut()
            .insert(index, ticket_id)
            .expect("Failed to insert executed transaction index.")
    });
}

pub fn set_trigger_principal(principal: Principal) {
    TRIGGER_PRINCIPAL.with(|c| {
        c.borrow_mut()
            .set(principal.clone())
            .expect("Failed to set TRIGGER_PRINCIPAL.")
    });
}

pub fn get_trigger_principal() -> Principal {
    TRIGGER_PRINCIPAL.with(|c| c.borrow().get().clone())
}

pub fn set_ckbtc_index_principal(principal: Principal) {
    CKBTC_LEDGER_PRINCIPAL.with(|c| {
        c.borrow_mut()
            .set(Some(principal.clone()))
            .expect("Failed to set CKBTC_LEDGER_PRINCIPAL.")
    });
}

pub fn get_ckbtc_ledger_principal() -> Principal {
    CKBTC_LEDGER_PRINCIPAL.with(|c| {
        c.borrow()
            .get()
            .expect("CKBTC_LEDGER_PRINCIPAL not initialized!")
            .clone()
    })
}

pub fn set_ckbtc_minter_principal(principal: Principal) {
    CKBTC_MINTER_PRINCIPAL.with(|c| {
        c.borrow_mut()
            .set(Some(principal.clone()))
            .expect("Failed to set CKBTC_MINTER_PRINCIPAL.")
    });
}

pub fn get_ckbtc_minter_principal() -> Principal {
    CKBTC_MINTER_PRINCIPAL.with(|c| {
        c.borrow()
            .get()
            .expect("CKBTC_MINTER_PRINCIPAL not initialized!")
            .clone()
    })
}

pub fn set_icp_customs_principal(principal: Principal) {
    ICP_CUSTOMS_PRINCIPAL.with(|c| {
        c.borrow_mut()
            .set(Some(principal.clone()))
            .expect("Failed to set ICP_CUSTOM_PRINCIPAL.")
    });
}

pub fn get_icp_custom_principal() -> Principal {
    ICP_CUSTOMS_PRINCIPAL.with(|c| {
        c.borrow()
            .get()
            .expect("ICP_CUSTOM_PRINCIPAL not initialized!")
            .clone()
    })
}
