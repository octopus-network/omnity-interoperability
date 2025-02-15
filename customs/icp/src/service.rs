use std::time::Duration;

use crate::lifecycle::init::InitArgs;
use crate::state::{get_finalized_mint_token_request, read_state, CustomsState};
use crate::updates::generate_ticket::{GenerateTicketError, GenerateTicketOk, GenerateTicketReq};
use crate::{hub, lifecycle, periodic_task, updates, PERIODIC_TASK_INTERVAL};
use candid::Principal;
use ic_canisters_http_types::{HttpRequest, HttpResponse};
use ic_cdk_macros::{init, post_upgrade, query, update};
use ic_cdk_timers::set_timer_interval;
use ic_ledger_types::{AccountIdentifier, Subaccount};
use omnity_types::MintTokenStatus::{Finalized, Unknown};
use omnity_types::{Chain, MintTokenStatus, Seq, Ticket, TicketId, Token};
use ic_canister_log::log;
use omnity_types::ic_log::INFO;

pub fn is_controller() -> Result<(), String> {
    if ic_cdk::api::is_controller(&ic_cdk::caller()) {
        Ok(())
    } else {
        Err("caller is not controller".to_string())
    }
}

#[init]
fn init(args: InitArgs) {
    lifecycle::init(args);
    set_timer_interval(Duration::from_secs(PERIODIC_TASK_INTERVAL), periodic_task);
}

#[post_upgrade]
fn post_upgrade() {
    set_timer_interval(Duration::from_secs(PERIODIC_TASK_INTERVAL), periodic_task);

    log!(
        INFO,
        "Finish Upgrade current version: {}",
        env!("CARGO_PKG_VERSION")
    );
}

fn check_anonymous_caller() {
    if ic_cdk::caller() == Principal::anonymous() {
        panic!("anonymous caller not allowed")
    }
}

#[update]
async fn generate_ticket_v2(args: GenerateTicketReq) -> Result<GenerateTicketOk, GenerateTicketError> {
    check_anonymous_caller();
    updates::generate_ticket_v2(args).await
}

#[update]
async fn generate_ticket(args: GenerateTicketReq) -> Result<GenerateTicketOk, GenerateTicketError> {
    check_anonymous_caller();
    updates::generate_ticket(args).await
}

#[update(guard = "is_controller")]
async fn refund_icp(principal: Principal)->Result<(ic_ledger_types::BlockIndex, u64), String> {
    updates::generate_ticket::refund_icp_from_subaccount(principal).await
}

#[query]
fn get_account_identifier(principal: Principal) -> AccountIdentifier {
    let subaccount = Subaccount::from(principal);
    AccountIdentifier::new(&ic_cdk::api::id(), &subaccount)
}

#[query]
fn get_account_identifier_text(principal: Principal) -> String {
    let subaccount = Subaccount::from(principal);
    AccountIdentifier::new(&ic_cdk::api::id(), &subaccount).to_hex()
}

#[query]
fn get_chain_list() -> Vec<Chain> {
    crate::state::get_chain_list()
}

#[query]
fn get_token_list() -> Vec<Token> {
    crate::state::get_token_list()
}

#[update(guard = "is_controller")]
fn set_icp_token(token_id: String) {
    crate::state::mutate_state(|state| {
        state.icp_token_id = Some(token_id);
    });
}

#[update(guard = "is_controller")]
fn set_ckbtc_token(token_id: String) {
    crate::state::mutate_state(|state| {
        state.ckbtc_token_id = Some(token_id);
    });
}

#[query(hidden = true)]
fn http_request(req: HttpRequest) -> HttpResponse {
    if ic_cdk::api::data_certificate().is_none() {
        ic_cdk::trap("update call rejected");
    }
    omnity_types::ic_log::http_request(req)
}

#[update(guard = "is_controller")]
pub async fn query_hub_tickets(from: u64, limit: u64) -> Vec<(Seq, Ticket)> {
    let hub_principal = read_state(|s| s.hub_principal);
    hub::query_tickets(hub_principal, from, limit)
        .await
        .unwrap()
}

#[update(guard = "is_controller")]
pub async fn handle_ticket(seq: u64)->Result<u64, String> {
    let hub_principal = read_state(|s| s.hub_principal);
    let r = hub::query_tickets(hub_principal, seq, 1)
        .await
        .unwrap()
        .first()
        .unwrap()
        .clone();
    match super::handle_redeem_ticket(&r.1).await {
        Ok(block_index) => {
            log::info!(
                "[handle_ticket] process successful for ticket: {}, block_index: {}",
                r.1,
                block_index
            );

            Ok(block_index)
        },
        Err(e) => {
            log::error!("[handle_ticket] failed to process ticket: {}, error: {}", r.1, e);
            Err(e)
        },
    }
}

#[query(guard = "is_controller")]
pub fn get_state() -> CustomsState {
    read_state(|s| s.clone())
}

#[query]
fn mint_token_status(ticket_id: TicketId) -> MintTokenStatus {
    match get_finalized_mint_token_request(&ticket_id) {
        None => Unknown,
        Some(i) => Finalized {
            tx_hash: i.to_string(),
        },
    }
}

// Enable Candid export
ic_cdk::export_candid!();

