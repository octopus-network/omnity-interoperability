use candid::CandidType;

use omnity_types::Chain;
use omnity_types::ChainState;
use omnity_types::ChainType;

use omnity_types::Directive;
use omnity_types::Fee;
use omnity_types::Ticket;
use omnity_types::ToggleState;
use omnity_types::Token;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::collections::HashMap;

pub type Seq = u64;

// pub type Directive = Proposal;
pub type ChainId = String;
pub type DstChain = ChainId;
pub type TokenId = String;

/// Directive Queue
/// K: DstChain, V:  BTreeMap<Seq, Directive>
pub type DireQueue = HashMap<DstChain, BTreeMap<Seq, Directive>>;
/// Ticket Queue
/// K: DstChain, V: BTreeMap<Seq, Ticket>
pub type TicketQueue = HashMap<DstChain, BTreeMap<Seq, Ticket>>;

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub enum Proposal {
    AddChain(ChainMeta),
    AddToken(TokenMeta),
    //TODO: UpdateChain(ChainMeta)
    //TOOD: UpdateToken(TokenMeta)
    ToggleChainState(ToggleState),
    UpdateFee(Fee),
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq, Eq, Hash)]
pub struct ChainMeta {
    pub canister_id: String,
    pub chain_id: ChainId,
    pub chain_type: ChainType,
    // the chain default state is true
    pub chain_state: ChainState,
    // settlement chain: export contract address
    // execution chain: port contract address
    pub contract_address: Option<String>,

    // optional counterparty chains
    pub counterparties: Option<Vec<ChainId>>,
}

impl core::fmt::Display for ChainMeta {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(
            f,
            "\nchain name:{} \nchain type:{:?} \nchain state:{:?} \ncontract address:{:?} \ncounterparties:{:#?}",
            self.chain_id, self.chain_type, self.chain_state, self.contract_address,self.counterparties
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

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq, Eq, Hash)]
pub struct TokenMeta {
    pub token_id: TokenId,
    pub symbol: String,
    // the token`s issuse chain
    pub issue_chain: ChainId,
    pub decimals: u8,
    pub icon: Option<String>,
    pub dst_chains: Vec<ChainId>,
    // pub total_amount: Option<u128>,
    // pub token_constract_address: Option<String>,
}
impl core::fmt::Display for TokenMeta {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(
            f,
            "\ntoken name:{} \nsymbol:{:?} \nissue chain:{} \ndecimals:{} \nicon:{:?} \n dst chains:{:?}",
            self.token_id, self.symbol, self.issue_chain, self.decimals, self.icon,self.dst_chains
        )
    }
}

impl Into<Token> for TokenMeta {
    fn into(self) -> Token {
        Token {
            token_id: self.token_id,
            symbol: self.symbol,
            issue_chain: self.issue_chain,
            decimals: self.decimals,
            icon: self.icon,
        }
    }
}
