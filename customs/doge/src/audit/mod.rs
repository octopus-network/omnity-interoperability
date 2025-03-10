use crate::state::DogeState;
use omnity_types::{Chain, Token};

pub fn add_chain(state: &mut DogeState, chain: Chain) {
    state.counterparties.insert(chain.chain_id.clone(), chain);
}

pub fn add_token(state: &mut DogeState, token: Token) {
    state.tokens.insert(token.token_id.clone(), token);
}
