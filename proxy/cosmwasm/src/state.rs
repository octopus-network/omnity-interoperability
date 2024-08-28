use ic_stable_structures::{
    memory_manager::{MemoryId, MemoryManager, VirtualMemory},
    Cell, DefaultMemoryImpl, StableBTreeMap,
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

// pub fn set_target_chain_id(target_chain_id: String) {
//     TARGET_CHAIN_ID.with(|c| {
//         c.borrow_mut()
//             .set(Some(target_chain_id.clone()))
//             .expect("Failed to set TARGET_CHAIN_ID.")
//     });
// }

// pub fn get_target_chain_id() -> String {
//     TARGET_CHAIN_ID.with(|c| {
//         c.borrow()
//             .get()
//             .expect("TARGET_CHAIN_ID not initialized!")
//             .clone()
//     })
// }
