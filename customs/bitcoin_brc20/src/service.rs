use crate::bitcoin_to_custom::finalize_lock;
use crate::generate_ticket::{GenerateTicketArgs, GenerateTicketError};
use crate::ord::builder::Utxo;
use crate::state::{
    init_ecdsa_public_key, mutate_state, read_state, replace_state, Brc20State, StateProfile,
};
use crate::tasks::start_tasks;
use crate::types::{LockTicketRequest, ReleaseTokenStatus, TokenResp, UtxoArgs};
use candid::{CandidType, Deserialize, Principal};
use ic_btc_interface::Txid;
use ic_canisters_http_types::{HttpRequest, HttpResponse};
use ic_cdk::api::management_canister::http_request;
use ic_cdk::api::management_canister::http_request::TransformArgs;
use ic_cdk_macros::{init, post_upgrade, pre_upgrade, query, update};
use omnity_types::{ChainId, Network, Seq};
use std::str::FromStr;

#[init]
fn init(args: InitArgs) {
    replace_state(Brc20State::init(args).expect("params error"));
    start_tasks();
}

#[pre_upgrade]
fn pre_upgrade() {
    read_state(|s| s.pre_upgrade());
}

#[post_upgrade]
fn post_upgrade() {
    Brc20State::post_upgrade();
    start_tasks();
}

#[query(hidden = true)]
fn http_request(req: HttpRequest) -> HttpResponse {
    if ic_cdk::api::data_certificate().is_none() {
        ic_cdk::trap("update call rejected");
    }
    omnity_types::ic_log::http_request(req)
}

#[update]
pub async fn generate_ticket(req: GenerateTicketArgs) -> Result<(), GenerateTicketError> {
    crate::generate_ticket::generate_ticket(req).await
}

#[query]
fn get_platform_fee(target_chain: ChainId) -> (Option<u128>, Option<String>) {
    read_state(|s| {
        s.get_transfer_fee_info(&target_chain)
    })
}

#[query]
pub fn get_deposit_addr() -> (String, String) {
    read_state(|s| {
        (
            s.deposit_addr.clone().unwrap(),
            s.deposit_pubkey.clone().unwrap(),
        )
    })
}

#[update(guard = "is_admin")]
pub async fn generate_deposit_addr() -> (String, String) {
    init_ecdsa_public_key().await;
    read_state(|s| {
        (
            s.deposit_addr.clone().unwrap(),
            s.deposit_pubkey.clone().unwrap(),
        )
    })
}

#[query(guard = "is_admin")]
pub fn brc20_state() -> StateProfile {
    read_state(|s| StateProfile::from(s))
}

#[update(guard = "is_admin")]
pub fn set_fee_collector(addr: String) {
    mutate_state(|s|s.fee_collector = addr);
}

#[query]
fn release_token_status(ticket_id: String) -> ReleaseTokenStatus {
    read_state(|s| s.unlock_tx_status(&ticket_id))
}

#[query(guard = "is_admin")]
pub fn pending_unlock_tickets(seq: Seq) -> String {
    let r = read_state(|s| s.flight_unlock_ticket_map.get(&seq).cloned().unwrap());
    serde_json::to_string(&r).unwrap()
}

#[query(guard = "is_admin")]
pub fn finalized_unlock_tickets(seq: Seq) -> String {
    let r = read_state(|s| s.finalized_unlock_ticket_map.get(&seq).cloned().unwrap());
    serde_json::to_string(&r).unwrap()
}

#[update(guard = "is_admin")]
pub fn update_fees(us: Vec<UtxoArgs>) {
    for a in us {
        let utxo: Utxo = a.into();
        mutate_state(|s| {
            if !s.deposit_addr_utxo.contains(&utxo) {
                s.deposit_addr_utxo.push(utxo);
            }
        });
    }
}

#[update(guard = "is_admin")]
pub async fn finalize_lock_request(txid: String) {
    let txid = Txid::from_str(txid.as_str()).unwrap();
    let deposit = read_state(|s| s.deposit_addr.clone().unwrap());
    let req = read_state(|s| s.pending_lock_ticket_requests.get(&txid).cloned().unwrap());
    finalize_lock(txid, req, deposit).await;
}

#[query(hidden = true)]
fn transform(raw: TransformArgs) -> http_request::HttpResponse {
    http_request::HttpResponse {
        status: raw.response.status.clone(),
        body: raw.response.body.clone(),
        headers: vec![],
    }
}

#[update(guard = "is_admin")]
pub async fn resend_unlock_ticket(seq: Seq, fee_rate: u64) -> String {
    let r = crate::custom_to_bitcoin::submit_unlock_ticket(seq, fee_rate)
        .await
        .unwrap()
        .unwrap();
    mutate_state(|s| s.flight_unlock_ticket_map.insert(seq, r.clone()));
    serde_json::to_string(&r).unwrap()
}

#[query]
fn get_token_list() -> Vec<TokenResp> {
    read_state(|s| s.tokens.values().map(|t| t.clone().into()).collect())
}

#[query(guard = "is_admin")]
fn query_finalized_lock_tickets(txid: Txid) -> Option<LockTicketRequest> {
    read_state(|s| s.finalized_lock_ticket_requests.get(&txid).cloned())
}

#[derive(CandidType, Deserialize)]
pub struct InitArgs {
    pub admins: Vec<Principal>,
    pub hub_principal: Principal,
    pub network: Network,
    pub chain_id: String,
    pub indexer_principal: Principal,
    pub fee_token: String,
}

fn is_admin() -> Result<(), String> {
    let c = ic_cdk::caller();
    match ic_cdk::api::is_controller(&c) || read_state(|s| s.admins.contains(&c)) {
        true => Ok(()),
        false => Err("permission deny".to_string()),
    }
}

ic_cdk::export_candid!();
