#![allow(unused)]
use candid::CandidType;

use serde::{Deserialize, Serialize};

use crate::config::{mutate_config, read_config};

#[derive(CandidType, Copy, Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub enum Permission {
    Query,
    Update,
}

pub fn is_admin() -> Result<(), String> {
    let caller = ic_cdk::api::caller();
    read_config(|s| {
        if s.get().admin == caller || ic_cdk::api::is_controller(&caller) {
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
    read_config(|s| {
        if s.get().admin != caller
            && !ic_cdk::api::is_controller(&caller)
            && !s
                .get()
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
    read_config(|s| {
        if s.get().admin != caller
            && !ic_cdk::api::is_controller(&caller)
            && !s.get().caller_perms.contains_key(&caller.to_string())
        {
            ic_cdk::eprintln!("{:?} Unauthorized!", caller.to_string());
            Err("Unauthorized!".into())
        } else {
            Ok(())
        }
    })
}

pub fn set_perms(caller: String, perm: Permission) {
    mutate_config(|s| {
        let mut config = s.get().to_owned();
        config.caller_perms.insert(caller.to_string(), perm);
        s.set(config);
    })
}
