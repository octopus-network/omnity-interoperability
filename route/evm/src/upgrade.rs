use std::collections::{BTreeMap, BTreeSet};

use candid::Principal;
use cketh_common::eth_rpc_client::providers::RpcApi;
use ic_cdk::api::management_canister::ecdsa::EcdsaKeyId;
use ic_stable_structures::StableBTreeMap;
use serde_derive::{Deserialize, Serialize};

use crate::eth_common::{EvmAddress, EvmTxType};
use crate::stable_memory::Memory;
use crate::state::EvmRouteState;
use crate::types::{
    Chain, ChainId, ChainState, Directive, PendingDirectiveStatus, PendingTicketStatus, Seq,
    Ticket, TicketId, Timestamp, Token, TokenId,
};

#[derive(Deserialize, Serialize)]
pub struct OldEvmRouteState {
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
    pub handled_evm_event: BTreeSet<String>,
    #[serde(skip, default = "crate::stable_memory::init_to_evm_tickets_queue")]
    pub tickets_queue: StableBTreeMap<u64, Ticket, Memory>,
    #[serde(skip, default = "crate::stable_memory::init_to_evm_directives_queue")]
    pub directives_queue: StableBTreeMap<u64, Directive, Memory>,
    #[serde(skip, default = "crate::stable_memory::init_pending_ticket_map")]
    pub pending_tickets_map: StableBTreeMap<TicketId, PendingTicketStatus, Memory>,
    #[serde(skip, default = "crate::stable_memory::init_pending_directive_map")]
    pub pending_directive_map: StableBTreeMap<Seq, PendingDirectiveStatus, Memory>,
    pub scan_start_height: u64,
    #[serde(skip)]
    pub is_timer_running: BTreeMap<String, bool>,
    pub evm_tx_type: EvmTxType,
    #[serde(skip)]
    pub latest_scan_height_update_time: Timestamp,
}

impl From<(OldEvmRouteState, u64)> for EvmRouteState {
    fn from(value: (OldEvmRouteState, u64)) -> Self {
        let old = value.0;
        Self {
            admins: old.admins,
            hub_principal: old.hub_principal,
            omnity_chain_id: old.omnity_chain_id,
            evm_chain_id: old.evm_chain_id,
            fee_token_id: old.fee_token_id,
            tokens: old.tokens,
            token_contracts: old.token_contracts,
            counterparties: old.counterparties,
            finalized_mint_token_requests: old.finalized_mint_token_requests,
            chain_state: old.chain_state,
            evm_rpc_addr: old.evm_rpc_addr,
            key_id: old.key_id,
            key_derivation_path: old.key_derivation_path,
            pubkey: old.pubkey,
            rpc_providers: old.rpc_providers,
            omnity_port_contract: old.omnity_port_contract,
            fee_token_factor: old.fee_token_factor,
            target_chain_factor: old.target_chain_factor,
            next_ticket_seq: old.next_ticket_seq,
            next_directive_seq: old.next_directive_seq,
            next_consume_ticket_seq: old.next_consume_ticket_seq,
            next_consume_directive_seq: old.next_consume_directive_seq,
            handled_evm_event: old.handled_evm_event,
            tickets_queue: old.tickets_queue,
            directives_queue: old.directives_queue,
            pending_tickets_map: old.pending_tickets_map,
            pending_directive_map: old.pending_directive_map,
            is_timer_running: old.is_timer_running,
            evm_tx_type: old.evm_tx_type,
            block_interval_secs: value.1,
            pending_events_on_chain: Default::default(),
        }
    }
}
