use std::{
    collections::{BTreeMap, HashMap},
    fmt::{self, Display, Formatter},
    str::FromStr,
};

use candid::CandidType;
use candid::Principal;
use ic_stable_structures::storable::Bound;
use ic_stable_structures::Storable;
use serde::{Deserialize, Serialize};
use sha2::Digest;
use std::borrow::Cow;
use thiserror::Error;

pub type CanisterId = Principal;
pub type Signature = Vec<u8>;
pub type Seq = u64;
pub type Timestamp = u64;
pub type ChainId = String;
pub type DstChain = ChainId;
pub type TokenId = String;
pub type TicketId = String;
pub type Account = String;

#[derive(CandidType, Deserialize, Serialize, PartialEq, Eq, Clone, Debug)]
pub enum Directive {
    AddChain(Chain),
    AddToken(Token),
    UpdateChain(Chain),
    UpdateToken(Token),
    ToggleChainState(ToggleState),
    UpdateFee(Factor),
}

impl Directive {
    pub fn to_topic(&self) -> Topic {
        match self {
            Self::AddChain(_) => Topic::AddChain,
            Self::AddToken(_) => Topic::AddToken,
            Self::ToggleChainState(_) => Topic::ToggleChainState,
            Self::UpdateFee(_) => Topic::UpdateFee,
            Self::UpdateChain(_) => Topic::UpdateChain,
            Self::UpdateToken(_) => Topic::UpdateToken,
        }
    }
}

impl Storable for Directive {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        let bytes = bincode::serialize(&self).expect("failed to serialize Directive");
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        bincode::deserialize(bytes.as_ref()).expect("failed to deserialize Directive")
    }

    const BOUND: Bound = Bound::Unbounded;
}
impl core::fmt::Display for Directive {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Directive::AddChain(chain) => write!(f, "AddChain({})", chain),
            Directive::AddToken(token) => write!(f, "AddToken({})", token),
            Directive::ToggleChainState(toggle_state) => {
                write!(f, "ToggleChainState({})", toggle_state)
            }
            Directive::UpdateFee(factor) => write!(f, "UpdateFee({})", factor),
            Directive::UpdateChain(chain) => write!(f, "UpdateChain({})", chain),
            Directive::UpdateToken(token) => write!(f, "UpdateToken({})", token),
        }
    }
}
impl Directive {
    pub fn hash(&self) -> String {
        let mut hasher = sha2::Sha256::new();
        hasher.update(self.to_string().as_bytes());
        let bytes: [u8; 32] = hasher.finalize().into();
        bytes.iter().map(|byte| format!("{:02x}", byte)).collect()
    }
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
        let bytes = bincode::serialize(&self).expect("failed to serialize DireKey");
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        bincode::deserialize(bytes.as_ref()).expect("failed to deserialize DireKey")
    }

    const BOUND: Bound = Bound::Unbounded;
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, Default)]
pub struct DireMap {
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
        let bytes = bincode::serialize(&self).expect("failed to serialize DireMap");
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        bincode::deserialize(bytes.as_ref()).expect("failed to deserialize DireMap")
    }

    const BOUND: Bound = Bound::Unbounded;
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Topic {
    AddChain,
    AddToken,
    UpdateChain,
    UpdateToken,
    ToggleChainState,
    UpdateFee,
}

impl Storable for Topic {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        let bytes = bincode::serialize(&self).expect("failed to serialize Topic");
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        bincode::deserialize(bytes.as_ref()).expect("failed to deserialize Topic")
    }

    const BOUND: Bound = Bound::Unbounded;
}
impl core::fmt::Display for Topic {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Topic::AddChain => write!(f, "AddChain"),
            Topic::AddToken => write!(f, "AddToken"),
            Topic::ToggleChainState => {
                write!(f, "ToggleChainState",)
            }
            Topic::UpdateFee => write!(f, "UpdateFee"),
            Topic::UpdateChain => write!(f, "UpdateChain"),
            Topic::UpdateToken => write!(f, "UpdateToken"),
        }
    }
}

#[derive(
    CandidType, Deserialize, Serialize, Default, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
pub enum TicketType {
    #[default]
    Normal,
    Resubmit,
}

#[derive(
    CandidType, Deserialize, Serialize, Default, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
