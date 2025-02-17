use crate::{
    state::{mutate_state, read_state, ReleaseTokenReq, TxStatus},
    types::omnity_types::Ticket,
};
use candid::CandidType;
use ic_solana::types::Pubkey;
use serde::Deserialize;
use std::str::FromStr;

#[derive(CandidType, Clone, Debug, Deserialize, PartialEq, Eq)]
pub enum AddReleaseTokenReqErr {
    AlreadyProcessing,
    AlreadyProcessed,
    InvalidAmount(String),
    InvalidSolAddress(String),
}

pub fn add_release_token_req(ticket: Ticket) -> Result<(), AddReleaseTokenReqErr> {
    if read_state(|s| s.release_token_requests.contains_key(&ticket.ticket_id)) {
        return Err(AddReleaseTokenReqErr::AlreadyProcessing);
    }
    if read_state(|s| s.finalized_requests.contains_key(&ticket.ticket_id)) {
        return Err(AddReleaseTokenReqErr::AlreadyProcessed);
    }

    let amount = u64::from_str_radix(ticket.amount.as_str(), 10)
        .map_err(|err| AddReleaseTokenReqErr::InvalidAmount(err.to_string()))?;
    let address = Pubkey::from_str(&ticket.receiver)
        .map_err(|err| AddReleaseTokenReqErr::InvalidSolAddress(err.to_string()))?;

    let req = ReleaseTokenReq {
        ticket_id: ticket.ticket_id,
        action: ticket.action,
        token_id: ticket.token,
        amount: amount,
        address: address,
        received_at: ic_cdk::api::time(),
        signature: None,
        submitted_at: None,
        status: TxStatus::Pending,
    };

    mutate_state(|s| s.release_token_requests.insert(req.ticket_id.clone(), req));
    Ok(())
}
