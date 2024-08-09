use candid::{CandidType, Nat, Principal};
use ic_cdk::api::call::RejectionCode;
use serde::{Deserialize, Serialize};

pub type Result<R> = std::result::Result<R, RouteError>;

#[derive(thiserror::Error, Debug, CandidType, Serialize, Deserialize)]
pub enum RouteError {
    #[error("Call {0} of {1} failed, code: {2:?}, message: {3}")]
    CallError(String, Principal, String, String),
    #[error("Http out call failed, code: {0:?}, message: {1}")]
    HttpOutCallError(String, String),
    #[error("Http status code: {0:?}, url: {1}, body: {2}")]
    HttpStatusError(Nat, String, String),
    #[error("{0}")]
    CustomError(String),
}
