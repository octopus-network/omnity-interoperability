use std::fmt::Display;

use candid::{CandidType, Deserialize, Nat, Principal};
use thiserror::Error;

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
    #[error("RpcResultParseError: {0}")]
    RpcResultParseError(String),
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
    #[error("Http out call exceed retry limit, url: {0}")]
    HttpOutExceedRetryLimit(String),
    #[error("InvalidBlockHash: {0}, error: {1}")]
    InvalidBlockHash(String, String),
    #[error("InvalidMerkleRoot: {0}, error: {1}")]
    InvalidMerkleRoot(String, String),
    #[error("InvalidBits: {0}, error: {1}")]
    InvalidBits(String, String),
    #[error("ValidationError: {0:?}")]
    ValidateError(#[from] ValidationError),
    #[error("MerkleBlockVerifyError: merkle block hash: {0}, saved block hash: {1}")]
    MerkleBlockVerifyError(String, String),
    #[error("BlockHashNotEqual at height({0}): saved: {1}, queried block({1}) pre hash: {2}")]
    BlockHashNotEqual(u64, String, String, String),
}

/// A block validation error.
#[derive(Debug, Clone, PartialEq, Eq, CandidType, Deserialize, Error)]
#[non_exhaustive]
pub enum ValidationError {
    /// The header hash is not below the target.
    BadProofOfWork,
    /// The `target` field of a block header did not match the expected difficulty.
    BadTarget,
    /// No auxpow on block with auxpow version.
    BadVersion,
    /// The block has an invalid auxpow.
    BadAuxPow(String),
}

impl Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::BadProofOfWork => write!(f, "bad proof of work"),
            ValidationError::BadTarget => write!(f, "bad target"),
            ValidationError::BadVersion => write!(f, "bad version"),
            ValidationError::BadAuxPow(s) => write!(f, "bad auxpow: {}", s),
        }
    }
}
