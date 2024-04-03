use crate::updates::{generate_ticket::GenerateTicketReq, mint_token::MintTokenRequest};
use candid::Principal;
use omnity_types::{Chain, Fee, ToggleState, Token};
use serde::{Deserialize, Serialize};

#[derive(candid::CandidType, Deserialize)]
pub struct GetEventsArg {
    pub start: u64,
    pub length: u64,
}

#[derive(candid::CandidType, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Event {
    #[serde(rename = "added_chain")]
    AddedChain(Chain),

    #[serde(rename = "added_token")]
    AddedToken { ledger_id: Principal, token: Token },

    #[serde(rename = "updated_fee")]
    UpdatedFee { fee: Fee },

    #[serde(rename = "toggle_chain_state")]
    ToggleChainState(ToggleState),

    #[serde(rename = "finalized_mint_token")]
    FinalizedMintToken(MintTokenRequest),

    #[serde(rename = "finalized_gen_ticket")]
    FinalizedGenTicket {
        block_index: u64,
        request: GenerateTicketReq,
    },
}
