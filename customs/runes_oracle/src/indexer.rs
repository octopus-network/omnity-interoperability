use crate::{tx::Transaction, Runes};
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
        let resp = reqwest::get(format!("{}/api/rest/tx/{}", self.url, txid))
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

    pub async fn get_runes(&self, rune_id: &String) -> Result<Runes, IndexerError> {
        let resp = reqwest::get(format!("{}/api/rest/runes/{}", self.url, rune_id))
            .await
            .map_err(IndexerError::RequestErr)?;
        match resp.status() {
            StatusCode::OK => {
                let text = resp.text().await.map_err(IndexerError::RequestErr)?;
                Ok(Runes::from_json(text.as_str()).map_err(IndexerError::JsonErr)?)
            }
            code => Err(IndexerError::ServerErr(code)),
        }
    }
}
