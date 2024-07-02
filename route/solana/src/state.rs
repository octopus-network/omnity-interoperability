use crate::{auth::Permission, lifecycle::InitArgs};
use candid::{CandidType, Principal};

use crate::event::{record_event, Event};
use omnity_types::{
    Chain, ChainId, ChainState, Factor, Ticket, TicketId, ToggleState, Token, TokenId,
};
use serde::{Deserialize, Serialize};
use std::{
    cell::RefCell,
    collections::{BTreeMap, HashMap},
};

thread_local! {
    static STATE: RefCell<Option<SolanaRouteState>> = RefCell::default();
}

#[derive(CandidType, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MintTokenStatus {
    Finalized { block_index: u64 },
    Unknown,
}

#[derive(CandidType, Clone, Debug, Deserialize, Serialize)]
pub struct SolanaRouteState {
    pub chain_id: String,

    pub hub_principal: Principal,

    // Next index of query tickets from hub
    pub next_ticket_seq: u64,

    // Next index of query directives from hub
    pub next_directive_seq: u64,

    pub counterparties: BTreeMap<ChainId, Chain>,

    pub tokens: BTreeMap<TokenId, Token>,

    pub token_ledgers: BTreeMap<TokenId, Principal>,

    pub finalized_mint_token_requests: BTreeMap<TicketId, u64>,

    pub fee_token_factor: Option<u128>,

    pub target_chain_factor: BTreeMap<ChainId, u128>,

    pub chain_state: ChainState,

    pub failed_tickets: Vec<Ticket>,

    #[serde(skip)]
    pub is_timer_running: bool,
    pub admin: Principal,
    pub caller_perms: HashMap<String, Permission>,
}

impl SolanaRouteState {
    pub fn validate_config(&self) {}

    pub fn add_chain(&mut self, chain: Chain) {
        record_event(&Event::AddedChain(chain.clone()));
        self.counterparties.insert(chain.chain_id.clone(), chain);
    }

    pub fn add_token(&mut self, token: Token) {
        self.tokens.insert(token.token_id.clone(), token);
    }

    pub fn toggle_chain_state(&mut self, toggle: ToggleState) {
        if toggle.chain_id == self.chain_id {
            self.chain_state = toggle.action.into();
        } else if let Some(chain) = self.counterparties.get_mut(&toggle.chain_id) {
            record_event(&Event::ToggleChainState(toggle.clone()));
            chain.chain_state = toggle.action.into();
        }
    }

    pub fn finalize_mint_token_req(&mut self, ticket_id: String, finalized_block_index: u64) {
        record_event(&Event::FinalizedMintToken {
            ticket_id: ticket_id.clone(),
            block_index: finalized_block_index,
        });
        self.finalized_mint_token_requests
            .insert(ticket_id, finalized_block_index);
    }

    pub fn update_fee(&mut self, fee: Factor) {
        record_event(&Event::UpdatedFee { fee: fee.clone() });
        match fee {
            Factor::UpdateTargetChainFactor(factor) => {
                self.target_chain_factor
                    .insert(factor.target_chain_id.clone(), factor.target_chain_factor);
            }

            Factor::UpdateFeeTokenFactor(token_factor) => {
                if token_factor.fee_token == "LICP" {
                    self.fee_token_factor = Some(token_factor.fee_token_factor);
                }
            }
        }
    }
}

// pub fn finalize_gen_ticket(ticket_id: String, request: GenerateTicketReq) {
//     record_event(&Event::FinalizedGenTicket { ticket_id, request })
// }

impl From<InitArgs> for SolanaRouteState {
    fn from(args: InitArgs) -> Self {
        Self {
            chain_id: args.chain_id,
            hub_principal: args.hub_principal,
            token_ledgers: Default::default(),
            next_ticket_seq: 0,
            next_directive_seq: 0,
            counterparties: Default::default(),
            tokens: Default::default(),
            finalized_mint_token_requests: Default::default(),
            fee_token_factor: None,
            target_chain_factor: Default::default(),
            chain_state: args.chain_state,
            failed_tickets: Default::default(),
            is_timer_running: false,
            admin: args.admin,
            caller_perms: HashMap::from([(args.admin.to_string(), Permission::Update)]),
        }
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
