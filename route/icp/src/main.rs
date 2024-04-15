use candid::Principal;
use ic_cdk::{caller, post_upgrade, pre_upgrade};
use ic_cdk_macros::{init, query, update};
use ic_cdk_timers::set_timer_interval;
use ic_ledger_types::AccountIdentifier;
use ic_log::writer::Logs;
use icp_route::lifecycle::{self, init::RouteArg};
use icp_route::state::eventlog::{Event, GetEventsArg};
use icp_route::state::{read_state, replace_state, take_state, MintTokenStatus, RouteState};
use icp_route::updates::generate_ticket::{
    principal_to_subaccount, GenerateTicketError, GenerateTicketOk, GenerateTicketReq,
};
use icp_route::updates::{self};
use icp_route::{periodic_task, storage, ICP_TRANSFER_FEE, PERIODIC_TASK_INTERVAL};
use log::{self};
use omnity_types::log::{init_log, StableLog};
use omnity_types::{Chain, ChainId, Token};
use std::time::Duration;

#[init]
fn init(args: RouteArg) {
    match args {
        RouteArg::Init(args) => {
            init_log(StableLog::default());
            storage::record_event(&Event::Init(args.clone()));
            lifecycle::init::init(args);
            set_timer_interval(Duration::from_secs(PERIODIC_TASK_INTERVAL), periodic_task);
        }
        RouteArg::Upgrade(_) => {
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
            .map_or(MintTokenStatus::Unknown, |req| MintTokenStatus::Finalized {
                block_index: req.finalized_block_index.unwrap(),
            })
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
pub fn get_log_records(offset: usize, limit: usize) -> Logs {
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
pub fn get_fee_account(principal: Option<Principal>) -> AccountIdentifier {
    let principal = principal.unwrap_or(caller());
    AccountIdentifier::new(&ic_cdk::api::id(), &principal_to_subaccount(&principal))
}

#[query]
pub fn get_redeem_fee(chain_id: ChainId) -> Option<u64> {
    read_state(|s| {
        s.target_chain_factor
            .get(&chain_id)
            // Add an additional transfer fee to make users bear the cost of transferring from route subaccount to route default account
            .map_or(None, |target_chain_factor| {
                s.fee_token_factor.map(|fee_token_factor| {
                    (target_chain_factor * fee_token_factor) as u64 + ICP_TRANSFER_FEE
                })
            })
    })
}

#[query]
pub fn get_hub_principal() -> Principal {
    read_state(|s| {
        s.hub_principal.clone()
    })
}

#[pre_upgrade]
fn pre_upgrade() {
    take_state(|state| ic_cdk::storage::stable_save((state,)).expect("failed to save state"))
}

#[post_upgrade]
fn post_upgrade(route_arg: Option<RouteArg>) {
    match route_arg.unwrap() {
        RouteArg::Init(_) => {
            panic!("expected UpgradeArgs, got InitArgs");
        }
        RouteArg::Upgrade(upgrade_args) => {
            let (mut stable_state,): (RouteState,) =
                ic_cdk::storage::stable_restore().expect("failed to restore state");

            stable_state.upgrade(upgrade_args.clone());

            replace_state(stable_state);

            storage::record_event(&Event::Upgrade(upgrade_args));
        }
    }
}

fn main() {}

// Enable Candid export
ic_cdk::export_candid!();
