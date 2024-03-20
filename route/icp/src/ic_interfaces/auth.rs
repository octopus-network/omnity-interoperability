use crate::*;

pub fn auth_port() -> Result {
    let caller = ic_cdk::api::caller();
    if port_addr_or_error()?.eq(&caller) {
        Ok(())
    } else {
        Err(Error::AuthError(caller.to_string()))
    }
}