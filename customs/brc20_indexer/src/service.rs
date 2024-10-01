use candid::CandidType;
use ic_cdk::api::management_canister::http_request;
use ic_cdk::api::management_canister::http_request::TransformArgs;
use ic_cdk_macros::{export_candid, init, post_upgrade, pre_upgrade, query, update};
use serde::{Deserialize, Serialize};
use crate::state::{BitcoinNetwork, IndexerState};
use crate::state::replace_state;
use crate::state::read_state;
use crate::unisat::query_transfer_event;
pub use omnity_types::brc20::*;
#[derive(CandidType, Serialize, Deserialize, Debug, Clone)]
pub struct InitArgs {
    pub api_key: String,
    pub network: BitcoinNetwork,
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
    query_transfer_event(args).await
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


ic_cdk::export_candid!();

