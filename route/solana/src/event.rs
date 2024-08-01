use crate::types::{Chain, Factor, ToggleState, Token};
use crate::{
    handler::ticket::GenerateTicketReq,
    lifecycle::{InitArgs, UpgradeArgs},
    memory::{init_event, Memory},
};

use ic_stable_structures::log::{Log, NoSuchEntry};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;

type EventLog = Log<Vec<u8>, Memory, Memory>;

thread_local! {
    // The event storage
    static EVENTS: RefCell<EventLog> =  RefCell::new(init_event())
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

#[derive(candid::CandidType, Deserialize)]
pub struct GetEventsArg {
    pub start: u64,
    pub length: u64,
}

#[derive(candid::CandidType, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Event {
    /// Indicates the route initialization with the specified arguments.  Must be
    /// the first event in the event log.
    #[serde(rename = "init")]
    Init(InitArgs),

    #[serde(rename = "upgrade")]
    Upgrade(UpgradeArgs),

    #[serde(rename = "added_chain")]
    AddedChain(Chain),

    #[serde(rename = "added_token")]
    AddedToken(Token),

    #[serde(rename = "updated_fee")]
    UpdatedFee { fee: Factor },

    #[serde(rename = "toggle_chain_state")]
    ToggleChainState(ToggleState),

    #[serde(rename = "finalized_mint_token")]
    FinalizedMintToken {
        ticket_id: String,
        signature: String,
    },
    #[serde(rename = "finalized_gen_ticket")]
    FinalizedGenTicket {
        ticket_id: String,
        request: GenerateTicketReq,
    },
}
