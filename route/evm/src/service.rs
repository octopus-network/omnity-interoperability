use std::str::FromStr;
use std::time::Duration;

use candid::{CandidType, Principal};
use cketh_common::eth_rpc_client::providers::RpcApi;
use evm_rpc_types::TransactionReceipt;
use ic_canisters_http_types::{HttpRequest, HttpResponse};
use ic_cdk::{init, post_upgrade, pre_upgrade, query, update};
use ic_cdk_timers::set_timer_interval;
use log::{error, info};
use serde_derive::Deserialize;

use crate::{Error, get_time_secs, hub};
use crate::const_args::{BATCH_QUERY_LIMIT, FETCH_HUB_DIRECTIVE_INTERVAL, FETCH_HUB_TICKET_INTERVAL, MONITOR_PRINCIPAL, SCAN_EVM_TASK_INTERVAL, SEND_EVM_TASK_INTERVAL};
use crate::eth_common::{EvmAddress, EvmTxType, get_balance};
use crate::evm_scan::{create_ticket_by_tx, get_transaction_receipt, scan_evm_task};
use crate::hub_to_route::{fetch_hub_directive_task, fetch_hub_ticket_task};
use crate::route_to_evm::{send_directive, send_ticket, to_evm_task};
use crate::stable_log::{init_log, StableLogWriter};
use crate::stable_memory::init_stable_log;
use crate::state::{
    EvmRouteState, init_chain_pubkey, minter_addr, mutate_state, read_state, replace_state,
    StateProfile,
};
use crate::types::{Chain, ChainId, Directive, LocalLogEntry, MetricsStatus, MintTokenStatus, Network, PendingDirectiveStatus, PendingTicketStatus, Seq, Ticket, TicketId, TokenResp};

#[init]
fn init(args: InitArgs) {
    replace_state(EvmRouteState::init(args).expect("params error"));
    init_log(Some(init_stable_log()));
    start_tasks();
}

#[pre_upgrade]
fn pre_upgrade() {
    read_state(|s| s.pre_upgrade());
}

#[post_upgrade]
fn post_upgrade() {
    EvmRouteState::post_upgrade();
    init_log(Some(init_stable_log()));
    start_tasks();
    info!("[evmroute] upgraded successed at {}", ic_cdk::api::time());
}

#[query]
fn http_request(req: HttpRequest) -> HttpResponse {
    StableLogWriter::http_request(req)
}

#[update(guard = "is_admin")]
fn update_consume_directive_seq(seq: Seq) {
    mutate_state(|s| s.next_consume_directive_seq = seq);
}

fn start_tasks() {
    set_timer_interval(
        Duration::from_secs(FETCH_HUB_TICKET_INTERVAL),
        fetch_hub_ticket_task,
    );
    set_timer_interval(
        Duration::from_secs(FETCH_HUB_DIRECTIVE_INTERVAL),
        fetch_hub_directive_task,
    );
    set_timer_interval(Duration::from_secs(SEND_EVM_TASK_INTERVAL), to_evm_task);
    set_timer_interval(Duration::from_secs(SCAN_EVM_TASK_INTERVAL), scan_evm_task);
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
    read_state(|s| {
        s.target_chain_factor
            .get(&chain_id)
            .map_or(None, |target_chain_factor| {
                s.fee_token_factor
                    .map(|fee_token_factor| (target_chain_factor * fee_token_factor) as u64)
            })
    })
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

fn is_admin() -> Result<(), String> {
    let c = ic_cdk::caller();
    match ic_cdk::api::is_controller(&c) || read_state(|s| s.admins.contains(&c)) {
        true => Ok(()),
        false => Err("permission deny".to_string()),
    }
}

fn is_monitor() -> Result<(), String> {
    let c = ic_cdk::caller();
    match c == Principal::from_text(MONITOR_PRINCIPAL).unwrap() {
        true => Ok(()),
        false => Err("permission deny".to_string()),
    }
}

#[update(guard = "is_monitor")]
async fn metrics() -> MetricsStatus {
    let chainkey_addr = minter_addr();
    let balance = get_balance(chainkey_addr).await.unwrap_or_default();
    MetricsStatus {
        latest_scan_interval_secs: 0,
        chainkey_addr_balance: balance.as_u128(),
    }
}

#[update]
async fn generate_ticket(hash: String) -> Result<(), String> {
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
        return Err("duplicate request".to_string());
    }
    let (ticket, _transaction_receipt) = create_ticket_by_tx(&tx_hash).await?;
    let hub_principal = read_state(|s| s.hub_principal);
    hub::pending_ticket(hub_principal, ticket)
        .await
        .map_err(|e| {
            error!("call hub error:{}", e.to_string());
            "call hub error".to_string()
        })?;
    mutate_state(|s| s.pending_events_on_chain.insert(tx_hash, get_time_secs()));
    Ok(())
}

#[update(guard = "is_admin")]
pub fn insert_pending_hash(tx_hash: String) {
    mutate_state(|s| s.pending_events_on_chain.insert(tx_hash, get_time_secs()));
}

#[update(guard = "is_admin")]
pub async fn test_receipt(tx_hash: String) -> String{
    let r = get_transaction_receipt(&tx_hash).await.unwrap();
    let t = r.unwrap();
    serde_json::to_string(&t).unwrap()
/*    let v: Vec<LocalLogEntry> = t.logs.clone().into_iter().map(|l|l.into()).collect();
    (t,v)*/
}


#[update(guard = "is_admin")]
pub async fn query_hub_tickets(start: u64) -> Vec<(Seq, Ticket)> {
    let hub_principal = read_state(|s| s.hub_principal);
    match hub::query_tickets(hub_principal, start, BATCH_QUERY_LIMIT).await {
        Ok(tickets) => {
            return tickets
        }
        Err(err) => {
            log::error!("[process tickets] failed to query tickets, err: {}", err);
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
    hub::update_tx_hash(hub_principal, ticket_id, tx_hash).await.unwrap();
}

#[update(guard = "is_admin")]
pub async fn resend_ticket_to_hub(tx_hash: String) {
    let (ticket, _tr) = create_ticket_by_tx(&tx_hash).await.unwrap();
    let _r: () = ic_cdk::call(crate::state::hub_addr(), "send_ticket", (ticket.clone(),))
        .await
        .map_err(|(_, s)| Error::HubError(s))
        .unwrap();
    info!("[evm_route] burn_ticket sent to hub success: {:?}", ticket);
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
