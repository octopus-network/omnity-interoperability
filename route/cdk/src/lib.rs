
use ic_cdk::api::call::RejectionCode;
use itertools::Itertools;
use thiserror::Error;

use crate::types::{Chain, Ticket};

pub mod cdk_scan;
pub mod types;
pub mod state;

pub mod call_error;
pub mod hub;
pub mod hub_to_route;
pub mod guard;
pub mod contracts;
pub mod updates;
pub mod audit;
pub mod stable_memory;
pub mod evm_address;
pub mod route_to_cdk;

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
pub fn init(target_chain: Chain, target_chain_id: u64) {
   /*// TODO TOKEN
    TARGET_CHAIN.with_borrow_mut(|id| *id = target_chain.clone());
    TARGET_CHAIN_ID.with_borrow_mut(|id| *id = target_chain_id);
    KEY_ID.with_borrow_mut(|k| {
        *k = Some(EcdsaKeyId {
            curve: EcdsaCurve::Secp256k1,
            name: target_chain,
        })
    });
    // TODO make derivation path compatiable with ETH account
    // TODO this might be vec![b"m".to_vec(), b"44'".to_vec(), ...]
    KEY_DERIVATION_PATH.with_borrow_mut(|p| p.push(b"m/44'/223'/0'/0/0".to_vec()));
    // TODO init hub & rpc addr
    // TODO init rpc providers*/
}

// don't call this in canister init function because ICP forbids IO during initialization
pub async fn init_key() -> Result {
/*    let arg = EcdsaPublicKeyArgument {
        canister_id: None,
        derivation_path: KEY_DERIVATION_PATH.with_borrow(|p| p.clone()),
        key_id: KEY_ID.with_borrow(|k| k.as_ref().expect("already initialized;qed").clone()),
    };
    let (r,) = ecdsa_public_key(arg)
        .await
        .map_err(|(_, e)| Error::ChainKeyError(e))?;
    PUBKEY.with_borrow_mut(|p| p.replace(r.public_key));*/
    Ok(())
}



pub fn max_ticket_id() -> u64 {
    //TODO
    0
    //TICKETS.with_borrow(|tickets| *tickets.keys().last().unwrap_or(&0))
}
*/