use super::eventlog::Event;
use super::RouteState;
use crate::{
    storage::record_event,
    updates::{generate_ticket::GenerateTicketReq, mint_token::MintTokenRequest},
};
use candid::Principal;
use omnity_types::{Chain, Factor, ToggleState, Token};

pub fn add_chain(state: &mut RouteState, chain: Chain) {
    record_event(&Event::AddedChain(chain.clone()));
    state.counterparties.insert(chain.chain_id.clone(), chain);
}

pub fn add_token(state: &mut RouteState, token: Token, ledger_id: Principal) {
    record_event(&Event::AddedToken {
        ledger_id,
        token: token.clone(),
    });
    let token_id = token.token_id.clone();
    state.tokens.insert(token_id.clone(), token);
    state.token_ledgers.insert(token_id, ledger_id);
}

pub fn toggle_chain_state(state: &mut RouteState, toggle: ToggleState) {
    if let Some(chain) = state.counterparties.get_mut(&toggle.chain_id) {
        record_event(&Event::ToggleChainState(toggle.clone()));
        chain.chain_state = toggle.action.into();
    }
}

pub fn finalize_mint_token_req(state: &mut RouteState, req: MintTokenRequest) {
    record_event(&Event::FinalizedMintToken(req.clone()));
    state
        .finalized_mint_token_requests
        .insert(req.ticket_id.clone(), req);
}

pub fn finalize_gen_ticket(block_index: u64, request: GenerateTicketReq) {
    record_event(&Event::FinalizedGenTicket {
        block_index,
        request,
    })
}

pub fn update_fee(state: &mut RouteState, fee: Factor) {
    record_event(&Event::UpdatedFee { fee: fee.clone() });
    match fee {
        Fee::ChainFactor(chain_factor) => {
            state
                .redeem_fees
                .entry(chain_factor.chain_id.clone())
                .or_default()
                .target_chain_factor = chain_factor.chain_factor;
        }
        Fee::TokenFactor(token_factor) => {
            state
                .redeem_fees
                .entry(token_factor.dst_chain_id.clone())
                .or_default()
                .fee_token_factor = token_factor.fee_token_factor;
        }
    }
}
