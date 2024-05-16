use ethers_core::abi::ethereum_types;
use ethers_core::utils::keccak256;
use ic_cdk::{init, post_upgrade, pre_upgrade, query, update};
use ic_cdk::api::management_canister::ecdsa::{ecdsa_public_key, EcdsaPublicKeyArgument};
use k256::PublicKey;

use crate::cdk_scan::{get_cdk_finalized_height};
use crate::Error;
use crate::hub_to_route::store_tickets;
use crate::state::{CdkRouteState, InitArgs, key_derivation_path, key_id, mutate_state, read_state, replace_state, StateProfile};
use crate::types::{Seq, Ticket};

#[init]
fn init(args: InitArgs) {
    replace_state(CdkRouteState::init(args).expect("params error"));

/*  set_timer_interval(Duration::from_secs(10), fetch_hub_periodic_task);
    set_timer_interval(Duration::from_secs(20), to_cdk_task);
    set_timer_interval(Duration::from_secs(30), scan_cdk_task);*/
}

#[pre_upgrade]
fn pre_upgrade() {
    read_state(|s| s.pre_upgrade());
}

#[post_upgrade]
fn post_upgrade() {
    CdkRouteState::post_upgrade();
    /*
   set_timer_interval(Duration::from_secs(10), fetch_hub_periodic_task);
   set_timer_interval(Duration::from_secs(20), to_cdk_task);
   set_timer_interval(Duration::from_secs(30), scan_cdk_task);*/
}

#[update]
async fn init_chain_pubkey() -> String {
    let arg = EcdsaPublicKeyArgument {
        canister_id: None,
        derivation_path: key_derivation_path(),
        key_id: key_id(),
    };
    let res = ecdsa_public_key(arg)
        .await
        .map_err(|(_, e)| Error::ChainKeyError(e));
    match res {
        Ok((t,)) => {
            mutate_state(|s| s.pubkey = t.public_key.clone());
            hex::encode(t.public_key)
        }
        Err(e) => e.to_string(),
    }
}

#[query]
fn pubkey_and_evm_addr() -> (String, String) {
    let key = read_state(|s| s.pubkey.clone());
    let key_str = format!("0x{}", hex::encode(key.as_slice()));
    use k256::elliptic_curve::sec1::ToEncodedPoint;
    let key =
        PublicKey::from_sec1_bytes(key.as_slice()).expect("failed to parse the public key as SEC1");
    let point = key.to_encoded_point(false);
    // we re-encode the key to the decompressed representation.
    let point_bytes = point.as_bytes();
    assert_eq!(point_bytes[0], 0x04);
    let hash = keccak256(&point_bytes[1..]);
    let addr = ethers_core::utils::to_checksum(&ethereum_types::Address::from_slice(&hash[12..32]), None);
    (key_str, addr)
}

#[query]
fn route_state() -> StateProfile {
    read_state(|s| StateProfile::from(s))
}

fn is_admin() -> Result<(), String> {
    let c = ic_cdk::caller();
    match read_state(|s| s.admin == c) {
        true => Ok(()),
        false => Err("permission deny".to_string()),
    }
}


#[update]
pub fn test_send_ticket(v: Vec<(Seq, Ticket)>) {
    store_tickets(v,0);
}

#[update]
pub async fn test_get_finalized_height() -> u64 {
    let r = get_cdk_finalized_height().await.unwrap();
    r
}

ic_cdk::export_candid!();
