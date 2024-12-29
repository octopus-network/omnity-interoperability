use thiserror::Error;
use candid::{CandidType, Deserialize, Nat, Principal};

use crate::types::Destination;

#[derive(CandidType, Clone, Default, Debug, Deserialize, PartialEq, Eq, Error)]
pub enum CustomsError {
    #[error("Call {0} of {1} failed, reason: {2:?}")]
    CallError(Principal, String, String),
    #[error("temp unavailable: {0}")]
    TemporarilyUnavailable(String),
    #[error("AlreadySubmitted")]
    AlreadySubmitted,
    #[error("AlreadyProcessed")]
    AlreadyProcessed,
    #[error("DepositUtxoNotFound, txid: {0}, destination: {1:?}")]
    DepositUtxoNotFound(String, Destination),
    #[error("TxNotFoundInMemPool")]
    TxNotFoundInMemPool,
    #[error("InvalidRuneId: {0}")]
    InvalidRuneId(String),
    #[error("InvalidTxId")]
    InvalidTxId,
    #[error("InvalidTxReceiver")]
    InvalidTxReceiver,
    #[error("UnsupportedChainId: {0}")]
    UnsupportedChainId(String),
    #[error("UnsupportedToken: {0}")]
    UnsupportedToken(String),
    #[error("SendTicketErr: {0}")]
    SendTicketErr(String),
    #[error("RpcError: {0}")]
    RpcError(String),
    #[error("AmountIsZero")]
    AmountIsZero,
    #[error("OrdTxError: {0}")]
    OrdTxError(String),
    #[error("NotBridgeTx")]
    NotBridgeTx,
    #[error("InvalidArgs: {0}")]
    InvalidArgs(String),
    #[error("NotPayFees")]
    NotPayFees,
    #[default]
    #[error("Unknown")]
    Unknown,
    #[error("CustomError: {0}")]
    CustomError(String),
    #[error("ECDSAPublicKeyNotFound")]
    ECDSAPublicKeyNotFound,
    #[error("Http out call failed, code: {0:?}, message: {1}, request: {2}")]
    HttpOutCallError(String, String, String),
    #[error("Http status code: {0:?}, url: {1}, body: {2}")]
    HttpStatusError(Nat, String, String),
    #[error("Http out call exceed limit")]
    HttpOutExceedLimit,
}