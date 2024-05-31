use crate::state::with_state;
use log::{error, info};

pub fn auth() -> Result<(), String> {
    let caller = ic_cdk::api::caller();
    info!("auth for caller: {:?}", caller.to_string());
    with_state(|s| {
        if s.admin != caller
            && !ic_cdk::api::is_controller(&caller)
            && !s.authorized_caller.contains_key(&caller.to_string())
        {
            error!("{:?} Unauthorized!", caller.to_string());
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
            error!("{:?} Not Admin!", caller.to_string());
            Err("Not Admin!".into())
        } else {
            Ok(())
        }
    })
}
