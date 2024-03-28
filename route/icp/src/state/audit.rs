use super::RouteState;
use candid::Principal;
use omnity_types::{Chain, ToggleState, Token};

pub fn add_chain(state: &mut RouteState, chain: Chain) {
    state.counterparties.insert(chain.chain_id.clone(), chain);
}

pub fn add_token(state: &mut RouteState, token: Token, ledger_id: Principal) {
    let token_id = token.token_id.clone();
    state.tokens.insert(token_id.clone(), token);
    state.token_ledgers.insert(token_id, ledger_id);
}

pub fn toggle_chain_state(state: &mut RouteState, toggle: ToggleState) {
    if let Some(chain) = state.counterparties.get_mut(&toggle.chain_id) {
        chain.chain_state = toggle.action.into();
    }
}
