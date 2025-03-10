use std::str::FromStr;
use std::time::Duration;

use candid::{CandidType, Principal};
use cketh_common::eth_rpc_client::providers::RpcApi;
use ic_canister_log::log;
use ic_canisters_http_types::{HttpRequest, HttpResponse};
use ic_cdk::{init, post_upgrade, pre_upgrade, query, update};
use ic_cdk::api::is_controller;
use ic_cdk::api::management_canister::http_request;
use ic_cdk::api::management_canister::http_request::TransformArgs;
use ic_cdk_timers::set_timer_interval;
use serde_derive::Deserialize;

use crate::{get_time_secs, hub};
use crate::const_args::{BATCH_QUERY_LIMIT, MONITOR_PRINCIPAL, PERIODIC_TASK_INTERVAL, SEND_EVM_TASK_NAME};
use crate::eth_common::{call_rpc_with_retry, EvmAddress, EvmTxType, get_balance};
use crate::evm_scan::{create_ticket_by_tx, scan_evm_task};
use crate::hub_to_route::{process_directives, process_tickets};
use crate::ic_log::{CRITICAL, INFO, WARNING};
use crate::route_to_evm::{send_directive, send_directives_to_evm, send_ticket, send_tickets_to_evm};
use crate::state::{
    EvmRouteState, get_redeem_fee, init_chain_pubkey, minter_addr, mutate_state, read_state,
    replace_state, StateProfile,
};
use crate::types::{Chain, ChainId, Directive, MetricsStatus, MintTokenStatus, Network, PendingDirectiveStatus, PendingTicketStatus, Seq, Ticket, TicketId, TokenResp};

#[init]
fn init(args: InitArgs) {
    replace_state(EvmRouteState::init(args).expect("params error"));
    start_tasks();
}

#[pre_upgrade]
fn pre_upgrade() {
    read_state(|s| s.pre_upgrade());
}

#[post_upgrade]
fn post_upgrade() {
    EvmRouteState::post_upgrade();
    start_tasks();
    log!(INFO, "[evmroute] upgraded successed at {}", ic_cdk::api::time());
}

#[query]
fn http_request(req: HttpRequest) -> HttpResponse {
    if ic_cdk::api::data_certificate().is_none() {
        ic_cdk::trap("update call rejected");
    }
    crate::ic_log::http_request(req)
}
#[update(guard = "is_admin")]
fn update_consume_directive_seq(seq: Seq) {
    mutate_state(|s| s.next_consume_directive_seq = seq);
}


#[update(guard = "is_admin")]
fn set_finality_blocks(b: u64) {
    mutate_state(|s| s.finality_blocks = Some(b));
}

fn start_tasks() {
    set_timer_interval(Duration::from_secs(PERIODIC_TASK_INTERVAL), bridge_ticket_to_evm_task);
    set_timer_interval(Duration::from_secs(PERIODIC_TASK_INTERVAL), scan_evm_task);
}

pub fn bridge_ticket_to_evm_task() {
    ic_cdk::spawn(async {
        let _guard = match crate::guard::TimerLogicGuard::new(SEND_EVM_TASK_NAME.to_string()) {
            Some(guard) => guard,
            None => return,
        };
        process_directives().await;
        process_tickets().await;
        send_directives_to_evm().await;
        send_tickets_to_evm().await;
    });
}

#[query]
fn get_ticket(ticket_id: String) -> Option<(u64, Ticket)> {
    let r = read_state(|s| {
        s.tickets_queue
            .iter()
            .filter(|(_seq, t)| t.ticket_id == ticket_id)
            .collect::<Vec<_>>()
    });
    r.first().cloned()
}

#[update(guard = "is_admin")]
async fn pubkey_and_evm_addr() -> (String, String) {
    let mut key = read_state(|s| s.pubkey.clone());
    if key.is_empty() {
        init_chain_pubkey().await;
        key = read_state(|s| s.pubkey.clone());
    }
    let key_str = format!("0x{}", hex::encode(key.as_slice()));
    let addr = minter_addr();
    (key_str, addr)
}

