use crate::hub;
use crate::state::{audit, GenTicketStatus, RunesBalance};
use crate::state::{mutate_state, read_state};
use candid::{CandidType, Deserialize};
use ic_btc_interface::{OutPoint, Txid};
use omnity_types::{Ticket, TicketType, TxAction};
use serde::Serialize;

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct UpdateRunesBalanceArgs {
    pub txid: Txid,
    pub balances: Vec<RunesBalance>,
}

#[derive(CandidType, Clone, Debug, Deserialize, PartialEq, Eq)]
pub enum UpdateRunesBalanceError {
    RequestNotFound,
    AleardyProcessed,
    MismatchWithGenTicketReq,
    UtxoNotFound,
    SendTicketErr(String),
}

pub async fn update_runes_balance(
    args: UpdateRunesBalanceArgs,
) -> Result<(), UpdateRunesBalanceError> {
    for balance in &args.balances {
        let outpoint = OutPoint {
            txid: args.txid,
            vout: balance.vout,
        };
        read_state(|s| match s.outpoint_destination.get(&outpoint) {
            Some(_) => Ok(()),
            None => Err(UpdateRunesBalanceError::UtxoNotFound),
        })?;
    }

    let req = read_state(|s| match s.generate_ticket_status(args.txid) {
        GenTicketStatus::Finalized => Err(UpdateRunesBalanceError::AleardyProcessed),
        GenTicketStatus::Unknown => Err(UpdateRunesBalanceError::RequestNotFound),
        GenTicketStatus::Pending(req) => Ok(req),
    })?;

    let amount = args.balances.iter().map(|b| b.amount).sum::<u128>();
    if amount != req.amount || args.balances.iter().any(|b| b.rune_id != req.rune_id) {
        mutate_state(|s| audit::remove_pending_request(s, &req.txid));
        return Err(UpdateRunesBalanceError::MismatchWithGenTicketReq);
    }

    let (hub_principal, chain_id) = read_state(|s| (s.hub_principal, s.chain_id.clone()));
    hub::send_ticket(
        hub_principal,
        Ticket {
            ticket_id: args.txid.to_string(),
            ticket_type: TicketType::Normal,
            ticket_time: ic_cdk::api::time(),
            src_chain: chain_id,
            dst_chain: req.target_chain_id.clone(),
            action: TxAction::Transfer,
            token: req.token_id.clone(),
            amount: req.amount.to_string(),
            sender: None,
            receiver: req.receiver.clone(),
            memo: None,
        },
    )
    .await
    .map_err(|err| UpdateRunesBalanceError::SendTicketErr(format!("{}", err)))?;

    mutate_state(|s| audit::finalize_ticket_request(s, &req, args.balances));

    Ok(())
}
