use std::time::Duration;

use candid::Principal;
use ic_cdk_macros::{init, query, update};
use ic_cdk_timers::set_timer_interval;
use icp_customs::lifecycle::init::CustomArg;
use icp_customs::{lifecycle, periodic_task, updates, PERIODIC_TASK_INTERVAL};
use icp_customs::{
    state::read_state,
    updates::generate_ticket::{GenerateTicketError, GenerateTicketOk, GenerateTicketReq},
};
use omnity_types::{Chain, Token};

#[init]
fn init(args: CustomArg) {
    match args {
        CustomArg::Init(args) => {
            lifecycle::init(args);
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
fn get_token_list() -> Vec<Token> {
    read_state(|s| {
        s.tokens
            .iter()
            .map(|(_, (token, _))| token.clone())
            .collect()
    })
}

fn main() {}

// Enable Candid export
ic_cdk::export_candid!();
