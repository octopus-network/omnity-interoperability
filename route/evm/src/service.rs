use std::time::Duration;

use candid::{CandidType, Principal};
use cketh_common::eth_rpc_client::providers::RpcApi;
use ethers_core::abi::ethereum_types;
use ethers_core::utils::keccak256;
use ic_canisters_http_types::{HttpRequest, HttpResponse};
use ic_cdk::{init, post_upgrade, pre_upgrade, query, update};
use ic_cdk_timers::set_timer_interval;
use k256::PublicKey;
use log::info;
use serde_derive::{Deserialize, Serialize};

use crate::const_args::{FETCH_HUB_TASK_INTERVAL, SCAN_EVM_TASK_INTERVAL, SEND_EVM_TASK_INTERVAL};
use crate::evm_scan::scan_evm_task;
use crate::hub_to_route::fetch_hub_periodic_task;
use crate::route_to_evm::{send_directive, send_ticket, to_evm_task};
use crate::stable_log::{init_log, StableLogWriter};
use crate::stable_memory::init_stable_log;
use crate::state::{
    EvmRouteState, init_chain_pubkey, mutate_state, read_state, replace_state,
    StateProfile,
};
use crate::types::{Chain, ChainId, Directive, MintTokenStatus, Network, PendingDirectiveStatus, PendingTicketStatus, Seq, Ticket, TicketId, TokenResp};

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
fn post_upgrade(args: Option<UpgradeArgs>) {
    EvmRouteState::post_upgrade(args);
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
        Duration::from_secs(FETCH_HUB_TASK_INTERVAL),
        fetch_hub_periodic_task,
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
    use ethers_core::utils::to_checksum;
    use k256::elliptic_curve::sec1::ToEncodedPoint;
    let mut key = read_state(|s| s.pubkey.clone());
    if key.is_empty() {
        init_chain_pubkey().await;
        key = read_state(|s| s.pubkey.clone());
    }
    let key_str = format!("0x{}", hex::encode(key.as_slice()));
    let key =
        PublicKey::from_sec1_bytes(key.as_slice()).expect("failed to parse the public key as SEC1");
    let point = key.to_encoded_point(false);
    let point_bytes = point.as_bytes();
    assert_eq!(point_bytes[0], 0x04);
    let hash = keccak256(&point_bytes[1..]);
    let addr = to_checksum(&ethereum_types::Address::from_slice(&hash[12..32]), None);
    (key_str, addr)
}

#[query]
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

fn is_admin() -> Result<(), String> {
    let c = ic_cdk::caller();
    match read_state(|s| s.admin == c) {
        true => Ok(()),
        false => Err("permission deny".to_string()),
    }
}


#[derive(CandidType, Deserialize)]
pub struct InitArgs {
    pub evm_chain_id: u64,
    pub admin: Principal,
    pub hub_principal: Principal,
    pub network: Network,
    pub evm_rpc_canister_addr: Principal,
    pub scan_start_height: u64,
    pub chain_id: String,
    pub rpc_url: String,
    pub fee_token_id: String,
}

#[derive(Clone, CandidType, Deserialize, Serialize)]
pub struct UpgradeArgs {
    pub omnity_port_contract_addr: Option<String>,
    pub rpc_services: Option<Vec<RpcApi>>,
}

ic_cdk::export_candid!();
