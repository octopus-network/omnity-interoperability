use std::collections::{BTreeMap, HashMap};

use candid::CandidType;
use ic_stable_structures::storable::Bound;
use ic_stable_structures::Storable;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

use thiserror::Error;

pub type Signature = Vec<u8>;
pub type Seq = u64;
pub type Timestamp = u64;
// pub type Proposal = Directive;
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

impl Storable for Directive {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        let mut bytes = vec![];
        let _ = ciborium::ser::into_writer(self, &mut bytes);
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        let dire = ciborium::de::from_reader(bytes.as_ref()).expect("failed to decode TokenKey");
        dire
    }

    const BOUND: Bound = Bound::Unbounded;
}

#[derive(
    CandidType, Deserialize, Serialize, Clone, Debug, PartialOrd, Ord, PartialEq, Eq, Hash,
)]
pub struct DireKey {
    pub chain_id: ChainId,
    pub seq: Seq,
}

impl Storable for DireKey {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        let mut bytes = vec![];
        let _ = ciborium::ser::into_writer(self, &mut bytes);
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        let dk = ciborium::de::from_reader(bytes.as_ref()).expect("failed to decode TokenKey");
        dk
    }

    const BOUND: Bound = Bound::Unbounded;
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, Default)]
pub struct DireMap {
    // pub seq: Seq,
    // pub dire: Directive,
    pub dires: BTreeMap<Seq, Directive>,
}

impl DireMap {
    pub fn from(seq: Seq, dire: Directive) -> Self {
        Self {
            dires: BTreeMap::from([(seq, dire)]),
        }
    }
}
impl Storable for DireMap {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        let mut bytes = vec![];
        let _ = ciborium::ser::into_writer(self, &mut bytes);
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        let dire = ciborium::de::from_reader(bytes.as_ref()).expect("failed to decode TokenKey");
        dire
    }

    const BOUND: Bound = Bound::Unbounded;
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

#[derive(
    CandidType, Deserialize, Serialize, Default, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
pub struct Ticket {
    pub ticket_id: TicketId,
    pub ticket_time: Timestamp,
    pub src_chain: ChainId,
    pub dst_chain: ChainId,
    pub action: TxAction,
    pub token: TokenId,
    pub amount: String,
    pub sender: Option<Account>,
    pub receiver: Account,
    pub memo: Option<Vec<u8>>,
}

impl Storable for Ticket {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        let mut bytes = vec![];
        let _ = ciborium::ser::into_writer(self, &mut bytes);
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        let ticket = ciborium::de::from_reader(bytes.as_ref()).expect("failed to decode TokenKey");
        ticket
    }

    const BOUND: Bound = Bound::Unbounded;
}

impl core::fmt::Display for Ticket {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(
            f,
            "\nticket id:{} \ncreated time:{} \nsrc chain:{} \ndst_chain:{} \naction:{:?} \ntoken:{} \namount:{} \nsender:{:?} \nrecevier:{} \nmemo:{:?}",
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

#[derive(
    CandidType, Deserialize, Serialize, Default, Clone, Debug, PartialOrd, Ord, PartialEq, Eq, Hash,
)]
pub struct SeqKey {
    pub chain_id: ChainId,
    pub seq: Seq,
}

impl SeqKey {
    pub fn from(chain_id: ChainId, seq: Seq) -> Self {
        Self { chain_id, seq }
    }
}

impl Storable for SeqKey {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        let mut bytes = vec![];
        let _ = ciborium::ser::into_writer(self, &mut bytes);
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        let tk = ciborium::de::from_reader(bytes.as_ref()).expect("failed to decode TokenKey");
        tk
    }

    const BOUND: Bound = Bound::Unbounded;
}

#[derive(CandidType, Deserialize, Serialize, Default, Clone, Debug)]
pub struct TicketMap {
    // pub seq: Seq,
    // pub ticket: Ticket,
    pub tickets: BTreeMap<Seq, Ticket>,
}

impl TicketMap {
    pub fn from(seq: Seq, ticket: Ticket) -> Self {
        Self {
            tickets: BTreeMap::from([(seq, ticket)]),
        }
    }
}

impl Storable for TicketMap {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        let mut bytes = vec![];
        let _ = ciborium::ser::into_writer(self, &mut bytes);
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        let ticket = ciborium::de::from_reader(bytes.as_ref()).expect("failed to decode TokenKey");
        ticket
    }

    const BOUND: Bound = Bound::Unbounded;
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

impl From<ToggleAction> for ChainState {
    fn from(value: ToggleAction) -> Self {
        match value {
            ToggleAction::Activate => ChainState::Active,
            ToggleAction::Deactivate => ChainState::Deactive,
        }
    }
}

#[derive(
    CandidType, Deserialize, Serialize, Default, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
pub enum TxAction {
    #[default]
    Transfer,
    Redeem,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, Default)]
pub struct Fee {
    pub dst_chain_id: ChainId,
    // quote currency or token
    pub fee_token: TokenId,
    pub target_chain_factor: u64,
    pub fee_token_factor: u64,
}

impl Storable for Fee {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        let mut bytes = vec![];
        let _ = ciborium::ser::into_writer(self, &mut bytes);
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        let fee = ciborium::de::from_reader(bytes.as_ref()).expect("failed to decode TokenKey");
        fee
    }

    const BOUND: Bound = Bound::Unbounded;
}

impl core::fmt::Display for Fee {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(
            f,
            "\ndst chain:{},\nfee token:{},\nfactor:{}",
            self.dst_chain_id, self.fee_token, self.target_chain_factor,
        )
    }
}

/// chain id spec:
/// for settlement chain, the chain id is: Bitcoin, Ethereum,or ICP
/// for execution chain, the chain id spec is: type-chain_name,eg: EVM-Base,Cosmos-Gaia, Substrate-Xxx
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
impl Chain {
    pub fn chain_name(&self) -> Option<&str> {
        match self.chain_type {
            ChainType::SettlementChain => Some(&self.chain_id),
            ChainType::ExecutionChain => self.chain_id.split('-').last(),
        }
    }
}

//TODO: update chain and token info
#[derive(CandidType, Deserialize, Serialize, Default, Clone, Debug, PartialEq, Eq)]
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

// token id spec is setllmentchain_name-potocol-symbol, eg: Ethereurm-ERC20-OCT , Bitcoin-RUNES-WHAT•ABOUT•THIS•RUNE
/// metadata stores extended information，for runes protocol token, it stores the runes id
#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub struct Token {
    pub token_id: TokenId,
    pub symbol: String,
    // the token`s issuse chain
    pub issue_chain: ChainId,
    pub decimals: u8,
    pub icon: Option<String>,
    pub metadata: Option<HashMap<String, String>>,
    // pub token_constract_address: Option<String>,
}

impl Token {
    /// return (settlmentchain,token protocol, token symbol)
    pub fn token_id_info(&self) -> Vec<&str> {
        self.token_id.split('-').collect()
    }
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

    #[error("generate directive error for : (`{0}`)")]
    GenerateDirectiveError(String),

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
    #[error("not found token: (`{0}`)")]
    NotFoundToken(String),
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
