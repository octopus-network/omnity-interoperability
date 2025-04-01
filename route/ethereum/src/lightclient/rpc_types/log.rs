use crate::lightclient::rpc_types::address::Address;
use bytes::Bytes;
use candid::CandidType;
use ethereum_common::address::EvmAddress;
use rlp::{Encodable, RlpStream};
use serde_derive::{Deserialize, Serialize};
use tree_hash::fixed_bytes::B256;

#[derive(Debug, Clone, Deserialize, CandidType, Serialize, PartialEq, Eq)]
#[serde(transparent)]
pub struct Data(#[serde(with = "crate::lightclient::rpc_types::serde_data")] pub Vec<u8>);

#[derive(Debug, CandidType, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct LogEntry {
    pub address: Address,

    pub topics: Vec<B256>,
    /// Contains one or more 32-byte non-indexed log arguments.
    pub data: Data,
    /// The block number in which this log appeared.
    /// None if the block is pending.
    #[serde(
        rename = "blockNumber",
        with = "crate::lightclient::rpc_types::serde_u64::opt_u64"
    )]
    pub block_number: Option<u64>,
    // 32 Bytes - hash of the transactions from which this log was created.
    // None when its pending log.
    #[serde(rename = "transactionHash")]
    pub transaction_hash: Option<B256>,
    // Integer of the transactions position within the block the log was created from.
    // None if the log is pending.
    #[serde(
        rename = "transactionIndex",
        with = "crate::lightclient::rpc_types::serde_u64::opt_u64"
    )]
    pub transaction_index: Option<u64>,
    /// 32 Bytes - hash of the block in which this log appeared.
    /// None if the block is pending.
    #[serde(rename = "blockHash")]
    pub block_hash: Option<B256>,
    /// Integer of the log index position in the block.
    /// None if the log is pending.
    #[serde(
        rename = "logIndex",
        with = "crate::lightclient::rpc_types::serde_u64::opt_u64"
    )]
    pub log_index: Option<u64>,
    /// "true" when the log was removed due to a chain reorganization.
    /// "false" if it's a valid log.
    #[serde(default)]
    pub removed: bool,
}

impl Into<ethereum_common::evm_log::LogEntry> for LogEntry {
    fn into(self) -> ethereum_common::evm_log::LogEntry {
        ethereum_common::evm_log::LogEntry {
            address: EvmAddress(self.address.0 .0),
            topics: self.topics,
            data: self.data.0,
            block_number: self.block_number,
            transaction_hash: self.transaction_hash.unwrap_or_default(),
            transaction_index: self.transaction_index,
            block_hash: self.block_hash,
            log_index: self.log_index,
            removed: self.removed,
        }
    }
}

impl Encodable for LogEntry {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(3);
        s.append(&self.address);
        s.append_list(&self.topics);
        let by = Bytes::copy_from_slice(&self.data.0);
        s.append(&by);
    }
}
