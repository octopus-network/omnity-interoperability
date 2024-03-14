use log::info;

use crate::with_state;

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

// #[update(guard = "is_owner")]
// pub async fn set_whitelist(principal: Principal, authorized: bool) -> Result<(), Error> {
//     info!("principal: {principal:?}, authorized {authorized:?}");
//     if authorized {
//         with_state_mut(|s| s.authorized_caller.insert(principal.to_string()));
//     } else {
//         with_state_mut(|s| s.authorized_caller.remove(&principal.to_string()));
//     }
//     Ok(())
// }

// #[update(guard = "is_owner")]
// pub async fn set_owner(principal: Principal) -> Result<(), Error> {
//     with_state_mut(|s| s.owner = Some(principal.to_string()));
//     info!("new owner: {principal:?}");
//     Ok(())
// }
