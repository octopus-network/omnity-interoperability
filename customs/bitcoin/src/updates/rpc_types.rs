use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Transaction {
    pub vout: Vec<TxOut>,
    pub status: TxStatus,
}

#[derive(Deserialize, Debug, Clone)]
pub struct TxOut {
    pub scriptpubkey_address: Option<String>,
    pub value: u64,
}

#[derive(Deserialize, Debug, Clone)]
pub struct TxStatus {
    pub block_height: u32,
    pub block_time: u64,
}
