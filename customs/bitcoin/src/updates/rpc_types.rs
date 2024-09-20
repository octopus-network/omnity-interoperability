use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Transaction {
    pub vout: Vec<TxOut>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct TxOut {
    pub scriptpubkey_address: Option<String>,
    pub value: u64,
}
