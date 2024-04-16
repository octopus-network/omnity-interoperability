use crate::{lifecycle::init::InitArgs, updates::generate_ticket::GenerateTicketReq};
use candid::Principal;
use omnity_types::{Chain, Factor, ToggleState, Token};
use serde::{Deserialize, Serialize};

#[derive(candid::CandidType, Deserialize)]
pub struct GetEventsArg {
    pub start: u64,
    pub length: u64,
}

#[derive(candid::CandidType, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Event {
    /// Indicates the route initialization with the specified arguments.  Must be
    /// the first event in the event log.
    #[serde(rename = "init")]
    Init(InitArgs),

    #[serde(rename = "added_chain")]
    AddedChain(Chain),

    #[serde(rename = "added_token")]
    AddedToken { ledger_id: Principal, token: Token },

    #[serde(rename = "updated_fee")]
    UpdatedFee { fee: Factor },

    #[serde(rename = "toggle_chain_state")]
    ToggleChainState(ToggleState),

    #[serde(rename = "finalized_mint_token")]
    FinalizedMintToken { ticket_id: String, block_index: u64 },

    #[serde(rename = "finalized_gen_ticket")]
    FinalizedGenTicket {
        ticket_id: String,
        request: GenerateTicketReq,
    },
}
