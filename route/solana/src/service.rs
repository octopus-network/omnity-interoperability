use candid::Principal;
// use ic_canisters_http_types::{HttpRequest, HttpResponse};

use ic_cdk::{init, post_upgrade, pre_upgrade, query, update};

use ic_log::writer::Logs;
use omnity_types::log::{init_log, LoggerConfigService};
use omnity_types::{Chain, ChainId, Error};
use solana_route::auth::{is_admin, set_perms, Permission};
use solana_route::event::{Event, GetEventsArg};
use solana_route::handler::directive::TokenResp;
use solana_route::handler::ticket::GenerateTicketError;
use solana_route::handler::{self, scheduler::schedule_jobs};

use omnity_types::Network;
use solana_route::event;
use solana_route::lifecycle::{self, RouteArg, UpgradeArgs};
use solana_route::memory::init_stable_log;
use solana_route::state::{mutate_state, read_state, take_state, MintTokenStatus};
use std::time::Duration;

#[init]
fn init(args: RouteArg) {
    init_log(Some(init_stable_log()));
    match args {
        RouteArg::Init(args) => {
            event::record_event(&Event::Init(args.clone()));
            lifecycle::init(args);
        }
        RouteArg::Upgrade(_) => {
            panic!("expected InitArgs got UpgradeArgs");
        }
    }
    schedule_jobs()
}

#[pre_upgrade]
fn pre_upgrade() {
    take_state(|state| ic_cdk::storage::stable_save((state,)).expect("failed to save state"))
}

#[post_upgrade]
fn post_upgrade(route_arg: Option<RouteArg>) {
    init_log(Some(init_stable_log()));
    let mut upgrade_arg: Option<UpgradeArgs> = None;
    if let Some(route_arg) = route_arg {
        upgrade_arg = match route_arg {
            RouteArg::Upgrade(upgrade_args) => upgrade_args,
            RouteArg::Init(_) => panic!("expected Option<UpgradeArgs> got InitArgs."),
        };
    }
    lifecycle::post_upgrade(upgrade_arg);

    schedule_jobs()
}

#[update(guard = "is_admin")]
fn set_job_interval(_interval: Duration) {
    // set_timer_interval(interval, periodic_task);
}

#[update(guard = "is_admin")]
pub async fn ecdsa_public_key(_network: Network) -> Result<Vec<u8>, Error> {
    todo!()
}

#[update(guard = "is_admin")]
pub async fn resend_tickets() -> Result<(), GenerateTicketError> {
    let tickets_sz = read_state(|s| s.failed_tickets.len());
    while !read_state(|s| s.failed_tickets.is_empty()) {
        let ticket = mutate_state(|rs| rs.failed_tickets.pop()).unwrap();

        let hub_principal = read_state(|s| (s.hub_principal));
        if let Err(err) = handler::ticket::send_ticket(hub_principal, ticket.clone())
            .await
            .map_err(|err| GenerateTicketError::SendTicketErr(format!("{}", err)))
        {
            mutate_state(|state| {
                state.failed_tickets.push(ticket.clone());
            });
            log::error!("failed to resend ticket: {}", ticket.ticket_id);
            return Err(err);
        }
    }
    log::info!("successfully resend {} tickets", tickets_sz);
    Ok(())
}

#[query]
fn get_chain_list() -> Vec<Chain> {
    read_state(|s| {
        s.counterparties
            .iter()
            .map(|(_, chain)| chain.clone())
            .collect()
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

#[query]
fn get_token_list() -> Vec<TokenResp> {
    read_state(|s| {
        s.tokens
            .iter()
            .map(|(_, token)| token.clone().into())
            .collect()
    })
}

#[query]
pub fn get_log_records(offset: usize, limit: usize) -> Logs {
    log::debug!("collecting {limit} log records");
    ic_log::take_memory_records(limit, offset)
}

#[query]
fn get_events(args: GetEventsArg) -> Vec<Event> {
    const MAX_EVENTS_PER_QUERY: usize = 2000;

    event::events()
        .skip(args.start as usize)
        .take(MAX_EVENTS_PER_QUERY.min(args.length as usize))
        .collect()
}

#[query]
pub fn get_redeem_fee(_chain_id: ChainId) -> Option<u64> {
    // read_state(|s| {
    //     s.target_chain_factor
    //         .get(&chain_id)
    //         // Add an additional transfer fee to make users bear the cost of transferring from route subaccount to route default account
    //         .map_or(None, |target_chain_factor| {
    //             s.fee_token_factor.map(|fee_token_factor| {
    //                 (target_chain_factor * fee_token_factor) as u64 + ICP_TRANSFER_FEE
    //             })
    //         })
    // })
    None
}

// #[query(hidden = true)]
// fn http_request(req: HttpRequest) -> HttpResponse {
//     StableLogWriter::http_request(req)
// }

#[update(guard = "is_admin")]
pub async fn set_logger_filter(filter: String) {
    LoggerConfigService::default().set_logger_filter(&filter);
}

#[update(guard = "is_admin")]
pub async fn set_permissions(caller: Principal, perm: Permission) {
    set_perms(caller.to_string(), perm)
}

fn main() {}

// Enable Candid export
ic_cdk::export_candid!();
