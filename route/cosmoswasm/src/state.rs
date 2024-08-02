use std::{cell::RefCell, collections::BTreeMap};

use candid::Principal;
use cosmrs::AccountId;
use omnity_types::{Chain, ChainId, ChainState, Factor, ToggleState, Token};
use serde_bytes::ByteBuf;

use crate::lifecycle::init::InitArgs;

thread_local! {
    static __STATE: RefCell<Option<RouteState>> = RefCell::default();
}

pub struct RouteState {
    pub schnorr_canister_principal: Principal,
    pub cw_port_contract_address: String,
    pub cw_chain_key_derivation_path: Vec<ByteBuf>,
    pub chain_id: String,
    pub cw_url: String,
    pub hub_principal: Principal,
    pub next_directive_seq: u64,
    pub counterparties: BTreeMap<ChainId, Chain>,
    pub tokens: BTreeMap<String, Token>,
    pub fee_token_factor: Option<u128>,
    pub target_chain_factor: BTreeMap<ChainId, u128>,
    pub chain_state: ChainState,
    pub is_timer_running: bool,
}

impl From<InitArgs> for RouteState {
    fn from(args: InitArgs) -> Self {
        Self {
            schnorr_canister_principal: args.schnorr_canister_principal,
            cw_port_contract_address: args.cosmoswasm_port_contract_address,
            cw_chain_key_derivation_path: [vec![1u8; 4]] // Example derivation path for signing
                .iter()
                .map(|v| ByteBuf::from(v.clone()))
                .collect(),
            chain_id: args.chain_id,
            cw_url: args.cw_url,
            hub_principal: args.hub_principal,
            next_directive_seq: 0,
            counterparties: Default::default(),
            tokens: todo!(),
            fee_token_factor: todo!(),
            target_chain_factor: todo!(),
            chain_state: ChainState::Active,
            is_timer_running: false,
        }
    }
}

/// Mutates (part of) the current state using `f`.
///
/// Panics if there is no state.
pub fn mutate_state<F, R>(f: F) -> R
where
    F: FnOnce(&mut RouteState) -> R,
{
    __STATE.with(|s| f(s.borrow_mut().as_mut().expect("State not initialized!")))
}

pub fn read_state<F, R>(f: F) -> R
where
    F: FnOnce(&RouteState) -> R,
{
    __STATE.with(|s| f(s.borrow().as_ref().expect("State not initialized!")))
}

pub fn replace_state(state: RouteState) {
    __STATE.with(|s| {
        *s.borrow_mut() = Some(state);
    });
}

pub fn get_contract_id() -> AccountId {
    read_state(|state| state.cw_port_contract_address.clone())
        .parse()
        .unwrap()
}

pub fn get_derivation_path() -> Vec<ByteBuf> {
    read_state(|state| state.cw_chain_key_derivation_path.clone())
}

pub fn add_chain(chain: Chain) {
    mutate_state(|state| {
        state.counterparties.insert(chain.chain_id.clone(), chain);
    });
}

pub fn add_token(address: String, token: Token) {
    mutate_state(|state| {
        state.tokens.insert(address, token);
    });
}

pub fn toggle_chain_state(toggle: ToggleState) {
    mutate_state(|state| {
        if toggle.chain_id == state.chain_id {
            state.chain_state = toggle.action.into();
        } else if let Some(chain) = state.counterparties.get_mut(&toggle.chain_id) {
            chain.chain_state = toggle.action.into();
        }
    });
}

pub fn update_fee(state: &mut RouteState, fee: Factor) {
    match fee {
        Factor::UpdateTargetChainFactor(factor) => {
            state
                .target_chain_factor
                .insert(factor.target_chain_id.clone(), factor.target_chain_factor);
        }

        Factor::UpdateFeeTokenFactor(token_factor) => {
            if token_factor.fee_token == "LICP" {
                state.fee_token_factor = Some(token_factor.fee_token_factor);
            }
        }
    }
}