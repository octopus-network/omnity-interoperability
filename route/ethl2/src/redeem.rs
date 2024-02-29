use super::Error;
use crate::types::Ticket;
use candid::Principal;

pub(crate) async fn deliver(hub: Principal, ticket: Ticket) -> Result<(), Error> {
    ic_cdk::call(hub, "deliver", ())
        .await
        .map_err(|(_, s)| Error::HubError(s))?;
    Ok(())
}

pub(crate) async fn check_witnesses(rpc: Principal) -> Result<(), Error> {
    Ok(())
}
