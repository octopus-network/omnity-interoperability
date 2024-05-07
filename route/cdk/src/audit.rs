//use super::eventlog::Event;
use crate::state::CdkRouteState;
use crate::types::{Chain, Factor, ToggleState, Token};

pub fn add_chain(state: &mut CdkRouteState, chain: Chain) {
   // record_event(&Event::AddedChain(chain.clone()));
    state.counterparties.insert(chain.chain_id.clone(), chain);
}

pub fn add_token(state: &mut CdkRouteState, token: Token) {
    //TODO
    /*    record_event(&Event::AddedToken {
        ledger_id,
        token: token.clone(),
    });*/
    let token_id = token.token_id.clone();
    state.tokens.insert(token_id.clone(), token);
}

pub fn toggle_chain_state(state: &mut CdkRouteState, toggle: ToggleState) {
    if toggle.chain_id == state.omnity_chain_id {
        state.chain_state = toggle.action.into();
    } else if let Some(chain) = state.counterparties.get_mut(&toggle.chain_id) {
       // record_event(&Event::ToggleChainState(toggle.clone()));
        chain.chain_state = toggle.action.into();
    }
}

pub fn finalize_mint_token_req(
    state: &mut CdkRouteState,
    ticket_id: String,
    finalized_block_index: u64,
) {
    //TODO
   /* record_event(&Event::FinalizedMintToken {
        ticket_id: ticket_id.clone(),
        block_index: finalized_block_index,
    });*/
    state
        .finalized_mint_token_requests
        .insert(ticket_id, finalized_block_index);
}

/*pub fn finalize_gen_ticket(ticket_id: String, request: GenerateTicketReq) {
   // record_event(&Event::FinalizedGenTicket { ticket_id, request })
}*/

pub fn update_fee(state: &mut CdkRouteState, fee: Factor) {
   // record_event(&Event::UpdatedFee { fee: fee.clone() });
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
