use candid::{Nat, Principal};
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
use icp_route::state::eventlog::{Event, GetEventsArg};
use icp_route::state::{mutate_state, read_state, take_state, MintTokenStatus};
use icp_route::updates::add_new_token::upgrade_icrc2_ledger;
use icp_route::updates::generate_ticket::{
    icp_get_redeem_fee, principal_to_subaccount, GenerateTicketError, GenerateTicketOk, GenerateTicketReq
};
use icp_route::updates::{self};
use icp_route::{
    hub, process_directive_msg_task, process_ticket_msg_task, storage, TokenResp,
    FEE_COLLECTOR_SUB_ACCOUNT, INTERVAL_QUERY_DIRECTIVE, INTERVAL_QUERY_TICKET,
};
use icrc_ledger_client_cdk::{CdkRuntime, ICRC1Client};
use icrc_ledger_types::icrc1::account::Account;
use icrc_ledger_types::icrc1::transfer::TransferArg;

pub use ic_canister_log::log;
pub use omnity_types::ic_log::{ERROR, INFO};
use omnity_types::{Chain, ChainId, Ticket};
use std::time::Duration;

#[init]
fn init(args: RouteArg) {
    match args {
        RouteArg::Init(args) => {
            storage::record_event(&Event::Init(args.clone()));
            lifecycle::init(args);
            set_timer_interval(
                Duration::from_secs(INTERVAL_QUERY_DIRECTIVE),
                process_directive_msg_task,
            );
            set_timer_interval(
                Duration::from_secs(INTERVAL_QUERY_TICKET),
                process_ticket_msg_task,
            );
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
    
    log!(INFO, "generate_ticket: {:?}", args);
    match updates::generate_ticket(args, false).await {
        Ok(r) => {
            log!(INFO, "generate_ticket success, result: {:?}", r);
            Ok(r)
        },
        Err(e) => {
            log!(ERROR, "generate_ticket failed, error: {:?}", e);
            Err(e)
        },
    }
}

#[update]
async fn generate_ticket_v2(args: GenerateTicketReq) -> Result<GenerateTicketOk, GenerateTicketError> {
    check_anonymous_caller();
    log!(INFO, "generate_ticket_v2: {:?}", args);
    match updates::generate_ticket(args, true).await {
        Ok(r) => {
            log!(INFO, "generate_ticket_v2 success, result: {:?}", r);
            Ok(r)
        },
        Err(e) => {
            log!(ERROR, "generate_ticket_v2 failed, error: {:?}", e);
            Err(e)
        },
    }
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
pub async fn update_icrc_ledger(
    ledger_id: Principal,
    upgrade_args: ic_icrc1_ledger::UpgradeArgs,
) -> Result<(), String> {
    if !read_state(|s| s.token_ledgers.iter().any(|(_, id)| *id == ledger_id)) {
        return Err("ledger id not found!".into());
    }
    upgrade_icrc2_ledger(ledger_id, upgrade_args).await
}

#[update(guard = "is_controller")]
pub async fn collect_ledger_fee(
    ledger_id: Principal,
    amount: Option<Nat>,
    receiver: Account,
) -> Result<(), String> {
    let client = ICRC1Client {
        runtime: CdkRuntime,
        ledger_canister_id: ledger_id,
    };

    let collector = Account {
        owner: ic_cdk::id(),
        subaccount: Some(FEE_COLLECTOR_SUB_ACCOUNT.clone()),
    };

    let transfer_amount = if amount.is_none() {
        let fee = client.fee().await.map_err(|(code, msg)| {
            format!(
                "failed to get fee from ledger: {} code: {} msg: {}",
                ledger_id, code, msg
            )
        })?;
        client.balance_of(collector).await.map_err(|(code, msg)| {
            format!(
                "failed to get balance of ledger: {} code: {} msg: {}",
                ledger_id, code, msg
            )
        })? - fee
    } else {
        amount.unwrap()
    };

    client
        .transfer(TransferArg {
            from_subaccount: Some(FEE_COLLECTOR_SUB_ACCOUNT.clone()),
            to: receiver,
            fee: None,
            created_at_time: Some(ic_cdk::api::time()),
            memo: None,
            amount: transfer_amount,
        })
        .await
        .map_err(|(code, msg)| {
            format!(
                "failed to transfer from ledger: {:?} code: {:?} msg: {:?}",
                ledger_id, code, msg
            )
        })?
        .map_err(|err| {
            format!(
                "failed to transfer from ledger: {:?} error: {:?}",
                ledger_id, err
            )
        })?;

    Ok(())
}

#[query(guard = "is_controller")]
pub fn query_failed_tickets() -> Vec<Ticket> {
    read_state(|s| s.failed_tickets.clone())
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
            log!(
                ERROR,
                "failed to resend ticket: {}, error: {:?}",
                ticket.ticket_id,
                err
            );
            return Err(err);
        }
    }
    log!(INFO, "successfully resend {} tickets", tickets_sz);
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
            .map(|(_, token)| {
                let mut token_resp = TokenResp::from(token.clone());
                token_resp.principal = s.token_ledgers.get(&token.token_id).clone().copied();
                token_resp
            })
            .collect()
    })
}

#[query]
fn get_token_ledger(token_id: String) -> Option<Principal> {
    read_state(|s| s.token_ledgers.get(&token_id).cloned())
}

#[query(hidden = true, guard = "is_controller")]
pub fn get_log_records(offset: usize, limit: usize) -> Logs {
    ic_log::take_memory_records(limit, offset)
}

#[query(hidden = true, guard = "is_controller")]
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
pub fn get_readable_fee_account(principal: Option<Principal>) -> String {
    let principal = principal.unwrap_or(caller());
    AccountIdentifier::new(&ic_cdk::api::id(), &principal_to_subaccount(&principal)).to_hex()
}

#[query]
pub fn get_redeem_fee(chain_id: ChainId) -> Option<u64> {
    icp_get_redeem_fee(chain_id)
}

#[query(hidden = true, guard = "is_controller")]
pub fn get_route_state() -> icp_route::state::RouteState {
    read_state(|s| s.clone())
}

#[query(hidden = true)]
fn http_request(req: HttpRequest) -> HttpResponse {
    if ic_cdk::api::data_certificate().is_none() {
        ic_cdk::trap("update call rejected");
    }
    omnity_types::ic_log::http_request(req)
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
    set_timer_interval(
        Duration::from_secs(INTERVAL_QUERY_DIRECTIVE),
        process_directive_msg_task,
    );
    set_timer_interval(
        Duration::from_secs(INTERVAL_QUERY_TICKET),
        process_ticket_msg_task,
    );
    log!(
        INFO,
        "Finish Upgrade current version: {}",
        env!("CARGO_PKG_VERSION")
    );
}

fn main() {}

// Enable Candid export
ic_cdk::export_candid!();
