use crate::state::{with_state, with_state_mut};
use candid::CandidType;
use log::{debug, error};
use serde::{Deserialize, Serialize};

#[derive(CandidType, Copy, Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub enum Permission {
    Query,
    Update,
}

pub fn is_admin() -> Result<(), String> {
    let caller = ic_cdk::api::caller();
    with_state(|s| {
        if s.admin != caller {
            error!("{:?} Not Admin!", caller.to_string());
            Err("Not Admin!".into())
        } else {
            Ok(())
        }
    })
}

pub fn is_runes_oracle() -> Result<(), String> {
    let caller = ic_cdk::api::caller();
    with_state(|s| {
        if !s.runes_oracles.contains(&caller) {
            Err("Not runes principal!".into())
        } else {
            Ok(())
        }
    })
}

pub fn auth_update() -> Result<(), String> {
    let caller = ic_cdk::api::caller();
    debug!("auth update for caller: {:?}", caller.to_string());
    with_state(|s| {
        if s.admin != caller
            && !ic_cdk::api::is_controller(&caller)
            && !s
                .caller_perms
                .get(&caller.to_string())
                .is_some_and(|perm| *perm == Permission::Update)
        {
            error!("{:?} Unauthorized!", caller.to_string());
            Err("Unauthorized!".into())
        } else {
            Ok(())
        }
    })
}

pub fn auth_query() -> Result<(), String> {
    let caller = ic_cdk::api::caller();
    debug!("auth query for caller: {:?}", caller.to_string());
    with_state(|s| {
        if s.admin != caller
            && !ic_cdk::api::is_controller(&caller)
            && !s.caller_perms.contains_key(&caller.to_string())
        {
            error!("{:?} Unauthorized!", caller.to_string());
            Err("Unauthorized!".into())
        } else {
            Ok(())
        }
    })
}

pub fn set_perms(caller: String, perm: Permission) {
    with_state_mut(|s| {
        s.caller_perms.insert(caller.to_string(), perm);
    })
}
