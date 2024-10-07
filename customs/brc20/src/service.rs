use bitcoin::{Amount, Txid};
use candid::{CandidType, Deserialize, Principal};
use ic_cdk_macros::{init, post_upgrade, pre_upgrade, query, update};

use crate::constants::DEFAULT_FEE;
use crate::generate_ticket::GenerateTicketArgs;
use crate::management::get_utxos;
use crate::ord::builder::Utxo;
use crate::state::{
    init_ecdsa_public_key, mutate_state, read_state, replace_state, Brc20State, StateProfile,
};
use crate::tasks::start_tasks;
use crate::types::ReleaseTokenStatus;
use bitcoin::hashes::Hash;
use ic_canister_log::log;
use ic_canisters_http_types::{HttpRequest, HttpResponse};
use ic_cdk::api::management_canister::http_request;
use ic_cdk::api::management_canister::http_request::TransformArgs;
use omnity_types::ic_log::{INFO};
use omnity_types::{Network, Seq, Ticket};

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
pub async fn generate_ticket(req: GenerateTicketArgs) {
    let r = crate::generate_ticket::generate_ticket(req).await;
    log!(INFO, "Fi error: {:?}", r);
}
#[update]
pub async fn generate_deposit_addr() -> (Option<String>, Option<String>) {
    init_ecdsa_public_key().await;
    read_state(|s| (s.deposit_addr.clone(), s.deposit_pubkey.clone()))
}

#[update]
pub fn test_update_main_addr(addr: String) {
    mutate_state(|s| s.deposit_addr = Some(addr));
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

#[query(hidden = true)]
fn transform(raw: TransformArgs) -> http_request::HttpResponse {
    http_request::HttpResponse {
        status: raw.response.status.clone(),
        body: raw.response.body.clone(),
        headers: vec![],
        ..Default::default()
    }
}

#[update(guard = "is_admin")]
pub async fn resend_unlock_ticket(seq: Seq) {
    crate::custom_to_bitcoin::send_ticket_to_bitcoin(seq, &DEFAULT_FEE)
        .await
        .unwrap();
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
