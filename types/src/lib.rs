use std::{
    collections::{BTreeMap, HashMap}, str::FromStr
};

use candid::CandidType;
use ic_stable_structures::storable::Bound;
use ic_stable_structures::Storable;
use serde::{Deserialize, Serialize};
use sha2::Digest;
use std::borrow::Cow;
use thiserror::Error;

pub mod rune_id;
pub mod signer;
pub mod ic_log;
pub mod brc20;
pub mod hub_types;

pub type Signature = Vec<u8>;
pub type Seq = u64;
pub type Timestamp = u64;
pub type ChainId = String;
pub type DstChain = ChainId;
pub type TokenId = String;
pub type TicketId = String;
pub type Account = String;

pub type Amount = u128;
pub type TxHash = String;


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
        let mut bytes = vec![];
        let _ = ciborium::ser::into_writer(self, &mut bytes);
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        let dire = ciborium::de::from_reader(bytes.as_ref()).expect("failed to decode Directive");
        dire
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
        hex::encode(bytes)
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
        let mut bytes = vec![];
        let _ = ciborium::ser::into_writer(self, &mut bytes);
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        let dk = ciborium::de::from_reader(bytes.as_ref()).expect("failed to decode DireKey");
        dk
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
        let mut bytes = vec![];
        let _ = ciborium::ser::into_writer(self, &mut bytes);
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        let dire = ciborium::de::from_reader(bytes.as_ref()).expect("failed to decode DireMap");
        dire
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
        let mut bytes = vec![];
        let _ = ciborium::ser::into_writer(self, &mut bytes);
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        let topic = ciborium::de::from_reader(bytes.as_ref()).expect("failed to decode Topic");
        topic
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
        let mut bytes = vec![];
        let _ = ciborium::ser::into_writer(self, &mut bytes);
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        let ticket = ciborium::de::from_reader(bytes.as_ref()).expect("failed to decode Ticket");
        ticket
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

#[derive(Serialize, Deserialize, Debug)]
pub struct Memo {
    pub memo: Option<String>,
    pub bridge_fee: u128,
}

