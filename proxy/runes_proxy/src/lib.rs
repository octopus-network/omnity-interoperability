mod oracle;
mod types;

use candid::{CandidType, Deserialize, Principal};
use ic_canisters_http_types::{HttpRequest, HttpResponse, HttpResponseBuilder};
use ic_cdk_macros::{init, post_upgrade, pre_upgrade, query};
use std::cell::RefCell;

thread_local! {
    static CUSTOMS_PRINCIPAL: RefCell<Option<Principal>> = RefCell::new(None);
    static INDEXER_PRINCIPAL: RefCell<Option<Principal>> = RefCell::new(None);
}

pub(crate) fn customs_principal() -> Principal {
    CUSTOMS_PRINCIPAL.with(|p| p.borrow().clone().expect("not initialized"))
}

pub(crate) fn indexer_principal() -> Principal {
    INDEXER_PRINCIPAL.with(|p| p.borrow().clone().expect("not initialized"))
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct Args {
    pub customs: Principal,
    pub indexer: Principal,
}

#[query(hidden = true)]
fn http_request(req: HttpRequest) -> HttpResponse {
    if ic_cdk::api::data_certificate().is_none() {
        ic_cdk::trap("update call rejected");
    }
    if req.path() == "/logs" {
        omnity_types::ic_log::http_request(req)
    } else {
        HttpResponseBuilder::not_found().build()
    }
}

#[init]
pub fn init(args: Args) {
    CUSTOMS_PRINCIPAL.with(|p| p.replace(Some(args.customs)));
    INDEXER_PRINCIPAL.with(|p| p.replace(Some(args.indexer)));
    oracle::fetch_then_submit(5);
}

#[pre_upgrade]
fn pre_upgrade() {
    let customs = CUSTOMS_PRINCIPAL.with(|p| p.take());
    let indexer = INDEXER_PRINCIPAL.with(|p| p.take());
    ic_cdk::storage::stable_save((customs, indexer)).unwrap();
}

#[post_upgrade]
fn post_upgrade() {
    let (customs, indexer): (Option<Principal>, Option<Principal>) =
        ic_cdk::storage::stable_restore().unwrap();
    CUSTOMS_PRINCIPAL.with(|p| p.replace(customs));
    INDEXER_PRINCIPAL.with(|p| p.replace(indexer));
    oracle::fetch_then_submit(5);
}

ic_cdk::export_candid!();
