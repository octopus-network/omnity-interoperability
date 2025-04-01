use candid_derive::CandidType;
use ic_stable_structures::storable::Bound;
use ic_stable_structures::Storable;
use omnity_types::TicketId;
use serde_derive::{Deserialize, Serialize};
use std::borrow::Cow;

#[derive(candid::CandidType, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetricsStatus {
    pub latest_scan_interval_secs: u64,
    pub chainkey_addr_balance: u128,
}

#[derive(CandidType, Deserialize, Serialize, PartialEq, Eq, Clone, Debug)]
pub struct PendingTicketStatus {
    pub evm_tx_hash: Option<String>,
    pub ticket_id: TicketId,
    pub seq: u64,
    pub error: Option<String>,
}

#[derive(CandidType, Deserialize, Serialize, PartialEq, Eq, Clone, Debug)]
pub struct PendingDirectiveStatus {
    pub evm_tx_hash: Option<String>,
    pub seq: u64,
    pub error: Option<String>,
}

impl Storable for PendingDirectiveStatus {
    fn to_bytes(&self) -> Cow<[u8]> {
        let mut bytes = vec![];
        let _ = ciborium::ser::into_writer(self, &mut bytes);
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        let pds = ciborium::de::from_reader(bytes.as_ref())
            .expect("failed to decode pending ticket status");
        pds
    }

    const BOUND: Bound = Bound::Unbounded;
}

impl Storable for PendingTicketStatus {
    fn to_bytes(&self) -> Cow<[u8]> {
        let mut bytes = vec![];
        let _ = ciborium::ser::into_writer(self, &mut bytes);
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        let pts = ciborium::de::from_reader(bytes.as_ref())
            .expect("failed to decode pending ticket status");
        pts
    }

    const BOUND: Bound = Bound::Unbounded;
}
