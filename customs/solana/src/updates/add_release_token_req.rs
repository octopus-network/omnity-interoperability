use crate::{
    solana_rpc,
    state::{mutate_state, read_state, ReleaseTokenReq, ReleaseTokenStatus},
    types::omnity_types::Ticket,
};
use candid::CandidType;
use ic_canister_log::log;
use ic_solana::{ic_log::ERROR, types::Pubkey};
use serde::Deserialize;
use std::str::FromStr;

#[derive(CandidType, Clone, Debug, Deserialize, PartialEq, Eq)]
pub enum AddReleaseTokenReqErr {
    AlreadyProcessing,
    AlreadyProcessed,
    InvalidAmount(String),
    InvalidSolAddress(String),
}

pub async fn add_release_token_req(ticket: Ticket) -> Result<(), AddReleaseTokenReqErr> {
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

    let mut req = ReleaseTokenReq {
        ticket_id: ticket.ticket_id,
        action: ticket.action,
        token_id: ticket.token,
        amount: amount,
        address: address,
        received_at: ic_cdk::api::time(),
        last_sent_at: 0,
        try_cnt: 0,
        status: ReleaseTokenStatus::Pending,
    };

    submit_release_token_tx(&mut req).await;
    Ok(())
}

pub async fn submit_release_token_tx(req: &mut ReleaseTokenReq) {
    match solana_rpc::redeem(req.ticket_id.clone(), req.address, req.amount).await {
        Err(err) => {
            log!(
                ERROR,
                "[submit_release_token_tx] failed to redeem token for ticket_id:{}, err: {}",
                req.ticket_id,
                err
            );
        }
        Ok(signature) => {
            req.status = ReleaseTokenStatus::Submitted(signature);
        }
    }
    mutate_state(|s| {
        req.last_sent_at = ic_cdk::api::time();
        req.try_cnt += 1;
        s.release_token_requests
            .insert(req.ticket_id.clone(), req.clone())
    });
}
