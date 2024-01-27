use crate::state::{audit, RunesBalance};
use crate::state::{mutate_state, read_state, FinalizedTicketStatus};
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
    PendingReqNotFound,
    MismatchWithPendingReq,
    UtxoNotFound,
}

pub async fn update_runes_balance(
    args: UpdateRunesBlanceArgs,
) -> Result<(), UpdateRunesBalanceError> {
    read_state(|s| {
        match s.outpoint_destination.get(&OutPoint {
            txid: args.tx_id,
            vout: args.vout,
        }) {
            Some(dest) => Ok(dest.clone()),
            None => Err(UpdateRunesBalanceError::UtxoNotFound),
        }
    })?;

    let req = read_state(|s| match s.pending_gen_ticket_requests.get(&args.tx_id) {
        Some(req) => Ok(req.clone()),
        None => Err(UpdateRunesBalanceError::PendingReqNotFound),
    })?;

    if args.balance.rune_id != req.runes_id || args.balance.value != req.amount {
        return Err(UpdateRunesBalanceError::MismatchWithPendingReq);
    }

    // TODO invoke hub to generate landing pass

    mutate_state(|s| {
        audit::update_runes_balance(
            s,
            OutPoint {
                txid: args.tx_id,
                vout: args.vout,
            },
            args.balance.clone(),
        );

        s.pending_gen_ticket_requests.remove(&args.tx_id);
        audit::finalize_ticket_request(s, &req, FinalizedTicketStatus::Finalized);
    });

    Ok(())
}
