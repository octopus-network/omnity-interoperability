use candid::Principal;
use ic_canisters_http_types::{HttpRequest, HttpResponse};
use ic_cdk::api::management_canister::main::{
    canister_info, update_settings, CanisterInfoRequest, CanisterSettings, UpdateSettingsArgument,
};
use ic_cdk::{caller, post_upgrade, pre_upgrade};
use ic_cdk_macros::{init, query, update};
use ic_cdk_timers::set_timer_interval;
use ic_ledger_types::AccountIdentifier;
use ic_log::writer::Logs;
use icp_route::lifecycle::{self, init::RouteArg, upgrade::UpgradeArgs};
use icp_route::memory::init_stable_log;
use icp_route::state::eventlog::{Event, GetEventsArg};
use icp_route::state::{mutate_state, read_state, take_state, MintTokenStatus};
use icp_route::updates::generate_ticket::{
    principal_to_subaccount, GenerateTicketError, GenerateTicketOk, GenerateTicketReq,
};
use icp_route::updates::{self};
use icp_route::{hub, periodic_task, storage, TokenResp, ICP_TRANSFER_FEE, PERIODIC_TASK_INTERVAL};
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
pub async fn resend_tickets() -> Result<(), GenerateTicketError> {
    let tickets_sz = read_state(|s| s.failed_tickets.len());
    while !read_state(|s| s.failed_tickets.is_empty()) {
        let ticket = mutate_state(|rs| rs.failed_tickets.pop()).unwrap();

        let hub_principal = read_state(|s| (s.hub_principal));
        if let Err(err) = hub::send_ticket(hub_principal, ticket.clone())
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
