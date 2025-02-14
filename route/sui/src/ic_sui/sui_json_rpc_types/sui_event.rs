use crate::ic_sui::fastcrypto::encoding::{Base58, Base64};
use crate::ic_sui::move_core_types::language_storage::StructTag;
use crate::ic_sui::sui_types::Identifier;

use crate::ic_sui::sui_types::base_types::{ObjectID, SuiAddress, TransactionDigest};
// use crate::ic_sui::sui_types::error::SuiResult;
use crate::ic_sui::sui_types::event::{Event, EventEnvelope, EventID};
use crate::ic_sui::sui_types::sui_serde::BigInt;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use serde_with::{serde_as, DisplayFromStr};
use std::fmt;
use std::fmt::Display;

use json_to_table::json_to_table;
use tabled::settings::Style as TableStyle;

use super::Page;
use crate::ic_sui::sui_types::sui_serde::SuiStructTag;

pub type EventPage = Page<SuiEvent, EventID>;

#[serde_as]
#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
#[serde(rename = "Event", rename_all = "camelCase")]
pub struct SuiEvent {
    /// Sequential event ID, ie (transaction seq number, event seq number).
    /// 1) Serves as a unique event ID for each fullnode
    /// 2) Also serves to sequence events for the purposes of pagination and querying.
    ///    A higher id is an event seen later by that fullnode.
    /// This ID is the "cursor" for event querying.
    pub id: EventID,
    /// Move package where this event was emitted.
    pub package_id: ObjectID,
    #[serde_as(as = "DisplayFromStr")]
    /// Move module where this event was emitted.
    pub transaction_module: Identifier,
    /// Sender's Sui address.
    pub sender: SuiAddress,
    #[serde_as(as = "SuiStructTag")]
    /// Move event type.
    pub type_: StructTag,
    /// Parsed json value of the event
    pub parsed_json: Value,
    /// Base 58 encoded bcs bytes of the move event
    #[serde(flatten)]
    pub bcs: BcsEvent,
    /// UTC timestamp in milliseconds since epoch (1/1/1970)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde_as(as = "Option<BigInt<u64>>")]
    pub timestamp_ms: Option<u64>,
}

impl From<EventEnvelope> for SuiEvent {
    fn from(ev: EventEnvelope) -> Self {
        Self {
            id: EventID {
                tx_digest: ev.tx_digest,
                event_seq: ev.event_num,
            },
            package_id: ev.event.package_id,
            transaction_module: ev.event.transaction_module,
            sender: ev.event.sender,
            type_: ev.event.type_,
            parsed_json: ev.parsed_json,
            bcs: BcsEvent::Base64 {
                bcs: ev.event.contents,
            },
            timestamp_ms: Some(ev.timestamp),
        }
    }
}

impl From<SuiEvent> for Event {
    fn from(val: SuiEvent) -> Self {
        Event {
            package_id: val.package_id,
            transaction_module: val.transaction_module,
            sender: val.sender,
            type_: val.type_,
            contents: val.bcs.into_bytes(),
        }
    }
}

impl Display for SuiEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let parsed_json = &mut self.parsed_json.clone();
        bytes_array_to_base64(parsed_json);
        let mut table = json_to_table(parsed_json);
        let style = TableStyle::modern();
        table.collapse().with(style);
        write!(f,
            " ┌──\n │ EventID: {}:{}\n │ PackageID: {}\n │ Transaction Module: {}\n │ Sender: {}\n │ EventType: {}\n",
            self.id.tx_digest, self.id.event_seq, self.package_id, self.transaction_module, self.sender, self.type_)?;
        if let Some(ts) = self.timestamp_ms {
            writeln!(f, " │ Timestamp: {}\n └──", ts)?;
        }
        writeln!(f, " │ ParsedJSON:")?;
        let table_string = table.to_string();
        let table_rows = table_string.split_inclusive('\n');
        for r in table_rows {
            write!(f, " │   {r}")?;
        }

        write!(f, "\n └──")
    }
}

#[serde_as]
#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "bcsEncoding")]
#[serde(from = "MaybeTaggedBcsEvent")]
pub enum BcsEvent {
    Base64 {
        #[serde_as(as = "Base64")]
        bcs: Vec<u8>,
    },
    Base58 {
        #[serde_as(as = "Base58")]
        bcs: Vec<u8>,
    },
}

