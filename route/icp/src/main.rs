use candid::Principal;
use ic_canisters_http_types::{HttpRequest, HttpResponse};
use ic_cdk::api::call::call;
use ic_cdk::api::management_canister::main::{
    canister_info, update_settings, CanisterIdRecord, CanisterInfoRequest, CanisterSettings,
    CanisterStatusResponse, UpdateSettingsArgument,
};
use ic_cdk::{caller, post_upgrade, pre_upgrade};
use ic_cdk_macros::{init, query, update};
use ic_cdk_timers::set_timer_interval;
use ic_ledger_types::AccountIdentifier;
use ic_log::writer::Logs;
use icp_route::lifecycle::{self, init::RouteArg, upgrade::UpgradeArgs};
use icp_route::memory::init_stable_log;
use icp_route::state::eventlog::{Event, GetEventsArg};
use icp_route::state::{read_state, take_state, MintTokenStatus};
use icp_route::updates::add_new_token::upgrade_icrc2_ledger;
use icp_route::updates::generate_ticket::{
    principal_to_subaccount, GenerateTicketError, GenerateTicketOk, GenerateTicketReq,
};
use icp_route::updates::{self};
use icp_route::{periodic_task, storage, TokenResp, ICP_TRANSFER_FEE, PERIODIC_TASK_INTERVAL};
use omnity_types::log::{init_log, StableLogWriter};
use omnity_types::{Chain, ChainId};
use std::time::Duration;

#[init]
fn init(args: RouteArg) {
    match args {
        RouteArg::Init(args) => {
            init_log(Some(init_stable_log()));
            storage::record_event(&Event::Init(args.clone()));
            lifecycle::init(args);
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

pub fn is_controller() -> Result<(), String> {
    if ic_cdk::api::is_controller(&ic_cdk::caller()) {
        Ok(())
    } else {
        Err("caller is not controller".to_string())
    }
}

#[update(guard = "is_controller")]
async fn stop_controlled_canister(icrc_canister_id: Principal) -> Result<(), String> {
    let args = CanisterIdRecord {
        canister_id: icrc_canister_id,
    };

    call(Principal::management_canister(), "stop_canister", (args,))
        .await
        .and_then(|((),)| Ok(()))
        .map_err(|(_, reason)| reason)
}

#[update(guard = "is_controller")]
async fn start_controlled_canister(icrc_canister_id: Principal) -> Result<(), String> {
    let args = CanisterIdRecord {
        canister_id: icrc_canister_id,
    };

    call(Principal::management_canister(), "start_canister", (args,))
        .await
        .and_then(|((),)| Ok(()))
        .map_err(|(_, reason)| reason)
}

#[update(guard = "is_controller")]
async fn delete_controlled_canister(icrc_canister_id: Principal) -> Result<(), String> {
    let args = CanisterIdRecord {
        canister_id: icrc_canister_id,
    };
    call(Principal::management_canister(), "delete_canister", (args,))
        .await
        .and_then(|((),)| Ok(()))
        .map_err(|(_, reason)| reason)
}

#[update(guard = "is_controller")]
pub async fn controlled_canister_status(
    icrc_canister_id: Principal,
) -> Result<CanisterStatusResponse, String> {
    let args = CanisterIdRecord {
        canister_id: icrc_canister_id,
    };
    call(Principal::management_canister(), "canister_status", (args,))
        .await
        .and_then(|(e,)| Ok(e))
        .map_err(|(_, reason)| reason)
}

#[update(guard = "is_controller")]
pub async fn add_controller(canister_id: Principal, controller: Principal) -> Result<(), String> {
    let args = CanisterInfoRequest {
        canister_id,
        num_requested_changes: None,
    };

    let canister_info = canister_info(args).await.map_err(|(_, reason)| reason)?;

    let mut controllers = canister_info.0.controllers;

    if !controllers.contains(&controller) {
        controllers.push(controller);
        let args = UpdateSettingsArgument {
            canister_id,
            settings: CanisterSettings {
                controllers: Some(controllers),
                compute_allocation: None,
                memory_allocation: None,
                freezing_threshold: None,
                reserved_cycles_limit: None,
            },
        };
        return update_settings(args).await.map_err(|(_, reason)| reason);
    } else {
        Ok(())
    }
}

#[update(guard = "is_controller")]
pub async fn remove_controller(
    canister_id: Principal,
    controller: Principal,
) -> Result<(), String> {
    let args = CanisterInfoRequest {
        canister_id,
        num_requested_changes: None,
    };

    let canister_info = canister_info(args).await.map_err(|(_, reason)| reason)?;

    let controllers = canister_info.0.controllers;

    if controllers.contains(&controller) {
        let args = UpdateSettingsArgument {
            canister_id,
            settings: CanisterSettings {
                controllers: Some(
                    controllers
                        .into_iter()
                        .filter(|c| c.ne(&controller))
                        .collect(),
                ),
                compute_allocation: None,
                memory_allocation: None,
                freezing_threshold: None,
                reserved_cycles_limit: None,
            },
        };
        return update_settings(args).await.map_err(|(_, reason)| reason);
    } else {
        Ok(())
    }
}

#[update(guard = "is_controller")]
pub async fn update_icrc_ledger(
    ledger_id: Principal,
    upgrade_args: ic_icrc1_ledger::UpgradeArgs,
) -> Result<(), String> {
    if !read_state(|s| s.token_ledgers.iter().any(|(_, id)| *id == ledger_id)) {
        return Err("leder id not found!".into());
    }

    upgrade_icrc2_ledger(ledger_id, upgrade_args).await
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
    StableLogWriter::http_request(req)
}

#[pre_upgrade]
fn pre_upgrade() {
    take_state(|state| ic_cdk::storage::stable_save((state,)).expect("failed to save state"))
}

#[post_upgrade]
fn post_upgrade(route_arg: Option<RouteArg>) {
    let mut upgrade_arg: Option<UpgradeArgs> = None;
    if let Some(route_arg) = route_arg {
        upgrade_arg = match route_arg {
            RouteArg::Upgrade(upgrade_args) => upgrade_args,
            RouteArg::Init(_) => panic!("expected Option<UpgradeArgs> got InitArgs."),
        };
    }
    lifecycle::post_upgrade(upgrade_arg);

    set_timer_interval(Duration::from_secs(PERIODIC_TASK_INTERVAL), periodic_task);
}

fn main() {}

// Enable Candid export
ic_cdk::export_candid!();
