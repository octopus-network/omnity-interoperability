use candid::CandidType;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(CandidType, Deserialize, Debug, Error)]
pub enum Error {
    #[error("the message is malformed and cannot be decoded error")]
    MalformedMessageBytes,
    #[error("unauthorized")]
    Unauthorized,
    #[error("custom error: (`{0}`)")]
    CustomError(String),
}

#[derive(CandidType, Deserialize, Serialize, Debug)]
pub struct BoardingPass {}

#[derive(CandidType, Deserialize, Serialize, Default, Debug)]
pub struct LandingPass {
    pub timestamp: u64,
    pub nonce: u64,
    pub source: String,
    pub target: String,
    pub token: String,
    pub receiver: String,
    pub amount: u64,
    pub signature: Option<Vec<u8>>,
}

#[derive(CandidType, Deserialize, Serialize, Default, Debug)]
pub enum ChainType {
    #[default]
    SettlementChain,
    ExecutionChain,
}
#[derive(CandidType, Deserialize, Serialize, Default, Debug)]
pub enum ChainStatus {
    #[default]
    Active,
    Suspend,
    Reinstate,
}

#[derive(CandidType, Deserialize, Serialize, Debug)]
pub enum Action {
    Transfer,
    Redeem,
}

#[derive(CandidType, Deserialize, Serialize, Debug)]
pub struct Fee {
    pub chain_id: String,
    pub fee_token: String,
    pub fee_amount: u64,
}

#[derive(CandidType, Deserialize, Serialize, Default, Debug)]
pub struct ChainInfo {
    pub chain_id: String,
    pub chain_name: String,
    pub chain_type: ChainType,
    pub chain_state: ChainStatus,
}

#[derive(CandidType, Deserialize, Serialize, Debug)]
pub struct TokenInfo {
    pub token_id: String,
    pub token_symbol: String,
    pub chain_id: String,
}

#[derive(CandidType, Deserialize, Serialize, Debug)]
pub enum Directive {
    AddChain(ChainInfo),
    AddToken(TokenInfo),
    SetChainStatus(ChainStatus),
    UpdateFee(Fee),
}
