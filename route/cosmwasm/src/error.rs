use candid::{CandidType, Nat, Principal};
use serde::{Deserialize, Serialize};

pub type Result<R> = std::result::Result<R, RouteError>;

#[derive(thiserror::Error, Debug, CandidType, Serialize, Deserialize)]
pub enum RouteError {
    #[error("Call {0} of {1} failed, code: {2:?}, message: {3}")]
    CallError(String, Principal, String, String),
    #[error("Http out call failed, code: {0:?}, message: {1}, request: {2}")]
    HttpOutCallError(String, String, String),
    #[error("Http status code: {0:?}, url: {1}, body: {2}")]
    HttpStatusError(Nat, String, String),
    #[error("Sign with ecdsa error, reject code {0}, message: {1}")]
    SignWithEcdsaError(String, String),
    #[error("Query ecdsa public key error, reject code {0}, message: {1}")]
    QueryEcdsaPublicKeyError(String, String),
    #[error("Event not found, kind: {0}")]
    EventNotFound(String),
    #[error("Attribute parse error, key: {0}, event kind: {1}, error: {2}")]
    AttributeParseError(String, String, String),
    #[error("Attribute not found, key: {0}, event kind: {1}")]
    AttributeNotFound(String, String),
    #[error("Failed to send ticket, error: {0}")]
    SendTicketErr(String),
    #[error("Failed to confirm mint token, seq: {0}, tx_hash: {1}")]
    ConfirmExecuteDirectiveErr(u64, String),
    #[error("Failed to confirm mint token, mint_token_request: {0}, tx_hash: {1}")]
    ConfirmMintTokenErr(String, String),
    #[error("{0}")]
    CustomError(String),
}
