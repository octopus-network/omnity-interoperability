use crate::constants::IC_GATEWAY;
use crate::eddsa::KeyType;
use crate::handler::gen_ticket::GenerateTicketReq;
use crate::memory::Memory;
use crate::solana_client::solana_rpc::{SolanaClient, TxError};
use crate::{
    auth::Permission,
    constants::{FEE_ACCOUNT, FEE_TOKEN, SCHNORR_KEY_NAME},
    guard::TaskType,
    lifecycle::InitArgs,
};
use candid::{CandidType, Principal};
use ic_canister_log::{export as export_logs, GlobalBuffer};
use ic_canisters_http_types::{HttpRequest, HttpResponse, HttpResponseBuilder};
use ic_cdk::api::management_canister::http_request::HttpHeader;
use ic_solana::logs::{
    Log, LogEntry, Priority as LogPriority, CRITICAL_BUF, DEBUG_BUF, ERROR_BUF, INFO_BUF,
    WARNING_BUF,
};
use ic_spl::compute_budget::compute_budget::Priority;
use ic_stable_structures::StableBTreeMap;

use crate::handler::mint_token::MintTokenRequest;
use crate::types::{
    Chain, ChainId, ChainState, Factor, Ticket, TicketId, ToggleState, Token, TokenId,
};
use ic_stable_structures::storable::Bound;
use ic_stable_structures::Storable;

use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::{
    cell::RefCell,
    collections::{BTreeMap, HashMap, HashSet},
};
use time::OffsetDateTime;

pub type CanisterId = Principal;
pub type Owner = String;
pub type MintAccount = String;
pub type AssociatedAccount = String;

thread_local! {
    static STATE: RefCell<Option<SolanaRouteState>> = RefCell::default();
}

#[derive(CandidType, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TxStatus {
    New,
    Pending,
    Finalized,
    TxFailed { e: TxError },
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

#[derive(CandidType, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct AccountInfo {
    pub account: String,
    pub retry_4_building: u64,
    pub retry_4_status: u64,
    pub signature: Option<String>,
    pub status: TxStatus,
}

impl Storable for AccountInfo {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        let mut bytes = vec![];
        let _ = ciborium::ser::into_writer(self, &mut bytes);
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        let tm = ciborium::de::from_reader(bytes.as_ref()).expect("failed to decode AccountInfo");
        tm
    }

    const BOUND: Bound = Bound::Unbounded;
}

#[derive(CandidType, Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct AtaKey {
    pub owner: String,
    pub token_mint: String,
}

impl Storable for AtaKey {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        let mut bytes = vec![];
        let _ = ciborium::ser::into_writer(self, &mut bytes);
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        let tm = ciborium::de::from_reader(bytes.as_ref()).expect("failed to decode AtaKey");
        tm
    }

    const BOUND: Bound = Bound::Unbounded;
}

impl AtaKey {
    pub fn new(owner: String, token_mint: String) -> Self {
        Self { owner, token_mint }
    }
}

#[derive(CandidType, Clone, Debug, Serialize, Deserialize)]
pub struct UpdateToken {
    pub token: Token,
    pub retry_4_building: u64,
    pub retry_4_status: u64,
    pub signature: Option<String>,
    pub status: TxStatus,
}

impl Storable for UpdateToken {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        let mut bytes = vec![];
        let _ = ciborium::ser::into_writer(self, &mut bytes);
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        let tm = ciborium::de::from_reader(bytes.as_ref()).expect("failed to decode UpdateToken");
        tm
    }

    const BOUND: Bound = Bound::Unbounded;
}

