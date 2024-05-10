use crate::contracts::OmnityPortContract;
use crate::evm_address::EvmAddress;
use crate::stable_memory::Memory;
use crate::types::{Chain, ChainState, Token, TokenId};
use crate::types::{
    ChainId, Directive, EcdsaKeyIds, PendingDirectiveStatus, PendingTicketStatus, Seq, Ticket,
    TicketId,
};
use anyhow::anyhow;
use candid::{CandidType, Principal};
use cketh_common::eth_rpc_client::providers::RpcApi;
use ic_cdk::api::management_canister::ecdsa::{EcdsaCurve, EcdsaKeyId};
use ic_stable_structures::StableBTreeMap;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::ptr::read;

thread_local! {
    static STATE: RefCell<Option<CdkRouteState>> = RefCell::new(None);
}

#[derive(CandidType, Deserialize)]
pub struct InitArgs {
    pub chain_id: String,
    pub hub_principal: Principal,
    pub evm_chain_id: u64,
    pub evm_rpc_canister_addr: Principal,
    pub omnity_port_contract: Vec<u8>,
    pub scan_start_height: u64,
    pub key_derivation_path: String,
    pub key_id_str: String,
}

#[derive(Deserialize, Serialize)]
pub struct CdkRouteState {
    pub hub_principal: Principal,
    pub omnity_chain_id: String,
    pub evm_chain_id: u64,
    pub tokens: BTreeMap<TokenId, Token>,
    pub counterparties: BTreeMap<ChainId, Chain>,
    pub finalized_mint_token_requests: BTreeMap<TicketId, u64>,
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
    pub handled_cdk_event: BTreeSet<String>,
    #[serde(skip, default = "crate::stable_memory::init_to_cdk_tickets_queue")]
    pub tickets_queue: StableBTreeMap<u64, Ticket, Memory>,
    #[serde(skip, default = "crate::stable_memory::init_to_cdk_directives_queue")]
    pub directives_queue: StableBTreeMap<u64, Directive, Memory>,
    #[serde(skip, default = "crate::stable_memory::init_pending_ticket_map")]
    pub pending_tickets_map: StableBTreeMap<TicketId, PendingTicketStatus, Memory>,
    #[serde(skip, default = "crate::stable_memory::init_pending_directive_map")]
    pub pending_directive_map: StableBTreeMap<Seq, PendingDirectiveStatus, Memory>,

    pub scan_start_height: u64,
    #[serde(skip)]
    pub is_timer_running: bool,
}

impl CdkRouteState {
    pub fn init(args: InitArgs) -> anyhow::Result<Self> {
        /*
        match self {
                Self::TestKeyLocalDevelopment => "dfx_test_key",
                Self::TestKey1 => "test_key_1",
                Self::ProductionKey1 => "key_1",*/
        if args.key_id_str != "dfx_test_key"
            || args.key_id_str != "test_key_1"
            || args.key_id_str != "key_1"
        {
            return Err(anyhow!("unspport key id "));
        }

        let key_id = EcdsaKeyId {
            curve: EcdsaCurve::Secp256k1,
            name: args.key_id_str.clone(),
        };
        let ret = CdkRouteState {
            hub_principal: args.hub_principal.clone(),
            omnity_chain_id: args.chain_id,
            evm_chain_id: args.evm_chain_id,
            tokens: Default::default(),
            counterparties: Default::default(),
            finalized_mint_token_requests: Default::default(),
            chain_state: ChainState::Active,
            evm_rpc_addr: args.evm_rpc_canister_addr,
            key_id,
            key_derivation_path: vec![],
            nonce: 0,
            pubkey: vec![],
            rpc_privders: vec![],
            omnity_port_contract: EvmAddress::try_from(args.omnity_port_contract)
                .expect("omnity port contract address error"),
            next_ticket_seq: 0,
            next_directive_seq: 0,
            next_consume_ticket_seq: 0,
            next_consume_directive_seq: 0,
            handled_cdk_event: Default::default(),
            tickets_queue: StableBTreeMap::init(crate::stable_memory::get_to_cdk_tickets_memory()),
            directives_queue: StableBTreeMap::init(
                crate::stable_memory::get_to_cdk_directives_memory(),
            ),
            pending_tickets_map: StableBTreeMap::init(
                crate::stable_memory::get_pending_ticket_map_memory(),
            ),
            pending_directive_map: StableBTreeMap::init(
                crate::stable_memory::get_pending_directive_map_memory(),
            ),
            scan_start_height: args.scan_start_height,
            is_timer_running: false,
        };
        Ok(ret)
    }
}

pub fn is_active() -> bool {
    read_state(|s| s.chain_state == ChainState::Active)
}

pub fn hub_addr() -> Principal {
    read_state(|s| s.hub_principal.clone())
}

pub fn rpc_addr() -> Principal {
    read_state(|s| s.evm_rpc_addr.clone())
}

pub fn rpc_providers() -> Vec<RpcApi> {
    read_state(|s| s.rpc_privders.clone())
}

pub fn target_chain_id() -> u64 {
    read_state(|s| s.evm_chain_id)
}

pub fn try_public_key() -> crate::Result<Vec<u8>> {
    Ok(read_state(|s| s.pubkey.clone()))
}

pub fn key_id() -> EcdsaKeyId {
    read_state(|s| s.key_id.clone())
}

pub fn key_derivation_path() -> Vec<Vec<u8>> {
    read_state(|s| s.key_derivation_path.clone())
}

pub fn mutate_state<F, R>(f: F) -> R
where
    F: FnOnce(&mut CdkRouteState) -> R,
{
    STATE.with(|s| f(s.borrow_mut().as_mut().expect("State not initialized!")))
}

pub fn read_state<F, R>(f: F) -> R
where
    F: FnOnce(&CdkRouteState) -> R,
{
    STATE.with(|s| f(s.borrow().as_ref().expect("State not initialized!")))
}

/// Replaces the current state.
pub fn replace_state(state: CdkRouteState) {
    STATE.with(|s| {
        *s.borrow_mut() = Some(state);
    });
}
