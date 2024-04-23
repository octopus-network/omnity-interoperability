use crate::state::with_state;
use log::info;

pub fn auth() -> Result<(), String> {
    let caller = ic_cdk::api::caller();
    info!("auth for caller: {:?}", caller.to_string());
    with_state(|s| {
        if s.admin != caller && !s.authorized_caller.contains_key(&caller.to_string()) {
            Err("Unauthorized!".into())
        } else {
            Ok(())
        }
    })
}

pub fn is_admin() -> Result<(), String> {
    let caller = ic_cdk::api::caller();
    with_state(|s| {
        if s.admin != caller {
            Err("Not Admin!".into())
        } else {
            Ok(())
        }
    })
}
