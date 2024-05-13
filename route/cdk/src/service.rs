use crate::Error;
use ic_cdk::api::management_canister::ecdsa::{ecdsa_public_key, EcdsaPublicKeyArgument};
use ic_cdk::{init, post_upgrade, pre_upgrade, query, update};

use crate::state::{key_derivation_path, key_id, mutate_state, read_state, CdkRouteState};

/*
#[init]
fn init(args: InitArgs) {
    mutate_state(|s| *s = CdkRouteState::init(args).expect("params error"));
/*  set_timer_interval(Duration::from_secs(10), fetch_hub_periodic_task);
    set_timer_interval(Duration::from_secs(20), to_cdk_task);
    set_timer_interval(Duration::from_secs(30), scan_cdk_task);*/
}
*/

#[init]
fn init() {
    mutate_state(|s| *s = CdkRouteState::default());
}
#[pre_upgrade]
fn pre_upgrade() {
    read_state(|s| s.pre_upgrade());
}

#[post_upgrade]
fn post_upgrade() {
    CdkRouteState::post_upgrade(); /*
                                   set_timer_interval(Duration::from_secs(10), fetch_hub_periodic_task);
                                   set_timer_interval(Duration::from_secs(20), to_cdk_task);
                                   set_timer_interval(Duration::from_secs(30), scan_cdk_task);*/
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
fn pubkey() -> String {
    let key = read_state(|s| s.pubkey.clone());
    hex::encode(key)
}

fn is_admin() -> Result<(), String> {
    let c = ic_cdk::caller();
    match read_state(|s| s.admin == c) {
        true => Ok(()),
        false => Err("permission deny".to_string()),
    }
}

ic_cdk::export_candid!();
