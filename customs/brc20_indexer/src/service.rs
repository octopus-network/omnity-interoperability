use candid::CandidType;
use ic_canister_log::log;
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
use crate::bestinslot::bestinsolt_query_transfer_event;
use crate::okx::okx_query_transfer_event;
use omnity_types::ic_log::{ERROR, INFO};
use crate::height::get_block_height;

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
    log!(INFO, "brc20 indexer canister upgrade successfully !!!");
}

#[update]
pub async fn get_indexed_transfer(args: QueryBrc20TransferArgs) -> Option<Brc20TransferEvent>{
    mix_indexer(&args).await
}

async fn mix_indexer(args: &QueryBrc20TransferArgs) -> Option<Brc20TransferEvent>{
    let height = get_block_height().await;
    if height == 0 {
        log!(INFO, "query height error: {}", height);
        return None;
    }
    let unisat_event = unisat_query_transfer_event(&args).await;
    let okx_event = okx_query_transfer_event(&args).await;
    if unisat_event.is_none() && okx_event.is_none() {
        log!(INFO, "unisat or okx error");
        return None;
    }
    if okx_event == unisat_event {
        return if height - okx_event.unwrap().height >= 4 {
            unisat_event
        } else {
            log!(INFO, "height no more than 4");
            None
        }
    }
    let bestinslot_event = bestinsolt_query_transfer_event(&args).await;
    if bestinslot_event.is_some() {
        if bestinslot_event == okx_event && height - okx_event.unwrap().height >= 4{
            return bestinslot_event;
        }
        if bestinslot_event == unisat_event && height - unisat_event.unwrap().height >= 4  {
            return bestinslot_event;
        }
    }
    log!(ERROR, "Not found brc20 event");
    return None;

}

#[update]
pub async fn height() -> u64{
    get_block_height().await
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
        headers: vec![]
    }
}

#[query(guard = "is_controller")]
pub fn proxy_url() -> String {
    crate::state::proxy_url()
}

pub fn is_controller() -> Result<(), String> {
    if ic_cdk::api::is_controller(&ic_cdk::caller()) {
        Ok(())
    } else {
        Err("caller is not controller".to_string())
    }
}

export_candid!();

