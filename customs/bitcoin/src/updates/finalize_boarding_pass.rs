use super::gen_boarding_pass::GenBoardingPassError;
use crate::state::{self, FinalizedBoardingPass};
use crate::state::{mutate_state, read_state, FinalizedBoardingPassStatus, RunesUtxo};
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

    let req = read_state(
        |s| match s.pending_boarding_pass_requests.get(&args.tx_id) {
            Some(req) => Ok(req.clone()),
            None => Err(GenBoardingPassError::PendingReqNotFound),
        },
    )?;

    let result = {
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
        Ok(())
    };

    mutate_state(|s| s.pending_boarding_pass_requests.remove(&args.tx_id));

    match result {
        Ok(()) => {
            mutate_state(|s| {
                for utxo in &args.runes_utxos {
                    s.available_runes_utxos
                        .entry(utxo.rune_id.clone())
                        .or_default()
                        .insert(utxo.clone());
                }
                s.finalize_boarding_pass_request(FinalizedBoardingPass {
                    request: req,
                    state: FinalizedBoardingPassStatus::Finalized,
                })
            });
        }
        Err(GenBoardingPassError::UtxoNotFound) => mutate_state(|s| {
            s.finalize_boarding_pass_request(FinalizedBoardingPass {
                request: req,
                state: FinalizedBoardingPassStatus::UtxoNotFound,
            })
        }),
        _ => {}
    }

    Ok(())
}
