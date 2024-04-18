use candid::Principal;
use ic_canisters_http_types::{HttpRequest, HttpResponse, HttpResponseBuilder};
use ic_cdk::api::management_canister::main::CanisterStatusResponse;
use ic_cdk::{caller, post_upgrade, pre_upgrade};
use ic_cdk_macros::{init, query, update};
use ic_cdk_timers::set_timer_interval;
use ic_ledger_types::AccountIdentifier;
use ic_log::writer::Logs;
use icp_route::lifecycle::{self, init::RouteArg};
use icp_route::memory::init_stable_log;
use icp_route::state::eventlog::{Event, GetEventsArg};
use icp_route::state::{read_state, replace_state, take_state, MintTokenStatus, RouteState};
use icp_route::updates::generate_ticket::{
    principal_to_subaccount, GenerateTicketError, GenerateTicketOk, GenerateTicketReq,
};
use icp_route::updates::{self};
use icp_route::{
    manage_icrc_canister, periodic_task, storage, TokenResp, ICP_TRANSFER_FEE, PERIODIC_TASK_INTERVAL,
};
use log::{self, info};
use omnity_types::log::{init_log, StableLogWriter};
use omnity_types::{Chain, ChainId};
use std::str::FromStr;
use std::time::Duration;

#[init]
fn init(args: RouteArg) {
    match args {
        RouteArg::Init(args) => {
            init_log(Some(init_stable_log()));
            storage::record_event(&Event::Init(args.clone()));
            lifecycle::init::init(args);
            set_timer_interval(Duration::from_secs(PERIODIC_TASK_INTERVAL), periodic_task);
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

pub fn is_controller() -> Result<(), String> {
    if ic_cdk::api::is_controller(&ic_cdk::caller()) {
        Ok(())
    } else {
        Err("caller is not controller".to_string())
    }
}

#[update(guard = "is_controller")]
async fn stop_icrc_canister(icrc_canister_id: Principal) -> Result<(), String> {
    let exist_token_canister = read_state(|s| {
        s.token_ledgers.values().find(|&e| e.eq(&icrc_canister_id)).is_some()
    });
    if !exist_token_canister {
        return Err("Icrc canister id not exist".to_string());
    }
    manage_icrc_canister::stop_icrc_canister(icrc_canister_id)
        .await
        .and_then(|_| Ok(()))
        .map_err(|(_, reason)| reason)
}

#[update(guard = "is_controller")]
async fn start_icrc_canister(icrc_canister_id: Principal) -> Result<(), String> {
    let exist_token_canister = read_state(|s| {
        s.token_ledgers.values().find(|&e| e.eq(&icrc_canister_id)).is_some()
    });
    if !exist_token_canister {
        return Err("Icrc canister id not exist".to_string());
    }
    manage_icrc_canister::start_icrc_canister(icrc_canister_id)
        .await
        .and_then(|_| Ok(()))
        .map_err(|(_, reason)| reason)
}

#[update(guard = "is_controller")]
async fn delete_icrc_canister(icrc_canister_id: Principal) -> Result<(), String> {
    let exist_token_canister = read_state(|s| {
        s.token_ledgers.values().find(|&e| e.eq(&icrc_canister_id)).is_some()
    });
    if !exist_token_canister {
        return Err("Icrc canister id not exist".to_string());
    }
    manage_icrc_canister::delete_icrc_canister(icrc_canister_id)
        .await
        .and_then(|_| Ok(()))
        .map_err(|(_, reason)| reason)
}

#[update(guard = "is_controller")]
pub async fn icrc_canister_status(
    icrc_canister_id: Principal,
) -> Result<CanisterStatusResponse, String> {
    let exist_token_canister = read_state(|s| {
        s.token_ledgers.values().find(|&e| e.eq(&icrc_canister_id)).is_some()
    });
    if !exist_token_canister {
        return Err("Icrc canister id not exist".to_string());
    }
    manage_icrc_canister::icrc_canister_status(icrc_canister_id)
        .await
        .and_then(|(e,)| Ok(e))
        .map_err(|(_, reason)| reason)
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

#[query(hidden = true)]
fn http_request(req: HttpRequest) -> HttpResponse {
    if req.path() == "/logs" {
        use serde_json;
        let max_skip_timestamp = parse_param::<u64>(&req, "time").unwrap_or(0);
        let offset = match parse_param::<usize>(&req, "offset") {
            Ok(value) => value,
            Err(err) => return err,
        };
        let limit = match parse_param::<usize>(&req, "limit") {
            Ok(value) => value,
            Err(err) => return err,
        };
        info!(
            "log req, max_skip_timestamp: {}, offset: {}, limit: {}",
            max_skip_timestamp, offset, limit
        );

        let logs = StableLogWriter::get_logs(max_skip_timestamp, offset, limit);
        HttpResponseBuilder::ok()
            .header("Content-Type", "application/json; charset=utf-8")
            .with_body_and_content_length(serde_json::to_string(&logs).unwrap_or_default())
            .build()
    } else {
        HttpResponseBuilder::not_found().build()
    }
}

fn parse_param<T: FromStr>(req: &HttpRequest, param_name: &str) -> Result<T, HttpResponse> {
    match req.raw_query_param(param_name) {
        Some(arg) => match arg.parse() {
            Ok(value) => Ok(value),
            Err(_) => Err(HttpResponseBuilder::bad_request()
                .with_body_and_content_length(format!(
                    "failed to parse the '{}' parameter",
                    param_name
                ))
                .build()),
        },
        None => Err(HttpResponseBuilder::bad_request()
            .with_body_and_content_length(format!("must provide the '{}' parameter", param_name))
            .build()),
    }
}

#[pre_upgrade]
fn pre_upgrade() {
    take_state(|state| ic_cdk::storage::stable_save((state,)).expect("failed to save state"))
}

#[post_upgrade]
fn post_upgrade() {
    let (stable_state,): (RouteState,) =
        ic_cdk::storage::stable_restore().expect("failed to restore state");

    replace_state(stable_state);
}

fn main() {}

// Enable Candid export
ic_cdk::export_candid!();
