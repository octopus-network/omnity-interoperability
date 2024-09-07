use crate::memory::{get_redeem_tickets, init_stable_log, mutate_state, read_state, GUARD_RUNNING_TASK, PERIODIC_JOB_MANAGER_MAP };
use crate::periodic_jobs::{start_process_directive_job, start_process_ticket_job};
use crate::{
    business::redeem_token::redeem_token_and_send_ticket,
    cosmwasm::{
        client::{query_cw_public_key, OSMO_ACCOUNT_PREFIX},
        TxHash,
    },
};
use cosmrs::tendermint;
use ic_canisters_http_types::{HttpRequest, HttpResponse};
use ic_cdk::{
    api::management_canister::http_request::TransformArgs, init, post_upgrade, query, update,
};
use lifecycle::init::InitArgs;
use omnity_types::{
    log::{init_log, StableLogWriter},
    TicketId,
};
use std::collections::{HashMap, HashSet};

use crate::{lifecycle, RouteState, UpdateCwSettingsArgs};

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
pub async fn start_process_directive() {
    start_process_directive_job();
}

#[update(guard = "is_controller")]
pub async fn start_process_ticket() {
    start_process_ticket_job();
}

#[update]
pub async fn redeem(tx_hash: TxHash) -> std::result::Result<TicketId, String> {
    let _ = match crate::guard::LogicGuard::new(format!("redeem_{}", tx_hash)) {
        Some(guard) => guard,
        None => return Err("redeem task is running".to_string()),
    };

    match redeem_token_and_send_ticket(tx_hash.clone()).await {
        Ok(ticket_id) => {
            log::info!(
                "send redeem ticket success: {:?}, tx_hash: {:?}",
                ticket_id,
                tx_hash
            );

            Ok(ticket_id)
        }
        Err(error) => {
            log::error!(
                "send redeem ticket failed: {:?}, tx_hash: {:?}",
                error,
                tx_hash
            );
            Err(error.to_string())
        }
    }
}

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
pub fn route_state() -> (RouteState, String, HashSet<String>) {
    (
        read_state(|s| s.clone()),
        PERIODIC_JOB_MANAGER_MAP.with(|m| format!("{:?}", m.borrow()).to_string()),
        GUARD_RUNNING_TASK.with(|g| g.borrow().clone()),
    )
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

        if let Some(multi_rpc_config) = args.multi_rpc_config {
            state.multi_rpc_config = multi_rpc_config;
        }
        log::info!("update cw settings, new state: {:?}", state);
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

#[query(guard = "is_controller")]
fn query_redeemed_tickets() -> HashMap<TxHash, TicketId> {
    get_redeem_tickets()
}

#[post_upgrade]
fn post_upgrade() {
    init_log(Some(init_stable_log()));

    lifecycle::upgrade::post_upgrade();

    start_process_directive_job();
    start_process_ticket_job();
  
    log::info!(
        "Finish Upgrade current version: {}",
        env!("CARGO_PKG_VERSION")
    );
}

ic_cdk::export_candid!();
