use crate::eth_common::EvmAddress;
use crate::stable_memory;
use crate::stable_memory::Memory;
use crate::types::{Chain, ChainState, Network, Token, TokenId};
use crate::types::{
    ChainId, Directive, PendingDirectiveStatus, PendingTicketStatus, Seq, Ticket, TicketId,
};
use candid::{CandidType, Principal};
use cketh_common::eth_rpc_client::providers::RpcApi;
use ethers_core::abi::ethereum_types;
use ethers_core::utils::keccak256;
use ic_cdk::api::management_canister::ecdsa::EcdsaKeyId;
use ic_stable_structures::writer::Writer;
use ic_stable_structures::StableBTreeMap;
use itertools::Itertools;
use k256::PublicKey;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::str::FromStr;

thread_local! {
    static STATE: RefCell<Option<CdkRouteState>> = RefCell::new(None);
}

#[derive(CandidType, Deserialize)]
pub struct InitArgs {
    pub evm_chain_id: u64,
    pub admin: Principal,
    pub omnity_port_contract: String,
    pub hub_principal: Principal,
    pub network: Network,
    pub evm_rpc_canister_addr: Principal,
    pub scan_start_height: u64,
    pub chain_id: String,
    pub rpc_url: String,
}

impl CdkRouteState {
    pub fn init(args: InitArgs) -> anyhow::Result<Self> {
        let ret = CdkRouteState {
            admin: args.admin,
            hub_principal: args.hub_principal,
            omnity_chain_id: args.chain_id,
            evm_chain_id: args.evm_chain_id,
            tokens: Default::default(),
            counterparties: Default::default(),
            finalized_mint_token_requests: Default::default(),
            chain_state: ChainState::Active,
            evm_rpc_addr: args.evm_rpc_canister_addr,
            key_id: args.network.key_id(),
            key_derivation_path: vec![b"m/44'/223'/0'/0/0".to_vec()], //TODO
            nonce: 0,
            pubkey: vec![],
            rpc_privders: vec![RpcApi {
                url: args.rpc_url.clone(),
                headers: None,
            }],
            omnity_port_contract: EvmAddress::from_str(args.omnity_port_contract.as_str())
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

    pub fn pre_upgrade(&self) {
        let mut state_bytes = vec![];
        let _ = ciborium::ser::into_writer(self, &mut state_bytes);
        let len = state_bytes.len() as u32;
        let mut memory = crate::stable_memory::get_upgrade_stash_memory();
        let mut writer = Writer::new(&mut memory, 0);
        writer
            .write(&len.to_le_bytes())
            .expect("failed to save hub state len");
        writer
            .write(&state_bytes)
            .expect("failed to save hub state");
    }

    pub fn post_upgrade() {
        use ic_stable_structures::Memory;
        let memory = stable_memory::get_upgrade_stash_memory();
        // Read the length of the state bytes.
        let mut state_len_bytes = [0; 4];
        memory.read(0, &mut state_len_bytes);
        let state_len = u32::from_le_bytes(state_len_bytes) as usize;

        // Read the bytes
        let mut state_bytes = vec![0; state_len];
        memory.read(4, &mut state_bytes);

        // Deserialize and set the state.
        let state: CdkRouteState =
            ciborium::de::from_reader(&*state_bytes).expect("failed to decode state");

        STATE.with(|s| *s.borrow_mut() = Some(state));
    }
}

#[derive(Deserialize, Serialize)]
pub struct CdkRouteState {
    pub admin: Principal,
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

impl From<&CdkRouteState> for StateProfile {
    fn from(v: &CdkRouteState) -> Self {
        let t = v.tickets_queue.iter().map(|v| v.clone()).collect_vec();
        StateProfile {
            admin: v.admin,
            hub_principal: v.hub_principal,
            omnity_chain_id: v.omnity_chain_id.clone(),
            evm_chain_id: v.evm_chain_id,
            tokens: v.tokens.clone(),
            counterparties: v.counterparties.clone(),
            finalized_mint_token_requests: v.finalized_mint_token_requests.clone(),
            chain_state: v.chain_state.clone(),
            evm_rpc_addr: v.evm_rpc_addr.clone(),
            key_id: v.key_id.clone(),
            key_derivation_path: v.key_derivation_path.clone(),
            nonce: v.nonce,
            pubkey: v.pubkey.clone(),
            omnity_port_contract: v.omnity_port_contract.clone(),
            next_ticket_seq: v.next_ticket_seq,
            next_directive_seq: v.next_directive_seq,
            next_consume_ticket_seq: v.next_consume_ticket_seq,
            next_consume_directive_seq: v.next_consume_directive_seq,
            rpc_providers: v.rpc_privders.clone(),
            tickets: t,
        }
    }
}

#[derive(Deserialize, Serialize, CandidType)]
pub struct StateProfile {
    pub admin: Principal,
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
    pub omnity_port_contract: EvmAddress,
    pub next_ticket_seq: u64,
    pub next_directive_seq: u64,
    pub next_consume_ticket_seq: u64,
    pub next_consume_directive_seq: u64,
    pub tickets: Vec<(u64, Ticket)>,
    pub rpc_providers: Vec<RpcApi>,
}

pub fn is_active() -> bool {
    read_state(|s| s.chain_state == ChainState::Active)
}

pub fn hub_addr() -> Principal {
    read_state(|s| s.hub_principal)
}

pub fn rpc_addr() -> Principal {
    read_state(|s| s.evm_rpc_addr)
}

pub fn minter_addr() -> String {
    let key = public_key();
    use k256::elliptic_curve::sec1::ToEncodedPoint;
    let key =
        PublicKey::from_sec1_bytes(key.as_slice()).expect("failed to parse the public key as SEC1");
    let point = key.to_encoded_point(false);
    // we re-encode the key to the decompressed representation.
    let point_bytes = point.as_bytes();
    assert_eq!(point_bytes[0], 0x04);
    let hash = keccak256(&point_bytes[1..]);
    ethers_core::utils::to_checksum(&ethereum_types::Address::from_slice(&hash[12..32]), None)
}
pub fn rpc_providers() -> Vec<RpcApi> {
    read_state(|s| s.rpc_privders.clone())
}

pub fn evm_chain_id() -> u64 {
    read_state(|s| s.evm_chain_id)
}

pub fn public_key() -> Vec<u8> {
    read_state(|s| s.pubkey.clone())
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

pub fn take_state<F, R>(f: F) -> R
where
    F: FnOnce(CdkRouteState) -> R,
{
    STATE.with(|s| f(s.take().expect("State not initialized!")))
}