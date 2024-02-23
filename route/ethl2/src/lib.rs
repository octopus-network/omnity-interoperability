mod redeem;
mod transport;
mod tx;

use candid::{CandidType, Principal};
use ic_cdk::api::management_canister::ecdsa::{
    ecdsa_public_key, sign_with_ecdsa, EcdsaCurve, EcdsaKeyId, EcdsaPublicKeyArgument,
    SignWithEcdsaArgument,
};
use omnity_types::*;
use std::{
    cell::RefCell,
    collections::{BTreeMap, HashMap},
};

thread_local! {
    static TOKENS: RefCell<HashMap<String, Vec<u8>>> = RefCell::new(Default::default());
    static TICKETS: RefCell<BTreeMap<u64, Ticket>> = RefCell::new(BTreeMap::new());
    static TARGET_CHAIN: RefCell<ChainId> = RefCell::new(ChainId::default());
    static TARGET_CHAIN_ID: RefCell<u64> = RefCell::new(0);
    static PUBKEY: RefCell<Option<Vec<u8>>> = RefCell::new(None);
    static KEY_ID: RefCell<Option<EcdsaKeyId>> = RefCell::new(None);
    static KEY_DERIVATION_PATH: RefCell<Vec<Vec<u8>>> = RefCell::new(Vec::new());
    // static HUB_ADDR: RefCell<Principal> = RefCell::new()
}

pub enum Error {
    HubOffline(String),
    EthRpcUnavailable,
    ChainKeyError(String),
    RouteNotInitialized,
}

pub fn init(target_chain: ChainId, target_chain_id: u64) {
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
}

// don't call this in canister init function because ICP forbids IO during initialization
pub async fn init_key() -> Result<(), Error> {
    let arg = EcdsaPublicKeyArgument {
        canister_id: None,
        derivation_path: KEY_DERIVATION_PATH.with_borrow(|p| p.clone()),
        key_id: KEY_ID.with_borrow(|k| k.as_ref().expect("already initialized;qed").clone()),
    };
    let (r,) = ecdsa_public_key(arg)
        .await
        .map_err(|(_, e)| Error::ChainKeyError(e))?;
    PUBKEY.with_borrow_mut(|p| p.replace(r.public_key));
    Ok(())
}

pub fn target_chain_id() -> u64 {
    TARGET_CHAIN_ID.with_borrow(|id| *id)
}

pub fn max_ticket_id() -> u64 {
    TICKETS.with_borrow(|tickets| tickets.keys().last().unwrap_or(&0) + 1)
}

pub fn try_public_key() -> Result<Vec<u8>, Error> {
    PUBKEY
        .with_borrow(|p| p.clone())
        .ok_or(Error::RouteNotInitialized)
}

pub fn try_key_id() -> Result<EcdsaKeyId, Error> {
    KEY_ID
        .with_borrow(|k| k.clone())
        .ok_or(Error::RouteNotInitialized)
}

pub fn key_derivation_path() -> Vec<Vec<u8>> {
    KEY_DERIVATION_PATH.with_borrow(|p| p.clone())
}
