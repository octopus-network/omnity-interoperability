use crate::address::main_destination;
use crate::state::{audit, RunesBalance};
use crate::state::{mutate_state, read_state, FinalizedBoardingPassStatus};
use candid::{CandidType, Deserialize};
use ic_btc_interface::{OutPoint, Txid};
use serde::Serialize;

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct UpdateRunesBlanceArgs {
    pub tx_id: Txid,
    pub vout: u32,
    pub balances: Vec<RunesBalance>,
}

#[derive(CandidType, Clone, Debug, Deserialize, PartialEq, Eq)]
pub enum UpdateRunesBalanceError {
    BalanceIsEmpty,
    PendingReqNotFound,
    UtxoNotFound,
}

pub async fn update_runes_balance(
    args: UpdateRunesBlanceArgs,
) -> Result<(), UpdateRunesBalanceError> {
    if args.balances.is_empty() {
        return Err(UpdateRunesBalanceError::BalanceIsEmpty);
    }

    let outpoint = OutPoint {
        txid: args.tx_id,
        vout: args.vout,
    };
    let dest = read_state(|s| match s.outpoint_destination.get(&outpoint) {
        Some(dest) => Ok(dest.clone()),
        None => Err(UpdateRunesBalanceError::UtxoNotFound),
    })?;

    mutate_state(|s| audit::update_runes_balance(s, &outpoint, args.balances));
    if dest.eq(&main_destination()) {
        return Ok(());
    }

    let req = read_state(
        |s| match s.pending_boarding_pass_requests.get(&args.tx_id) {
            Some(req) => Ok(req.clone()),
            None => Err(UpdateRunesBalanceError::PendingReqNotFound),
        },
    )?;

    // TODO invoke hub to generate landing pass

    mutate_state(|s| s.pending_boarding_pass_requests.remove(&args.tx_id));

    mutate_state(|s| {
        audit::finalize_boarding_pass_request(s, &req, FinalizedBoardingPassStatus::Finalized)
    });

    Ok(())
}
