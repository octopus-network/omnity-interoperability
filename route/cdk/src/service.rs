use std::str::FromStr;
use candid::Principal;
use ethers_core::abi::ethereum_types;
use ethers_core::types::spoof::nonce;
use ethers_core::types::U256;
use ethers_core::utils::keccak256;
use ic_cdk::{init, post_upgrade, pre_upgrade, query, update};
use ic_cdk::api::management_canister::ecdsa::{ecdsa_public_key, EcdsaPublicKeyArgument};
use ic_cdk::api::management_canister::main::CanisterId;
use k256::PublicKey;

use crate::cdk_scan::{get_cdk_finalized_height, get_gasprice};
use crate::contracts::{gen_eip1559_tx, gen_execute_directive_data, gen_mint_token_data};
use crate::Error;
use crate::eth_common::{broadcast, EvmAddress, get_account_nonce, sign_transaction};
use crate::hub_to_route::store_tickets;
use crate::state::{CdkRouteState, InitArgs, key_derivation_path, key_id, minter_addr, mutate_state, read_state, replace_state, StateProfile};
use crate::types::{Directive, Seq, Ticket};

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
async fn init_chain_pubkey(canister_id: Principal) -> String {
    let arg = EcdsaPublicKeyArgument {
        canister_id: Some(canister_id),
        derivation_path: key_derivation_path(),
        key_id: key_id(),
    };
    let res = ecdsa_public_key(arg)
        .await
        .map_err(|(_, e)| Error::ChainKeyError(e));
    match res {
        Ok((t,)) => {
            mutate_state(|s|s.pubkey = t.public_key.clone());
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

#[update]
fn set_omnity_port_contract_addr(addr: String) {
    mutate_state(|s|s.omnity_port_contract = EvmAddress::from_str(addr.as_str()).unwrap());
}

fn is_admin() -> Result<(), String> {
    let c = ic_cdk::caller();
    match read_state(|s| s.admin == c) {
        true => Ok(()),
        false => Err("permission deny".to_string()),
    }
}

#[update]
pub async fn test_send_directive_to_cdk(d: Directive, seq: Seq) -> String {
    let data = gen_execute_directive_data(&d, U256::from(seq));
    let nonce = get_account_nonce(minter_addr()).await.unwrap();
    let tx = gen_eip1559_tx(data, get_gasprice().await.ok(), nonce);
    let raw = sign_transaction(tx).await.unwrap();
    let hash = broadcast(raw).await.unwrap();
    hash
}

pub async fn test_send_ticket_to_cdk(t: Ticket, seq: Seq) -> String {
    let data = gen_mint_token_data(&t);
    let nonce = get_account_nonce(minter_addr()).await.unwrap();
    let tx = gen_eip1559_tx(data, get_gasprice().await.ok(), nonce);
    let raw = sign_transaction(tx).await.unwrap();
    let hash = broadcast(raw).await.unwrap();
    hash

}
#[update]
pub async fn test_get_finalized_height() -> u64 {
    let r = get_cdk_finalized_height().await.unwrap();
    r
}

ic_cdk::export_candid!();
