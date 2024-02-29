mod redeem;
mod transport;
mod tx;
mod types;

use crate::types::{Chain, Ticket};
use candid::{CandidType, Principal};
use cketh_common::eth_rpc_client::providers::{RpcApi, RpcService};
use ic_cdk::api::management_canister::ecdsa::{
    ecdsa_public_key, sign_with_ecdsa, EcdsaCurve, EcdsaKeyId, EcdsaPublicKeyArgument,
    SignWithEcdsaArgument,
};
use std::{
    cell::RefCell,
    collections::{BTreeMap, HashMap},
    rc::Rc,
};

thread_local! {
    // TODO implement directives
    static TOKENS: RefCell<HashMap<String, Vec<u8>>> = RefCell::new(Default::default());
    static TICKETS: RefCell<Rc<BTreeMap<u64, Ticket>>> = RefCell::new(Rc::new(BTreeMap::new()));
    static BROADCASTED_TXS: RefCell<Rc<HashMap<String, Ticket>>> = RefCell::new(Rc::new(HashMap::new()));
    static NONCE: RefCell<u64> = RefCell::new(0);
    static ACTIVE: RefCell<bool> = RefCell::new(false);
    // init on startup
    static TARGET_CHAIN: RefCell<Chain> = RefCell::new(Chain::default());
    static TARGET_CHAIN_ID: RefCell<u64> = RefCell::new(0);
    static KEY_ID: RefCell<Option<EcdsaKeyId>> = RefCell::new(None);
    static KEY_DERIVATION_PATH: RefCell<Vec<Vec<u8>>> = RefCell::new(Vec::new());
    static HUB_ADDR: RefCell<Option<Principal>> = RefCell::new(None);
    static RPC_ADDR: RefCell<Option<Principal>> = RefCell::new(None);
    static RPC_PROVIDERS: RefCell<Vec<RpcApi>> = RefCell::new(vec![]);
    // init on call
    static PUBKEY: RefCell<Option<Vec<u8>>> = RefCell::new(None);
}

#[derive(Clone, Debug)]
pub enum Error {
    HubError(String),
    EthRpcError(String),
    ChainKeyError(String),
    RouteNotInitialized,
}

pub fn init(target_chain: Chain, target_chain_id: u64) {
    // TODO TOKEN
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
    // TODO init rpc providers
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

pub fn is_active() -> bool {
    ACTIVE.with_borrow(|active| *active)
}

pub fn hub_addr() -> Option<Principal> {
    HUB_ADDR.with_borrow(|addr| addr.clone())
}

pub fn rpc_addr() -> Option<Principal> {
    RPC_ADDR.with_borrow(|addr| addr.clone())
}

pub fn rpc_providers() -> Vec<RpcApi> {
    RPC_PROVIDERS.with_borrow(|p| p.clone())
}

pub fn get_broadcasted_txs() -> Rc<HashMap<String, Ticket>> {
    BROADCASTED_TXS.with_borrow(|txs| txs.clone())
}

pub fn target_chain() -> String {
    TARGET_CHAIN.with_borrow(|id| id.clone())
}

pub fn target_chain_id() -> u64 {
    TARGET_CHAIN_ID.with_borrow(|id| *id)
}

pub fn max_ticket_id() -> u64 {
    TICKETS.with_borrow(|tickets| *tickets.keys().last().unwrap_or(&0))
}

pub fn try_public_key() -> Result<Vec<u8>, Error> {
    PUBKEY
        .with_borrow(|p| p.clone())
        .ok_or(Error::RouteNotInitialized)
}

pub fn key_id() -> EcdsaKeyId {
    KEY_ID.with_borrow(|k| k.as_ref().expect("init on write;qed").clone())
}

pub fn key_derivation_path() -> Vec<Vec<u8>> {
    KEY_DERIVATION_PATH.with_borrow(|p| p.clone())
}

pub fn fetch_and_incr_nonce() -> u64 {
    NONCE.with_borrow_mut(|n| {
        let nonce = *n;
        *n += 1;
        nonce
    })
}

/// call this function every time after resuming or activating this canister
pub fn start_pull(secs: u64) {
    ic_cdk_timers::set_timer(std::time::Duration::from_secs(secs), || {
        ic_cdk::spawn(async move {
            if is_active() {
                if transport::transport().await {
                    start_pull(1);
                } else {
                    start_pull(5);
                }
            }
        });
    });
}
