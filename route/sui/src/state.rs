#![allow(unused)]
use crate::config::{mutate_config, read_config, SuiRouteConfig};
use crate::handler::burn_token::BurnTx;
use crate::handler::clear_ticket::ClearTx;
use crate::handler::gen_ticket::GenerateTicketReq;
use crate::ic_sui::ck_eddsa::KeyType;
use crate::lifecycle::InitArgs;
use crate::memory::Memory;
use candid::{CandidType, Principal};
use ic_stable_structures::StableBTreeMap;
use ic_stable_structures::StableCell;

use crate::handler::mint_token::MintTokenRequest;
use crate::types::{Chain, ChainId, Ticket, TicketId, ToggleState, Token, TokenId};
use ic_stable_structures::storable::Bound;
use ic_stable_structures::Storable;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::{cell::RefCell, collections::HashSet};

pub type CanisterId = Principal;
pub type Owner = String;
pub type MintAccount = String;
pub type AssociatedAccount = String;

thread_local! {

    static STATE: RefCell<Option<SuiRouteState>> = RefCell::default();
}

#[derive(CandidType, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TxStatus {
    New,
    Pending,
    Finalized,
    TxFailed { e: String },
}

#[derive(CandidType, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccountType {
    MintAccount,
    AssociatedAccount,
}

#[derive(CandidType, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccountStatus {
    Confirmed,
    Unknown,
}

#[derive(
    CandidType, Clone, Debug, Serialize, Deserialize, PartialOrd, Ord, PartialEq, Eq, Hash,
)]
pub enum UpdateType {
    Name(String),
    Symbol(String),
    Icon(String),
    Description(String),
}

impl Storable for UpdateType {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        let bytes = bincode::serialize(&self).expect("failed to serialize UpdateType");
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        bincode::deserialize(bytes.as_ref()).expect("failed to deserialize UpdateType")
    }

    const BOUND: Bound = Bound::Unbounded;
}

#[derive(CandidType, Clone, Debug, Serialize, Deserialize)]
pub struct UpdateTokenStatus {
    pub token_id: TokenId,
    pub retry: u64,
    pub degist: Option<String>,
    pub status: TxStatus,
}

impl Storable for UpdateTokenStatus {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        let bytes = bincode::serialize(&self).expect("failed to serialize UpdateTokenStatus");
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        bincode::deserialize(bytes.as_ref()).expect("failed to deserialize UpdateTokenStatus")
    }

    const BOUND: Bound = Bound::Unbounded;
}

impl UpdateTokenStatus {
    pub fn new(token_id: TokenId) -> Self {
        Self {
            token_id,
            // update_type,
            retry: 0,
            degist: None,
            status: TxStatus::New,
        }
    }
}

#[derive(CandidType, Clone, Debug, Serialize, Deserialize)]
pub struct TokenResp {
    pub token_id: TokenId,
    pub symbol: String,
    pub decimals: u8,
    pub icon: Option<String>,
    pub rune_id: Option<String>,
}

impl From<Token> for TokenResp {
    fn from(value: Token) -> Self {
        TokenResp {
            token_id: value.token_id,
            symbol: value.symbol,
            decimals: value.decimals,
            icon: value.icon,
            rune_id: value.metadata.get("rune_id").map(|rune_id| rune_id.clone()),
        }
    }
}

#[derive(CandidType, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SuiToken {
    pub package: String,
    pub module: String,
    pub functions: HashSet<String>,
    pub treasury_cap: String,
    pub metadata: String,
    pub type_tag: String,
    pub upgrade_cap: String,
}

impl Storable for SuiToken {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        let bytes = bincode::serialize(&self).expect("failed to serialize SuiTokenInfo");
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        bincode::deserialize(bytes.as_ref()).expect("failed to deserialize SuiTokenInfo")
    }

    const BOUND: Bound = Bound::Unbounded;
}

