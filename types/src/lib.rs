use candid::CandidType;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::collections::HashMap;
use thiserror::Error;

pub type Signature = Vec<u8>;
pub type Seq = u64;
pub type Timestamp = u64;
pub type Directive = Proposal;
pub type ChainId = String;
pub type TokenId = String;
pub type TicketId = String;
pub type Account = String;

/// Directive Queue
/// K: chainid and seq, V:  HashMap<Seq, Directive>
pub type DireQueue = HashMap<ChainId, BTreeMap<Seq, Directive>>;
/// Ticket Queue
/// K: chainid and seq, V: HashMap<Seq, Ticket>
pub type TicketQueue = HashMap<ChainId, BTreeMap<Seq, Ticket>>;

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub enum Proposal {
    AddChain(ChainInfo),
    AddToken(TokenMetaData),
    ChangeChainStatus(Status),
    UpdateFee(Fee),
}

#[derive(CandidType, Deserialize, Serialize, Default, Clone, Debug)]
pub struct Ticket {
    pub ticket_id: TicketId,
    pub created_time: Timestamp,
    pub src_chain: ChainId,
    pub dst_chain: ChainId,
    pub action: Action,
    pub token: TokenId,
    pub amount: String,
    pub sender: Account,
    pub receiver: Account,
    pub memo: Option<Vec<u8>>,
}

impl core::fmt::Display for Ticket {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(
            f,
            "ticket id:{},\ncreated time:{},\nsrc chain:{},\ndst_chain:{},\naction:{:?},\ntoken:{},\namount:{},\nsender:{},\nrecevier:{},\nmemo:{:?}",
            self.ticket_id,
            self.created_time,
            self.src_chain,
            self.dst_chain,
            self.action,
            self.token,
            self.amount,
            self.sender,
            self.receiver,
            self.memo,
        )
    }
}

#[derive(CandidType, Deserialize, Serialize, Default, Clone, Debug)]
pub enum ChainType {
    #[default]
    SettlementChain,
    ExecutionChain,
}

#[derive(CandidType, Deserialize, Serialize, Default, Clone, Debug)]
pub enum Status {
    #[default]
    Active,
    Suspend,
    Reinstate,
}

#[derive(CandidType, Deserialize, Serialize, Default, Clone, Debug)]
pub enum Action {
    #[default]
    Transfer,
    Redeem,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct Fee {
    pub dst_chain: ChainId,
    // quote currency or token
    pub fee_token: TokenId,
    // base fee = 1 wei
    pub factor: i64,
    // quote token amoute
    // pub fee_amount: u64,
}

impl core::fmt::Display for Fee {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(
            f,
            "dst chain:{},\nfee token:{},\nfactor:{}",
            self.dst_chain, self.fee_token, self.factor,
        )
    }
}

#[derive(CandidType, Deserialize, Serialize, Default, Clone, Debug)]
pub struct ChainInfo {
    pub chain_name: ChainId,
    pub chain_type: ChainType,
    pub chain_state: Status,
    // Optional: settlement chain export contract address
    // pub export_address: Option<String>,
    // Optional: execution chain port contract address
    // pub port_address: Option<String>,
}

impl core::fmt::Display for ChainInfo {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(
            f,
            "chain name:{},\nfchain type:{:?},\nchain state:{:?}",
            self.chain_name, self.chain_type, self.chain_state,
        )
    }
}

#[derive(CandidType, Deserialize, Serialize, Default, Clone, Debug)]
pub struct ChainStatus {
    pub chain: ChainId,
    pub state: Status,
}
impl core::fmt::Display for ChainStatus {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(f, "chain:{},\nchain state:{:?}", self.chain, self.state,)
    }
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct TokenMetaData {
    pub name: TokenId,
    pub symbol: String,
    // the token`s issuse chain
    pub issue_chain: ChainId,
    pub decimals: u8,
    pub icon: Option<String>,
    // pub total_amount: Option<u128>,
    // pub token_constract_address: Option<String>,
}
impl core::fmt::Display for TokenMetaData {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(
            f,
            "token name:{},\nsymbol:{:?},\nissue chain:{},\ndecimals:{},\nicon:{:?}",
            self.name, self.symbol, self.issue_chain, self.decimals, self.icon
        )
    }
}

#[derive(CandidType, Deserialize, Debug, Error)]
pub enum Error {
    #[error("proposal error: (`{0}`)")]
    ProposalError(String),
    #[error("not supported proposal")]
    NotSupportedProposal,
    #[error("the message is malformed and cannot be decoded error")]
    MalformedMessageBytes,
    #[error("unauthorized")]
    Unauthorized,
    #[error("custom error: (`{0}`)")]
    CustomError(String),
}