pub struct Ticket {
    pub ticket_id: TicketId,
    pub ticket_type: TicketType,
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
        let bytes = bincode::serialize(&self).expect("failed to serialize Ticket");
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        bincode::deserialize(bytes.as_ref()).expect("failed to deserialize Ticket")
    }

    const BOUND: Bound = Bound::Unbounded;
}

impl core::fmt::Display for Ticket {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(
            f,
            "\nticket id:{} \nticket type:{:?} \ncreated time:{} \nsrc chain:{} \ndst_chain:{} \naction:{:?} \ntoken:{} \namount:{} \nsender:{:?} \nrecevier:{} \nmemo:{:?}",
            self.ticket_id,
            self.ticket_type,
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
        let bytes = bincode::serialize(&self).expect("failed to serialize SeqKey");
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        bincode::deserialize(bytes.as_ref()).expect("failed to deserialize SeqKey")
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
        let bytes = bincode::serialize(&self).expect("failed to serialize TicketMap");
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        bincode::deserialize(bytes.as_ref()).expect("failed to deserialize TicketMap")
    }

    const BOUND: Bound = Bound::Unbounded;
}

#[derive(
    CandidType, Deserialize, Serialize, Default, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
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
    Burn,
    Mint,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub enum Factor {
    UpdateTargetChainFactor(TargetChainFactor),
    UpdateFeeTokenFactor(FeeTokenFactor),
}

impl Storable for Factor {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        let bytes = bincode::serialize(&self).expect("failed to serialize Factor");
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        bincode::deserialize(bytes.as_ref()).expect("failed to deserialize Factor")
    }

    const BOUND: Bound = Bound::Unbounded;
}
impl core::fmt::Display for Factor {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        match self {
            Factor::UpdateTargetChainFactor(chain_factor) => write!(f, "{}", chain_factor),
            Factor::UpdateFeeTokenFactor(token_factor) => write!(f, "{}", token_factor),
        }
    }
}
#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq, Eq, Default)]
pub struct TargetChainFactor {
    pub target_chain_id: ChainId,
    pub target_chain_factor: u128,
}

impl Storable for TargetChainFactor {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        let bytes = bincode::serialize(&self).expect("failed to serialize TargetChainFactor");
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        bincode::deserialize(bytes.as_ref()).expect("failed to deserialize TargetChainFactor")
    }

    const BOUND: Bound = Bound::Unbounded;
}

impl core::fmt::Display for TargetChainFactor {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(
            f,
            "\nchain id:{},\nchain factor:{}",
            self.target_chain_id, self.target_chain_factor,
        )
    }
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq, Eq, Default)]
pub struct FeeTokenFactor {
    pub fee_token: TokenId,
    pub fee_token_factor: u128,
}

impl Storable for FeeTokenFactor {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        let bytes = bincode::serialize(&self).expect("failed to serialize FeeTokenFactor");
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        bincode::deserialize(bytes.as_ref()).expect("failed to deserialize FeeTokenFactor")
    }

    const BOUND: Bound = Bound::Unbounded;
}

impl core::fmt::Display for FeeTokenFactor {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(
            f,
            "\nfee token:{},\nfee_token_factor:{}",
            self.fee_token, self.fee_token_factor,
        )
    }
}

/// chain id spec:
/// for settlement chain, the chain id is: Bitcoin, Ethereum,or ICP
/// for execution chain, the chain id spec is: type-chain_name,eg: EVM-Base,Cosmos-Gaia, Substrate-Xxx
#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Chain {
    pub chain_id: ChainId,
    pub canister_id: String,
    pub chain_type: ChainType,
    // the chain default state is true
    pub chain_state: ChainState,
    // settlement chain: export contract address
    // execution chain: port contract address
    pub contract_address: Option<String>,

    // optional counterparty chains
    pub counterparties: Option<Vec<ChainId>>,
    // fee token
    pub fee_token: Option<TokenId>,
}
impl Chain {
    pub fn chain_name(&self) -> Option<&str> {
        match self.chain_type {
            ChainType::SettlementChain => Some(&self.chain_id),
            ChainType::ExecutionChain => self.chain_id.split('-').last(),
        }
    }
}

impl Storable for Chain {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        let bytes = bincode::serialize(&self).expect("failed to serialize Chain");
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        bincode::deserialize(bytes.as_ref()).expect("failed to deserialize Chain")
    }

    const BOUND: Bound = Bound::Unbounded;
}

