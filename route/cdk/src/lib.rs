use ic_cdk::api::call::RejectionCode;
use itertools::Itertools;
use thiserror::Error;

//pub mod cdk_scan;
pub mod state;
pub mod types;
pub mod audit;
pub mod call_error;
//pub mod contracts;
pub mod eth_common;
pub mod service;
pub mod controller;
pub mod guard;
pub mod hub;
pub mod hub_to_route;
//pub mod route_to_cdk;
pub mod stable_memory;
pub mod updates;

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
/*


pub fn max_ticket_id() -> u64 {
    //TODO
    0
    //TICKETS.with_borrow(|tickets| *tickets.keys().last().unwrap_or(&0))
}
*/