impl UpdateToken {
    pub fn new(token: Token) -> Self {
        Self {
            token,
            retry_4_building: 0,
            retry_4_status: 0,
            signature: None,
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

#[derive(CandidType, Clone, Debug, Serialize, Deserialize)]
pub struct TokenUri {
    pub name: String,
    pub symbol: String,
    pub uri: String,
}

impl From<Token> for TokenUri {
    fn from(value: Token) -> Self {
        TokenUri {
            name: value.name,
            symbol: value.symbol,
            uri: format!(
                "https://{}.{}/token_meta?id={}",
                ic_cdk::api::id().to_text(),
                IC_GATEWAY,
                value.token_id.to_string()
            ),
        }
    }
}

#[derive(CandidType, Clone, Debug, Serialize, Deserialize)]
pub struct TokenMeta {
    pub name: String,
    pub symbol: String,
    pub description: String,
    pub image: String,
}

impl From<Token> for TokenMeta {
    fn from(value: Token) -> Self {
        TokenMeta {
            name: value.name,
            symbol: value.symbol,
            description: value.token_id,
            image: value.icon.unwrap_or_default(),
        }
    }
}

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, Default, PartialEq, Eq)]
pub struct Seqs {
    pub next_ticket_seq: u64,
    pub next_directive_seq: u64,
}

#[derive(candid::CandidType, Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct RpcProvider {
    pub host: String,
    pub api_key_param: Option<String>,
    pub headers: Option<Vec<HttpHeader>>,
}

impl RpcProvider {
    pub fn rpc_url(&self) -> String {
        format!(
            "https://{}{}",
            self.host,
            self.api_key_param
                .clone()
                .map_or("".into(), |param| format!("/?{}", param))
        )
    }
}

pub const KEY_TYPE_NAME: &str = "Native";
#[derive(CandidType, Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub enum SnorKeyType {
    ChainKey,
    Native,
}

