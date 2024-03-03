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
pub type DstChain = ChainId;
pub type TokenId = String;
pub type TicketId = String;
pub type Account = String;

/// Directive Queue
/// K: DstChain, V:  BTreeMap<Seq, Directive>
pub type DireQueue = HashMap<DstChain, BTreeMap<Seq, Directive>>;
/// Ticket Queue
/// K: DstChain, V: BTreeMap<Seq, Ticket>
pub type TicketQueue = HashMap<DstChain, BTreeMap<Seq, Ticket>>;

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub enum Proposal {
    AddChain(ChainInfo),
    AddToken(TokenMeta),
    ToggleChainState(ToggleState),
    UpdateFee(Fee),
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub enum Topic {
    // AddChain(Option<ChainType>)
    AddChain(Option<ChainType>),
    AddToken(Option<TokenId>),
    UpdateFee(Option<TokenId>),
    ActivateChain,
    DeactivateChain,
}

#[derive(CandidType, Deserialize, Serialize, Default, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Ticket {
    pub ticket_id: TicketId,
    pub ticket_time: Timestamp,
    pub src_chain: ChainId,
    pub dst_chain: ChainId,
    pub action: TxAction,
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
            self.ticket_time,
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

#[derive(CandidType, Deserialize, Serialize, Default, Clone, Debug, PartialEq, Eq, Hash)]
pub enum ChainType {
    #[default]
    SettlementChain,
    ExecutionChain,
}

#[derive(CandidType, Deserialize, Serialize, Default, Clone, Debug, PartialEq, Eq, Hash)]
pub enum ChainState {
    #[default]
    Active,
    Deactive,
}

#[derive(CandidType, Deserialize, Serialize, Default, Clone, Debug, PartialEq, Eq)]
pub enum StateAction {
    // #[default]
    // Active,
    #[default]
    Activate,
    Deactivate,
}

#[derive(CandidType, Deserialize, Serialize, Default, Clone, Debug, PartialEq, Eq, Hash)]
pub enum TxAction {
    #[default]
    Transfer,
    Redeem,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct Fee {
    pub dst_chain_id: ChainId,
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
            self.dst_chain_id, self.fee_token, self.factor,
        )
    }
}

#[derive(CandidType, Deserialize, Serialize, Default, Clone, Debug, PartialEq, Eq, Hash)]
pub struct ChainInfo {
    pub chain_id: ChainId,
    pub chain_type: ChainType,
    // the chain default state is true
    pub chain_state: ChainState,
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
            self.chain_id, self.chain_type, self.chain_state,
        )
    }
}

#[derive(CandidType, Deserialize, Serialize, Default, Clone, Debug)]
pub struct ToggleState {
    pub chain_id: ChainId,
    pub action: StateAction,
}
impl core::fmt::Display for ToggleState {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(f, "chain:{},\nchain state:{:?}", self.chain_id, self.action,)
    }
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq, Eq, Hash)]
pub struct TokenMeta {
    pub token_id: TokenId,
    pub symbol: String,
    // the token`s issuse chain
    pub issue_chain: ChainId,
    pub decimals: u8,
    pub icon: Option<String>,
    // pub total_amount: Option<u128>,
    // pub token_constract_address: Option<String>,
}
impl core::fmt::Display for TokenMeta {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(
            f,
            "token name:{},\nsymbol:{:?},\nissue chain:{},\ndecimals:{},\nicon:{:?}",
            self.token_id, self.symbol, self.issue_chain, self.decimals, self.icon
        )
    }
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq, Eq, Hash)]
pub struct TokenOnChain {
    pub token_id: TokenId,
    // the chain of the token be locked
    pub chain_id: ChainId,
    pub amount: u64,
    // pub chain_type: ChainType,
}

#[derive(CandidType, Deserialize, Serialize, Default, Clone, Debug)]
pub struct ChainCondition {
    // chain_id: Option<ChainId>,
    pub chain_type: Option<ChainType>,
    pub chain_state: Option<ChainState>,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct TokenCondition {
    pub token_id: Option<TokenId>,
    pub chain_id: Option<ChainId>,
    // pub chain_type: Option<ChainType>,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct TxCondition {
    pub src_chain: Option<ChainId>,
    pub dst_chain: Option<ChainId>,
    // chain_type: Option<ChainType>,
    pub token_id: Option<TokenId>,
    // time range: from .. end
    pub time_range: Option<(u64, u64)>,
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
    #[error("not found chain: (`{0}`)")]
    NotFoundChain(String),
    #[error("custom error: (`{0}`)")]
    CustomError(String),
}
