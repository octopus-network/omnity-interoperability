use crate::*;
use omnity_types::Directive;

pub async fn send_ticket(hub_principal: Principal, ticket: Ticket) -> Result<()> {
    // TODO determine how many cycle it will cost.
    let cost_cycles = 4_000_000_000_u64;

    let resp: (std::result::Result<(), omnity_types::Error>,) =
        ic_cdk::api::call::call_with_payment(hub_principal, "send_ticket", (ticket,), cost_cycles)
            .await
            .map_err(|(code, message)| {
                RouteError::CallError(
                    "send_ticket".to_string(),
                    hub_principal,
                    format!("{:?}", code).to_string(),
                    message,
                )
            })?;
    let data = resp
        .0
        .map_err(|err| RouteError::CustomError(format!("Error calling send_ticket: {:?}", err)))?;
    Ok(data)
}

pub async fn query_tickets(
    hub_principal: Principal,
    offset: u64,
    limit: u64,
) -> Result<Vec<(Seq, Ticket)>> {
    let resp: (std::result::Result<Vec<(Seq, Ticket)>, omnity_types::Error>,) =
        ic_cdk::api::call::call(
            hub_principal,
            "query_tickets",
            (None::<Option<ChainId>>, offset, limit),
        )
        .await
        .map_err(|(code, message)| {
            RouteError::CallError(
                "query_tickets".to_string(),
                hub_principal,
                format!("{:?}", code).to_string(),
                message,
            )
        })?;

    resp.0.map_err(|err| {
        RouteError::CustomError(format!("Error calling query_tickets: {:?}", err).to_string())
    })
}

pub async fn query_directives(
    hub_principal: Principal,
    offset: u64,
    limit: u64,
) -> Result<Vec<(Seq, Directive)>> {
    let resp: (std::result::Result<Vec<(Seq, Directive)>, omnity_types::Error>,) =
        ic_cdk::api::call::call(
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
        .map_err(|(code, message)| {
            RouteError::CallError(
                "query_directives".to_string(),
                hub_principal,
                format!("{:?}", code).to_string(),
                message,
            )
        })?;
    let data = resp.0.map_err(|err| {
        RouteError::CustomError(format!("Error calling query_directives: {:?}", err).to_string())
    })?;
    log::info!("query_directives: {:?}", data);
    Ok(data)
}

pub async fn update_tx_hash(
    hub_principal: Principal,
    ticket_id: TicketId,
    mint_tx_hash: String,
) -> Result<()> {
    let resp: (std::result::Result<(), omnity_types::Error>,) =
        ic_cdk::api::call::call(hub_principal, "update_tx_hash", (ticket_id, mint_tx_hash))
            .await
            .map_err(|(code, message)| {
                RouteError::CallError(
                    "update_tx_hash".to_string(),
                    hub_principal,
                    format!("{:?}", code).to_string(),
                    message,
                )
            })?;
    resp.0.map_err(|err| {
        RouteError::CustomError(format!("Error calling update_tx_hash: {:?}", err).to_string())
    })?;
    Ok(())
}