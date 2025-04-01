use crate::address::EvmAddress;
use serde_derive::{Deserialize, Serialize};
use tree_hash::fixed_bytes::B256;

#[derive(Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct LogEntry {
    pub address: EvmAddress,
    pub topics: Vec<B256>,
    pub data: Vec<u8>,
    pub block_number: Option<u64>,
    pub transaction_hash: B256,
    pub transaction_index: Option<u64>,
    pub block_hash: Option<B256>,
    pub log_index: Option<u64>,
    pub removed: bool,
}
