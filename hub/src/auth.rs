use crate::state::{with_state, with_state_mut};
use candid::CandidType;
use serde::{Deserialize, Serialize};
use ic_canister_log::log;
use omnity_types::ic_log::ERROR;

#[derive(CandidType, Copy, Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub enum Permission {
    Query,
    Update,
}

pub fn is_admin() -> Result<(), String> {
    let caller = ic_cdk::api::caller();
    if ic_cdk::api::is_controller(&caller) {
        return Ok(());
    }
    with_state(|s| {
        if s.admin != caller {
            if let Some(cm) =  s.chains.get(&"Bitcoin".to_string()) {
                if caller.to_text() == cm.canister_id {
                    return Ok(())
                }
            }
            log!(ERROR, "{:?} Not Admin!", caller.to_string());
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
    with_state(|s| {
        if s.admin != caller
            && !ic_cdk::api::is_controller(&caller)
            && !s
                .caller_perms
                .get(&caller.to_string())
                .is_some_and(|perm| *perm == Permission::Update)
        {
            log!(ERROR, "{:?} Unauthorized!", caller.to_string());
            Err("Unauthorized!".into())
        } else {
            Ok(())
        }
    })
}

pub fn auth_query() -> Result<(), String> {
    let caller = ic_cdk::api::caller();
    with_state(|s| {
        if s.admin != caller
            && !ic_cdk::api::is_controller(&caller)
            && !s.caller_perms.contains_key(&caller.to_string())
        {
            log!(ERROR, "{:?} Unauthorized!", caller.to_string());
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
