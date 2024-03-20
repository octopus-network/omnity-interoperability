use crate::*;

pub fn convert_ticket_to_transfer_arg(ticket: Ticket) -> Result<TransferArg> {

    Ok(TransferArg {
        amount: NumTokens::from(ticket.amount.parse::<u128>().map_err(|e| Error::Custom(e.into()) )?),
        from_subaccount: None,
        to: Account { 
            owner: Principal::from_text(ticket.receiver).map_err(|e| Error::Custom(e.into()) )?, 
            subaccount: None 
        },
        fee: None,
        created_at_time: None,
        memo: None,
    })

}