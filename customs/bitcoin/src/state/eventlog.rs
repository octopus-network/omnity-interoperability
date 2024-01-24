use crate::destination::Destination;
use crate::lifecycle::init::InitArgs;
use crate::lifecycle::upgrade::UpgradeArgs;
use crate::state::{
    ChangeOutput, CustomState, FinalizedBtcRetrieval, FinalizedStatus, ReleaseTokenRequest,
    SubmittedBtcTransaction,
};
use ic_btc_interface::{OutPoint, Txid, Utxo};
use serde::{Deserialize, Serialize};

use super::{
    FinalizedBoardingPass, FinalizedBoardingPassStatus, GenBoardingPassRequest, RunesBalance,
};

#[derive(candid::CandidType, Deserialize)]
pub struct GetEventsArg {
    pub start: u64,
    pub length: u64,
}

#[derive(candid::CandidType, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Event {
    /// Indicates the minter initialization with the specified arguments.  Must be
    /// the first event in the event log.
    #[serde(rename = "init")]
    Init(InitArgs),

    /// Indicates the minter upgrade with specified arguments.
    #[serde(rename = "upgrade")]
    Upgrade(UpgradeArgs),

    /// Indicates that the customs received new UTXOs to the specified destination.
    #[serde(rename = "received_utxos")]
    ReceivedUtxos {
        /// That destination owning the UTXOs.
        #[serde(rename = "destination")]
        destination: Destination,
        #[serde(rename = "utxos")]
        utxos: Vec<Utxo>,
    },

    #[serde(rename = "received_runes_utxos")]
    ReceivedRunesToken {
        #[serde(rename = "runes")]
        balances: Vec<RunesBalance>,
        #[serde(rename = "outpoint")]
        outpoint: OutPoint,
    },

    #[serde(rename = "accepted_gen_boarding_pass_request")]
    AcceptedGenBoardingPassRequest(GenBoardingPassRequest),

    /// Indicates that the minter accepted a new retrieve_btc request.
    /// The minter emits this event _after_ it burnt ckBTC.
    #[serde(rename = "accepted_release_token_request")]
    AcceptedReleaseTokenRequest(ReleaseTokenRequest),

    /// Indicates that the minter removed a previous retrieve_btc request
    /// because the retrieval amount was not enough to cover the transaction
    /// fees.
    #[serde(rename = "removed_retrieve_btc_request")]
    RemovedRetrieveBtcRequest {
        #[serde(rename = "block_index")]
        block_index: u64,
    },

    #[serde(rename = "removed_boarding_pass_request")]
    FinalizedBoardingPassRequest {
        #[serde(rename = "tx_id")]
        tx_id: Txid,
        #[serde(rename = "status")]
        status: FinalizedBoardingPassStatus,
    },

    /// Indicates that the minter sent out a new transaction to the Bitcoin
    /// network.
    #[serde(rename = "sent_transaction")]
    SentBtcTransaction {
        /// Block indices of retrieve_btc requests that caused the transaction.
        #[serde(rename = "requests")]
        request_block_indices: Vec<u64>,
        /// The Txid of the Bitcoin transaction.
        #[serde(rename = "txid")]
        txid: Txid,
        /// UTXOs used for the transaction.
        #[serde(rename = "utxos")]
        utxos: Vec<Utxo>,
        /// The output with the minter's change, if any.
        #[serde(rename = "change_output")]
        #[serde(skip_serializing_if = "Option::is_none")]
        change_output: Option<ChangeOutput>,
        /// The IC time at which the minter submitted the transaction.
        #[serde(rename = "submitted_at")]
        submitted_at: u64,
        /// The fee per vbyte (in millisatoshi) that we used for the transaction.
        #[serde(rename = "fee")]
        #[serde(skip_serializing_if = "Option::is_none")]
        fee_per_vbyte: Option<u64>,
    },

    /// Indicates that the minter sent out a new transaction to replace an older transaction
    /// because the old transaction did not appear on the Bitcoin blockchain.
    #[serde(rename = "replaced_transaction")]
    ReplacedBtcTransaction {
        /// The Txid of the old Bitcoin transaction.
        #[serde(rename = "old_txid")]
        old_txid: Txid,
        /// The Txid of the new Bitcoin transaction.
        #[serde(rename = "new_txid")]
        new_txid: Txid,
        /// The output with the minter's change.
        #[serde(rename = "change_output")]
        change_output: ChangeOutput,
        /// The IC time at which the minter submitted the transaction.
        #[serde(rename = "submitted_at")]
        submitted_at: u64,
        /// The fee per vbyte (in millisatoshi) that we used for the transaction.
        #[serde(rename = "fee")]
        fee_per_vbyte: u64,
    },

    /// Indicates that the minter received enough confirmations for a bitcoin
    /// transaction.
    #[serde(rename = "confirmed_transaction")]
    ConfirmedBtcTransaction {
        #[serde(rename = "txid")]
        txid: Txid,
    },
}

