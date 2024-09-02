use crate::*;
use business::{process_directive::process_directive_task, ticket_task::process_ticket_task};
use cosmrs::tendermint;
use cosmwasm::{
    client::{query_cw_public_key, OSMO_ACCOUNT_PREFIX},
    port::{ExecuteMsg, PortContractExecutor},
    TxHash,
};
use ic_canisters_http_types::{HttpRequest, HttpResponse};
use ic_cdk::{
    api::management_canister::http_request::{TransformArgs, TransformContext},
    init, post_upgrade, query, update,
};
use ic_cdk_timers::set_timer_interval;
use lifecycle::init::InitArgs;
use memory::{init_stable_log, insert_redeem_ticket, mutate_state, read_state};
use omnity_types::{
    log::{init_log, StableLogWriter},
    Directive,
};
use std::time::Duration;

#[init]
pub async fn init(args: InitArgs) {
    lifecycle::init::init(args);

    init_log(Some(init_stable_log()));
}

#[update]
pub async fn cache_public_key_and_start_timer() {
    let public_key_response = query_cw_public_key()
        .await
        .expect("failed to query cw public key");

    mutate_state(|state| {
        state.cw_public_key_vec = Some(public_key_response.public_key.clone());
    });

    set_timer_interval(
        Duration::from_secs(const_args::INTERVAL_QUERY_DIRECTIVE),
        process_directive_task,
    );
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

    hub::send_ticket(hub_principal, ticket.clone())
        .await
        .map_err(|e| e.to_string())?;

    insert_redeem_ticket(tx_hash, ticket.ticket_id.clone());

    Ok(ticket.ticket_id)
}

// fn check_anonymous_caller() {
//     if ic_cdk::caller() == Principal::anonymous() {
//         panic!("anonymous caller not allowed")
//     }
// }

#[update]
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

#[query]
pub fn route_status() -> RouteState {
    read_state(|s| s.clone())
}

#[update]
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

#[update]
async fn test_execute_directive(
    seq: String,
    d: Directive,
) -> std::result::Result<TxHash, String> {
    let _seq: u64 = seq.to_string().parse().unwrap() ;
    let msg = ExecuteMsg::ExecDirective {
        seq: _seq,
        directive: d.into(),
    };

    let client = CosmWasmClient::cosmos_wasm_port_client();

    let contract_id = get_contract_id();

    let public_key_response = query_cw_public_key().await.map_err(|e| e.to_string())?;

    let tendermint_public_key: tendermint::PublicKey =
        tendermint::public_key::PublicKey::from_raw_secp256k1(
            public_key_response.public_key.as_slice(),
        )
        .unwrap();

    let tx_hash = client
        .execute_msg(contract_id, msg, tendermint_public_key)
        .await.map_err(|e| e.to_string());
    tx_hash

}

#[update]
async fn test_http_outcall(
    url: String,
) -> std::result::Result<ic_cdk::api::management_canister::http_request::HttpResponse, String> {
    let request_headers = vec![HttpHeader {
        name: "content-type".to_string(),
        value: "application/json".to_string(),
    }];

    let request = CanisterHttpRequestArgument {
        url: url,
        max_response_bytes: None,
        method: HttpMethod::GET,
        headers: request_headers,
        body: None,
        transform: Some(TransformContext::from_name(
            "cleanup_response".to_owned(),
            vec![],
        )),
    };

    http_request_with_status_check(request)
        .await
        .map_err(|e| e.to_string())
}

#[post_upgrade]
fn post_upgrade() {
    init_log(Some(init_stable_log()));

    lifecycle::upgrade::post_upgrade();
    mutate_state(|state| {
        state.next_directive_seq = 0;
    });

    set_timer_interval(
        Duration::from_secs(const_args::INTERVAL_QUERY_DIRECTIVE),
        process_directive_task,
    );
    set_timer_interval(
        Duration::from_secs(const_args::INTERVAL_QUERY_TICKET),
        process_ticket_task,
    );
    log::info!("Finish Upgrade current version: {}", const_args::VERSION);
}

ic_cdk::export_candid!();