impl BcsEvent {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self::Base64 { bcs: bytes }
    }

    pub fn bytes(&self) -> &[u8] {
        match self {
            BcsEvent::Base64 { bcs } => bcs.as_ref(),
            BcsEvent::Base58 { bcs } => bcs.as_ref(),
        }
    }

    pub fn into_bytes(self) -> Vec<u8> {
        match self {
            BcsEvent::Base64 { bcs } => bcs,
            BcsEvent::Base58 { bcs } => bcs,
        }
    }
}

#[allow(unused)]
#[serde_as]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", untagged)]
enum MaybeTaggedBcsEvent {
    Tagged(TaggedBcsEvent),
    Base58 {
        #[serde_as(as = "Base58")]
        bcs: Vec<u8>,
    },
}

#[serde_as]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "bcsEncoding")]
enum TaggedBcsEvent {
    Base64 {
        #[serde_as(as = "Base64")]
        bcs: Vec<u8>,
    },
    Base58 {
        #[serde_as(as = "Base58")]
        bcs: Vec<u8>,
    },
}

impl From<MaybeTaggedBcsEvent> for BcsEvent {
    fn from(event: MaybeTaggedBcsEvent) -> BcsEvent {
        let bcs = match event {
            MaybeTaggedBcsEvent::Tagged(TaggedBcsEvent::Base58 { bcs })
            | MaybeTaggedBcsEvent::Base58 { bcs } => bcs,
            MaybeTaggedBcsEvent::Tagged(TaggedBcsEvent::Base64 { bcs }) => bcs,
        };

        // Bytes are already decoded, force into Base64 variant to avoid serializing to base58
        Self::Base64 { bcs }
    }
}

/// Convert a json array of bytes to Base64
fn bytes_array_to_base64(v: &mut Value) {
    match v {
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => (),
        Value::Array(vals) => {
            if let Some(vals) = vals.iter().map(try_into_byte).collect::<Option<Vec<_>>>() {
                *v = json!(Base64::from_bytes(&vals).encoded())
            } else {
                for val in vals {
                    bytes_array_to_base64(val)
                }
            }
        }
        Value::Object(map) => {
            for val in map.values_mut() {
                bytes_array_to_base64(val)
            }
        }
    }
}

/// Try to convert a json Value object into an u8.
fn try_into_byte(v: &Value) -> Option<u8> {
    let num = v.as_u64()?;
    (num <= 255).then_some(num as u8)
}

#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EventFilter {
    /// Return all events.
    All([Box<EventFilter>; 0]),

    /// Return events that match any of the given filters. Only supported on event subscriptions.
    Any(Vec<EventFilter>),

    /// Query by sender address.
    Sender(SuiAddress),
    /// Return events emitted by the given transaction.
    Transaction(
        ///digest of the transaction, as base-64 encoded string
        TransactionDigest,
    ),
    /// Return events emitted in a specified Move module.
    /// If the event is defined in Module A but emitted in a tx with Module B,
    /// query `MoveModule` by module B returns the event.
    /// Query `MoveEventModule` by module A returns the event too.
    MoveModule {
        /// the Move package ID
        package: ObjectID,
        /// the module name
        #[serde_as(as = "DisplayFromStr")]
        module: Identifier,
    },
    /// Return events with the given Move event struct name (struct tag).
    /// For example, if the event is defined in `0xabcd::MyModule`, and named
    /// `Foo`, then the struct tag is `0xabcd::MyModule::Foo`.
    MoveEventType(#[serde_as(as = "SuiStructTag")] StructTag),
    /// Return events with the given Move module name where the event struct is defined.
    /// If the event is defined in Module A but emitted in a tx with Module B,
    /// query `MoveEventModule` by module A returns the event.
    /// Query `MoveModule` by module B returns the event too.
    MoveEventModule {
        /// the Move package ID
        package: ObjectID,
        /// the module name

        #[serde_as(as = "DisplayFromStr")]
        module: Identifier,
    },
    /// Return events emitted in [start_time, end_time] interval
    #[serde(rename_all = "camelCase")]
    TimeRange {
        /// left endpoint of time interval, milliseconds since epoch, inclusive

        #[serde_as(as = "BigInt<u64>")]
        start_time: u64,
        /// right endpoint of time interval, milliseconds since epoch, exclusive

        #[serde_as(as = "BigInt<u64>")]
        end_time: u64,
    },
}

pub trait Filter<T> {
    fn matches(&self, item: &T) -> bool;
}
