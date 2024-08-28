use std::borrow::Cow;

use candid::CandidType;
use ic_stable_structures::Storable;
use ic_stable_structures::storable::Bound;
use serde_derive::{Deserialize, Serialize};

use omnity_types::{TicketId, Token, TokenId};

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



#[derive(candid::CandidType, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetricsStatus {
    pub latest_scan_interval_secs: u64,
    pub chainkey_addr_balance: u128,
}

#[derive(CandidType, Clone, Debug, Serialize, Deserialize)]
pub struct TokenResp {
    pub token_id: TokenId,
    pub symbol: String,
    pub decimals: u8,
    pub icon: Option<String>,
    pub rune_id: Option<String>,
    pub evm_contract: Option<String>,
}


impl From<Token> for TokenResp {
    fn from(value: Token) -> Self {
        TokenResp {
            token_id: value.token_id,
            symbol: value.symbol,
            decimals: value.decimals,
            icon: value.icon,
            rune_id: value.metadata.get("rune_id").cloned(),
            evm_contract: None,
        }
    }
}


