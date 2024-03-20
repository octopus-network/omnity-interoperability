use ic_cdk::api::call::RejectionCode;
use thiserror::Error;

pub type Result<T = ()> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Hub error: {0}")]
    HubError(String),
    #[error("Auth error: {0}")]
    AuthError(String),
    #[error("Evm rpc canister error: {0}")]
    EvmRpcCanisterError(String),
    #[error ("Evm rpc error: {0}")]
    EvmRpcError(String),
    #[error("Chain key error: {0}")]
    ChainKeyError(String),
    #[error("Parse event error: {0}")]
    ParseEventError(String),
    #[error("Route storage variable({0}) not initialized")]
    RouteNotInitialized(String),
    #[error("IC call error: {0:?}, {1}")]
    IcCallError(RejectionCode, String),

    #[error(transparent)]
    Custom(#[from] anyhow::Error),
}