impl From<KeyType> for SnorKeyType {
    fn from(key_type: KeyType) -> Self {
        match key_type {
            KeyType::ChainKey => SnorKeyType::ChainKey,
            KeyType::Native(_) => SnorKeyType::Native,
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct SolanaRouteState {
    pub chain_id: String,
    pub hub_principal: Principal,
    pub seqs: Seqs,
    pub fee_token_factor: Option<u128>,
    pub target_chain_factor: BTreeMap<ChainId, u128>,
    pub chain_state: ChainState,
    pub schnorr_key_name: String,
    pub sol_canister: Principal,
    pub fee_account: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub solana_client_cache: Option<(KeyType, SolanaClient)>,
    // Locks preventing concurrent execution timer tasks
    pub active_tasks: HashSet<TaskType>,
    pub admin: Principal,
    pub caller_perms: HashMap<String, Permission>,

    pub enable_debug: bool,
    pub priority: Option<Priority>,
    pub key_type: KeyType,

    pub providers: Vec<RpcProvider>,
    pub proxy: String,
    pub minimum_response_count: u32,

    // stable storage
    #[serde(skip, default = "crate::memory::init_ticket_queue")]
    pub tickets_queue: StableBTreeMap<u64, Ticket, Memory>,
    #[serde(skip, default = "crate::memory::init_failed_tickets")]
    pub tickets_failed_to_hub: StableBTreeMap<String, Ticket, Memory>,
    #[serde(skip, default = "crate::memory::init_counterparties")]
    pub counterparties: StableBTreeMap<ChainId, Chain, Memory>,
    #[serde(skip, default = "crate::memory::init_tokens")]
    pub tokens: StableBTreeMap<TokenId, Token, Memory>,
    #[serde(skip, default = "crate::memory::init_update_tokens_v2")]
    pub update_token_queue: StableBTreeMap<TokenId, UpdateToken, Memory>,
    #[serde(skip, default = "crate::memory::init_token_mint_accounts_v2")]
    pub token_mint_accounts: StableBTreeMap<TokenId, AccountInfo, Memory>,
    #[serde(skip, default = "crate::memory::init_associated_accounts_v2")]
    pub associated_accounts: StableBTreeMap<AtaKey, AccountInfo, Memory>,
    #[serde(skip, default = "crate::memory::init_mint_token_requests_v2")]
    pub mint_token_requests: StableBTreeMap<TicketId, MintTokenRequest, Memory>,
    #[serde(skip, default = "crate::memory::init_gen_ticket_reqs")]
    pub gen_ticket_reqs: StableBTreeMap<TicketId, GenerateTicketReq, Memory>,
    #[serde(skip, default = "crate::memory::init_seed")]
    pub seeds: StableBTreeMap<String, [u8; 64], Memory>,
}

impl From<InitArgs> for SolanaRouteState {
    fn from(args: InitArgs) -> Self {
        Self {
            chain_id: args.chain_id,
            hub_principal: args.hub_principal,
            seqs: Seqs::default(),
            fee_token_factor: None,
            target_chain_factor: Default::default(),
            chain_state: args.chain_state,
            schnorr_key_name: args
                .schnorr_key_name
                .unwrap_or(SCHNORR_KEY_NAME.to_string()),
            sol_canister: args.sol_canister,
            active_tasks: Default::default(),
            admin: args.admin,
            caller_perms: HashMap::from([(args.admin.to_string(), Permission::Update)]),
            fee_account: args.fee_account.unwrap_or(FEE_ACCOUNT.to_string()),
            solana_client_cache: None,

            enable_debug: false,
            priority: Some(Priority::None),
            key_type: KeyType::ChainKey,
            providers: args.providers,
            proxy: args.proxy,
            minimum_response_count: args.minimum_response_count,
            // init stable storage
            tickets_queue: StableBTreeMap::init(crate::memory::get_ticket_queue_memory()),
            tickets_failed_to_hub: StableBTreeMap::init(crate::memory::get_failed_tickets_memory()),
            counterparties: StableBTreeMap::init(crate::memory::get_counterparties_memory()),
            tokens: StableBTreeMap::init(crate::memory::get_tokens_memory()),
            update_token_queue: StableBTreeMap::init(crate::memory::get_update_tokens_v2_memory()),
            token_mint_accounts: StableBTreeMap::init(
                crate::memory::get_token_mint_accounts_v2_memory(),
            ),
            associated_accounts: StableBTreeMap::init(
                crate::memory::get_associated_accounts_v2_memory(),
            ),
            mint_token_requests: StableBTreeMap::init(
                crate::memory::get_mint_token_requests_v2_memory(),
            ),
            gen_ticket_reqs: StableBTreeMap::init(crate::memory::get_gen_ticket_req_memory()),
            seeds: StableBTreeMap::init(crate::memory::get_seeds_memory()),
        }
    }
}

impl SolanaRouteState {
    pub fn validate_config(&self) {}

    pub fn add_chain(&mut self, chain: Chain) {
        self.counterparties
            .insert(chain.chain_id.to_owned(), chain.to_owned());
    }

    pub fn add_token(&mut self, token: Token) {
        self.tokens.insert(token.token_id.clone(), token.clone());
    }

    pub fn toggle_chain_state(&mut self, toggle: ToggleState) {
        if toggle.chain_id == self.chain_id {
            self.chain_state = toggle.action.into();
        } else if let Some(chain) = self.counterparties.get(&toggle.chain_id).as_mut() {
            chain.chain_state = toggle.action.into();
            // update chain state
            self.counterparties
                .insert(chain.chain_id.to_string(), chain.to_owned());
        }
    }

    pub fn sol_token_account(&self, ticket_id: &String) -> Option<AccountInfo> {
        self.token_mint_accounts.get(ticket_id)
    }

    pub fn update_mint_token_req(&mut self, ticket_id: String, req: MintTokenRequest) {
        self.mint_token_requests.insert(ticket_id, req);
    }

    pub fn update_fee(&mut self, fee: Factor) {
        match fee {
            Factor::UpdateTargetChainFactor(factor) => {
                self.target_chain_factor.insert(
                    factor.target_chain_id.to_owned(),
                    factor.target_chain_factor,
                );
            }

            Factor::UpdateFeeTokenFactor(token_factor) => {
                if token_factor.fee_token == FEE_TOKEN {
                    self.fee_token_factor = Some(token_factor.fee_token_factor);
                }
            }
        }
    }
    pub fn get_fee(&self, chain_id: ChainId) -> Option<u128> {
        read_state(|s| {
            s.target_chain_factor
                .get(&chain_id)
                .map_or(None, |target_chain_factor| {
                    s.fee_token_factor
                        .map(|fee_token_factor| target_chain_factor * fee_token_factor)
                })
        })
    }
}

pub fn take_state<F, R>(f: F) -> R
where
    F: FnOnce(SolanaRouteState) -> R,
{
    STATE.with(|s| f(s.take().expect("State not initialized!")))
}

pub fn mutate_state<F, R>(f: F) -> R
where
    F: FnOnce(&mut SolanaRouteState) -> R,
{
    STATE.with(|s| f(s.borrow_mut().as_mut().expect("State not initialized!")))
}

pub fn read_state<F, R>(f: F) -> R
where
    F: FnOnce(&SolanaRouteState) -> R,
{
    STATE.with(|s| f(s.borrow().as_ref().expect("State not initialized!")))
}

pub fn replace_state(state: SolanaRouteState) {
    STATE.with(|s| {
        *s.borrow_mut() = Some(state);
    });
}

pub fn http_log(req: HttpRequest, enable_debug: bool) -> HttpResponse {
    use std::str::FromStr;
    let max_skip_timestamp = match req.raw_query_param("time") {
        Some(arg) => match u64::from_str(arg) {
            Ok(value) => value,
            Err(_) => {
                return HttpResponseBuilder::bad_request()
                    .with_body_and_content_length("failed to parse the 'time' parameter")
                    .build()
            }
        },
        None => 0,
    };

    let limit = match req.raw_query_param("limit") {
        Some(arg) => match u64::from_str(arg) {
            Ok(value) => value,
            Err(_) => {
                return HttpResponseBuilder::bad_request()
                    .with_body_and_content_length("failed to parse the 'time' parameter")
                    .build()
            }
        },
        None => 1000,
    };

    let offset = match req.raw_query_param("offset") {
        Some(arg) => match u64::from_str(arg) {
            Ok(value) => value,
            Err(_) => {
                return HttpResponseBuilder::bad_request()
                    .with_body_and_content_length("failed to parse the 'time' parameter")
                    .build()
            }
        },
        None => 0,
    };

    let mut entries: Log = Default::default();
    if enable_debug {
        merge_log(&mut entries, &DEBUG_BUF, LogPriority::DEBUG);
    }
    merge_log(&mut entries, &INFO_BUF, LogPriority::INFO);
    merge_log(&mut entries, &WARNING_BUF, LogPriority::WARNING);
    merge_log(&mut entries, &ERROR_BUF, LogPriority::ERROR);
    merge_log(&mut entries, &CRITICAL_BUF, LogPriority::CRITICAL);
    entries
        .entries
        .retain(|entry| entry.timestamp >= max_skip_timestamp);
    entries
        .entries
        .sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
    let logs = entries
        .entries
        .into_iter()
        .skip(offset as usize)
        .take(limit as usize)
        .collect::<Vec<_>>();
    HttpResponseBuilder::ok()
        .header("Content-Type", "application/json; charset=utf-8")
        .with_body_and_content_length(serde_json::to_string(&logs).unwrap_or_default())
        .build()
}

fn merge_log(entries: &mut Log, buffer: &'static GlobalBuffer, priority: LogPriority) {
    let canister_id = ic_cdk::api::id();
    for entry in export_logs(buffer) {
        entries.entries.push(LogEntry {
            timestamp: entry.timestamp,
            canister_id: canister_id.to_string(),
            time_str: OffsetDateTime::from_unix_timestamp_nanos(entry.timestamp as i128)
                .unwrap()
                .to_string(),
            counter: entry.counter,
            priority: priority,
            file: entry.file.to_string(),
            line: entry.line,
            message: entry.message,
        });
    }
}
