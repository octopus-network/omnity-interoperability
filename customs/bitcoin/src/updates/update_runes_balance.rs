use crate::hub;
use crate::logs::P1;
use crate::state::{audit, GenTicketStatus, RunesBalance};
use crate::state::{mutate_state, read_state};
use candid::{CandidType, Deserialize};
use ic_btc_interface::{OutPoint, Txid};
use ic_canister_log::log;
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
    FinalizeTicketErr(String),
    RequestNotConfirmed,
    BalancesIsEmpty,
}

pub async fn update_runes_balance(
    args: UpdateRunesBalanceArgs,
) -> Result<(), UpdateRunesBalanceError> {
    if args.balances.len() == 0 {
        return Err(UpdateRunesBalanceError::BalancesIsEmpty);
    }

    let req = read_state(|s| match s.generate_ticket_status(args.txid) {
        GenTicketStatus::Finalized(_) => Err(UpdateRunesBalanceError::AleardyProcessed),
        GenTicketStatus::Unknown => Err(UpdateRunesBalanceError::RequestNotFound),
        GenTicketStatus::Pending(_) => Err(UpdateRunesBalanceError::RequestNotConfirmed),
        GenTicketStatus::Confirmed(req) => Ok(req),
    })?;

    for balance in &args.balances {
        let outpoint = OutPoint {
            txid: args.txid,
            vout: balance.vout,
        };
        if req
            .new_utxos
            .iter()
            .find(|u| u.outpoint == outpoint)
            .is_none()
        {
            return Err(UpdateRunesBalanceError::UtxoNotFound);
        }
    }

    let amount = args.balances.iter().map(|b| b.amount).sum::<u128>();
    if amount != req.amount || args.balances.iter().any(|b| b.rune_id != req.rune_id) {
        mutate_state(|s| audit::remove_confirmed_request(s, &req.txid));
        return Err(UpdateRunesBalanceError::MismatchWithGenTicketReq);
    }

    let hub_principal = read_state(|s| s.hub_principal);
    hub::finalize_ticket(hub_principal, args.txid.to_string())
        .await
        .map_err(|err| UpdateRunesBalanceError::FinalizeTicketErr(format!("{}", err)))?;

    log!(
        P1,
        "[update_runes_balance] send ticket to hub: {}",
        args.txid.to_string()
    );

    mutate_state(|s| audit::finalize_ticket_request(s, &req, args.balances));

    Ok(())
}
