use crate::lightclient::rpc_types::log::LogEntry;
use candid::CandidType;
use rlp::RlpStream;
use serde_derive::{Deserialize, Serialize};
use tree_hash::fixed_bytes::LogBloom;

#[derive(Clone, Debug, PartialEq, Serialize, CandidType, Eq, Deserialize)]
pub struct TransactionReceipt {
    #[serde(rename = "blockHash")]
    pub block_hash: String,
    #[serde(rename = "blockNumber")]
    pub block_number: String,
    #[serde(rename = "gasUsed")]
    pub gas_used: String,

    #[serde(
        default,
        rename = "cumulativeGasUsed",
        with = "crate::lightclient::rpc_types::serde_u64::u64"
    )]
    pub cumulative_gas_used: u64,
    #[serde(with = "crate::lightclient::rpc_types::serde_u64::opt_u64")]
    pub status: Option<u64>,
    #[serde(rename = "transactionHash")]
    pub transaction_hash: String,
    #[serde(rename = "contractAddress")]
    pub contract_address: Option<String>,
    pub from: String,
    pub logs: Vec<LogEntry>,
    #[serde(rename = "logsBloom")]
    pub logs_bloom: LogBloom,
    pub to: String,
    #[serde(rename = "transactionIndex")]
    pub transaction_index: String,
    #[serde(with = "crate::lightclient::rpc_types::serde_u64::opt_u64")]
    pub r#type: Option<u64>,
}

pub fn encode_receipt(receipt: &TransactionReceipt) -> Vec<u8> {
    let mut stream = RlpStream::new();
    stream.begin_list(4);
    stream.append(&receipt.status.unwrap());
    stream.append(&receipt.cumulative_gas_used);
    stream.append(&receipt.logs_bloom);
    stream.append_list(&receipt.logs);
    let legacy_receipt_encoded = stream.out();
    let tx_type = receipt.r#type.unwrap();
    match tx_type {
        0 => legacy_receipt_encoded.to_vec(),
        _ => [&tx_type.to_be_bytes()[7..8], &legacy_receipt_encoded].concat(),
    }
}
