use crate::memory::Memory;
use crate::{
    auth::Permission,
    constants::{FEE_TOKEN, SCHNORR_KEY_NAME},
    guard::TaskType,
    lifecycle::InitArgs,
};
use candid::{CandidType, Principal};

use ic_stable_structures::StableBTreeMap;

use crate::types::{
    Chain, ChainId, ChainState, Factor, Ticket, TicketId, ToggleState, Token, TokenId,
};
use serde::{Deserialize, Serialize};
use std::{
    cell::RefCell,
    collections::{BTreeMap, HashMap, HashSet},
};
pub type CanisterId = Principal;
pub type Owner = String;
pub type TokenMint = String;
pub type AssociatedTokenAccount = String;

thread_local! {
    static STATE: RefCell<Option<SolanaRouteState>> = RefCell::default();
}

#[derive(CandidType, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MintTokenStatus {
    Finalized { signature: String },
    Unknown,
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

#[derive(Deserialize, Serialize)]
pub struct SolanaRouteState {
    pub chain_id: String,

    pub hub_principal: Principal,

    // Next index of query tickets from hub
    pub next_ticket_seq: u64,
    pub next_consume_ticket_seq: u64,

    // Next index of query directives from hub
    pub next_directive_seq: u64,

    pub counterparties: BTreeMap<ChainId, Chain>,

    pub tokens: BTreeMap<TokenId, Token>,

    pub token_mint_map: BTreeMap<TokenId, TokenMint>,

    pub associated_account: BTreeMap<(Owner, TokenMint), AssociatedTokenAccount>,

    pub finalized_mint_token_requests: BTreeMap<TicketId, String>,

    pub fee_token_factor: Option<u128>,

    pub target_chain_factor: BTreeMap<ChainId, u128>,

    pub chain_state: ChainState,

    pub failed_tickets: Vec<Ticket>,

    pub schnorr_canister: Principal,
    pub schnorr_key_name: String,

    pub sol_canister: Principal,

    // Locks preventing concurrent execution timer tasks
    pub active_tasks: HashSet<TaskType>,
    pub admin: Principal,
    pub caller_perms: HashMap<String, Permission>,

    #[serde(skip, default = "crate::memory::init_ticket_queue")]
    pub tickets_queue: StableBTreeMap<u64, Ticket, Memory>,
}

impl From<InitArgs> for SolanaRouteState {
    fn from(args: InitArgs) -> Self {
        Self {
            chain_id: args.chain_id,
            hub_principal: args.hub_principal,
            token_mint_map: Default::default(),

            next_ticket_seq: 0,
            next_consume_ticket_seq: 0,
            next_directive_seq: 0,
            counterparties: Default::default(),
            tokens: Default::default(),
            finalized_mint_token_requests: Default::default(),
            fee_token_factor: None,
            target_chain_factor: Default::default(),
            chain_state: args.chain_state,
            failed_tickets: Default::default(),
            schnorr_canister: args.schnorr_canister,
            schnorr_key_name: args
                .schnorr_key_name
                .unwrap_or(SCHNORR_KEY_NAME.to_string()),
            sol_canister: args.sol_canister,
            active_tasks: Default::default(),
            admin: args.admin,
            caller_perms: HashMap::from([(args.admin.to_string(), Permission::Update)]),
            tickets_queue: StableBTreeMap::init(crate::memory::get_ticket_queue_memory()),
            associated_account: Default::default(),
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
        } else if let Some(chain) = self.counterparties.get_mut(&toggle.chain_id) {
            chain.chain_state = toggle.action.into();
        }
    }

    pub fn sol_token_address(&self, ticket_id: &String) -> Option<String> {
        self.token_mint_map.get(ticket_id).cloned()
    }

    pub fn finalize_mint_token_req(&mut self, ticket_id: String, signature: String) {
        self.finalized_mint_token_requests
            .insert(ticket_id, signature);
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

// just for test or dev, replace it for production with Principal::management_canister()
pub fn management_canister() -> CanisterId {
    read_state(|s| s.schnorr_canister)
}

// pub fn finalize_gen_ticket(ticket_id: String, request: GenerateTicketReq) {
//     record_event(&Event::FinalizedGenTicket { ticket_id, request })
// }

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