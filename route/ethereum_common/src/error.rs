use ic_cdk::api::call::RejectionCode;
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum Error {
    #[error("Hub error: {0}")]
    HubError(String),
    #[error("Evm rpc canister error: {0}")]
    EvmRpcCanisterError(String),
    #[error("Evm rpc error: {0}")]
    EvmRpcError(String),
    #[error("Chain key error: {0}")]
    ChainKeyError(String),
    #[error("Parse event error: {0}")]
    ParseEventError(String),
    #[error("Route not initialized")]
    RouteNotInitialized,
    #[error("IC call error: {0:?}, {1}")]
    IcCallError(RejectionCode, String),
    #[error("generate rpc request data error: {0}")]
    RequestDataError(String),
    #[error("custom error: {0}")]
    Custom(String),
    #[error("Temporay error")]
    Temporary,
    #[error("Fatal error: {0}")]
    Fatal(String),
}
