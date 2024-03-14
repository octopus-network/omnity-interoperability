use candid::CandidType;

use serde::{Deserialize, Serialize};

use thiserror::Error;

pub type Signature = Vec<u8>;
pub type Seq = u64;
pub type Timestamp = u64;
// pub type Directive = Proposal;
pub type ChainId = String;
pub type DstChain = ChainId;
pub type TokenId = String;
pub type TicketId = String;
pub type Account = String;

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub enum Directive {
    AddChain(Chain),
    AddToken(Token),
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
            "\nticket id:{} \ncreated time:{} \nsrc chain:{} \ndst_chain:{} \naction:{:?} \ntoken:{} \namount:{} \nsender:{} \nrecevier:{} \nmemo:{:?}",
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
pub enum ToggleAction {
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
            "\ndst chain:{},\nfee token:{},\nfactor:{}",
            self.dst_chain_id, self.fee_token, self.factor,
        )
    }
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Chain {
    // pub canister_id: String,
    pub chain_id: ChainId,
    pub chain_type: ChainType,
    // the chain default state is true
    pub chain_state: ChainState,
    // settlement chain: export contract address
    // execution chain: port contract address
    pub contract_address: Option<String>,
}

#[derive(CandidType, Deserialize, Serialize, Default, Clone, Debug)]
pub struct ToggleState {
    pub chain_id: ChainId,
    pub action: ToggleAction,
}
impl core::fmt::Display for ToggleState {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(
            f,
            "\nchain:{},\nchain state:{:?}",
            self.chain_id, self.action,
        )
    }
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Token {
    pub token_id: TokenId,
    pub symbol: String,
    // the token`s issuse chain
    pub issue_chain: ChainId,
    pub decimals: u8,
    pub icon: Option<String>,
    // pub dst_chains: Vec<ChainId>,
    // pub total_amount: Option<u128>,
    // pub token_constract_address: Option<String>,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq, Eq, Hash)]
pub struct TokenOnChain {
    // the chain of the token be locked
    pub chain_id: ChainId,
    pub token_id: TokenId,
    pub amount: u128,
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
    #[error("The chain(`{0}`) already exists")]
    ChainAlreadyExisting(String),
    #[error("The token(`{0}`) already exists")]
    TokenAlreadyExisting(String),

    #[error("not supported proposal")]
    NotSupportedProposal,
    #[error("proposal error: (`{0}`)")]
    ProposalError(String),

    #[error("the message is malformed and cannot be decoded error")]
    MalformedMessageBytes,
    #[error("unauthorized")]
    Unauthorized,
    #[error("The `{0}` is deactive")]
    DeactiveChain(String),
    #[error("The ticket id (`{0}`) already exists!")]
    AlreadyExistingTicketId(String),
    #[error("not found chain: (`{0}`)")]
    NotFoundChain(String),
    #[error("not found account: (`{0}`)")]
    NotFoundAccount(String),
    #[error("not found account(`{0}`) token(`{1}`) on the chain(`{2}`")]
    NotFoundAccountToken(String, String, String),
    #[error("Not found this token(`{0}`) on chain(`{1}`) ")]
    NotFoundChainToken(String, String),
    #[error("Insufficient token (`{0}`) on chain (`{1}`) !)")]
    NotSufficientTokens(String, String),
    #[error("The ticket amount(`{0}`) parse error: `{1}`")]
    TicketAmountParseError(String, String),
    #[error("custom error: (`{0}`)")]
    CustomError(String),
}
