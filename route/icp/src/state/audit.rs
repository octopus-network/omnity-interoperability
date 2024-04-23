use super::eventlog::Event;
use super::RouteState;
use crate::{storage::record_event, updates::generate_ticket::GenerateTicketReq};
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
    if toggle.chain_id == state.chain_id {
        state.chain_state = toggle.action.into();
    } else if let Some(chain) = state.counterparties.get_mut(&toggle.chain_id) {
        record_event(&Event::ToggleChainState(toggle.clone()));
        chain.chain_state = toggle.action.into();
    }
}

pub fn finalize_mint_token_req(
    state: &mut RouteState,
    ticket_id: String,
    finalized_block_index: u64,
) {
    record_event(&Event::FinalizedMintToken {
        ticket_id: ticket_id.clone(),
        block_index: finalized_block_index,
    });
    state
        .finalized_mint_token_requests
        .insert(ticket_id, finalized_block_index);
}

pub fn finalize_gen_ticket(ticket_id: String, request: GenerateTicketReq) {
    record_event(&Event::FinalizedGenTicket { ticket_id, request })
}

pub fn update_fee(state: &mut RouteState, fee: Factor) {
    record_event(&Event::UpdatedFee { fee: fee.clone() });
    match fee {
        Factor::UpdateTargetChainFactor(factor) => {
            state
                .target_chain_factor
                .insert(factor.target_chain_id.clone(), factor.target_chain_factor);
        }

        Factor::UpdateFeeTokenFactor(token_factor) => {
            state.fee_token_factor = Some(token_factor.fee_token_factor);
        }
    }
}
