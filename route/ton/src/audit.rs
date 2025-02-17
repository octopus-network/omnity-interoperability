use omnity_types::{Chain, Factor, Token};

use crate::state::{TonRouteState, TON_NATIVE_TOKEN};

pub fn add_chain(state: &mut TonRouteState, chain: Chain) {
    state.counterparties.insert(chain.chain_id.clone(), chain);
}

pub fn add_token(state: &mut TonRouteState, token: Token) {
    state.tokens.insert(token.token_id.clone(), token);
}

pub fn update_fee(state: &mut TonRouteState, fee: Factor) {
    match fee {
        Factor::UpdateTargetChainFactor(factor) => {
            state
                .target_chain_factor
                .insert(factor.target_chain_id.clone(), factor.target_chain_factor);
        }
        Factor::UpdateFeeTokenFactor(token_factor) => {
            if token_factor.fee_token == *TON_NATIVE_TOKEN {
                state.fee_token_factor = Some(token_factor.fee_token_factor);
            }
        }
    }
}
