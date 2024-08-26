use crate::state::EvmRouteState;
use omnity_types::{Chain, Factor, ToggleState, Token};

pub fn add_chain(state: &mut EvmRouteState, chain: Chain) {
    state.counterparties.insert(chain.chain_id.clone(), chain);
}

pub fn add_token(state: &mut EvmRouteState, token: Token) {
    state.tokens.insert(token.token_id.clone(), token);
}

pub fn toggle_chain_state(state: &mut EvmRouteState, toggle: ToggleState) {
    if toggle.chain_id == state.omnity_chain_id {
        state.chain_state = toggle.action.into();
    } else if let Some(chain) = state.counterparties.get_mut(&toggle.chain_id) {
        chain.chain_state = toggle.action.into();
    }
}

pub fn update_fee(state: &mut EvmRouteState, fee: Factor) {
    match fee {
        Factor::UpdateTargetChainFactor(factor) => {
            state
                .target_chain_factor
                .insert(factor.target_chain_id.clone(), factor.target_chain_factor);
        }
        Factor::UpdateFeeTokenFactor(token_factor) => {
            if token_factor.fee_token == state.fee_token_id.clone() {
                state.fee_token_factor = Some(token_factor.fee_token_factor);
            }
        }
    }
}
