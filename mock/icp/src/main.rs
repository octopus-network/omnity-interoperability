use candid::Principal;
use ic_cdk::{init, query, update};
use ic_cdk_timers::set_timer_interval;
use icp_mock::types::{
    mutate_state, read_state, Args, CallError, MintTokenStatus, Reason, TimerLogicGuard,
};
use log::info;
use omnity_types::{log::init_log, Directive, Seq, Ticket, TicketId, Topic};

use std::time::Duration;

pub const INTERVAL_QUERY_DIRECTIVE: u64 = 5;
pub const INTERVAL_QUERY_TICKET: u64 = 5;
pub const DIRE_CHAIN_ID: &str = "eICP";
pub const TICKET_CHAIN_ID: &str = "Bitcoin";

pub async fn query_directives(
    hub_principal: Principal,
    method: String,
    offset: u64,
    limit: u64,
) -> Result<Vec<(Seq, Directive)>, CallError> {
    // let (hub_principal, query_directive) =
    //     read_state(|s| (s.hub_principal, s.directive_method.to_string()));
    // let offset = 0u64;
    // let limit = 12u64;
    let resp: (Result<Vec<(Seq, Directive)>, omnity_types::Error>,) = ic_cdk::api::call::call(
        hub_principal,
        &method,
        (Some(DIRE_CHAIN_ID), None::<Option<Topic>>, offset, limit),
    )
    .await
    .map_err(|(code, message)| CallError {
        method: method.to_string(),
        reason: Reason::from_reject(code, message),
    })?;

    let data = resp.0.map_err(|err| CallError {
        method: method,
        reason: Reason::CanisterError(err.to_string()),
    })?;

    Ok(data)
}

fn handle_directive() {
    ic_cdk::spawn(async {
        let _guard = match TimerLogicGuard::new("FETCH_HUB_DIRECTIVE".to_string()) {
            Some(guard) => guard,
            None => return,
        };
        let (hub_principal, query_directive) =
            read_state(|s| (s.hub_principal, s.directive_method.to_string()));
        let offset = 0u64;
        let limit = 12u64;
        match query_directives(hub_principal, query_directive.to_string(), offset, limit).await {
            Ok(directives) => {
                info!("{} result : {:?}", query_directive, directives);
            }
            Err(err) => {
                info!(" failed to {}, err: {:?}", query_directive, err);
            }
        }
    })
}
pub async fn query_tickets(
    hub_principal: Principal,
    method: String,
    offset: u64,
    limit: u64,
) -> Result<Vec<(Seq, Ticket)>, CallError> {
    // let (hub_principal, query_ticket) =
    //     read_state(|s| (s.hub_principal, s.ticket_method.to_string()));
    // let offset = 0u64;
    // let limit = 6u64;
    let resp: (Result<Vec<(Seq, Ticket)>, omnity_types::Error>,) = ic_cdk::api::call::call(
        hub_principal,
        &method,
        (Some(TICKET_CHAIN_ID), offset, limit),
    )
    .await
    .map_err(|(code, message)| CallError {
        method: method.to_string(),
        reason: Reason::from_reject(code, message),
    })?;
    let data = resp.0.map_err(|err| CallError {
        method: method,
        reason: Reason::CanisterError(err.to_string()),
    })?;
    Ok(data)
}
fn handle_tickets() {
    ic_cdk::spawn(async {
        let _guard = match TimerLogicGuard::new("FETCH_HUB_TICKET".to_string()) {
            Some(guard) => guard,
            None => return,
        };
        let (hub_principal, query_ticket) =
            read_state(|s| (s.hub_principal, s.ticket_method.to_string()));
        let offset = 0u64;
        let limit = 6u64;
        match query_tickets(hub_principal, query_ticket.to_string(), offset, limit).await {
            Ok(tickets) => {
                info!("{} result : {:?}", query_ticket, tickets);
            }
            Err(err) => {
                info!(" failed to {}, err: {:?}", query_ticket, err);
            }
        }
    })
}

fn schedule_jobs() {
    set_timer_interval(
        Duration::from_secs(INTERVAL_QUERY_DIRECTIVE),
        handle_directive,
    );
    set_timer_interval(Duration::from_secs(INTERVAL_QUERY_TICKET), handle_tickets);
}

#[init]
fn init(args: Args) {
    init_log(None);
    mutate_state(|s| {
        s.hub_principal = args.hub_principal;
        s.directive_method = args.directive_method;
        s.ticket_method = args.ticket_method;
    });
    schedule_jobs()
}

#[update]
fn mock_finalized_mint_token(ticket_id: TicketId, block_idx: u64) {
    mutate_state(|s| {
        s.finalized_mint_token_requests.insert(ticket_id, block_idx);
    })
}

#[query]
fn mint_token_status(ticket_id: String) -> MintTokenStatus {
    read_state(|s| {
        s.finalized_mint_token_requests
            .get(&ticket_id)
            .map_or(MintTokenStatus::Unknown, |&block_index| {
                MintTokenStatus::Finalized { block_index }
            })
    })
}

fn main() {}
ic_cdk::export_candid!();