#[derive(Debug)]
pub enum ReplayLogError {
    /// There are no events in the event log.
    EmptyLog,
    /// The event log is inconsistent.
    InconsistentLog(String),
}

/// Reconstructs the minter state from an event log.
pub fn replay(mut events: impl Iterator<Item = Event>) -> Result<CustomState, ReplayLogError> {
    let mut state = match events.next() {
        Some(Event::Init(args)) => CustomState::from(args),
        Some(evt) => {
            return Err(ReplayLogError::InconsistentLog(format!(
                "The first event is not Init: {:?}",
                evt
            )))
        }
        None => return Err(ReplayLogError::EmptyLog),
    };

    for event in events {
        match event {
            Event::Init(args) => {
                state.reinit(args);
            }
            Event::Upgrade(args) => state.upgrade(args),
            Event::ReceivedUtxos {
                destination, utxos, ..
            } => state.add_utxos(destination, utxos),
            Event::ReceivedRunesToken {
                balances: runes,
                outpoint,
            } => {
                state.update_runes_balance(&outpoint, runes);
            }
            Event::AcceptedGenBoardingPassRequest(req) => {
                state.pending_boarding_pass_requests.insert(req.tx_id, req);
            }
            Event::FinalizedBoardingPassRequest { tx_id, status } => {
                let req = state
                    .pending_boarding_pass_requests
                    .remove(&tx_id)
                    .ok_or_else(|| {
                        ReplayLogError::InconsistentLog(format!(
                            "Attempted to remove a non-pending boarding pass request {}",
                            tx_id
                        ))
                    })?;
                state.push_finalized_boarding_pass(FinalizedBoardingPass {
                    request: req,
                    status,
                });
            }
            Event::AcceptedReleaseTokenRequest(req) => {
                state.push_back_pending_request(req);
            }
            Event::RemovedRetrieveBtcRequest { block_index } => {
                let request = state.remove_pending_request(block_index).ok_or_else(|| {
                    ReplayLogError::InconsistentLog(format!(
                        "Attempted to remove a non-pending retrieve_btc request {}",
                        block_index
                    ))
                })?;

                state.push_finalized_release_token(FinalizedBtcRetrieval {
                    request,
                    status: FinalizedStatus::AmountTooLow,
                })
            }
            Event::SentBtcTransaction {
                request_block_indices,
                txid,
                utxos,
                fee_per_vbyte,
                change_output,
                submitted_at,
            } => {
                let mut retrieve_btc_requests = Vec::with_capacity(request_block_indices.len());
                for block_index in request_block_indices {
                    let request = state.remove_pending_request(block_index).ok_or_else(|| {
                        ReplayLogError::InconsistentLog(format!(
                            "Attempted to send a non-pending retrieve_btc request {}",
                            block_index
                        ))
                    })?;
                    retrieve_btc_requests.push(request);
                }
                for utxo in utxos.iter() {
                    state.available_utxos.remove(&utxo.outpoint);
                }
                state.push_submitted_transaction(SubmittedBtcTransaction {
                    requests: retrieve_btc_requests,
                    txid,
                    used_utxos: utxos,
                    fee_per_vbyte,
                    change_output,
                    submitted_at,
                });
            }
            Event::ReplacedBtcTransaction {
                old_txid,
                new_txid,
                change_output,
                submitted_at,
                fee_per_vbyte,
            } => {
                let (requests, used_utxos) = match state
                    .submitted_transactions
                    .iter()
                    .find(|tx| tx.txid == old_txid)
                {
                    Some(tx) => (tx.requests.clone(), tx.used_utxos.clone()),
                    None => {
                        return Err(ReplayLogError::InconsistentLog(format!(
                            "Cannot replace a non-existent transaction {}",
                            &old_txid
                        )))
                    }
                };

                state.replace_transaction(
                    &old_txid,
                    SubmittedBtcTransaction {
                        txid: new_txid,
                        requests,
                        used_utxos,
                        change_output: Some(change_output),
                        submitted_at,
                        fee_per_vbyte: Some(fee_per_vbyte),
                    },
                );
            }
            Event::ConfirmedBtcTransaction { txid } => {
                state.finalize_transaction(&txid);
            }
        }
    }

    Ok(state)
}
