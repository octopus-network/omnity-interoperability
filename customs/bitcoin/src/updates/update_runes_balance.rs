use crate::state::{audit, FinalizedTicketStatus, GenTicketStatus, RunesBalance};
use crate::state::{mutate_state, read_state};
use crate::{management, BTC_TOKEN};
use candid::{CandidType, Deserialize};
use ic_btc_interface::{OutPoint, Txid};
use omnity_types::{Ticket, TxAction};
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
        GenTicketStatus::Invalid | GenTicketStatus::Finalized => {
            Err(UpdateRunesBalanceError::AleardyProcessed)
        }
        GenTicketStatus::Unknown => Err(UpdateRunesBalanceError::RequestNotFound),
        GenTicketStatus::Pending(req) => Ok(req),
    })?;

    let amount = args.balances.iter().map(|b| b.amount).sum::<u128>();
    let result = if args.balances.iter().all(|b| b.rune_id == req.rune_id) && amount == req.amount {
        let hub_principal = read_state(|s| s.hub_principal);
        management::send_ticket(
            hub_principal,
            Ticket {
                ticket_id: args.txid.to_string(),
                ticket_time: ic_cdk::api::time(),
                src_chain: String::from(BTC_TOKEN),
                dst_chain: req.target_chain_id.clone(),
                action: TxAction::Transfer,
                token: req.rune_id.to_string(),
                amount: req.amount.to_string(),
                sender: String::default(),
                receiver: req.receiver.clone(),
                memo: None,
            },
        )
        .await
        .map_err(|err| UpdateRunesBalanceError::SendTicketErr(format!("{}", err)))?;
        Ok(())
    } else {
        Err(UpdateRunesBalanceError::MismatchWithGenTicketReq)
    };

    mutate_state(|s| match result {
        Ok(_) => audit::finalize_ticket_request(s, &req, args.balances),
        Err(_) => audit::remove_ticket_request(s, &req, FinalizedTicketStatus::Invalid),
    });

    result
}
