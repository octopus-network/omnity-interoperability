use candid::CandidType;

use serde::{Deserialize, Serialize};

use crate::state::{mutate_state, read_state};

#[derive(CandidType, Copy, Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub enum Permission {
    Query,
    Update,
}

pub fn is_admin() -> Result<(), String> {
    let caller = ic_cdk::api::caller();
    read_state(|s| {
        if s.admin == caller
            || ic_cdk::api::is_controller(&caller)
            || s.caller_perms
                .get(&caller.to_string())
                .is_some_and(|perm| *perm == Permission::Update)
        {
            Ok(())
        } else {
            Err("Not Admin!".into())
        }
    })
}

pub fn auth_update() -> Result<(), String> {
    let caller = ic_cdk::api::caller();

    mutate_state(|s| {
        if s.admin != caller
            && !ic_cdk::api::is_controller(&caller)
            && !s
                .caller_perms
                .get(&caller.to_string())
                .is_some_and(|perm| *perm == Permission::Update)
        {
            Err("Unauthorized!".into())
        } else {
            Ok(())
        }
    })
}

pub fn auth_query() -> Result<(), String> {
    let caller = ic_cdk::api::caller();
    mutate_state(|s| {
        if s.admin != caller
            && !ic_cdk::api::is_controller(&caller)
            && !s.caller_perms.contains_key(&caller.to_string())
        {
            Err("Unauthorized!".into())
        } else {
            Ok(())
        }
    })
}

pub fn set_perms(caller: String, perm: Permission) {
    mutate_state(|s| {
        s.caller_perms.insert(caller.to_string(), perm);
    })
}
