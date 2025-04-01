use candid_derive::CandidType;
use omnity_types::{Token, TokenId};
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(CandidType, Clone, Debug, Serialize, Deserialize)]
pub struct TokenResp {
    pub token_id: TokenId,
    pub symbol: String,
    pub decimals: u8,
    pub icon: Option<String>,
    pub rune_id: Option<String>,
    pub evm_contract: Option<String>,
    pub metadata: HashMap<String, String>,
}

impl From<Token> for TokenResp {
    fn from(value: Token) -> Self {
        TokenResp {
            token_id: value.token_id,
            symbol: value.symbol,
            decimals: value.decimals,
            icon: value.icon,
            rune_id: value.metadata.get("rune_id").cloned(),
            metadata: value.metadata,
            evm_contract: None,
        }
    }
}
