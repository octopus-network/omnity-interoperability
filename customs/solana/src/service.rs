use crate::{
    address::{fee_address_path, main_address_path},
    lifecycle::{self, init::CustomArg, upgrade::UpgradeArgs},
    process_directive_msg_task, process_release_token_task, process_ticket_msg_task, solana_rpc,
    state::{mutate_state, read_state, CollectionTx, GenTicketStatus, ReleaseTokenStatus},
    types::omnity_types::{Chain, Token},
    updates::{
        self,
        generate_ticket::{GenerateTicketArgs, GenerateTicketError},
        get_sol_address::GetSolAddressArgs,
        submit_release_token_tx,
    },
    INTERVAL_PROCESSING,
};
use ic_canister_log::log;
use ic_canisters_http_types::{HttpRequest, HttpResponse};
use ic_cdk::{init, post_upgrade, pre_upgrade, query, update};
use ic_cdk_timers::set_timer_interval;
use ic_solana::ic_log::{self, INFO};

pub fn is_controller() -> Result<(), String> {
    if ic_cdk::api::is_controller(&ic_cdk::caller()) {
        Ok(())
    } else {
        Err("caller is not controller".to_string())
    }
}

fn schedule_tasks() {
    set_timer_interval(INTERVAL_PROCESSING, process_release_token_task);
    set_timer_interval(INTERVAL_PROCESSING, process_ticket_msg_task);
    set_timer_interval(INTERVAL_PROCESSING, process_directive_msg_task);
}

#[init]
fn init(args: CustomArg) {
    match args {
        CustomArg::Init(args) => {
            lifecycle::init(args);
            schedule_tasks();
        }
        CustomArg::Upgrade(_) => {
            panic!("expected InitArgs got UpgradeArgs");
        }
    }
}

#[pre_upgrade]
fn pre_upgrade() {
    log!(INFO, "begin to execute pre_upgrade...");
    lifecycle::pre_upgrade();
    log!(INFO, "pre_upgrade end!");
}

#[post_upgrade]
fn post_upgrade(args: Option<CustomArg>) {
    log!(INFO, "begin to execute post_upgrade with :{:?}", args);
    let mut upgrade_arg: Option<UpgradeArgs> = None;
    if let Some(custom_arg) = args {
        upgrade_arg = match custom_arg {
            CustomArg::Upgrade(upgrade_args) => upgrade_args,
            CustomArg::Init(_) => panic!("expected Option<UpgradeArgs> got InitArgs."),
        };
    }

    lifecycle::post_upgrade(upgrade_arg);
    schedule_tasks();
    log!(INFO, "upgrade successfully!");
}

#[update]
async fn get_sol_address(args: GetSolAddressArgs) -> String {
    updates::get_sol_address(args).await.to_string()
}

#[update]
pub async fn get_fee_address() -> String {
    let pk = solana_rpc::ecdsa_public_key(fee_address_path()).await;
    pk.to_string()
}

#[update]
pub async fn get_main_address() -> String {
    let pk = solana_rpc::ecdsa_public_key(main_address_path()).await;
    pk.to_string()
}

#[update]
async fn generate_ticket(args: GenerateTicketArgs) -> Result<(), GenerateTicketError> {
    updates::generate_ticket(args).await
}

#[query]
fn generate_ticket_status(ticket_id: String) -> GenTicketStatus {
    if let Some(args) = read_state(|s| s.finalized_gen_tickets.get(&ticket_id)) {
        GenTicketStatus::Finalized(args)
    } else {
        GenTicketStatus::Unknown
    }
}

#[query]
fn release_token_status(ticket_id: String) -> ReleaseTokenStatus {
    if let Some(_) = read_state(|s| s.finalized_requests.get(&ticket_id)) {
        return ReleaseTokenStatus::Finalized;
    }
    read_state(|s| {
        s.release_token_requests
            .get(&ticket_id)
            .map_or(ReleaseTokenStatus::Unknown, |r| r.status.clone())
    })
}

#[query(guard = "is_controller")]
fn submitted_collection_txs() -> Vec<CollectionTx> {
    read_state(|s| s.submitted_collection_txs.values().cloned().collect())
}

#[update(guard = "is_controller")]
async fn resubmit_release_token_tx(ticket_id: String) -> Result<(), String> {
    match read_state(|s| s.release_token_requests.get(&ticket_id).cloned()) {
        None => Err("can't find failed request".into()),
        Some(mut req) => {
            submit_release_token_tx(&mut req).await;
            Ok(())
        }
    }
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
    read_state(|s| s.tokens.iter().map(|(_, token)| token.clone()).collect())
}

#[update(guard = "is_controller", hidden = true)]
pub async fn update_forward(forward: Option<String>) {
    mutate_state(|s| s.forward = forward)
}

#[update(guard = "is_controller", hidden = true)]
pub fn debug(enable: bool) {
    mutate_state(|s| s.enable_debug = enable);
}

#[query(hidden = true)]
fn http_request(req: HttpRequest) -> HttpResponse {
    if ic_cdk::api::data_certificate().is_none() {
        ic_cdk::trap("update call rejected");
    }
    let endable_debug = read_state(|s| s.enable_debug);
    ic_log::http_log(req, endable_debug)
}

// Enable Candid export
ic_cdk::export_candid!();
