use crate::{util::convert_ticket_to_transfer_arg, *};

pub(crate) async fn query_tickets(target: ChainId, seq: u64) -> Result<BTreeMap<u64, Ticket>> {
    let (r,): (BTreeMap<u64, Ticket>,) =
    ic_cdk::call(hub_addr_or_error()?, "query_tickets", (target, seq, seq + ticket_query_limit() as u64,))
        .await
        .map_err(|(_, e)| Error::HubError(e))?;
    Ok(r)
}

pub(crate) async fn send_transaction_by_ticket(ticket: Ticket) -> omnity_route_common::error::Result {

    let (_,): (BlockIndex,) = ic_cdk::call(
        port_addr_or_error()?, 
        "icrc1_transfer", 
        (convert_ticket_to_transfer_arg(ticket)?,) 
    ).await.map_err(|e| Error::IcCallError(e.0, e.1))?;

    Ok(())
}

pub(crate) async fn transport() -> omnity_route_common::error::Result {
    let seq = ticket_sequence();
    let chain_id = target_chain_id();

    let ticket_map:BTreeMap<u64, Ticket> = query_tickets(chain_id, seq).await?;

    for (sequence, ticket) in ticket_map.into_iter() {
        match send_transaction_by_ticket(ticket).await {
            Ok(_) => {
                set_ticket_sequence(sequence);
            }
            Err(error) => {
                return Err(error);
            }
        }
    }
    Ok(())
}
