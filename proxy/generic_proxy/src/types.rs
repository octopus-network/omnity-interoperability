use std::borrow::Cow;

use crate::*;
use omnity_types::TicketId;
use utils::{get_chain_time_seconds, sha256};
use ic_btc_interface::Utxo;

#[derive(PartialOrd, Ord, CandidType, Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct OmnityAccount {
    pub chain_id: ChainId,
    pub account: Account,
}

impl OmnityAccount {
    pub fn new(chain_id: ChainId, account: Account) -> Self {
        OmnityAccount { chain_id, account }
    }

    pub fn get_mapping_subaccount(&self) -> Subaccount {
        sha256(
            format!("{}#{}", self.chain_id, self.account)
                .to_string()
                .as_bytes(),
        )
    }
}

impl Storable for OmnityAccount {
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

#[derive(CandidType, Deserialize, Serialize, PartialEq, Eq, Clone, Debug)]
pub struct UpdateBalanceJob {
    pub omnity_account: OmnityAccount,
    pub failed_times: u32,
    pub next_execute_time: u64,
}

impl UpdateBalanceJob {
    const MAX_FAILED_TIMES: u32 = 20;
    const INIT_DELAY: u64 = 60 * 30;
    const FAILED_DELAY: u64 = 60 * 3;
    pub fn new(omnity_account: OmnityAccount) -> Self {
        UpdateBalanceJob {
            omnity_account,
            failed_times: 0,
            next_execute_time: get_chain_time_seconds() + UpdateBalanceJob::INIT_DELAY,
        }
    }

    pub fn executable(&self) -> bool {
        get_chain_time_seconds() >= self.next_execute_time
    }

    pub fn handle_execute_failed_and_continue(&mut self) -> bool {
        self.failed_times += 1;
        if self.failed_times >= UpdateBalanceJob::MAX_FAILED_TIMES {
            return false;
        }
        self.next_execute_time = get_chain_time_seconds() + UpdateBalanceJob::FAILED_DELAY;
        return true;
    }
}

#[derive(CandidType, Deserialize, Serialize, PartialEq, Eq, Clone, Debug)]
pub struct MintedUtxo {
    pub block_index: u64,
    pub minted_amount: u64,
    pub utxo: Utxo,
}

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

#[derive(CandidType, Deserialize, Serialize, PartialEq, Eq, Clone, Debug)]
pub struct UtxoRecordList(pub Vec<UtxoRecord>);

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
pub struct TicketRecord {
    pub ticket_id: TicketId,
    pub minted_utxos: Vec<MintedUtxo>,
}

#[derive(CandidType, Deserialize, Serialize, PartialEq, Eq, Clone, Debug)]
pub struct TicketRecordList(pub Vec<TicketRecord>);

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