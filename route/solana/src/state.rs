use crate::handler::gen_ticket::{GenerateTicketReq, TransactionDetail};
use crate::memory::Memory;
use crate::{
    auth::Permission,
    constants::{FEE_ACCOUNT, FEE_TOKEN, SCHNORR_KEY_NAME},
    guard::TaskType,
    lifecycle::InitArgs,
};
use candid::{CandidType, Principal};

use ic_canister_log::log;
use ic_solana::ic_log::{DEBUG, ERROR};
use ic_solana::rpc_client::JsonRpcResponse;
use ic_stable_structures::StableBTreeMap;

use crate::handler::gen_ticket::Instruction;
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
    TxFailed { e: String },
}

#[derive(CandidType, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccountStatus {
    Confirmed,
    Unknown,
}

#[derive(CandidType, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct AccountInfo {
    pub account: String,
    pub retry: u64,
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
    pub retry: u64,
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
    pub fn new(token: Token, retry: u64) -> Self {
        Self { token, retry }
    }
    pub fn update_token(&mut self, token: Token, retry: u64) {
        self.token = token;
        self.retry = retry;
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

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, Default, PartialEq, Eq)]
pub struct Seqs {
    pub next_ticket_seq: u64,
    pub next_directive_seq: u64,
}

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, Default, PartialEq, Eq)]
pub struct MultiRpcConfig {
    pub rpc_list: Vec<String>,
    pub minimum_response_count: u32,
}

impl MultiRpcConfig {
    pub fn new(rpc_list: Vec<String>, minimum_response_count: u32) -> Result<Self, String> {
        let s = Self {
            rpc_list,
            minimum_response_count,
        };
        s.check_config_valid()?;

        Ok(s)
    }

    pub fn check_config_valid(&self) -> Result<(), String> {
        if self.minimum_response_count == 0 {
            return Err("minimum_response_count should be greater than 0".to_string());
        }
        if self.rpc_list.len() < self.minimum_response_count as usize {
            return Err(
                "rpc_list length should be greater than minimum_response_count".to_string(),
            );
        }
        Ok(())
    }

    pub fn valid_and_get_result(
        &self,
        response_list: &Vec<anyhow::Result<String>>,
    ) -> Result<Vec<Instruction>, String> {
        self.check_config_valid()?;
        let mut instructions_list = vec![];
        // let mut success_response_body_list = vec![];

        for response in response_list {
            // if response.is_err() {

            //     continue;
            // }
            log!(
                DEBUG,
                "[state::valid_and_get_result] input response: {:?}",
                response
            );
            match response {
                Ok(resp) => match serde_json::from_str::<JsonRpcResponse<TransactionDetail>>(&resp)
                {
                    Ok(t) => {
                        if let Some(e) = t.error {
                            return Err(format!("{}", e.message));
                        } else {
                            match t.result {
                                None => {
                                    return Err(format!(
                                        "{}",
                                        "[state::valid_and_get_result] tx result is None"
                                            .to_string()
                                    ))
                                }
                                Some(tx_detail) => {
                                    log!(
                                        DEBUG,
                                        "[state::valid_and_get_result] tx detail: {:?}",
                                        tx_detail
                                    );
                                    instructions_list
                                        .push(tx_detail.transaction.message.instructions);
                                    // success_response_body_list.push(t.result.to_owned())
                                }
                            }
                        }
                    }
                    Err(e) => {
                        log!(
                            ERROR,
                            "[state::valid_and_get_result] serde_json::from_str error: {:?}",
                            e.to_string()
                        );
                        continue;
                    }
                },
                Err(e) => {
                    log!(
                        ERROR,
                        "[state::valid_and_get_result] response error: {:?}",
                        e.to_string()
                    );
                    continue;
                }
            }
        }

        if instructions_list.len() < self.minimum_response_count as usize {
            return Err(format!(
                "Not enough valid response, expected: {}, actual: {}",
                self.minimum_response_count,
                instructions_list.len()
            ));
        }

        // The minimum_response_count should greater than 0
        let mut i = 1;
        while i < instructions_list.len() {
            if instructions_list[i - 1] != instructions_list[i] {
                return Err("Response mismatch".to_string());
            }
            i += 1;
        }

        Ok(instructions_list[0].to_owned())
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
    // Locks preventing concurrent execution timer tasks
    pub active_tasks: HashSet<TaskType>,
    pub admin: Principal,
    pub caller_perms: HashMap<String, Permission>,
    pub multi_rpc_config: MultiRpcConfig,
    pub forward: Option<String>,

    // stable storage
    #[serde(skip, default = "crate::memory::init_ticket_queue")]
    pub tickets_queue: StableBTreeMap<u64, Ticket, Memory>,
    #[serde(skip, default = "crate::memory::init_failed_tickets")]
    pub tickets_failed_to_hub: StableBTreeMap<String, Ticket, Memory>,
    #[serde(skip, default = "crate::memory::init_counterparties")]
    pub counterparties: StableBTreeMap<ChainId, Chain, Memory>,
    #[serde(skip, default = "crate::memory::init_tokens")]
    pub tokens: StableBTreeMap<TokenId, Token, Memory>,
    #[serde(skip, default = "crate::memory::init_update_tokens")]
    pub update_token_queue: StableBTreeMap<TokenId, UpdateToken, Memory>,
    #[serde(skip, default = "crate::memory::init_token_mint_accounts")]
    pub token_mint_accounts: StableBTreeMap<TokenId, AccountInfo, Memory>,
    #[serde(skip, default = "crate::memory::init_associated_accounts")]
    pub associated_accounts: StableBTreeMap<AtaKey, AccountInfo, Memory>,
    #[serde(skip, default = "crate::memory::init_mint_token_requests")]
    pub mint_token_requests: StableBTreeMap<TicketId, MintTokenRequest, Memory>,
    #[serde(skip, default = "crate::memory::init_gen_ticket_reqs")]
    pub gen_ticket_reqs: StableBTreeMap<TicketId, GenerateTicketReq, Memory>,
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
            multi_rpc_config: MultiRpcConfig::default(),
            forward: None,

            // init stable storage
            tickets_queue: StableBTreeMap::init(crate::memory::get_ticket_queue_memory()),
            tickets_failed_to_hub: StableBTreeMap::init(crate::memory::get_failed_tickets_memory()),
            counterparties: StableBTreeMap::init(crate::memory::get_counterparties_memory()),
            tokens: StableBTreeMap::init(crate::memory::get_tokens_memory()),
            update_token_queue: StableBTreeMap::init(crate::memory::get_update_tokens_memory()),
            token_mint_accounts: StableBTreeMap::init(
                crate::memory::get_token_mint_accounts_memory(),
            ),
            associated_accounts: StableBTreeMap::init(
                crate::memory::get_associated_accounts_memory(),
            ),
            mint_token_requests: StableBTreeMap::init(
                crate::memory::get_mint_token_requests_memory(),
            ),
            gen_ticket_reqs: StableBTreeMap::init(crate::memory::get_gen_ticket_req_memory()),
        }
    }
}

impl SolanaRouteState {
    pub fn validate_config(&self) {}

    pub fn add_chain(&mut self, chain: Chain) {
        self.counterparties
            .insert(chain.chain_id.clone(), chain.clone());
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
