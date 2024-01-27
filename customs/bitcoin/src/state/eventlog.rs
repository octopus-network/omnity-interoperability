use crate::destination::Destination;
use crate::lifecycle::init::InitArgs;
use crate::lifecycle::upgrade::UpgradeArgs;
use crate::state::{
    CustomState, FinalizedBtcRetrieval, FinalizedStatus, ReleaseTokenRequest, RunesChangeOutput,
    SubmittedBtcTransaction,
};
use ic_btc_interface::{OutPoint, Txid, Utxo};
use serde::{Deserialize, Serialize};

use super::{
    BtcChangeOutput, FinalizedTicket, FinalizedTicketStatus, GenTicketRequest, RunesBalance,
    RunesId, RunesUtxo,
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
        #[serde(rename = "is_runes")]
        is_runes: bool,
    },

    #[serde(rename = "received_runes_utxos")]
    ReceivedRunesToken {
        #[serde(rename = "outpoint")]
        outpoint: OutPoint,
        #[serde(rename = "balance")]
        balance: RunesBalance,
    },

    #[serde(rename = "accepted_gen_boarding_pass_request")]
    AcceptedGenTicketRequest(GenTicketRequest),

    /// Indicates that the minter accepted a new retrieve_btc request.
    /// The minter emits this event _after_ it burnt ckBTC.
    #[serde(rename = "accepted_release_token_request")]
    AcceptedReleaseTokenRequest(ReleaseTokenRequest),

    /// Indicates that the minter removed a previous retrieve_btc request
    /// because the retrieval amount was not enough to cover the transaction
    /// fees.
    #[serde(rename = "removed_release_token_request")]
    RemovedReleaseTokenRequest {
        #[serde(rename = "release_id")]
        release_id: Vec<u8>,
    },

    #[serde(rename = "finalized_ticket_request")]
    FinalizedTicketRequest {
        #[serde(rename = "tx_id")]
        tx_id: Txid,
        #[serde(rename = "status")]
        status: FinalizedTicketStatus,
    },

    /// Indicates that the minter sent out a new transaction to the Bitcoin
    /// network.
    #[serde(rename = "sent_transaction")]
    SentBtcTransaction {
        #[serde(rename = "runes_id")]
        runes_id: RunesId,
        /// Release id list of release_token requests that caused the transaction.
        #[serde(rename = "requests")]
        request_release_ids: Vec<Vec<u8>>,
        /// The Txid of the Bitcoin transaction.
        #[serde(rename = "txid")]
        txid: Txid,
        /// Runes UTXOs used for the transaction.
        #[serde(rename = "runes_utxos")]
        runes_utxos: Vec<RunesUtxo>,
        /// BTC UTXOs used for the transaction.
        #[serde(rename = "btc_utxos")]
        btc_utxos: Vec<Utxo>,
        /// The output with the minter's change, if any.
        #[serde(rename = "runes_change_output")]
        runes_change_output: RunesChangeOutput,
        #[serde(rename = "btc_change_output")]
        #[serde(skip_serializing_if = "Option::is_none")]
        btc_change_output: Option<BtcChangeOutput>,
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
        #[serde(rename = "runes_change_output")]
        runes_change_output: RunesChangeOutput,
        #[serde(rename = "btc_change_output")]
        btc_change_output: Option<BtcChangeOutput>,
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
                destination,
                utxos,
                is_runes,
            } => state.add_utxos(destination, utxos, is_runes),
            Event::ReceivedRunesToken { outpoint, balance } => {
                state.update_runes_balance(outpoint, balance);
            }
            Event::AcceptedGenTicketRequest(req) => {
                state.pending_gen_ticket_requests.insert(req.tx_id, req);
            }
            Event::FinalizedTicketRequest { tx_id, status } => {
                let req = state
                    .pending_gen_ticket_requests
                    .remove(&tx_id)
                    .ok_or_else(|| {
                        ReplayLogError::InconsistentLog(format!(
                            "Attempted to remove a non-pending boarding pass request {}",
                            tx_id
                        ))
                    })?;
                state.push_finalized_boarding_pass(FinalizedTicket {
                    request: req,
                    status,
                });
            }
            Event::AcceptedReleaseTokenRequest(req) => {
                state.push_back_pending_request(req);
            }
            Event::RemovedReleaseTokenRequest { release_id } => {
                let request = state.remove_pending_request(release_id.clone()).ok_or_else(|| {
                    ReplayLogError::InconsistentLog(format!(
                        "Attempted to remove a non-pending release_token request {:?}",
                        release_id
                    ))
                })?;

                state.push_finalized_release_token(FinalizedBtcRetrieval {
                    request,
                    status: FinalizedStatus::AmountTooLow,
                })
            }
            Event::SentBtcTransaction {
                runes_id,
                request_release_ids,
                txid,
                runes_utxos,
                btc_utxos,
                fee_per_vbyte,
                runes_change_output,
                btc_change_output,
                submitted_at,
            } => {
                let mut release_token_requests = Vec::with_capacity(request_release_ids.len());
                for release_id in request_release_ids {
                    let request = state.remove_pending_request(release_id.clone()).ok_or_else(|| {
                        ReplayLogError::InconsistentLog(format!(
                            "Attempted to send a non-pending retrieve_btc request {:?}",
                            release_id
                        ))
                    })?;
                    release_token_requests.push(request);
                }
                for utxo in runes_utxos.iter() {
                    state.available_runes_utxos.remove(utxo);
                }
                for utxo in btc_utxos.iter() {
                    state.available_btc_utxos.remove(utxo);
                }
                state.push_submitted_transaction(SubmittedBtcTransaction {
                    runes_id,
                    requests: release_token_requests,
                    txid,
                    runes_utxos,
                    btc_utxos,
                    fee_per_vbyte,
                    runes_change_output,
                    btc_change_output,
                    submitted_at,
                });
            }
            Event::ReplacedBtcTransaction {
                old_txid,
                new_txid,
                runes_change_output,
                btc_change_output,
                submitted_at,
                fee_per_vbyte,
            } => {
                let (requests, runes_utxos, btc_utxos) = match state
                    .submitted_transactions
                    .iter()
                    .find(|tx| tx.txid == old_txid)
                {
                    Some(tx) => (
                        tx.requests.clone(),
                        tx.runes_utxos.clone(),
                        tx.btc_utxos.clone(),
                    ),
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
                        runes_id: runes_change_output.runes_id,
                        txid: new_txid,
                        requests,
                        runes_utxos,
                        btc_utxos,
                        runes_change_output,
                        btc_change_output,
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
