use std::borrow::Cow;
use candid::{CandidType, Deserialize};
use ic_btc_interface::{Txid, Utxo};
use ic_stable_structures::Storable;
use ic_stable_structures::storable::Bound;
use serde::Serialize;
use omnity_types::{TicketId, TokenId};
use omnity_types::rune_id::RuneId;

pub type Brc20Ticker = String;

#[derive(CandidType, Deserialize, Serialize, PartialEq, Eq, Clone, Debug)]
pub struct PendingTicketStatus {
    pub bitcoin_tx_hash: Option<String>,
    pub ticket_id: TicketId,
    pub seq: u64,
    pub error: Option<String>,
}

#[derive(CandidType, Deserialize, Serialize, PartialEq, Eq, Clone, Debug)]
pub struct PendingDirectiveStatus {
    pub bitcoin_tx_hash: Option<String>,
    pub seq: u64,
    pub error: Option<String>,
}

#[derive(CandidType, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GenTicketStatus {
    /// The custom has no data for this request.
    /// The request is either invalid or too old.
    Unknown,
    /// The request is in the queue.
    Pending(GenTicketRequest),
    Confirmed(GenTicketRequest),
    Finalized(GenTicketRequest),
}


#[derive(candid::CandidType, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenTicketRequest {
    pub target_chain_id: String,
    pub receiver: String,
    pub token_id: TokenId,
    pub ticker: Brc20Ticker,
    pub amount: u128,
    pub txid: Txid,
    pub received_at: u64,
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
