use crate::state::with_state;
use log::info;

pub fn auth() -> Result<(), String> {
    let caller = ic_cdk::api::caller();
    info!("auth for caller: {:?}", caller.to_string());
    with_state(|s| {
        if !s.owner.eq(&Some(caller.to_string()))
            && !s.authorized_caller.contains_key(&caller.to_string())
        {
            Err("Unauthorized!".into())
        } else {
            Ok(())
        }
    })
}

pub fn is_owner() -> Result<(), String> {
    let caller = ic_cdk::api::caller();
    with_state(|s| {
        if !s.owner.eq(&Some(caller.to_string())) {
            Err("Not Owner!".into())
        } else {
            Ok(())
        }
    })
}
