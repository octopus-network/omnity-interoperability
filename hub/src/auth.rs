use crate::state::{with_state, with_state_mut};
use candid::CandidType;
use log::{debug, error};
use serde::{Deserialize, Serialize};

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
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

pub fn auth_update() -> Result<(), String> {
    let caller = ic_cdk::api::caller();
    debug!("auth update for caller: {:?}", caller.to_string());
    with_state(|s| {
        if s.admin != caller
            && !ic_cdk::api::is_controller(&caller)
            && !s.caller_perms.contains_key(&caller.to_string())
        {
            error!("{:?} Unauthorized!", caller.to_string());
            Err("Unauthorized!".into())
        } else {
            s.caller_perms
                .get(&caller.to_string())
                .map_or(Err("Unauthorized!".into()), |perms| {
                    if perms.iter().any(|perm| perm == &Permission::Update) {
                        return Ok(());
                    }
                    Err("Unauthorized Update!".into())
                })
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
            s.caller_perms
                .get(&caller.to_string())
                .map_or(Err("Unauthorized!".into()), |perms| {
                    if perms.iter().any(|perm| perm == &Permission::Query) {
                        return Ok(());
                    }
                    Err("Unauthorized Query!".into())
                })
        }
    })
}

pub fn set_perms(caller: String, perms: Vec<Permission>) {
    with_state_mut(|s| {
        s.caller_perms.insert(caller.to_string(), perms);
    })
}
