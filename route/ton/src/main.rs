use std::time::Duration;

use candid::{CandidType, Principal};
use ic_canister_log::log;
use ic_canisters_http_types::{HttpRequest, HttpResponse};
use ic_cdk::api::management_canister::http_request;
use ic_cdk::api::management_canister::http_request::TransformArgs;
use ic_cdk::{init, post_upgrade, pre_upgrade, query, update};
use ic_cdk_timers::set_timer_interval;
use log::info;
use serde_derive::Deserialize;

use crate::base::const_args::{
    FETCH_HUB_DIRECTIVE_INTERVAL, FETCH_HUB_TICKET_INTERVAL, SCAN_TON_TASK_INTERVAL,
    SEND_TON_TASK_INTERVAL,
};
use crate::chainkey::{init_chain_pubkey, minter_addr};
use crate::hub_to_route::{fetch_hub_directive_task, fetch_hub_ticket_task};
use crate::route_to_ton::{inner_send_ticket, send_ticket, to_ton_task};
use crate::state::{
    bridge_fee, mutate_state, read_state, replace_state, StateProfile, TonRouteState,
};
use crate::ton_to_route::scan_mint_events_task;
use crate::toncenter::{check_bridge_fee, create_ticket_by_generate_ticket, get_account_seqno};
use crate::types::{MintTokenStatus, PendingDirectiveStatus, PendingTicketStatus, TokenResp};
use omnity_types::ic_log::INFO;
use omnity_types::{Chain, ChainId, Directive, Seq, Ticket};

pub mod audit;
pub mod base;
pub mod call_error;
mod chainkey;
pub mod guard;
pub mod hub;
pub mod hub_to_route;
pub mod route_to_ton;
pub mod stable_memory;
pub mod state;
mod ton_common;
mod ton_to_route;
mod ton_transaction;
mod toncenter;
pub mod types;
pub mod updates;

#[init]
fn init(args: InitArgs) {
    replace_state(TonRouteState::init(args).expect("params error"));
    start_tasks();
}

#[pre_upgrade]
fn pre_upgrade() {
    read_state(|s| s.pre_upgrade());
}

#[post_upgrade]
fn post_upgrade() {
    TonRouteState::post_upgrade();
    start_tasks();
    info!("[ton_route] upgraded successed at {}", ic_cdk::api::time());
}

#[query(hidden = true)]
fn http_request(req: HttpRequest) -> HttpResponse {
    omnity_types::ic_log::http_request(req)
}

#[update(guard = "is_admin")]
fn update_consume_directive_seq(seq: Seq) {
    mutate_state(|s| s.next_consume_directive_seq = seq);
}

#[update(guard = "is_admin")]
async fn query_account_seqno(addr: String) -> Result<i32, String> {
    get_account_seqno(&addr).await.map_err(|e| e.to_string())
}

#[query]
fn transform(raw: TransformArgs) -> http_request::HttpResponse {
    http_request::HttpResponse {
        status: raw.response.status.clone(),
        body: raw.response.body.clone(),
        headers: vec![],
    }
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
    set_timer_interval(Duration::from_secs(SEND_TON_TASK_INTERVAL), to_ton_task);
    set_timer_interval(
        Duration::from_secs(SCAN_TON_TASK_INTERVAL),
        scan_mint_events_task,
    );
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
async fn pubkey_and_ton_addr() -> (String, String) {
    let mut key = read_state(|s| s.pubkey.clone());
    if key.is_empty() {
        init_chain_pubkey().await;
        key = read_state(|s| s.pubkey.clone());
    }
    let key_str = format!("0x{}", hex::encode(key.as_slice()));
    let addr = minter_addr();
    (key_str, addr)
}

#[query(guard = "is_admin")]
fn route_state() -> StateProfile {
    read_state(|s| StateProfile::from(s))
}

#[query(guard = "is_admin")]
fn query_pending_ticket(from: usize, limit: usize) -> Vec<(Seq, PendingTicketStatus)> {
    read_state(|s| {
        s.pending_tickets_map
            .iter()
            .skip(from)
            .take(limit)
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
            .collect()
    })
}

#[update(guard = "is_admin")]
async fn resend_ticket(seq: Seq) -> Result<Option<String>, String> {
    let ticket = read_state(|s| s.tickets_queue.get(&seq)).unwrap();
    inner_send_ticket(ticket, seq)
        .await
        .map_err(|e| e.to_string())
}

#[query]
fn get_chain_list() -> Vec<Chain> {
    read_state(|s| s.counterparties.values().cloned().collect())
}

#[query]
fn get_token_list() -> Vec<TokenResp> {
    read_state(|s| {
        s.tokens
            .iter()
            .map(|(token_id, token)| {
                let mut resp: TokenResp = token.clone().into();
                resp.ton_contract = s.token_jetton_master_map.get(token_id).cloned();
                resp
            })
            .collect()
    })
}

#[query]
fn mint_token_status(ticket_id: String) -> MintTokenStatus {
    read_state(|s| {
        s.finalized_mint_requests
            .get(&ticket_id)
            .map_or(MintTokenStatus::Unknown, |tx_hash| {
                MintTokenStatus::Finalized { tx_hash }
            })
    })
}

#[query]
fn get_fee(chain_id: ChainId) -> (Option<u64>, String) {
    let r = bridge_fee(&chain_id);
    (Some(r.unwrap_or(50000000u64)), minter_addr())
}

#[update]
async fn generate_ticket(params: GenerateTicketArgs) -> Result<Ticket, String> {
    let tx_hash = params.tx_hash.clone();
    if read_state(|s| s.handled_ton_event.contains(&tx_hash)) {
        return Err("duplicate request".to_string());
    }
    let ticket = create_ticket_by_generate_ticket(&params)
        .await
        .map_err(|e| e.to_string())?;
    check_bridge_fee(&tx_hash, &params.target_chain_id)
        .await
        .map_err(|e| e.to_string())?;
    let hub_principal = read_state(|s| s.hub_principal);
    hub::pending_ticket(hub_principal, ticket.clone())
        .await
        .map_err(|e| {
            log!(INFO, "call hub error:{}", e.to_string());
            "call hub error".to_string()
        })?;
    hub::finalize_ticket(crate::state::hub_addr(), ticket.ticket_id.clone())
        .await
        .map_err(|e| e.to_string())?;
    log!(
        INFO,
        "[bitfinity route] transport_ticket sent to hub success: {:?}",
        ticket
    );
    mutate_state(|s| s.handled_ton_event.insert(tx_hash));
    Ok(ticket)
}

#[update(guard = "is_admin")]
fn set_token_master(token_id: String, master: String) {
    mutate_state(|s| s.token_jetton_master_map.insert(token_id, master));
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

fn is_admin() -> Result<(), String> {
    let c = ic_cdk::caller();
    match ic_cdk::api::is_controller(&c) || read_state(|s| s.admins.contains(&c)) {
        true => Ok(()),
        false => Err("permission deny".to_string()),
    }
}

#[derive(CandidType, Deserialize)]
pub struct InitArgs {
    pub admins: Vec<Principal>,
    pub hub_principal: Principal,
}

#[derive(CandidType, Deserialize)]
pub struct GenerateTicketArgs {
    pub tx_hash: String,
    pub token_id: String,
    pub sender: String,
    pub amount: u128,
    pub target_chain_id: String,
    pub receiver: String,
}

fn main() {}

ic_cdk::export_candid!();
