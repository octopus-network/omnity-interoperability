use super::Error;
use candid::Principal;
use omnity_types::*;

pub(crate) async fn deliver(hub: Principal, ticket: Ticket) -> Result<(), Error> {
    ic_cdk::call(hub, "deliver", ())
        .await
        .map_err(|(_, s)| Error::HubOffline(s))?;
    Ok(())
}

pub(crate) async fn check_witnesses(rpc: Principal) -> Result<(), Error> {
    Ok(())
}
