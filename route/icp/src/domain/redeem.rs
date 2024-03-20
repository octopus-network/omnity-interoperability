use crate::*;


pub(crate) fn generate_ticket() -> Result {
    // assert caller is port canister
  

    Ok(())

}


pub(crate) async fn send_ticket(ticket: Ticket) -> Result {

    let (_,): ((),) = ic_cdk::call(
        hub_addr_or_error()?, 
        "send_ticket", 
        (ticket,) 
    ).await.map_err(|e| Error::IcCallError(e.0, e.1))?;

    Ok(())

}
