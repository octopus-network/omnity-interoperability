use omnity_types::{Chain, Token};
use crate::state::Brc20State;

pub fn add_chain(state: &mut Brc20State, chain: Chain) {
    state.counterparties.insert(chain.chain_id.clone(), chain);
}

pub fn add_token(state: &mut Brc20State, token: Token) {
    state.tokens.insert(token.token_id.clone(), token);
}
