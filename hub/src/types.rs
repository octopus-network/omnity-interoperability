use ic_stable_structures::storable::Bound;
use ic_stable_structures::Storable;
use omnity_types::Chain;
use omnity_types::ChainState;
use omnity_types::ChainType;

use candid::CandidType;
use ic_stable_structures::StableBTreeMap;
use omnity_types::DireKey;
use omnity_types::Directive;
use omnity_types::Fee;
use omnity_types::SeqKey;
use omnity_types::Ticket;
use omnity_types::ToggleState;
use omnity_types::Token;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::HashMap;

use crate::memory::Memory;

pub type Seq = u64;
pub type Amount = u128;
pub type ChainId = String;
pub type DstChain = ChainId;
pub type TokenId = String;
// pub type Directive = Proposal;

/// Directive Queue
/// K: DstChain, V:  BTreeMap<Seq, Directive>
// pub type DireQueue = StableBTreeMap<DstChain, BTreeMap<Seq, Directive>, Memory>;
pub type DireQueue = StableBTreeMap<DireKey, Directive, Memory>;
/// Ticket Queue
/// K: DstChain, V: BTreeMap<Seq, Ticket>
// pub type TicketQueue = StableBTreeMap<DstChain, BTreeMap<Seq, Ticket>, Memory>;
pub type TicketQueue = StableBTreeMap<SeqKey, Ticket, Memory>;

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub enum Proposal {
    AddChain(ChainMeta),
    AddToken(TokenMeta),
    //TODO: UpdateChain(ChainMeta)
    //TOOD: UpdateToken(TokenMeta)
    ToggleChainState(ToggleState),
    UpdateFee(Fee),
}

impl Storable for Proposal {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        let mut bytes = vec![];
        let _ = ciborium::ser::into_writer(self, &mut bytes);
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        let proposal =
            ciborium::de::from_reader(bytes.as_ref()).expect("failed to decode TokenKey");
        proposal
    }

    const BOUND: Bound = Bound::Unbounded;
}

/// chain id spec:
/// for settlement chain, the chain id is: Bitcoin, Ethereum,or ICP
/// for execution chain, the chain id spec is: type-chain_name,eg: EVM-Base,Cosmos-Gaia, Substrate-Xxx
#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq, Eq, Hash)]
pub struct ChainMeta {
    pub chain_id: ChainId,
    pub canister_id: String,
    pub chain_type: ChainType,
    // the chain default state is active
    pub chain_state: ChainState,
    // settlement chain: export contract address
    // execution chain: port contract address
    pub contract_address: Option<String>,

    // optional counterparty chains
    pub counterparties: Option<Vec<ChainId>>,
}

impl Storable for ChainMeta {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        let mut bytes = vec![];
        let _ = ciborium::ser::into_writer(self, &mut bytes);
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        let cm = ciborium::de::from_reader(bytes.as_ref()).expect("failed to decode TokenKey");
        cm
    }

    const BOUND: Bound = Bound::Unbounded;
}

impl core::fmt::Display for ChainMeta {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(
            f,
            "\ncanister id:{} \nchain name:{} \nchain type:{:?} \nchain state:{:?} \ncontract address:{:?} \ncounterparties:{:?}",
            self.canister_id,self.chain_id, self.chain_type, self.chain_state, self.contract_address,self.counterparties
        )
    }
}

impl Into<Chain> for ChainMeta {
    fn into(self) -> Chain {
        Chain {
            chain_id: self.chain_id,
            chain_type: self.chain_type,
            chain_state: self.chain_state,
            contract_address: self.contract_address,
        }
    }
}

#[derive(CandidType, Deserialize, Serialize, Default, Clone, Debug)]
pub struct ChainWithSeq {
    pub canister_id: String,
    pub chain_id: ChainId,
    pub chain_type: ChainType,
    pub chain_state: ChainState,
    pub contract_address: Option<String>,
    pub counterparties: Option<Vec<ChainId>>,
    pub latest_dire_seq: Seq,
    pub latest_ticket_seq: Seq,
}

impl From<ChainMeta> for ChainWithSeq {
    fn from(value: ChainMeta) -> Self {
        Self {
            canister_id: value.canister_id,
            chain_id: value.chain_id,
            chain_type: value.chain_type,
            chain_state: value.chain_state,
            contract_address: value.contract_address,
            counterparties: value.counterparties,
            latest_dire_seq: 0,
            latest_ticket_seq: 0,
        }
    }
}
impl Into<Chain> for ChainWithSeq {
    fn into(self) -> Chain {
        Chain {
            chain_id: self.chain_id.to_string(),
            chain_type: self.chain_type.clone(),
            chain_state: self.chain_state.clone(),
            contract_address: self.contract_address.clone(),
        }
    }
}
impl Storable for ChainWithSeq {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        let mut bytes = vec![];
        let _ = ciborium::ser::into_writer(self, &mut bytes);
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        let cs = ciborium::de::from_reader(bytes.as_ref()).expect("failed to decode TokenKey");
        cs
    }

    const BOUND: Bound = Bound::Unbounded;
}
/// token id spec is setllmentchain_name-potocol-symbol, eg:  Bitcoin-RUNES-WHAT•ABOUT•THIS•RUNE,Ethereurm-ERC20-OCT,ICP-ICRC2-XO
/// metadata stores extended information，for runes protocol token, it stores the runes id
#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub struct TokenMeta {
    pub token_id: TokenId,
    pub symbol: String,
    // the token`s setllment chain
    pub settlement_chain: ChainId,
    pub decimals: u8,
    pub icon: Option<String>,
    pub metadata: Option<HashMap<String, String>>,
    pub dst_chains: Vec<ChainId>,
    // pub token_constract_address: Option<String>,
}

impl Storable for TokenMeta {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        let mut bytes = vec![];
        let _ = ciborium::ser::into_writer(self, &mut bytes);
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        let tm = ciborium::de::from_reader(bytes.as_ref()).expect("failed to decode TokenKey");
        tm
    }

    const BOUND: Bound = Bound::Unbounded;
}

impl core::fmt::Display for TokenMeta {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(
            f,
            "\ntoken name:{} \nsymbol:{:?} \nissue chain:{} \ndecimals:{} \nicon:{:?} \nmetadata:{:?} \ndst chains:{:?}",
            self.token_id, self.symbol, self.settlement_chain, self.decimals, self.icon,self.metadata,self.dst_chains
        )
    }
}

impl Into<Token> for TokenMeta {
    fn into(self) -> Token {
        Token {
            token_id: self.token_id,
            symbol: self.symbol,
            issue_chain: self.settlement_chain,
            decimals: self.decimals,
            icon: self.icon,
            metadata: self.metadata,
        }
    }
}

/// This struct as HashMap key to find the token or else info
#[derive(
    CandidType, Deserialize, Serialize, Clone, Debug, PartialOrd, Ord, PartialEq, Eq, Hash,
)]
pub struct TokenKey {
    pub chain_id: ChainId,
    pub token_id: TokenId,
}

impl TokenKey {
    pub fn from(chain_id: ChainId, token_id: TokenId) -> Self {
        Self { chain_id, token_id }
    }
}

impl Storable for TokenKey {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        let mut bytes = vec![];
        let _ = ciborium::ser::into_writer(self, &mut bytes);
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        let token_key =
            ciborium::de::from_reader(bytes.as_ref()).expect("failed to decode TokenKey");
        token_key
    }

    const BOUND: Bound = Bound::Unbounded;
}
