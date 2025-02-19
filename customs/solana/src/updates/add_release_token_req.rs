use crate::{
    address::main_address_path,
    solana_rpc::{self, init_solana_client},
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
        signature: None,
        submitted_at: None,
        status: ReleaseTokenStatus::Unknown,
    };

    submit_release_token_tx(&mut req).await;
    Ok(())
}

pub async fn submit_release_token_tx(req: &mut ReleaseTokenReq) {
    let client = init_solana_client().await;
    let main_path = main_address_path();
    let main_address = solana_rpc::ecdsa_public_key(main_path.clone()).await;
    match client
        .transfer(main_address, main_path.clone(), req.address, req.amount)
        .await
    {
        Err(err) => {
            log!(
                ERROR,
                "[submit_release_token_tx] failed to transfer token for ticket_id:{}, err: {:?}",
                req.ticket_id,
                err
            );
            req.status = ReleaseTokenStatus::Failed(err.to_string());
        }
        Ok(signature) => {
            req.status = ReleaseTokenStatus::Submitted;
            req.signature = Some(signature);
            req.submitted_at = Some(ic_cdk::api::time());
        }
    }
    mutate_state(|s| {
        s.release_token_requests
            .insert(req.ticket_id.clone(), req.clone())
    });
}
