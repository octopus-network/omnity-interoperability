use std::collections::BTreeMap;

use candid::{CandidType, Principal};
use omnity_types::{Chain, ChainId, ChainState, TicketId, Token, TokenId};
use serde::{Deserialize, Serialize};

use crate::state::RouteState;

#[derive(CandidType, Clone, Debug, Deserialize, Serialize)]
pub struct OldRouteState {
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

    #[serde(skip)]
    pub is_timer_running: bool,
}

impl Into<RouteState> for OldRouteState {
    fn into(self) -> RouteState {
        RouteState {
            chain_id: self.chain_id,
            hub_principal: self.hub_principal,
            next_ticket_seq: self.next_ticket_seq,
            next_directive_seq: self.next_directive_seq,
            counterparties: self.counterparties,
            tokens: self.tokens,
            token_ledgers: self.token_ledgers,
            finalized_mint_token_requests: self.finalized_mint_token_requests,
            fee_token_factor: self.fee_token_factor,
            target_chain_factor: self.target_chain_factor,
            chain_state: self.chain_state,
            failed_tickets: Vec::new(),
            is_timer_running: self.is_timer_running,
        }
    }
}
