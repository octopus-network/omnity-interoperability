use std::str::FromStr;
use std::time::Duration;

use ethers_core::abi::ethereum_types;
use ethers_core::utils::keccak256;
use ic_cdk::api::management_canister::ecdsa::{ecdsa_public_key, EcdsaPublicKeyArgument};
use ic_cdk::{init, post_upgrade, pre_upgrade, query, update};
use ic_cdk_timers::set_timer_interval;
use k256::PublicKey;

use crate::eth_common::EvmAddress;
use crate::evm_scan::scan_evm_task;
use crate::hub_to_route::fetch_hub_periodic_task;
use crate::route_to_evm::{send_one_directive, to_evm_task};
use crate::state::{
    key_derivation_path, key_id, mutate_state, read_state, replace_state, EvmRouteState, InitArgs,
    StateProfile,
};
use crate::types::{Seq, Ticket};
use crate::Error;

#[init]
fn init(args: InitArgs) {
    replace_state(EvmRouteState::init(args).expect("params error"));
    start_tasks();
}

#[pre_upgrade]
fn pre_upgrade() {
    read_state(|s| s.pre_upgrade());
}

#[post_upgrade]
fn post_upgrade() {
    EvmRouteState::post_upgrade();
    start_tasks();
}

fn start_tasks() {
    set_timer_interval(Duration::from_secs(10), fetch_hub_periodic_task);
    set_timer_interval(Duration::from_secs(20), to_evm_task);
    set_timer_interval(Duration::from_secs(30), scan_evm_task);
}

#[update(guard = "is_admin")]
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
fn get_ticket(ticket_id: String) -> Option<(u64, Ticket)> {
    let r = read_state(|s| {
        s.tickets_queue
            .iter()
            .filter(|(_seq, t)| t.ticket_id == ticket_id)
            .collect::<Vec<_>>()
    });
    r.first().cloned()
}

#[query]
fn pubkey_and_evm_addr() -> (String, String) {
    let key = read_state(|s| s.pubkey.clone());
    let key_str = format!("0x{}", hex::encode(key.as_slice()));
    use k256::elliptic_curve::sec1::ToEncodedPoint;
    let key =
        PublicKey::from_sec1_bytes(key.as_slice()).expect("failed to parse the public key as SEC1");
    let point = key.to_encoded_point(false);
    let point_bytes = point.as_bytes();
    assert_eq!(point_bytes[0], 0x04);
    let hash = keccak256(&point_bytes[1..]);
    let addr =
        ethers_core::utils::to_checksum(&ethereum_types::Address::from_slice(&hash[12..32]), None);
    (key_str, addr)
}

#[query]
fn route_state() -> StateProfile {
    read_state(|s| StateProfile::from(s))
}

#[update(guard = "is_admin")]
async fn resend_directive(seq: Seq) {
    send_one_directive(seq).await;
}

#[update(guard = "is_admin")]
fn set_omnity_port_contract_addr(addr: String) {
    mutate_state(|s| s.omnity_port_contract = EvmAddress::from_str(addr.as_str()).unwrap());
}

#[update(guard = "is_admin")]
fn set_scan_height(height: u64) {
    mutate_state(|s| s.scan_start_height = height);
}
#[update]
fn set_evm_chain_id(chain_id: u64) {
    mutate_state(|s| s.evm_chain_id = chain_id);
}

fn is_admin() -> Result<(), String> {
    let c = ic_cdk::caller();
    match read_state(|s| s.admin == c) {
        true => Ok(()),
        false => Err("permission deny".to_string()),
    }
}

ic_cdk::export_candid!();
