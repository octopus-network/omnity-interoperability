use candid::CandidType;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub type Signature = Vec<u8>;
pub type Seq = u64;

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
    pub trans_id: String,
    pub timestamp: u64,
    pub nonce: u64,
    pub src_chain_id: String,
    pub dst_chain_id: String,
    pub action: Action,
    pub token: String,
    pub memo: Option<Vec<u8>>,
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
pub enum Status {
    #[default]
    Active,
    Suspend,
    Reinstate,
}

#[derive(CandidType, Deserialize, Serialize, Default, Debug)]
pub enum Action {
    #[default]
    Transfer,
    Redeem,
}

#[derive(CandidType, Deserialize, Serialize, Debug)]
pub struct Fee {
    pub chain_id: String,
    pub fee_token: String,
    pub fee_amount: u64,
    pub signature: Option<Vec<u8>>,
}

#[derive(CandidType, Deserialize, Serialize, Default, Debug)]
pub struct ChainInfo {
    pub chain_id: String,
    pub chain_type: ChainType,
    pub chain_name: String,
    pub chain_state: Status,
    pub seq: u64,
    pub signature: Option<Vec<u8>>,
}

#[derive(CandidType, Deserialize, Serialize, Default, Debug)]
pub struct ChainStatue {
    pub chain_id: String,
    pub chain_state: Status,
    pub seq: u64,
    pub signature: Option<Vec<u8>>,
}

#[derive(CandidType, Deserialize, Serialize, Debug)]
pub struct TokenInfo {
    pub token_id: String,
    pub token_symbol: String,
    pub chain_id: String,
    pub meta: Option<u8>,
    pub seq: u64,
    pub signature: Option<Vec<u8>>,
}

#[derive(CandidType, Deserialize, Serialize, Debug)]
pub enum Directive {
    AddChain(ChainInfo),
    AddToken(TokenInfo),
    SetChainStatus(ChainStatue),
    UpdateFee(Fee),
}
