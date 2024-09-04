use crate::business::{process_directive::process_directive_task, ticket_task::process_ticket_task};
use cosmrs::tendermint;
use crate::cosmwasm::{
    client::{query_cw_public_key, OSMO_ACCOUNT_PREFIX},
    port::PortContractExecutor,
    TxHash,
};
use ic_canisters_http_types::{HttpRequest, HttpResponse};
use ic_cdk::{
    api::management_canister::http_request::TransformArgs, init, post_upgrade, query, update,
};
use ic_cdk_timers::set_timer_interval;
use lifecycle::init::InitArgs;
use crate::memory::{init_stable_log, insert_redeem_ticket, mutate_state, read_state};
use omnity_types::{
    log::{init_log, StableLogWriter},
    Ticket, TicketId,
};
use std::time::Duration;

use crate::{
    const_args, hub, lifecycle, RouteState, UpdateCwSettingsArgs
};

#[init]
pub async fn init(args: InitArgs) {
    lifecycle::init::init(args);

    init_log(Some(init_stable_log()));
}

pub fn is_controller() -> std::result::Result<(), String> {
    if ic_cdk::api::is_controller(&ic_cdk::caller()) {
        Ok(())
    } else {
        Err("caller is not controller".to_string())
    }
}

#[update(guard = "is_controller")]
pub async fn cache_public_key() {
    let public_key_response = query_cw_public_key()
        .await
        .expect("failed to query cw public key");

    mutate_state(|state| {
        state.cw_public_key_vec = Some(public_key_response.public_key.clone());
    });
}

#[update(guard = "is_controller")]
pub async fn start_process_directive_task() {
    set_timer_interval(
        Duration::from_secs(const_args::INTERVAL_QUERY_DIRECTIVE),
        process_directive_task,
    );
}

#[update(guard = "is_controller")]
pub async fn start_process_ticket_task() {
    set_timer_interval(
        Duration::from_secs(const_args::INTERVAL_QUERY_TICKET),
        process_ticket_task,
    );
}

#[update]
pub async fn redeem(tx_hash: TxHash) -> std::result::Result<TicketId, String> {
    let port_contract_executor = PortContractExecutor::from_state().map_err(|e| e.to_string())?;
    let event = port_contract_executor
        .query_redeem_token_event(tx_hash.clone())
        .await
        .map_err(|e| e.to_string())?;

    let (hub_principal, chain_id) = read_state(|s| (s.hub_principal, s.chain_id.clone()));
    let ticket = Ticket {
        ticket_id: tx_hash.clone(),
        ticket_type: omnity_types::TicketType::Normal,
        ticket_time: ic_cdk::api::time(),
        src_chain: chain_id,
        dst_chain: event.target_chain.clone(),
        action: omnity_types::TxAction::Redeem,
        token: event.token_id.clone(),
        amount: event.amount.to_string(),
        sender: Some(event.sender),
        receiver: event.receiver,
        memo: None,
    };

    log::info!(
        "try to send redeem ticket: {:?}, tx_hash: {:?}",
        ticket,
        tx_hash
    );

    hub::send_ticket(hub_principal, ticket.clone())
        .await
        .map_err(|e| e.to_string())?;
    log::info!(
        "send redeem ticket success: {:?}, tx_hash: {:?}",
        ticket,
        tx_hash
    );

    insert_redeem_ticket(tx_hash, ticket.ticket_id.clone());

    Ok(ticket.ticket_id)
}

// fn check_anonymous_caller() {
//     if ic_cdk::caller() == Principal::anonymous() {
//         panic!("anonymous caller not allowed")
//     }
// }

#[update(guard = "is_controller")]
pub async fn osmosis_account_id() -> std::result::Result<String, String> {
    let public_key_response = query_cw_public_key()
        .await
        .expect("failed to query cw public key");

    let tendermint_public_key = tendermint::public_key::PublicKey::from_raw_secp256k1(
        public_key_response.public_key.as_slice(),
    )
    .expect("failed to init tendermint public key");

    let sender_public_key = cosmrs::crypto::PublicKey::from(tendermint_public_key);
    let sender_account_id = sender_public_key.account_id(OSMO_ACCOUNT_PREFIX).unwrap();
    Ok(sender_account_id.to_string())
}

#[query(guard = "is_controller")]
pub fn route_state() -> RouteState {
    read_state(|s| s.clone())
}

#[update(guard = "is_controller")]
pub fn update_cw_settings(args: UpdateCwSettingsArgs) {
    mutate_state(|state| {
        if let Some(cw_rpc_url) = args.cw_rpc_url {
            state.cw_rpc_url = cw_rpc_url;
        }

        if let Some(cw_rest_url) = args.cw_rest_url {
            state.cw_rest_url = cw_rest_url;
        }

        if let Some(cw_port_contract_address) = args.cw_port_contract_address {
            state.cw_port_contract_address = cw_port_contract_address;
        }
    });
}

#[query(hidden = true)]
fn http_request(req: HttpRequest) -> HttpResponse {
    StableLogWriter::http_request(req)
}

#[query(hidden = true)]
fn cleanup_response(
    mut args: TransformArgs,
) -> ic_cdk::api::management_canister::http_request::HttpResponse {
    // The response header contains non-deterministic fields that make it impossible to reach consensus!
    // Errors seem deterministic and do not contain data that can break consensus.
    // Clear non-deterministic fields from the response headers.

    args.response.headers.clear();
    args.response
}

#[post_upgrade]
fn post_upgrade() {
    init_log(Some(init_stable_log()));

    lifecycle::upgrade::post_upgrade();

    set_timer_interval(
        Duration::from_secs(const_args::INTERVAL_QUERY_DIRECTIVE),
        process_directive_task,
    );
    set_timer_interval(
        Duration::from_secs(const_args::INTERVAL_QUERY_TICKET),
        process_ticket_task,
    );
    log::info!("Finish Upgrade current version: {}", env!("CARGO_PKG_VERSION"));
}

ic_cdk::export_candid!();
