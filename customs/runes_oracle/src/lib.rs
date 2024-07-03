pub mod client;
pub mod executor;
pub mod indexer;
pub mod tx;

use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct RunesResponse {
    pub runes: Vec<Runes>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Runes {
    pub divisibility: u8,
    pub spaced_rune: String,
}

impl Runes {
    pub fn from_json(json_str: &str) -> Result<Self, serde_json::Error> {
        let response: RunesResponse = serde_json::from_str(json_str)?;
        assert!(response.runes.len() == 1, "Expected 1 runes");
        Ok(response.runes[0].clone())
    }
}
