use candid::{Deserialize, Principal};
use ic_canister_log::log;
use ic_solana::ic_log::DEBUG;

use crate::auth::Permission;
use crate::guard::TaskType;
use crate::handler::mint_token::MintTokenRequest;
use crate::lifecycle::InitArgs;
use crate::state::{
    AccountInfo, AtaKey, MintAccount, MultiRpcConfig, Owner, SolanaRouteState, UpdateToken,
};
use crate::types::{Chain, ChainId, ChainState, Ticket, TicketId, Token, TokenId};

use crate::memory::Memory;
use ic_stable_structures::StableBTreeMap;
use serde::Serialize;
use std::collections::HashMap;
use std::collections::{BTreeMap, HashSet};

#[derive(Serialize, Deserialize)]
pub struct PreState {
    pub chain_id: String,
    pub hub_principal: Principal,
    // Next index of query tickets from hub
    pub next_ticket_seq: u64,
    pub next_consume_ticket_seq: u64,
    // Next index of query directives from hub
    pub next_directive_seq: u64,
    pub counterparties: BTreeMap<ChainId, Chain>,
    pub tokens: BTreeMap<TokenId, Token>,
    pub update_token_queue: BTreeMap<TokenId, (Token, u64)>,
    pub token_mint_accounts: BTreeMap<TokenId, AccountInfo>,
    pub associated_accounts: BTreeMap<(Owner, MintAccount), AccountInfo>,
    pub mint_token_requests: BTreeMap<TicketId, MintTokenRequest>,
    pub fee_token_factor: Option<u128>,
    pub target_chain_factor: BTreeMap<ChainId, u128>,
    pub chain_state: ChainState,
    pub tickets_failed_to_hub: Vec<Ticket>,
    pub schnorr_canister: Principal,
    pub schnorr_key_name: String,
    pub sol_canister: Principal,
    pub fee_account: String,

    // Locks preventing concurrent execution timer tasks
    pub active_tasks: HashSet<TaskType>,
    pub admin: Principal,
    pub caller_perms: HashMap<String, Permission>,
    #[serde(skip, default = "crate::memory::init_ticket_queue")]
    pub tickets_queue: StableBTreeMap<u64, Ticket, Memory>,
}

pub fn migrate(pre_state: PreState) -> SolanaRouteState {
    let init_args = InitArgs {
        admin: pre_state.admin,
        chain_id: pre_state.chain_id,
        hub_principal: pre_state.hub_principal,
        chain_state: pre_state.chain_state,
        schnorr_key_name: Some(pre_state.schnorr_key_name),
        sol_canister: pre_state.sol_canister,
        // caller_perms: pre_state.caller_perms,
        fee_account: Some(pre_state.fee_account),
        multi_rpc_config: MultiRpcConfig::default(),
        forward: None,
    };
    let mut new_state = SolanaRouteState::from(init_args);
    // let mut new_state = SolanaRouteState {
    //     chain_id: pre_state.chain_id,
    //     hub_principal: pre_state.hub_principal,
    //     next_ticket_seq: pre_state.next_ticket_seq,
    //     next_consume_ticket_seq: pre_state.next_consume_ticket_seq,
    //     next_directive_seq: pre_state.next_directive_seq,
    //     fee_token_factor: pre_state.fee_token_factor,
    //     target_chain_factor: pre_state.target_chain_factor,
    //     chain_state: pre_state.chain_state,
    //     schnorr_key_name: pre_state.schnorr_key_name,
    //     sol_canister: pre_state.sol_canister,
    //     fee_account: pre_state.fee_account,
    //     active_tasks: pre_state.active_tasks,
    //     admin: pre_state.admin,
    //     caller_perms: pre_state.caller_perms,
    //     multi_rpc_config: MultiRpcConfig::default(),
    //     forward: None,

    //     // stable storage
    //     tickets_queue: pre_state.tickets_queue,
    //     tickets_failed_to_hub: StableBTreeMap::init(crate::memory::get_failed_tickets_memory()),
    //     counterparties: StableBTreeMap::init(crate::memory::get_counterparties_memory()),
    //     tokens: StableBTreeMap::init(crate::memory::get_tokens_memory()),
    //     update_token_queue: StableBTreeMap::init(crate::memory::get_update_tokens_memory()),
    //     token_mint_accounts: StableBTreeMap::init(crate::memory::get_token_mint_accounts_memory()),
    //     associated_accounts: StableBTreeMap::init(crate::memory::get_associated_accounts_memory()),
    //     mint_token_requests: StableBTreeMap::init(crate::memory::get_mint_token_requests_memory()),
    // };

    // new_state.chain_id = pre_state.chain_id;
    new_state.hub_principal = pre_state.hub_principal;
    new_state.next_ticket_seq = pre_state.next_ticket_seq;
    new_state.next_consume_ticket_seq = pre_state.next_consume_ticket_seq;
    new_state.next_directive_seq = pre_state.next_directive_seq;
    new_state.fee_token_factor = pre_state.fee_token_factor;
    new_state.target_chain_factor = pre_state.target_chain_factor;
    // new_state.chain_state = pre_state.chain_state;
    // new_state.schnorr_key_name = pre_state.schnorr_key_name;
    new_state.sol_canister = pre_state.sol_canister;
    // new_state.fee_account = pre_state.fee_account;
    new_state.active_tasks = pre_state.active_tasks;
    new_state.admin = pre_state.admin;
    new_state.caller_perms = pre_state.caller_perms;

    log!(DEBUG, "migrate tickets_queue ...");
    new_state.tickets_queue = pre_state.tickets_queue;

    log!(DEBUG, "migrate failed_ticket ...");
    for failed_ticket in pre_state.tickets_failed_to_hub {
        new_state
            .tickets_failed_to_hub
            .insert(failed_ticket.ticket_id.clone(), failed_ticket);
    }

    log!(DEBUG, "migrate counterparties ...");
    for (chain_id, chain) in pre_state.counterparties {
        new_state.counterparties.insert(chain_id, chain);
    }

    log!(DEBUG, "migrate tokens ...");
    for (token_id, token) in pre_state.tokens {
        new_state.tokens.insert(token_id, token);
    }

    log!(DEBUG, "migrate update_token_queue ...");
    for (token_id, (token, retry)) in pre_state.update_token_queue {
        new_state
            .update_token_queue
            .insert(token_id, UpdateToken::new(token, retry));
    }

    log!(DEBUG, "migrate token_mint_accounts ...");
    for (token_id, account_info) in pre_state.token_mint_accounts {
        new_state.token_mint_accounts.insert(token_id, account_info);
    }

    log!(DEBUG, "migrate associated_accounts ...");
    for ((owner, mint_account), account_info) in pre_state.associated_accounts {
        new_state.associated_accounts.insert(
            AtaKey {
                owner: owner,
                token_mint: mint_account,
            },
            account_info,
        );
    }

    log!(DEBUG, "migrate mint_token_requests ...");
    for (ticket_id, req) in pre_state.mint_token_requests {
        new_state.mint_token_requests.insert(ticket_id, req);
    }

    new_state
}
