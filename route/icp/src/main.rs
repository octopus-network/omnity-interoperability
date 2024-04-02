use candid::Principal;
use ic_cdk_macros::{init, query, update};
use ic_cdk_timers::set_timer_interval;
use ic_log::writer::Logs;
use icp_route::lifecycle::{self, init::RouteArg};
use icp_route::log_util::init_log;
use icp_route::state::eventlog::{Event, GetEventsArg};
use icp_route::state::{read_state, MintTokenStatus};
use icp_route::updates::generate_ticket::{
    principal_to_subaccount, GenerateTicketError, GenerateTicketOk, GenerateTicketReq,
};
use icp_route::updates::{self};
use icp_route::{periodic_task, storage, PERIODIC_TASK_INTERVAL};
use log::{self};
use omnity_types::{Chain, ChainId, Token};
use std::time::Duration;

#[init]
fn init(args: RouteArg) {
    match args {
        RouteArg::Init(args) => {
            init_log();
            lifecycle::init::init(args);
            set_timer_interval(Duration::from_secs(PERIODIC_TASK_INTERVAL), periodic_task);
        }
        RouteArg::Upgrade() => {
            panic!("expected InitArgs got UpgradeArgs");
        }
    }
}

fn check_anonymous_caller() {
    if ic_cdk::caller() == Principal::anonymous() {
        panic!("anonymous caller not allowed")
    }
}

#[update]
async fn generate_ticket(args: GenerateTicketReq) -> Result<GenerateTicketOk, GenerateTicketError> {
    check_anonymous_caller();
    updates::generate_ticket(args).await
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
            .map_or(MintTokenStatus::Unknown, |req| req.status.clone())
    })
}

#[query]
fn get_token_list() -> Vec<Token> {
    read_state(|s| s.tokens.iter().map(|(_, token)| token.clone()).collect())
}

#[query]
fn get_token_ledger(token_id: String) -> Option<Principal> {
    read_state(|s| s.token_ledgers.get(&token_id).cloned())
}

#[query]
pub fn get_log_records(limit: usize, offset: usize) -> Logs {
    log::debug!("collecting {limit} log records");
    ic_log::take_memory_records(limit, offset)
}

#[query]
fn get_events(args: GetEventsArg) -> Vec<Event> {
    const MAX_EVENTS_PER_QUERY: usize = 2000;

    storage::events()
        .skip(args.start as usize)
        .take(MAX_EVENTS_PER_QUERY.min(args.length as usize))
        .collect()
}

#[query]
pub fn get_deposit_subaccount(principal: Principal) -> ic_ledger_types::Subaccount {
    principal_to_subaccount(&principal)
}

#[query]
pub fn get_redeem_fee(chain_id: ChainId) -> u64 {
    read_state(|s| {
        s.redeem_fees
            .get(&chain_id)
            .expect("unreachable: redeem fee not found")
            .clone()
            .factor
            * (ic_ledger_types::Tokens::SUBDIVIDABLE_BY as i64)
    }) as u64 + icp_route::ICP_TRANSFER_FEE
}

fn main() {}

// Enable Candid export
ic_cdk::export_candid!();
