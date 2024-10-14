use bitcoin::{Amount, Txid};
use candid::{CandidType, Deserialize, Principal};
use ic_cdk_macros::{init, post_upgrade, pre_upgrade, query, update};

use crate::constants::DEFAULT_FEE;
use crate::generate_ticket::{GenerateTicketArgs, GenerateTicketError};
use crate::management::get_utxos;
use crate::ord::builder::Utxo;
use crate::state::{
    init_ecdsa_public_key, mutate_state, read_state, replace_state, Brc20State, StateProfile,
};
use crate::tasks::start_tasks;
use crate::types::{FeesArgs, ReleaseTokenStatus, UtxoArgs, TokenResp};
use bitcoin::hashes::Hash;
use ic_canisters_http_types::{HttpRequest, HttpResponse};
use ic_cdk::api::management_canister::http_request;
use ic_cdk::api::management_canister::http_request::TransformArgs;
use omnity_types::{Network, Seq, Ticket, TokenId};
use crate::bitcoin_to_custom::finalize_lock_ticket_request;
use crate::custom_to_bitcoin::CustomToBitcoinResult;

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
#[update(guard = "is_admin")]
pub async fn generate_deposit_addr() -> (Option<String>, Option<String>) {
    init_ecdsa_public_key().await;
    read_state(|s| (s.deposit_addr.clone(), s.deposit_pubkey.clone()))
}

#[query(guard = "is_admin")]
pub fn brc20_state() -> StateProfile {
    read_state(|s| StateProfile::from(s))
}

#[update]
pub async fn test_create_tx(ticket: Ticket, seq: Seq) {
    mutate_state(|s| s.tickets_queue.insert(seq, ticket));
}

#[update]
pub async fn test_update_utxos() -> String {
    let (nw, deposit_addr) = read_state(|s| (s.btc_network, s.deposit_addr.clone().unwrap()));
    let utxos = get_utxos(nw, &deposit_addr, 0u32).await;
    match utxos.clone() {
        Ok(r) => {
            let v = r
                .utxos
                .into_iter()
                .map(|u| Utxo {
                    id: Txid::from_slice(u.outpoint.txid.as_ref()).unwrap(),
                    index: u.outpoint.vout,
                    amount: Amount::from_sat(u.value),
                })
                .collect::<Vec<Utxo>>();
            mutate_state(|s| s.deposit_addr_utxo = v.clone());
            serde_json::to_string(&v).unwrap()
        }
        Err(e) => {
            panic!("query utxo error {:?}", e);
        }
    }
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
pub fn update_brc20_indexer(principal: Principal) {
    mutate_state(|s|s.indexer_principal = principal);
}

#[update(guard = "is_admin")]
pub async fn test_finalize_lock() {
    finalize_lock_ticket_request().await
}
#[update]
pub async fn transfer_fee(session_key: String) -> u64 {
    3333
}

#[update]
pub async fn build_commit_tx(
    session_key: String,
    vins: Vec<UtxoArgs>,
    token_id: TokenId,
    amount: String,
    sender: String,
    target_chain: String,
    receiver: String,
) -> CustomToBitcoinResult<String> {
    let fee = FeesArgs {
        commit_fee: 1000,
        reveal_fee: 1000,
        spend_fee: 1000,
    };
    crate::psbt::build_commit(
        session_key,
        vins,
        token_id,
        amount,
        sender,
        target_chain,
        receiver,
        fee,
    ).await
}

#[update]
pub async fn build_reveal_transfer(session_key: String,
                                   commit_tx_id: String,) -> CustomToBitcoinResult<Vec<String>>{
    let fee = FeesArgs {
        commit_fee: 1000,
        reveal_fee: 1000,
        spend_fee: 1000,
    };
    crate::psbt::build_reveal_transfer(
        session_key, commit_tx_id, fee
    ).await
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
pub async fn resend_unlock_ticket(seq: Seq) -> String {
    let r = crate::custom_to_bitcoin::submit_unlock_ticket(seq, &DEFAULT_FEE)
        .await
        .unwrap()
        .unwrap();
    mutate_state(|s| s.flight_unlock_ticket_map.insert(seq, r.clone()));
    serde_json::to_string(&r).unwrap()
}

#[query]
fn get_token_list() -> Vec<TokenResp> {
    read_state(|s| {
        s.tokens
            .iter()
            .map(|(token_id, token)| {
                let mut resp: TokenResp = token.clone().into();
                resp
            })
            .collect()
    })
}


#[derive(CandidType, Deserialize)]
pub struct InitArgs {
    pub admins: Vec<Principal>,
    pub hub_principal: Principal,
    pub network: Network,
    pub chain_id: String,
    pub indexer_principal: Principal,
}

fn is_admin() -> Result<(), String> {
    let c = ic_cdk::caller();
    match ic_cdk::api::is_controller(&c) || read_state(|s| s.admins.contains(&c)) {
        true => Ok(()),
        false => Err("permission deny".to_string()),
    }
}

ic_cdk::export_candid!();
