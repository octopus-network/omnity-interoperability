use candid::{CandidType, Principal};
use ic_stable_structures::storable::Bound;
use ic_stable_structures::Storable;
use omnity_types::ChainState;
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;

use crate::lifecycle::init::InitArgs;

#[derive(CandidType, Clone, Debug, Deserialize, Serialize)]
pub struct RouteState {
    pub cw_port_contract_address: String,
    pub cw_chain_key_derivation_path: Vec<ByteBuf>,
    pub chain_id: String,
    pub cw_rpc_url: String,
    pub cw_rest_url: String,
    pub hub_principal: Principal,
    pub next_directive_seq: u64,

    pub chain_state: ChainState,
    pub next_ticket_seq: u64,
    pub is_timer_running: bool,
    pub cw_public_key_vec: Option<Vec<u8>>,
}

impl Storable for RouteState {
    const BOUND: Bound = Bound::Unbounded;

    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        serde_cbor::to_vec(self)
            .expect("Failed to serialize token ledger.")
            .into()
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        serde_cbor::from_slice(&bytes).expect("Failed to deserialize token ledger.")
    }
}

impl From<InitArgs> for RouteState {
    fn from(args: InitArgs) -> Self {
        Self {
            cw_port_contract_address: args.cosmwasm_port_contract_address,
            cw_chain_key_derivation_path: [vec![1u8; 4]] // Example derivation path for signing
                .iter()
                .map(|v| ByteBuf::from(v.clone()))
                .collect(),
            chain_id: args.chain_id,
            cw_rpc_url: args.cw_rpc_url,
            cw_rest_url: args.cw_rest_url,
            hub_principal: args.hub_principal,
            next_directive_seq: 0,
            chain_state: ChainState::Active,
            next_ticket_seq: 0,
            is_timer_running: false,
            cw_public_key_vec: None,
        }
    }
}