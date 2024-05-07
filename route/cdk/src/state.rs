use std::cell::RefCell;
use std::collections::BTreeMap;
use crate::types::{ChainId, Directive, PendingTicketStatus, Ticket, TicketId};
use candid::{CandidType, Principal};
use cketh_common::eth_rpc_client::providers::RpcApi;
use ic_cdk::api::management_canister::ecdsa::{EcdsaCurve, EcdsaKeyId};
use ic_stable_structures::StableBTreeMap;
use serde::{Deserialize, Serialize};
use crate::contracts::OmnityPortContract;
use crate::evm_address::EvmAddress;
use crate::stable_memory::Memory;
use crate::types::{Chain, ChainState, Token, TokenId};

thread_local! {
    static STATE: RefCell<CdkRouteState> = RefCell::new(CdkRouteState::new());
}

#[derive( Deserialize, Serialize)]
pub struct CdkRouteState {
    pub hub_principal: Principal,
    pub omnity_chain_id: String,
    pub evm_chain_id: u64,
    pub tokens: BTreeMap<TokenId, Token>,
    pub counterparties: BTreeMap<ChainId, Chain>,
    pub fee_token_factor: Option<u128>,
    pub finalized_mint_token_requests: BTreeMap<TicketId, u64>,
    pub target_chain_factor: BTreeMap<ChainId, u128>,
    pub chain_state: ChainState,
    pub evm_rpc_addr: Principal,
    pub key_id: EcdsaKeyId,
    pub key_derivation_path: Vec<Vec<u8>>,
    pub nonce: u64,
    pub pubkey: Vec<u8>,
    pub rpc_privders: Vec<RpcApi>,
    pub omnity_port_contract: EvmAddress,
    pub next_ticket_seq: u64,
    pub next_directive_seq: u64,
    pub next_consume_ticket_seq: u64,
    pub next_consume_directive_seq: u64,
    #[serde(skip, default = "crate::stable_memory::init_to_cdk_tickets_queue")]
    pub tickets_queue: StableBTreeMap<u64, Ticket, Memory>,
    #[serde(skip, default = "crate::stable_memory::init_to_cdk_directives_queue")]
    pub directives_queue: StableBTreeMap<u64, Directive, Memory>,

    #[serde(skip, default = "crate::stable_memory::init_pending_ticket_map")]
    pub pending_tickets_map: StableBTreeMap<TicketId, PendingTicketStatus, Memory>,

    #[serde(skip)]
    pub is_timer_running: bool,
}


impl CdkRouteState {

    pub(crate) fn new() -> Self {
        CdkRouteState {
            hub_principal: Principal::anonymous(),
            omnity_chain_id: "".to_string(),
            next_ticket_seq: 0,
            next_directive_seq: 0,
            next_consume_ticket_seq: 0,
            next_consume_directive_seq: 0,
            tokens: Default::default(),
            counterparties: Default::default(),
            fee_token_factor: None,
            finalized_mint_token_requests: Default::default(),
            target_chain_factor: Default::default(),
            chain_state: Default::default(),
            evm_rpc_addr: Principal::anonymous(),
            key_id: EcdsaKeyId {
                curve: EcdsaCurve::Secp256k1,
                name: "".to_string(),
            },
            key_derivation_path: vec![],
            nonce: 0,
            pubkey: vec![],
            rpc_privders: vec![],
            tickets_queue: StableBTreeMap::init(crate::stable_memory::get_to_cdk_tickets_memory()),
            directives_queue: StableBTreeMap::init(crate::stable_memory::get_to_cdk_directives_memory()),
            pending_tickets_map: StableBTreeMap::init(crate::stable_memory::get_pending_ticket_map_memory()),
            is_timer_running: false,
            evm_chain_id: 0,

            omnity_port_contract: EvmAddress::default(),
        }
    }
}

pub fn is_active() -> bool {
    STATE.with(|s|s.borrow().chain_state == ChainState::Active)
}

pub fn hub_addr() -> Principal {
    STATE.with(|s| s.borrow().hub_principal.clone())
}

pub fn rpc_addr() -> Principal {
    STATE.with(|s| s.borrow().evm_rpc_addr.clone())
}

pub fn rpc_providers() -> Vec<RpcApi> {
    STATE.with_borrow(|s|s.rpc_privders.clone())
}

pub fn target_chain_id() -> u64 {
    STATE.with_borrow(|s|s.evm_chain_id)
}

pub fn try_public_key() -> crate::Result<Vec<u8>> {
    Ok(
        STATE.with(|s| s.borrow().pubkey.clone())
    )
}

pub fn key_id() -> EcdsaKeyId {
    STATE.with(|s|
        s.borrow().key_id.clone()
    )
}

pub fn key_derivation_path() -> Vec<Vec<u8>> {
    read_state(|s|s.key_derivation_path.clone())
}

pub fn fetch_and_incr_nonce() -> u64 {
    STATE.with(|s| {
        let nonce = s.borrow().nonce+1;
        s.borrow_mut().nonce = nonce;
        nonce
    })
}

pub fn read_state<F, R>(f: F) -> R
    where
        F: FnOnce(&CdkRouteState) -> R,
{
    STATE.with(|s| f(&*s.borrow()))
}

pub fn mutate_state<F, R>(f: F) -> R
    where
        F: FnOnce(&mut CdkRouteState) -> R,
{
    STATE.with(|s| f(&mut *s.borrow_mut()))
}