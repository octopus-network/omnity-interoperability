use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::str::FromStr;

use candid::{CandidType, Principal};
use cketh_common::eth_rpc_client::providers::RpcApi;
use ethers_core::abi::ethereum_types;
use ethers_core::utils::keccak256;
use ic_cdk::api::management_canister::ecdsa::{
    ecdsa_public_key, EcdsaKeyId, EcdsaPublicKeyArgument,
};
use ic_stable_structures::StableBTreeMap;
use ic_stable_structures::writer::Writer;
use k256::PublicKey;
use serde::{Deserialize, Serialize};

use crate::{Error, stable_memory};
use crate::eth_common::{EvmAddress, EvmTxType};
use crate::service::InitArgs;
use crate::stable_memory::Memory;
use crate::types::{Chain, ChainState, Token, TokenId};
use crate::types::{
    ChainId, Directive, PendingDirectiveStatus, PendingTicketStatus, Seq, Ticket, TicketId,
};

thread_local! {
    static STATE: RefCell<Option<EvmRouteState >> = RefCell::new(None);
}

impl EvmRouteState {
    pub fn init(args: InitArgs) -> anyhow::Result<Self> {
        let omnity_port_contract = match args.port_addr {
            None => EvmAddress([0u8; 20]),
            Some(addr) => EvmAddress::from_str(addr.as_str()).expect("port address is invalid"),
        };
        let ret = EvmRouteState {
            admins: args.admins,
            hub_principal: args.hub_principal,
            omnity_chain_id: args.chain_id,
            evm_chain_id: args.evm_chain_id,
            fee_token_id: args.fee_token_id,
            tokens: Default::default(),
            token_contracts: Default::default(),
            counterparties: Default::default(),
            finalized_mint_token_requests: Default::default(),
            chain_state: ChainState::Active,
            evm_rpc_addr: args.evm_rpc_canister_addr,
            key_id: args.network.key_id(),
            key_derivation_path: vec![b"m/44'/223'/0'/0/0".to_vec()],
            pubkey: vec![],
            rpc_providers: args.rpcs,
            omnity_port_contract,
            fee_token_factor: None,
            target_chain_factor: Default::default(),
            next_ticket_seq: 0,
            next_directive_seq: 0,
            next_consume_ticket_seq: 0,
            next_consume_directive_seq: 0,
            finality_blocks: None,
            handled_evm_event: Default::default(),
            tickets_queue: StableBTreeMap::init(crate::stable_memory::get_to_evm_tickets_memory()),
            directives_queue: StableBTreeMap::init(
                crate::stable_memory::get_to_evm_directives_memory(),
            ),
            pending_tickets_map: StableBTreeMap::init(
                crate::stable_memory::get_pending_ticket_map_memory(),
            ),
            pending_directive_map: StableBTreeMap::init(
                crate::stable_memory::get_pending_directive_map_memory(),
            ),
            is_timer_running: Default::default(),
            evm_tx_type: args.evm_tx_type,
            block_interval_secs: args.block_interval_secs,
            pending_events_on_chain: Default::default(),
            evm_transfer_gas_percent: 110,
            total_required_count: 0,
            minimum_response_count: 0,
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
        let mut state_bytes = vec![0; state_len];
        memory.read(4, &mut state_bytes);
        let state: EvmRouteState =
            ciborium::de::from_reader(&*state_bytes).expect("failed to decode state");
        //  let state = EvmRouteState::from((state, gasfee_percent));
        replace_state(state);
    }

    pub fn pull_tickets(&self, from: usize, limit: usize) -> Vec<(Seq, Ticket)> {
        self.tickets_queue
            .iter()
            .skip(from)
            .take(limit)
            .map(|(seq, t)| (seq, t.clone()))
            .collect()
    }

    pub fn pull_directives(&self, from: usize, limit: usize) -> Vec<(Seq, Directive)> {
        self.directives_queue
            .iter()
            .skip(from)
            .take(limit)
            .map(|(seq, d)| (seq, d.clone()))
            .collect()
    }
}

pub async fn init_chain_pubkey() {
    let arg = EcdsaPublicKeyArgument {
        canister_id: None,
        derivation_path: key_derivation_path(),
        key_id: key_id(),
    };
    let res = ecdsa_public_key(arg)
        .await
        .map_err(|(_, e)| Error::ChainKeyError(e))
        .unwrap();
    mutate_state(|s| s.pubkey = res.0.public_key.clone());
}

#[derive(Deserialize, Serialize)]
pub struct EvmRouteState {
    pub admins: Vec<Principal>,
    pub hub_principal: Principal,
    pub omnity_chain_id: String,
    pub evm_chain_id: u64,
    pub fee_token_id: String,
    pub tokens: BTreeMap<TokenId, Token>,
    pub token_contracts: BTreeMap<TokenId, String>,
    pub counterparties: BTreeMap<ChainId, Chain>,
    pub finalized_mint_token_requests: BTreeMap<TicketId, String>,
    pub chain_state: ChainState,
    pub evm_rpc_addr: Principal,
    pub key_id: EcdsaKeyId,
    pub key_derivation_path: Vec<Vec<u8>>,
    pub pubkey: Vec<u8>,
    pub rpc_providers: Vec<RpcApi>,
    pub omnity_port_contract: EvmAddress,
    pub fee_token_factor: Option<u128>,
    pub target_chain_factor: BTreeMap<ChainId, u128>,
    pub next_ticket_seq: u64,
    pub next_directive_seq: u64,
    pub next_consume_ticket_seq: u64,
    pub next_consume_directive_seq: u64,
    #[serde(default)]
    pub finality_blocks: Option<u64>,
    pub handled_evm_event: BTreeSet<String>,
    #[serde(skip, default = "crate::stable_memory::init_to_evm_tickets_queue")]
    pub tickets_queue: StableBTreeMap<u64, Ticket, Memory>,
    #[serde(skip, default = "crate::stable_memory::init_to_evm_directives_queue")]
    pub directives_queue: StableBTreeMap<u64, Directive, Memory>,
    #[serde(skip, default = "crate::stable_memory::init_pending_ticket_map")]
    pub pending_tickets_map: StableBTreeMap<TicketId, PendingTicketStatus, Memory>,
    #[serde(skip, default = "crate::stable_memory::init_pending_directive_map")]
    pub pending_directive_map: StableBTreeMap<Seq, PendingDirectiveStatus, Memory>,
    #[serde(skip)]
    pub is_timer_running: BTreeMap<String, bool>,
    pub evm_tx_type: EvmTxType,
    pub block_interval_secs: u64,
    pub pending_events_on_chain: BTreeMap<String, u64>,
    pub evm_transfer_gas_percent: u64,
    #[serde(default = "default_rpcs_count")]
    pub total_required_count: usize,
    #[serde(default = "default_rpcs_count")]
    pub minimum_response_count: usize,
}

pub fn default_rpcs_count() -> usize { 1usize }

impl From<&EvmRouteState> for StateProfile {
    fn from(v: &EvmRouteState) -> Self {
        StateProfile {
            admins: v.admins.clone(),
            hub_principal: v.hub_principal,
            omnity_chain_id: v.omnity_chain_id.clone(),
            evm_chain_id: v.evm_chain_id,
            tokens: v.tokens.clone(),
            token_contracts: v.token_contracts.clone(),
            finality_blocks: v.finality_blocks.clone(),
            counterparties: v.counterparties.clone(),
            chain_state: v.chain_state.clone(),
            evm_rpc_addr: v.evm_rpc_addr,
            key_id: v.key_id.clone(),
            key_derivation_path: v.key_derivation_path.clone(),
            pubkey: v.pubkey.clone(),
            omnity_port_contract: v.omnity_port_contract.clone(),
            next_ticket_seq: v.next_ticket_seq,
            next_directive_seq: v.next_directive_seq,
            next_consume_ticket_seq: v.next_consume_ticket_seq,
            next_consume_directive_seq: v.next_consume_directive_seq,
            rpc_providers: v.rpc_providers.clone(),
            fee_token_factor: v.fee_token_factor,
            target_chain_factor: v.target_chain_factor.clone(),
            evm_tx_type: v.evm_tx_type,
            evm_gasfee_percent: v.evm_transfer_gas_percent,
            total_required_count: v.total_required_count,
            minimum_response_count: v.minimum_response_count,
        }
    }
}

#[derive(Deserialize, Serialize, CandidType)]
pub struct StateProfile {
    pub admins: Vec<Principal>,
    pub hub_principal: Principal,
    pub omnity_chain_id: String,
    pub evm_chain_id: u64,
    pub tokens: BTreeMap<TokenId, Token>,
    pub token_contracts: BTreeMap<TokenId, String>,
    pub counterparties: BTreeMap<ChainId, Chain>,
    pub chain_state: ChainState,
    pub evm_rpc_addr: Principal,
    pub key_id: EcdsaKeyId,
    pub key_derivation_path: Vec<Vec<u8>>,
    pub pubkey: Vec<u8>,
    pub omnity_port_contract: EvmAddress,
    pub next_ticket_seq: u64,
    pub finality_blocks: Option<u64>,
    pub next_directive_seq: u64,
    pub next_consume_ticket_seq: u64,
    pub next_consume_directive_seq: u64,
    pub rpc_providers: Vec<RpcApi>,
    pub fee_token_factor: Option<u128>,
    pub target_chain_factor: BTreeMap<ChainId, u128>,
    pub evm_tx_type: EvmTxType,
    pub evm_gasfee_percent: u64,
    pub total_required_count: usize,
    pub minimum_response_count: usize,
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
    read_state(|s| s.rpc_providers.clone())
}

pub fn evm_chain_id() -> u64 {
    read_state(|s| s.evm_chain_id)
}

pub fn evm_transfer_gas_factor() -> u64 {
    read_state(|s| s.evm_transfer_gas_percent)
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
    F: FnOnce(&mut EvmRouteState) -> R,
{
    STATE.with(|s| f(s.borrow_mut().as_mut().expect("State not initialized!")))
}

pub fn read_state<F, R>(f: F) -> R
where
    F: FnOnce(&EvmRouteState) -> R,
{
    STATE.with(|s| f(s.borrow().as_ref().expect("State not initialized!")))
}

/// Replaces the current state.
pub fn replace_state(state: EvmRouteState) {
    STATE.with(|s| {
        *s.borrow_mut() = Some(state);
    });
}

pub fn take_state<F, R>(f: F) -> R
where
    F: FnOnce(EvmRouteState) -> R,
{
    STATE.with(|s| f(s.take().expect("State not initialized!")))
}

pub fn get_redeem_fee(chain_id: ChainId) -> Option<u64> {
    read_state(|s| {
        s.target_chain_factor
            .get(&chain_id)
            .map_or(None, |target_chain_factor| {
                s.fee_token_factor
                    .map(|fee_token_factor| (target_chain_factor * fee_token_factor) as u64)
            })
    })
}