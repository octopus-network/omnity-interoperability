use candid::{CandidType, Deserialize};
use ic_btc_interface::Txid;
use serde::Serialize;
use std::str::FromStr;

use crate::state::{audit, mutate_state, read_state, RuneId};

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct UpdatePendingTicketArgs {
    pub txid: String,
    pub rune_id: Option<String>,
    pub amount: Option<u128>,
}

#[derive(CandidType, Clone, Debug, Deserialize, PartialEq, Eq)]
pub enum UpdatePendingTicketError {
    InvalidTxId,
    TicketNotFound,
    InvalidRuneId(String),
}

pub async fn update_pending_ticket(
    args: UpdatePendingTicketArgs,
) -> Result<(), UpdatePendingTicketError> {
    let txid = Txid::from_str(&args.txid).map_err(|_| UpdatePendingTicketError::InvalidTxId)?;

    let mut request = read_state(|s| match s.pending_gen_ticket_requests.get(&txid) {
        Some(request) => Ok(request.clone()),
        None => Err(UpdatePendingTicketError::TicketNotFound),
    })?;

    if let Some(rune_id) = args.rune_id {
        let rune_id = RuneId::from_str(&rune_id)
            .map_err(|e| UpdatePendingTicketError::InvalidRuneId(e.to_string()))?;
        request.rune_id = rune_id;
    }

    if let Some(amount) = args.amount {
        request.amount = amount;
    }

    mutate_state(|s| audit::accept_generate_ticket_request(s, request));
    Ok(())
}
