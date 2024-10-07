use candid::CandidType;
use ic_cdk::api::management_canister::http_request;
use ic_cdk::api::management_canister::http_request::TransformArgs;
use ic_cdk_macros::{export_candid, init, post_upgrade, pre_upgrade, query, update};
use serde::{Deserialize, Serialize};
use crate::state::{BitcoinNetwork, IndexerState, mutate_state};
use crate::state::replace_state;
use crate::state::read_state;
use crate::unisat::unisat_query_transfer_event;
pub use omnity_types::brc20::*;
use ic_canisters_http_types::{HttpRequest, HttpResponse};

#[derive(CandidType, Serialize, Deserialize, Debug, Clone)]
pub struct InitArgs {
    pub network: BitcoinNetwork,
    pub proxy_url: String,
}

#[init]
fn init(init_args: InitArgs) {
    replace_state(IndexerState::init(init_args).expect("params error"));
}

#[pre_upgrade]
fn pre_upgrade() {
    read_state(|s| s.pre_upgrade());
}

#[post_upgrade]
fn post_upgrade() {
    IndexerState::post_upgrade();
}

#[update]
pub async fn get_indexed_transfer(args: QueryBrc20TransferArgs) -> Option<Brc20TransferEvent>{
    unisat_query_transfer_event(args).await
}

#[query(hidden = true)]
fn http_request(req: HttpRequest) -> HttpResponse {
    if ic_cdk::api::data_certificate().is_none() {
        ic_cdk::trap("update call rejected");
    }
    omnity_types::ic_log::http_request(req)
}

#[update(guard = "is_controller")]
pub fn set_api_key(rpc_name: String, key: String) {
    mutate_state(|s|s.api_keys.insert(rpc_name, key));
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

pub fn is_controller() -> Result<(), String> {
    if ic_cdk::api::is_controller(&ic_cdk::caller()) {
        Ok(())
    } else {
        Err("caller is not controller".to_string())
    }
}

export_candid!();

