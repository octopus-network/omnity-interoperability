use crate::runes_etching::transactions::SendEtchingRequest;
use crate::state::eventlog::Event;
use ic_stable_structures::{
    log::{Log as StableLog, NoSuchEntry},
    memory_manager::{MemoryId, MemoryManager, VirtualMemory},
    DefaultMemoryImpl, StableBTreeMap, StableVec,
};
use std::cell::RefCell;
use std::ops::Deref;

const LOG_INDEX_MEMORY_ID: MemoryId = MemoryId::new(0);
const LOG_DATA_MEMORY_ID: MemoryId = MemoryId::new(1);
const ETCHING_FEE_UTXOS_MEMORY_ID: MemoryId = MemoryId::new(50);
const PENDING_ETCHING_REQUESTS_MEMORY_ID: MemoryId = MemoryId::new(51);
const FINALIZED_ETCHING_REQUESTS_MEMORY_ID: MemoryId = MemoryId::new(52);

pub type VMem = VirtualMemory<DefaultMemoryImpl>;
type EventLog = StableLog<Vec<u8>, VMem, VMem>;

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    /// The log of the customs state modifications.
    static EVENTS: RefCell<EventLog> = MEMORY_MANAGER
        .with(|m|
              RefCell::new(
                  StableLog::init(
                      m.borrow().get(LOG_INDEX_MEMORY_ID),
                      m.borrow().get(LOG_DATA_MEMORY_ID)
                  ).expect("failed to initialize stable log")
              )
        );
}

fn with_memory_manager<R>(f: impl FnOnce(&MemoryManager<DefaultMemoryImpl>) -> R) -> R {
    MEMORY_MANAGER.with(|cell| f(cell.borrow().deref()))
}

pub struct EventIterator {
    buf: Vec<u8>,
    pos: u64,
}

impl Iterator for EventIterator {
    type Item = Event;

    fn next(&mut self) -> Option<Event> {
        EVENTS.with(|events| {
            let events = events.borrow();

            match events.read_entry(self.pos, &mut self.buf) {
                Ok(()) => {
                    self.pos = self.pos.saturating_add(1);
                    Some(decode_event(&self.buf))
                }
                Err(NoSuchEntry) => None,
            }
        })
    }

    fn nth(&mut self, n: usize) -> Option<Event> {
        self.pos = self.pos.saturating_add(n as u64);
        self.next()
    }
}

/// Encodes an event into a byte array.
fn encode_event(event: &Event) -> Vec<u8> {
    let mut buf = Vec::new();
    ciborium::ser::into_writer(event, &mut buf).expect("failed to encode a customs event");
    buf
}

/// # Panics
///
/// This function panics if the event decoding fails.
fn decode_event(buf: &[u8]) -> Event {
    ciborium::de::from_reader(buf).expect("failed to decode a customs event")
}

/// Returns an iterator over all customs events.
pub fn events() -> impl Iterator<Item = Event> {
    EventIterator {
        buf: vec![],
        pos: 0,
    }
}

/// Returns the current number of events in the log.
pub fn count_events() -> u64 {
    EVENTS.with(|events| events.borrow().len())
}

/// Records a new customs event.
pub fn record_event(event: &Event) {
    let bytes = encode_event(event);
    EVENTS.with(|events| {
        events
            .borrow()
            .append(&bytes)
            .expect("failed to append an entry to the event log")
    });
}

pub fn init_etching_fee_utxos() -> StableVec<crate::runes_etching::Utxo, VMem> {
    StableVec::init(with_memory_manager(|m| m.get(ETCHING_FEE_UTXOS_MEMORY_ID))).unwrap()
}

pub fn init_pending_etching_requests() -> StableBTreeMap<String, SendEtchingRequest, VMem> {
    StableBTreeMap::init(with_memory_manager(|m| {
        m.get(PENDING_ETCHING_REQUESTS_MEMORY_ID)
    }))
}

pub fn init_finalized_etching_requests() -> StableBTreeMap<String, SendEtchingRequest, VMem> {
    StableBTreeMap::init(with_memory_manager(|m| {
        m.get(FINALIZED_ETCHING_REQUESTS_MEMORY_ID)
    }))
}