impl Memo {
    pub fn convert_to_memo_json(self) -> Result<String, Box<dyn std::error::Error>> {
        let memo_json= serde_json::to_string_pretty(&self).map_err(|e| {
            format!(
                "[generate_ticket] memo convert error: {}",
                e.to_string()
            )
        })?;
        Ok(memo_json) 
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
        let tk = ciborium::de::from_reader(bytes.as_ref()).expect("failed to decode SeqKey");
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
        let ticket = ciborium::de::from_reader(bytes.as_ref()).expect("failed to decode TicketMap");
        ticket
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
    #[deprecated]
    RedeemIcpChainKeyAssets(IcpChainKeyToken)
}

#[derive(
    CandidType, Deserialize, Serialize, Default, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
pub enum IcpChainKeyToken {
    #[default]
    CKBTC
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub enum Factor {
    UpdateTargetChainFactor(TargetChainFactor),
    UpdateFeeTokenFactor(FeeTokenFactor),
}

impl Storable for Factor {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        let mut bytes = vec![];
        let _ = ciborium::ser::into_writer(self, &mut bytes);
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        let fee = ciborium::de::from_reader(bytes.as_ref()).expect("failed to decode Factor");
        fee
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
        let mut bytes = vec![];
        let _ = ciborium::ser::into_writer(self, &mut bytes);
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        let fee =
            ciborium::de::from_reader(bytes.as_ref()).expect("failed to decode TargetChainFactor");
        fee
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
        let mut bytes = vec![];
        let _ = ciborium::ser::into_writer(self, &mut bytes);
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        let fee =
            ciborium::de::from_reader(bytes.as_ref()).expect("failed to decode FeeTokenFactor");
        fee
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

impl Storable for Chain {
    const BOUND: Bound = Bound::Unbounded;

    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        let mut bytes = vec![];
        let _ = ciborium::ser::into_writer(self, &mut bytes);
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        let dire = ciborium::de::from_reader(bytes.as_ref()).expect("failed to decode Chain");
        dire
    }
}

impl Chain {
    pub fn chain_name(&self) -> Option<&str> {
        match self.chain_type {
            ChainType::SettlementChain => Some(&self.chain_id),
            ChainType::ExecutionChain => self.chain_id.split('-').last(),
        }
    }
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
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub icon: Option<String>,
    pub metadata: HashMap<String, String>,
}

impl Storable for Token {
    const BOUND: Bound = Bound::Unbounded;

    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        let mut bytes = vec![];
        let _ = ciborium::ser::into_writer(self, &mut bytes);
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        let dire = ciborium::de::from_reader(bytes.as_ref()).expect("failed to decode Token");
        dire
    }
}

impl Token {
    /// return (settlmentchain,token protocol, token symbol)
    pub fn token_id_info(&self) -> Vec<&str> {
        self.token_id.split('-').collect()
    }
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

use candid::Principal;
pub type CanisterId = Principal;

#[derive(CandidType, Serialize, Debug)]
struct ECDSAPublicKey {
    pub canister_id: Option<CanisterId>,
    pub derivation_path: Vec<Vec<u8>>,
    pub key_id: EcdsaKeyId,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct ECDSAPublicKeyReply {
    pub public_key: Vec<u8>,
    pub chain_code: Vec<u8>,
}

#[derive(CandidType, Serialize, Debug)]
pub struct SignWithECDSA {
    pub message_hash: Vec<u8>,
    pub derivation_path: Vec<Vec<u8>>,
    pub key_id: EcdsaKeyId,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct SignWithECDSAReply {
    pub signature: Vec<u8>,
}

#[derive(CandidType, Serialize, Debug)]
pub struct PublicKeyReply {
    pub public_key: Vec<u8>,
}

impl From<Vec<u8>> for PublicKeyReply {
    fn from(public_key: Vec<u8>) -> Self {
        Self { public_key }
    }
}

#[derive(CandidType, Serialize, Debug)]
pub struct SignatureReply {
    pub signature: Vec<u8>,
}

impl From<Vec<u8>> for SignatureReply {
    fn from(signature: Vec<u8>) -> Self {
        Self { signature }
    }
}

#[derive(CandidType, Serialize, Debug)]
pub struct SignatureVerificationReply {
    pub is_signature_valid: bool,
}

impl From<bool> for SignatureVerificationReply {
    fn from(is_signature_valid: bool) -> Self {
        Self { is_signature_valid }
    }
}

#[derive(CandidType, Serialize, Debug, Clone)]
pub struct EcdsaKeyId {
    pub curve: EcdsaCurve,
    pub name: String,
}

#[derive(CandidType, Serialize, Debug, Clone)]
pub enum EcdsaCurve {
    #[serde(rename = "secp256k1")]
    Secp256k1,
}

pub enum EcdsaKeyIds {
    #[allow(unused)]
    TestKeyLocalDevelopment,
    #[allow(unused)]
    TestKey1,
    #[allow(unused)]
    ProductionKey1,
}

impl EcdsaKeyIds {
    pub fn to_key_id(&self) -> EcdsaKeyId {
        EcdsaKeyId {
            curve: EcdsaCurve::Secp256k1,
            name: match self {
                Self::TestKeyLocalDevelopment => "dfx_test_key",
                Self::TestKey1 => "test_key_1",
                Self::ProductionKey1 => "key_1",
            }
            .to_string(),
        }
    }
}

#[derive(CandidType, Clone, Copy, Deserialize, Debug, Eq, PartialEq, Serialize, Hash)]
pub enum Network {
    #[serde(rename = "local")]
    Local,
    #[serde(rename = "testnet")]
    Testnet,
    #[serde(rename = "mainnet")]
    Mainnet,
}

impl Network {
    pub fn key_id(&self) -> EcdsaKeyId {
        match self {
            Network::Local => EcdsaKeyIds::TestKeyLocalDevelopment.to_key_id(),
            Network::Testnet => EcdsaKeyIds::TestKey1.to_key_id(),
            Network::Mainnet => EcdsaKeyIds::ProductionKey1.to_key_id(),
        }
    }
}

impl core::fmt::Display for Network {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Self::Local => write!(f, "local"),
            Self::Testnet => write!(f, "testnet"),
            Self::Mainnet => write!(f, "mainnet"),
        }
    }
}

impl FromStr for Network {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "regtest" => Ok(Network::Local),
            "testnet" => Ok(Network::Testnet),
            "mainnet" => Ok(Network::Mainnet),
            _ => Err(Error::CustomError("Bad network".to_string())),
        }
    }
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

#[derive(candid::CandidType, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MintTokenStatus {
    Finalized { tx_hash: String },
    Unknown,
}
