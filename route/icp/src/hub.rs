use crate::call_error::{CallError, Reason};
use candid::Principal;
use omnity_types::Directive;
use omnity_types::TicketId;
use omnity_types::Topic;
use omnity_types::{self, ChainId, Seq, Ticket};

pub async fn send_ticket(hub_principal: Principal, ticket: Ticket) -> Result<(), CallError> {
    // TODO determine how many cycle it will cost.
    let cost_cycles = 4_000_000_000_u64;

    let resp: (Result<(), omnity_types::Error>,) =
        ic_cdk::api::call::call_with_payment(hub_principal, "send_ticket", (ticket,), cost_cycles)
            .await
            .map_err(|(code, message)| CallError {
                method: "send_ticket".to_string(),
                reason: Reason::from_reject(code, message),
            })?;
    let data = resp.0.map_err(|err| CallError {
        method: "send_ticket".to_string(),
        reason: Reason::CanisterError(err.to_string()),
    })?;
    Ok(data)
}

pub async fn query_tickets(
    hub_principal: Principal,
    offset: u64,
    limit: u64,
) -> Result<Vec<(Seq, Ticket)>, CallError> {
    let resp: (Result<Vec<(Seq, Ticket)>, omnity_types::Error>,) = ic_cdk::api::call::call(
        hub_principal,
        "query_tickets",
        (None::<Option<ChainId>>, offset, limit),
    )
    .await
    .map_err(|(code, message)| CallError {
        method: "query_tickets".to_string(),
        reason: Reason::from_reject(code, message),
    })?;
    let data = resp.0.map_err(|err| CallError {
        method: "query_tickets".to_string(),
        reason: Reason::CanisterError(err.to_string()),
    })?;
    Ok(data)
}

pub async fn query_directives(
    hub_principal: Principal,
    offset: u64,
    limit: u64,
) -> Result<Vec<(Seq, Directive)>, CallError> {
    let resp: (Result<Vec<(Seq, Directive)>, omnity_types::Error>,) = ic_cdk::api::call::call(
        hub_principal,
        "query_directives",
        (
            None::<Option<ChainId>>,
            None::<Option<Topic>>,
            offset,
            limit,
        ),
    )
    .await
    .map_err(|(code, message)| CallError {
        method: "query_directives".to_string(),
        reason: Reason::from_reject(code, message),
    })?;
    let data = resp.0.map_err(|err| CallError {
        method: "query_directives".to_string(),
        reason: Reason::CanisterError(err.to_string()),
    })?;
    Ok(data)
}

pub async fn update_tx_hash(
    hub_principal: Principal,
    ticket_id: TicketId,
    mint_tx_hash: String,
) -> Result<(), CallError> {
    let resp: (Result<(), omnity_types::Error>,) =
        ic_cdk::api::call::call(hub_principal, "update_tx_hash", (ticket_id, mint_tx_hash))
            .await
            .map_err(|(code, message)| CallError {
                method: "update_tx_hash".to_string(),
                reason: Reason::from_reject(code, message),
            })?;
    resp.0.map_err(|err| CallError {
        method: "update_tx_hash".to_string(),
        reason: Reason::CanisterError(err.to_string()),
    })?;
    Ok(())
}
