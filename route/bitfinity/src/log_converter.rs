use did::transaction::TransactionReceiptLog;
use tree_hash::fixed_bytes::B256;
use ethereum_common::address::EvmAddress;
use ethereum_common::evm_log::LogEntry;


pub fn transform_transaction_log(log: &TransactionReceiptLog) -> LogEntry {
    ethereum_common::evm_log::LogEntry {
        address: EvmAddress(log.address.0.0),
        topics: log.topics.iter().map(|h|B256::from_slice(h.0.as_bytes())).collect(),
        data: log.data.0.to_vec(),
        block_number: Some(log.block_number.0.0[0]),
        transaction_hash: B256::from_slice(log.transaction_hash.0.as_bytes()),
        transaction_index: Option::from(log.transaction_index.0.0[0]),
        block_hash: Option::from(B256::from_slice(log.block_hash.0.as_bytes())),
        log_index: Option::from(log.log_index.0.as_u64()),
        removed: log.removed,
    }
}