#[derive(Deserialize, Serialize)]
pub struct SuiRouteState {
    // stable storage
    // #[serde(skip, default = "crate::memory::init_config")]
    // pub route_config: StableCell<SuiRouteConfig, Memory>,
    #[serde(skip, default = "crate::memory::init_ticket_queue")]
    pub tickets_queue: StableBTreeMap<u64, Ticket, Memory>,
    #[serde(skip, default = "crate::memory::init_failed_tickets")]
    pub tickets_failed_to_hub: StableBTreeMap<String, Ticket, Memory>,
    #[serde(skip, default = "crate::memory::init_counterparties")]
    pub counterparties: StableBTreeMap<ChainId, Chain, Memory>,
    #[serde(skip, default = "crate::memory::init_tokens")]
    pub tokens: StableBTreeMap<TokenId, Token, Memory>,
    #[serde(skip, default = "crate::memory::init_sui_tokens")]
    pub sui_tokens: StableBTreeMap<TokenId, SuiToken, Memory>,
    #[serde(skip, default = "crate::memory::init_update_tokens")]
    pub update_token_queue: StableBTreeMap<UpdateType, UpdateTokenStatus, Memory>,
    #[serde(skip, default = "crate::memory::init_mint_token_requests")]
    pub mint_token_requests: StableBTreeMap<TicketId, MintTokenRequest, Memory>,
    #[serde(skip, default = "crate::memory::init_gen_ticket_reqs")]
    pub gen_ticket_reqs: StableBTreeMap<TicketId, GenerateTicketReq, Memory>,
    #[serde(skip, default = "crate::memory::init_seed")]
    pub seeds: StableBTreeMap<String, [u8; 64], Memory>,
    #[serde(skip, default = "crate::memory::init_sui_addresses")]
    pub sui_route_addresses: StableBTreeMap<KeyType, Vec<u8>, Memory>,
    #[serde(skip, default = "crate::memory::init_clr_ticket_queue")]
    pub clr_ticket_queue: StableBTreeMap<String, ClearTx, Memory>,
    #[serde(skip, default = "crate::memory::init_burn_tokens")]
    pub burn_tokens: StableBTreeMap<String, BurnTx, Memory>,
}

impl SuiRouteState {
    pub fn init() -> Self {
        Self {
            tickets_queue: StableBTreeMap::init(crate::memory::get_ticket_queue_memory()),
            tickets_failed_to_hub: StableBTreeMap::init(crate::memory::get_failed_tickets_memory()),
            counterparties: StableBTreeMap::init(crate::memory::get_counterparties_memory()),
            tokens: StableBTreeMap::init(crate::memory::get_tokens_memory()),
            sui_tokens: StableBTreeMap::init(crate::memory::get_sui_tokens_memory()),
            update_token_queue: StableBTreeMap::init(crate::memory::get_update_tokens_memory()),
            mint_token_requests: StableBTreeMap::init(
                crate::memory::get_mint_token_requests_memory(),
            ),
            gen_ticket_reqs: StableBTreeMap::init(crate::memory::get_gen_ticket_req_memory()),
            seeds: StableBTreeMap::init(crate::memory::get_seeds_memory()),
            sui_route_addresses: StableBTreeMap::init(crate::memory::get_sui_addresses_memory()),
            clr_ticket_queue: StableBTreeMap::init(crate::memory::get_clr_ticket_queue_memory()),
            burn_tokens: StableBTreeMap::init(crate::memory::get_burn_tokens_memory()),
        }
    }
    pub fn add_chain(&mut self, chain: Chain) {
        self.counterparties
            .insert(chain.chain_id.to_owned(), chain.to_owned());
    }

    pub fn add_token(&mut self, token: Token) {
        self.tokens.insert(token.token_id.to_owned(), token);
    }

    pub fn toggle_chain_state(&mut self, toggle: ToggleState) {
        let chain_id = read_config(|c| c.get().chain_id.to_owned());
        if toggle.chain_id == chain_id {
            mutate_config(|c| {
                let mut config = c.get().to_owned();
                config.chain_state = toggle.action.into();
                c.set(config);
            });
        } else if let Some(chain) = self.counterparties.get(&toggle.chain_id).as_mut() {
            chain.chain_state = toggle.action.into();
            // update chain state
            self.counterparties
                .insert(chain.chain_id.to_string(), chain.to_owned());
        }
    }

    pub fn update_mint_token_req(&mut self, ticket_id: String, req: MintTokenRequest) {
        self.mint_token_requests.insert(ticket_id, req);
    }
}

pub fn take_state<F, R>(f: F) -> R
where
    F: FnOnce(SuiRouteState) -> R,
{
    STATE.with(|s| f(s.take().expect("State not initialized!")))
}

pub fn mutate_state<F, R>(f: F) -> R
where
    F: FnOnce(&mut SuiRouteState) -> R,
{
    STATE.with(|s| f(s.borrow_mut().as_mut().expect("State not initialized!")))
}

pub fn read_state<F, R>(f: F) -> R
where
    F: FnOnce(&SuiRouteState) -> R,
{
    STATE.with(|s| f(s.borrow().as_ref().expect("State not initialized!")))
}

pub fn replace_state(state: SuiRouteState) {
    STATE.with(|s| {
        *s.borrow_mut() = Some(state);
    });
}
