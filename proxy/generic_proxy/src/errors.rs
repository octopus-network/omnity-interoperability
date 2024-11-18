use crate::*;

pub type Result<R> = std::result::Result<R, Errors>;

#[derive(thiserror::Error, Debug, CandidType, Serialize, Deserialize)]
pub enum Errors {
    #[error("Call {0} of {1} failed, code: {2:?}, message: {3}")]
    CallError(String, Principal, String, String),
    #[error("Account Id({0}) Parse Error: {1}")]
    AccountIdParseError(String, String),
    #[error("Canister call {0}::{1} failed, code: {2:?}, message: {3}")]
    CanisterCallError(String, String, String, String),
    #[error("ckBTC update the account: ({0}) balance error: {1}")]
    CkBtcUpdateBalanceError(String, String),
    #[error("Failed to convert Nat({0})")]
    NatConversionError(String),
    #[error("{0}")]
    CustomError(String),
}
