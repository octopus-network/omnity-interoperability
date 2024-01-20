use super::gen_boarding_pass::GenBoardingPassError;
use crate::state;
use crate::state::{mutate_state, read_state, RunesUtxo};
use candid::{CandidType, Deserialize};
use ic_btc_interface::{OutPoint, Txid};
use serde::Serialize;

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct FinalizeBoardingPassArgs {
    pub tx_id: Txid,
    pub runes_utxos: Vec<RunesUtxo>,
}

pub async fn finalize_boarding_pass(
    args: FinalizeBoardingPassArgs,
) -> Result<(), GenBoardingPassError> {
    state::read_state(|s| s.mode.is_transport_available_for())
        .map_err(GenBoardingPassError::TemporarilyUnavailable)?;

    let req = read_state(|s| match s.pending_transport_requests.get(&args.tx_id) {
        Some(req) => Ok(req.clone()),
        None => Err(GenBoardingPassError::PendingReqNotFound),
    })?;


    for utxo in &args.runes_utxos {
        if !state::read_state(|s| {
            s.outpoint_destination.contains_key(&OutPoint {
                txid: args.tx_id,
                vout: utxo.vout,
            })
        }) {
            return Err(GenBoardingPassError::UtxoNotFound);
        }
    }

    // TODO invoke hub to generate landing pass

    mutate_state(|s| {
        for utxo in &args.runes_utxos {
            s.available_runes_utxos
                .entry(utxo.token.clone())
                .or_default()
                .insert(utxo.clone());
        }
        s.pending_transport_requests.remove(&args.tx_id);
        // TODO add to finalize requests
    });

    Ok(())
}
