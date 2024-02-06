use crate::state::{audit, FinalizedTicketStatus, GenTicketStatus, RunesBalance};
use crate::state::{mutate_state, read_state};
use candid::{CandidType, Deserialize};
use ic_btc_interface::{OutPoint, Txid};
use serde::Serialize;

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct UpdateRunesBlanceArgs {
    pub tx_id: Txid,
    pub vout: u32,
    pub balance: RunesBalance,
}

#[derive(CandidType, Clone, Debug, Deserialize, PartialEq, Eq)]
pub enum UpdateRunesBalanceError {
    RequestNotFound,
    AleardyProcessed,
    MismatchWithTicketReq,
    UtxoNotFound,
}

pub async fn update_runes_balance(
    args: UpdateRunesBlanceArgs,
) -> Result<(), UpdateRunesBalanceError> {
    let outpoint = OutPoint {
        txid: args.tx_id,
        vout: args.vout,
    };
    read_state(|s| match s.outpoint_destination.get(&outpoint) {
        Some(_) => Ok(()),
        None => Err(UpdateRunesBalanceError::UtxoNotFound),
    })?;

    let req = read_state(|s| match s.generate_ticket_status(args.tx_id) {
        GenTicketStatus::Finalized => Err(UpdateRunesBalanceError::AleardyProcessed),
        GenTicketStatus::Unknown => Err(UpdateRunesBalanceError::RequestNotFound),
        GenTicketStatus::Pending(req) => Ok(req),
    })?;

    let result = {
        // TODO invoke hub to generate landing pass
        if args.balance.runes_id != req.runes_id || args.balance.value != req.value {
            Err(UpdateRunesBalanceError::MismatchWithTicketReq)
        } else {
            Ok(())
        }
    };

    mutate_state(|s| match result {
        Ok(_) => audit::finalize_ticket_request(s, &req, args.vout),
        Err(UpdateRunesBalanceError::MismatchWithTicketReq) => {
            audit::remove_ticket_request(s, &req, FinalizedTicketStatus::MismatchWithTicketReq)
        }
        _ => {}
    });

    result
}
