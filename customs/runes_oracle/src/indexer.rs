use crate::tx::Transaction;
use ic_btc_interface::Txid;
use reqwest::StatusCode;

pub struct Indexer {
    url: String,
}

#[derive(Debug)]
pub enum IndexerError {
    RequestErr(reqwest::Error),
    ServerErr(StatusCode),
    JsonErr(serde_json::Error),
}

impl Indexer {
    pub fn new(url: String) -> Self {
        Self { url }
    }

    pub async fn get_transaction(&self, txid: Txid) -> Result<Transaction, IndexerError> {
        let resp = reqwest::get(format!("{}/api/tx/{}", self.url, txid))
            .await
            .map_err(IndexerError::RequestErr)?;
        match resp.status() {
            StatusCode::OK => {
                let text = resp.text().await.map_err(IndexerError::RequestErr)?;
                Ok(Transaction::from_json(text.as_str()).map_err(IndexerError::JsonErr)?)
            }
            code => Err(IndexerError::ServerErr(code)),
        }
    }
}
