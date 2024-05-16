use ic_cdk::api::call::RejectionCode;
use thiserror::Error;

pub mod audit;
pub mod call_error;
pub mod cdk_scan;
pub mod contract_types;
pub mod contracts;
pub mod eth_common;
pub mod guard;
pub mod hub;
pub mod hub_to_route;
pub mod route_to_cdk;
pub mod service;
pub mod stable_memory;
pub mod state;
pub mod types;
pub mod updates;
pub mod test_functions;

type Result<T = ()> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
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
    #[error(transparent)]
    Custom(#[from] anyhow::Error),
}
