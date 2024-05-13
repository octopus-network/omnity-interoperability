use ic_cdk::{query, update};
use ic_cdk::api::management_canister::ecdsa::{ecdsa_public_key, EcdsaPublicKeyArgument};
use crate::Error;
use crate::state::{key_derivation_path, key_id, mutate_state, read_state};

#[update(guard = "is_admin")]
async fn init_chain_pubkey() -> String{
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
            mutate_state(|s|s.pubkey = t.public_key.clone());
            hex::encode(t.public_key)
        }
        Err(e) => {
            e.to_string()
        }
    }
}

#[query]
fn pubkey() -> String{
    let key = read_state(|s|s.pubkey.clone());
    hex::encode(key)
}

fn is_admin() -> Result<(), String>{
    let c = ic_cdk::caller();
    match read_state(|s|s.admin == c) {
        true => {Ok(())}
        false => { Err("permission deny".to_string())}
    }
}

ic_cdk::export_candid!();