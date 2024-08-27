use std::time::Duration;

use candid::Principal;
use ic_cdk_macros::{init, query, update};
use ic_cdk_timers::set_timer_interval;
use icp_customs::lifecycle::init::InitArgs;
use icp_customs::updates::generate_ticket::{
    GenerateTicketError, GenerateTicketOk, GenerateTicketReq,
};
use ic_ledger_types::{AccountIdentifier, Subaccount};
use icp_customs::lifecycle::init::CustomArg;
use icp_customs::{lifecycle, periodic_task, updates, PERIODIC_TASK_INTERVAL};
use omnity_types::{Chain, Token};

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
fn get_account_identifier(principal: Principal) -> AccountIdentifier {
    let subaccount = Subaccount::from(principal);
    AccountIdentifier::new(&ic_cdk::api::id(), &subaccount)
}

#[query]
fn get_chain_list() -> Vec<Chain> {
    icp_customs::state::get_chain_list()
}

#[query]
fn get_token_list() -> Vec<Token> {
    icp_customs::state::get_token_list()
}

#[update(guard = "is_controller")]
fn set_icp_token(token_id: String) {
    icp_customs::state::mutate_state(|state| {
        state.icp_token_id = Some(token_id);
    });
}

#[update(guard = "is_controller")]
fn set_ckbtc_token(token_id: String) {
    icp_customs::state::mutate_state(|state| {
        state.ckbtc_token_id = Some(token_id);
    });
}

fn main() {}

// Enable Candid export
ic_cdk::export_candid!();
