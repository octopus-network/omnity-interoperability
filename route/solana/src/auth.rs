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
        if s.admin == caller || ic_cdk::api::is_controller(&caller) {
            Ok(())
        } else {
            ic_cdk::eprintln!("{:?} Not Admin!", caller.to_string());
            Err("Not Admin!".into())
        }
    })
}

pub fn auth_update() -> Result<(), String> {
    let caller = ic_cdk::api::caller();
    ic_cdk::println!("auth update for caller: {:?}", caller.to_string());
    mutate_state(|s| {
        if s.admin != caller
            && !ic_cdk::api::is_controller(&caller)
            && !s
                .caller_perms
                .get(&caller.to_string())
                .is_some_and(|perm| *perm == Permission::Update)
        {
            ic_cdk::eprintln!("{:?} Unauthorized!", caller.to_string());
            Err("Unauthorized!".into())
        } else {
            Ok(())
        }
    })
}

pub fn auth_query() -> Result<(), String> {
    let caller = ic_cdk::api::caller();
    ic_cdk::println!("auth query for caller: {:?}", caller.to_string());
    mutate_state(|s| {
        if s.admin != caller
            && !ic_cdk::api::is_controller(&caller)
            && !s.caller_perms.contains_key(&caller.to_string())
        {
            ic_cdk::eprintln!("{:?} Unauthorized!", caller.to_string());
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