#[update(guard = "is_admin")]
fn set_port_address(port_addr: String) {
    mutate_state(|s| s.omnity_port_contract = EvmAddress::from_str(port_addr.as_str()).unwrap())
}

#[query(guard = "is_admin")]
fn route_state() -> StateProfile {
    read_state(|s| StateProfile::from(s))
}

#[query(guard = "is_admin")]
fn query_pending_ticket(from: usize, limit: usize) -> Vec<(TicketId, PendingTicketStatus)> {
    read_state(|s| {
        s.pending_tickets_map
            .iter()
            .skip(from)
            .take(limit)
            .map(|kv| kv)
            .collect()
    })
}

#[query(guard = "is_admin")]
fn query_pending_directive(from: usize, limit: usize) -> Vec<(Seq, PendingDirectiveStatus)> {
    read_state(|s| {
        s.pending_directive_map
            .iter()
            .skip(from)
            .take(limit)
            .map(|kv| kv)
            .collect()
    })
}

#[update(guard = "is_admin")]
async fn resend_ticket(seq: Seq) {
    send_ticket(seq).await.unwrap();
}

#[update(guard = "is_admin")]
async fn resend_directive(seq: Seq) {
    send_directive(seq).await.unwrap();
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
fn get_token_list() -> Vec<TokenResp> {
    read_state(|s| {
        s.tokens
            .iter()
            .map(|(token_id, token)| {
                let mut resp: TokenResp = token.clone().into();
                resp.evm_contract = s.token_contracts.get(token_id).cloned();
                resp
            })
            .collect()
    })
}

#[query]
fn mint_token_status(ticket_id: String) -> MintTokenStatus {
    read_state(|s| {
        s.finalized_mint_token_requests
            .get(&ticket_id)
            .cloned()
            .map_or(MintTokenStatus::Unknown, |tx_hash| {
                MintTokenStatus::Finalized { tx_hash }
            })
    })
}

#[query]
fn get_fee(chain_id: ChainId) -> Option<u64> {
    get_redeem_fee(chain_id)
}

#[query(guard = "is_admin")]
fn query_tickets(from: usize, to: usize) -> Vec<(Seq, Ticket)> {
    read_state(|s| s.pull_tickets(from, to))
}

#[query(guard = "is_admin")]
fn query_directives(from: usize, to: usize) -> Vec<(Seq, Directive)> {
    read_state(|s| s.pull_directives(from, to))
}

#[update(guard = "is_admin")]
async fn sync_mint_status(hash: String) {
    crate::evm_scan::sync_mint_status(hash).await;
}
#[update(guard = "is_admin")]
fn update_admins(admins: Vec<Principal>) {
    mutate_state(|s| s.admins = admins);
}

#[update(guard = "is_admin")]
fn update_fee_token(fee_token: String) {
    mutate_state(|s| s.fee_token_id = fee_token);
}

#[update(guard = "is_admin")]
fn update_rpcs(rpcs: Vec<RpcApi>) {
    mutate_state(|s| s.rpc_providers = rpcs);
}

#[update(guard = "is_admin")]
fn update_rpc_check_rate(min_resp_count: usize, total_required_rpc_count: usize) {
    assert!(min_resp_count > 0 && total_required_rpc_count >= min_resp_count, "params errorr");
    let rpc_size = read_state(|s| s.rpc_providers.len());
    assert!(rpc_size >= total_required_rpc_count, "rpc count is not enough");
    mutate_state(|s| {
        s.minimum_response_count = min_resp_count;
        s.total_required_count = total_required_rpc_count;
    });
}

fn is_admin() -> Result<(), String> {
    let c = ic_cdk::caller();
    match ic_cdk::api::is_controller(&c) || read_state(|s| s.admins.contains(&c)) {
        true => Ok(()),
        false => Err("permission deny".to_string()),
    }
}

fn is_monitor() -> Result<(), String> {
    let c = ic_cdk::caller();
    if is_controller(&c) {
        return Ok(())
    }
    match c == Principal::from_text(MONITOR_PRINCIPAL).unwrap() {
        true => Ok(()),
        false => Err("permission deny".to_string()),
    }
}

#[update(guard = "is_monitor")]
async fn metrics() -> MetricsStatus {
    let chainkey_addr = minter_addr();
    let balance = call_rpc_with_retry(chainkey_addr, get_balance)
        .await
        .unwrap_or_default();
    MetricsStatus {
        latest_scan_interval_secs: 0,
        chainkey_addr_balance: balance.as_u128(),
    }
}

#[update]
async fn generate_ticket(hash: String) -> Result<(), String> {
    log!(INFO, "[generate ticket] received generate ticket request: {}", &hash);
    let tx_hash = hash.to_lowercase();
    if read_state(|s| s.pending_events_on_chain.get(&tx_hash).is_some()) {
        return Ok(());
    }
    assert!(tx_hash.starts_with("0x"));
    assert_eq!(
        hex::decode(tx_hash.strip_prefix("0x").unwrap())
            .unwrap()
            .len(),
        32
    );
    if read_state(|s| s.handled_evm_event.contains(&tx_hash)) {
        return Err("The ticket id already exists".to_string());
    }
    let (ticket, _transaction_receipt) = create_ticket_by_tx(&tx_hash).await?;
    let hub_principal = read_state(|s| s.hub_principal);
    hub::pending_ticket(hub_principal, ticket)
        .await
        .map_err(|e| {
            log!(CRITICAL, "call hub error:{}", e.to_string());
            "call hub error".to_string()
        })?;
    mutate_state(|s| s.pending_events_on_chain.insert(tx_hash, get_time_secs()));
    Ok(())
}

#[update(guard = "is_admin")]
pub fn insert_pending_hash(tx_hash: String) {
    mutate_state(|s| s.pending_events_on_chain.insert(tx_hash, get_time_secs()));
}


#[query]
fn transform(raw: TransformArgs) -> http_request::HttpResponse {
    http_request::HttpResponse {
        status: raw.response.status.clone(),
        body: raw.response.body.clone(),
        headers: vec![],
        ..Default::default()
    }
}

#[update(guard = "is_admin")]
pub async fn query_hub_tickets(start: u64) -> Vec<(Seq, Ticket)> {
    let hub_principal = read_state(|s| s.hub_principal);
    match hub::query_tickets(hub_principal, start, BATCH_QUERY_LIMIT).await {
        Ok(tickets) => return tickets,
        Err(err) => {
            log!(WARNING, "[process tickets] failed to query tickets, err: {}", err);
            return vec![];
        }
    }
}

#[update(guard = "is_admin")]
pub fn query_handled_event(tx_hash: String) -> Option<String> {
    read_state(|s| s.handled_evm_event.get(&tx_hash).cloned())
}

#[update(guard = "is_admin")]
pub async fn rewrite_tx_hash(ticket_id: String, tx_hash: String) {
    let hub_principal = read_state(|s| s.hub_principal);
    hub::update_tx_hash(hub_principal, ticket_id, tx_hash)
        .await
        .unwrap();
}

#[derive(CandidType, Deserialize)]
pub struct InitArgs {
    pub evm_chain_id: u64,
    pub admins: Vec<Principal>,
    pub hub_principal: Principal,
    pub network: Network,
    pub evm_rpc_canister_addr: Principal,
    pub chain_id: String,
    pub rpcs: Vec<RpcApi>,
    pub fee_token_id: String,
    pub port_addr: Option<String>,
    pub evm_tx_type: EvmTxType,
    pub block_interval_secs: u64,
}

ic_cdk::export_candid!();
