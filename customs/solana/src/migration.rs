use crate::{
    guard::TaskType,
    memory::{init_finalized_gen_tickets, init_finalized_requests},
    state::{CustomsState, ReleaseTokenReq, RpcProvider},
    types::omnity_types::{Chain, ChainId, ChainState, TicketId, Token, TokenId},
};
use candid::Principal;
use ic_solana::types::Pubkey;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashSet},
    str::FromStr,
};

#[derive(Deserialize, Serialize)]
pub struct OldState {
    pub chain_id: String,
    pub hub_principal: Principal,
    pub schnorr_key_name: String,
    pub sol_canister: Principal,
    pub forward: Option<String>,
    pub port_program_id: Pubkey,
    pub chain_state: ChainState,
    pub counterparties: BTreeMap<ChainId, Chain>,
    pub tokens: BTreeMap<TokenId, Token>,
    pub release_token_requests: BTreeMap<TicketId, ReleaseTokenReq>,
    pub rpc_list: Vec<String>,
    pub min_response_count: u32,
    pub enable_debug: bool,

    // Next index of query tickets from hub
    pub next_ticket_seq: u64,

    // Next index of query directives from hub
    pub next_directive_seq: u64,

    pub active_tasks: HashSet<TaskType>,
}

impl From<OldState> for CustomsState {
    fn from(args: OldState) -> Self {
        Self {
            chain_id: args.chain_id,
            hub_principal: args.hub_principal,
            schnorr_key_name: args.schnorr_key_name,
            sol_canister: Principal::from_str("by6od-j4aaa-aaaaa-qaadq-cai").unwrap(),
            port_program_id: args.port_program_id,
            chain_state: args.chain_state,
            counterparties: args.counterparties,
            tokens: args.tokens,
            release_token_requests: args.release_token_requests,
            providers: vec![RpcProvider {
                host: "api.devnet.solana.com".into(),
                api_key_param: None,
            }],
            proxy_rpc: "https://solana-idempotent-proxy-219952077564.us-central1.run.app/api"
                .into(),
            min_response_count: args.min_response_count,
            enable_debug: false,
            next_ticket_seq: args.next_ticket_seq,
            next_directive_seq: args.next_directive_seq,
            active_tasks: Default::default(),
            finalized_requests: init_finalized_requests(),
            finalized_gen_tickets: init_finalized_gen_tickets(),
        }
    }
}