impl core::fmt::Display for Chain {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(
            f,
            "\nchain id:{} \ncanister id:{} \nchain type:{:?} \nchain state:{:?} \ncontract address:{:?} \ncounterparties:{:?} \nfee_token:{:?}",
            self.chain_id,self.canister_id, self.chain_type, self.chain_state, self.contract_address,self.counterparties,self.fee_token,
        )
    }
}

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
    pub name: String,
    pub symbol: String,

    pub decimals: u8,
    pub icon: Option<String>,
    pub metadata: HashMap<String, String>,
}

impl Token {
    /// return (settlmentchain,token protocol, token symbol)
    pub fn token_id_info(&self) -> Vec<&str> {
        self.token_id.split('-').collect()
    }
}

impl Storable for Token {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        let bytes = bincode::serialize(&self).expect("failed to serialize Token");
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        // let s = ciborium::de::from_reader(bytes.as_ref()).expect("failed to decode SuiToken");
        // s
        bincode::deserialize(bytes.as_ref()).expect("failed to deserialize Token")
    }

    const BOUND: Bound = Bound::Unbounded;
}

impl core::fmt::Display for Token {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(
            f,
            "\ttoken id:{} \ntoken name:{} \nsymbol:{:?} \ndecimals:{} \nicon:{:?} \nmetadata:{:?}",
            self.token_id, self.name, self.symbol, self.decimals, self.icon, self.metadata
        )
    }
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq, Eq, Hash)]
pub struct TokenOnChain {
    // the chain of the token be locked
    pub chain_id: ChainId,
    pub token_id: TokenId,
    pub amount: u128,
}

#[derive(CandidType, Deserialize, Serialize, Default, Clone, Debug)]
pub struct ChainCondition {
    pub chain_type: Option<ChainType>,
    pub chain_state: Option<ChainState>,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct TokenCondition {
    pub token_id: Option<TokenId>,
    pub chain_id: Option<ChainId>,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct TxCondition {
    pub src_chain: Option<ChainId>,
    pub dst_chain: Option<ChainId>,
    pub token_id: Option<TokenId>,
    // time range: from .. end
    pub time_range: Option<(u64, u64)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseRuneIdError;

impl fmt::Display for ParseRuneIdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        "provided rune_id was not valid".fmt(f)
    }
}

impl std::error::Error for ParseRuneIdError {
    fn description(&self) -> &str {
        "failed to parse rune_id"
    }
}

#[derive(
    candid::CandidType,
    Clone,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Copy,
    Default,
    Serialize,
    Deserialize,
)]
pub struct RuneId {
    pub block: u64,
    pub tx: u32,
}

impl RuneId {
    pub fn delta(self, next: RuneId) -> Option<(u128, u128)> {
        let block = next.block.checked_sub(self.block)?;

        let tx = if block == 0 {
            next.tx.checked_sub(self.tx)?
        } else {
            next.tx
        };

        Some((block.into(), tx.into()))
    }
}

impl Display for RuneId {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.block, self.tx,)
    }
}

impl FromStr for RuneId {
    type Err = ParseRuneIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (height, index) = s.split_once(':').ok_or_else(|| ParseRuneIdError)?;

        Ok(Self {
            block: height.parse().map_err(|_| ParseRuneIdError)?,
            tx: index.parse().map_err(|_| ParseRuneIdError)?,
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Memo {
    pub memo: Option<String>,
    pub bridge_fee: u128,
}

#[derive(CandidType, Deserialize, Debug, Error)]
pub enum Error {
    #[error("The topic (`{0}`) already Subscribed")]
    RepeatSubscription(String),

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
    #[error("Not fount the ticket id (`{0}`) !")]
    NotFoundTicketId(String),
    #[error("The resubmit ticket id must exist!")]
    ResubmitTicketIdMustExist,
    #[error("The resubmit ticket must same as the old ticket!")]
    ResubmitTicketMustSame,
    #[error("The resumit ticket sent too often")]
    ResubmitTicketSentTooOften,
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
    #[error("ecdsa_public_key failed : (`{0}`)")]
    EcdsaPublicKeyError(String),
    #[error("sign_with_ecdsa failed: (`{0}`)")]
    SighWithEcdsaError(String),
    #[error("custom error: (`{0}`)")]
    CustomError(String),
}
