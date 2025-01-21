use std::borrow::Cow;
use std::collections::BTreeSet;
use std::collections::HashMap;

use candid::CandidType;
use ic_stable_structures::Storable;
use ic_stable_structures::storable::Bound;
use serde::{Deserialize, Serialize};

use crate::{Chain, ChainId, ChainState, ChainType, Factor, ToggleState, Token, TokenId};



#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub enum Proposal {
    AddChain(ChainMeta),
    AddToken(TokenMeta),
    UpdateChain(ChainMeta),
    UpdateToken(TokenMeta),
    ToggleChainState(ToggleState),
    UpdateFee(Factor),
}

impl Storable for Proposal {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        let mut bytes = vec![];
        let _ = ciborium::ser::into_writer(self, &mut bytes);
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        let proposal =
            ciborium::de::from_reader(bytes.as_ref()).expect("failed to decode Proposal");
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
    // fee token
    pub fee_token: Option<TokenId>,
}

impl ChainMeta {
    pub fn add_counterparty(&mut self, chain_id: ChainId) {
        match &mut self.counterparties {
            None => {
                self.counterparties = Some(vec![chain_id])
            }
            Some(v) => {
                if !v.contains(&chain_id) {
                    v.push(chain_id)
                }
            }
        }
    }

    pub fn contains_counterparty(&self, chain_id: &ChainId) -> bool {
        match &self.counterparties {
            None => { false }
            Some(c) => {
                c.contains(chain_id)
            }
        }
    }
}

impl Storable for ChainMeta {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        let mut bytes = vec![];
        let _ = ciborium::ser::into_writer(self, &mut bytes);
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        let cm = ciborium::de::from_reader(bytes.as_ref()).expect("failed to decode ChainMeta");
        cm
    }

    const BOUND: Bound = Bound::Unbounded;
}

impl core::fmt::Display for ChainMeta {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(
            f,
            "\nchain id:{} \ncanister id:{} \nchain type:{:?} \nchain state:{:?} \ncontract address:{:?} \ncounterparties:{:?} \nfee_token:{:?}",
            self.chain_id,self.canister_id, self.chain_type, self.chain_state, self.contract_address,self.counterparties,self.fee_token,
        )
    }
}

impl Into<Chain> for ChainMeta {
    fn into(self) -> Chain {
        Chain {
            chain_id: self.chain_id,
            canister_id: self.canister_id,
            chain_type: self.chain_type,
            chain_state: self.chain_state,
            contract_address: self.contract_address,
            counterparties: self.counterparties,
            fee_token: self.fee_token,
        }
    }
}

/// token id spec is setllmentchain_name-potocol-symbol, eg:  Bitcoin-RUNES-WHAT•ABOUT•THIS•RUNE,Ethereurm-ERC20-OCT,ICP-ICRC2-XO
/// metadata stores extended information，for runes protocol token, it stores the runes id
#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub struct TokenMeta {
    pub token_id: TokenId,
    pub name: String,
    pub symbol: String,
    // the token`s setllment chain
    pub issue_chain: ChainId,
    pub decimals: u8,
    pub icon: Option<String>,
    pub metadata: HashMap<String, String>,
    pub dst_chains: Vec<ChainId>,
}

impl Storable for TokenMeta {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        let mut bytes = vec![];
        let _ = ciborium::ser::into_writer(self, &mut bytes);
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        let tm = ciborium::de::from_reader(bytes.as_ref()).expect("failed to decode TokenMeta");
        tm
    }

    const BOUND: Bound = Bound::Unbounded;
}

impl core::fmt::Display for TokenMeta {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(
            f,
            "\ntoken id:{} \ntoken name:{} \nsymbol:{:?} \nissue chain:{} \ndecimals:{} \nicon:{:?} \nmetadata:{:?} \ndst chains:{:?}",
            self.token_id, self.name,self.symbol, self.issue_chain, self.decimals, self.icon,self.metadata,self.dst_chains
        )
    }
}

impl Into<Token> for TokenMeta {
    fn into(self) -> Token {
        Token {
            token_id: self.token_id,
            name: self.name,
            symbol: self.symbol,
            decimals: self.decimals,
            icon: self.icon,
            metadata: self.metadata,
        }
    }
}

#[derive(CandidType, Clone, Debug, Deserialize, Serialize)]
pub struct TokenResp {
    pub token_id: TokenId,
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub icon: Option<String>,
    pub rune_id: Option<String>,
}

impl From<Token> for TokenResp {
    fn from(value: Token) -> Self {
        TokenResp {
            token_id: value.token_id,
            name: value.name,
            symbol: value.symbol,
            decimals: value.decimals,
            icon: value.icon,
            rune_id: value.metadata.get("rune_id").cloned(),
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

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq, Eq, Default)]
pub struct ChainTokenFactor {
    pub target_chain_id: ChainId,
    pub fee_token: TokenId,
    pub fee_token_factor: u128,
}

impl Storable for ChainTokenFactor {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        let mut bytes = vec![];
        let _ = ciborium::ser::into_writer(self, &mut bytes);
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        let fee =
            ciborium::de::from_reader(bytes.as_ref()).expect("failed to decode ChainTokenFactor");
        fee
    }

    const BOUND: Bound = Bound::Unbounded;
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq, Eq, Default)]
pub struct Subscribers {
    pub subs: BTreeSet<String>,
}

impl Storable for Subscribers {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        let mut bytes = vec![];
        let _ = ciborium::ser::into_writer(self, &mut bytes);
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        let subs = ciborium::de::from_reader(bytes.as_ref()).expect("failed to decode Subscribers");
        subs
    }

    const BOUND: Bound = Bound::Unbounded;
}

