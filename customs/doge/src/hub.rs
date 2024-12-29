use crate::call_error::{CallError, Reason};
use candid::utils::ArgumentEncoder;
use candid::CandidType;
use candid::Principal;
use ic_canister_log::log;
use omnity_types::ic_log::INFO;
use omnity_types::Directive;
use omnity_types::TicketId;
use omnity_types::Topic;
use omnity_types::{self, ChainId, Seq, Ticket};

pub async fn query_tickets(
    hub_principal: Principal,
    offset: u64,
    limit: u64,
) -> Result<Vec<(Seq, Ticket)>, CallError> {
    call(
        hub_principal,
        "query_tickets".into(),
        (None::<Option<ChainId>>, offset, limit),
    )
    .await
}

pub async fn update_tx_hash(
    hub_principal: Principal,
    ticket_id: TicketId,
    mint_tx_hash: String,
) -> Result<(), CallError> {
    let r = call(
        hub_principal,
        "update_tx_hash".into(),
        (ticket_id.clone(), mint_tx_hash.clone()),
    )
    .await;
    if r.is_ok() {
        log!(
            INFO,
            "[rewrite tx_hash] write doge unlock ticket({}) tx hash({}) success.",
            ticket_id,
            mint_tx_hash
        );
    }
    r
}

pub async fn query_directives(
    hub_principal: Principal,
    offset: u64,
    limit: u64,
) -> Result<Vec<(Seq, Directive)>, CallError> {
    call(
        hub_principal,
        "query_directives".into(),
        (
            None::<Option<ChainId>>,
            None::<Option<Topic>>,
            offset,
            limit,
        ),
    )
    .await
}

pub async fn pending_ticket(hub_principal: Principal, ticket: Ticket) -> Result<(), CallError> {
    call(hub_principal, "pending_ticket".into(), (ticket,)).await
}

// pub async fn send_ticket(hub_principal: Principal, ticket: Ticket) -> Result<(), CallError> {
//     let hub_principal = read_state(|s| s.hub_principal);
//     call(hub_principal, "send_ticket".into(), (ticket,)).await
// }

pub async fn finalize_ticket(hub_principal: Principal, ticket_id: String) -> Result<(), CallError> {
    call(hub_principal, "finalize_ticket".into(), (ticket_id,)).await
}

async fn call<T: ArgumentEncoder, R>(
    hub_principal: Principal,
    method: String,
    args: T,
) -> Result<R, CallError>
where
    R: for<'a> candid::Deserialize<'a> + CandidType,
{
    let resp: (Result<R, omnity_types::Error>,) =
        ic_cdk::api::call::call(hub_principal, &method, args)
            .await
            .map_err(|(code, message)| CallError {
                method: method.to_string(),
                reason: Reason::from_reject(code, message),
            })?;
    let data = resp.0.map_err(|err| CallError {
        method: method.to_string(),
        reason: Reason::CanisterError(err.to_string()),
    })?;
    Ok(data)
}
