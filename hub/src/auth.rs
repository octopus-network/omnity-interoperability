use crate::{with_state, with_state_mut, Error};
use candid::types::principal::Principal;
use ic_cdk::{ update};
use log::info;

pub fn auth() -> Result<(), String> {
    let caller = ic_cdk::api::caller();
    with_state(|s| {
        if !s.owner.eq(&Some(caller)) && !s.whitelist.contains(&caller) {
            Err("Unauthorized!".into())
        } else {
            Ok(())
        }
    })
}

pub fn is_owner() -> Result<(), String> {
    let caller = ic_cdk::api::caller();
    with_state(|s| {
        if !s.owner.eq(&Some(caller)) {
            Err("Not Owner!".into())
        } else {
            Ok(())
        }
    })
}

#[update(guard = "is_owner")]
pub async fn set_whitelist(principal: Principal, authorized: bool) -> Result<(), Error> {
    info!("principal: {principal:?}, authorized {authorized:?}");
    if authorized {
        with_state_mut(|s| s.whitelist.insert(principal));
    } else {
        with_state_mut(|s| s.whitelist.remove(&principal));
    }
    Ok(())
}

#[update(guard = "is_owner")]
pub async fn set_owner(principal: Principal) -> Result<(), Error> {
    with_state_mut(|s| s.owner = Some(principal));
    info!("new owner: {principal:?}");
    Ok(())
}
