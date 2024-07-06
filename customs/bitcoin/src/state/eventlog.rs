use super::{
    BtcChangeOutput, GenTicketRequest, GenTicketRequestV2, RuneId, RuneTxRequest, RunesBalance,
    RunesUtxo, SubmittedBtcTransactionV2, RUNES_TOKEN,
};
use crate::destination::Destination;
use crate::lifecycle::init::InitArgs;
use crate::lifecycle::upgrade::UpgradeArgs;
use crate::state::{CustomsState, ReleaseTokenRequest, RunesChangeOutput};
use ic_btc_interface::{Txid, Utxo};
use omnity_types::{Chain, TicketId, ToggleState, Token};
use serde::{Deserialize, Serialize};

#[derive(candid::CandidType, Deserialize)]
pub struct GetEventsArg {
    pub start: u64,
    pub length: u64,
}

#[derive(candid::CandidType, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Event {
    /// Indicates the customs initialization with the specified arguments.  Must be
    /// the first event in the event log.
    #[serde(rename = "init")]
    Init(InitArgs),

    /// Indicates the customs upgrade with specified arguments.
    #[serde(rename = "upgrade")]
    Upgrade(UpgradeArgs),

    #[serde(rename = "added_chain")]
    AddedChain(Chain),

    #[serde(rename = "added_token")]
    AddedToken { rune_id: RuneId, token: Token },

    #[serde(rename = "toggle_chain_state")]
    ToggleChainState(ToggleState),

    #[serde(rename = "update_next_directive_seq")]
    UpdateNextDirectiveSeq(u64),

    #[serde(rename = "update_next_ticket_seq")]
    UpdateNextTicketSeq(u64),

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

    #[serde(rename = "updated_runes_balance")]
    UpdatedRunesBalance {
        #[serde(rename = "txid")]
        txid: Txid,
        #[serde(rename = "balance")]
        balance: RunesBalance,
    },

    #[serde(rename = "accepted_generate_ticket_request")]
    AcceptedGenTicketRequest(GenTicketRequest),

    #[serde(rename = "accepted_generate_ticket_request_v2")]
    AcceptedGenTicketRequestV2(GenTicketRequestV2),

    /// Indicates that the customs accepted a new release_token request.
    /// The customs emits this event _after_ it send to hub.
    #[serde(rename = "accepted_release_token_request")]
    AcceptedReleaseTokenRequest(ReleaseTokenRequest),

    #[serde(rename = "accepted_rune_tx_request")]
    AcceptedRuneTxRequest(RuneTxRequest),

    #[serde(rename = "finalized_ticket_request")]
    FinalizedTicketRequest {
        #[serde(rename = "txid")]
        txid: Txid,
        #[serde(rename = "balances")]
        balances: Vec<RunesBalance>,
    },

    #[serde(rename = "removed_ticket_request")]
    RemovedTicketRequest {
        #[serde(rename = "txid")]
        txid: Txid,
    },

    /// Indicates that the  sent out a new transaction to the Bitcoin
    /// network.
    #[serde(rename = "sent_transaction")]
    SentBtcTransaction {
        #[serde(rename = "rune_id")]
        rune_id: RuneId,
        /// Release id list of release_token requests that caused the transaction.
        #[serde(rename = "requests")]
        request_release_ids: Vec<TicketId>,
        /// The Txid of the Bitcoin transaction.
        #[serde(rename = "txid")]
        txid: Txid,
        /// Runes UTXOs used for the transaction.
        #[serde(rename = "runes_utxos")]
        runes_utxos: Vec<RunesUtxo>,
        /// BTC UTXOs used for the transaction.
        #[serde(rename = "btc_utxos")]
        btc_utxos: Vec<Utxo>,
        /// The output with the customs's change, if any.
        #[serde(rename = "runes_change_output")]
        runes_change_output: RunesChangeOutput,
        #[serde(rename = "btc_change_output")]
        btc_change_output: BtcChangeOutput,
        /// The IC time at which the customs submitted the transaction.
        #[serde(rename = "submitted_at")]
        submitted_at: u64,
        /// The fee per vbyte (in millisatoshi) that we used for the transaction.
        #[serde(rename = "fee")]
        #[serde(skip_serializing_if = "Option::is_none")]
        fee_per_vbyte: Option<u64>,
    },

    /// Indicates that the customs sent out a new transaction to replace an older transaction
    /// because the old transaction did not appear on the Bitcoin blockchain.
    #[serde(rename = "replaced_transaction")]
    ReplacedBtcTransaction {
        /// The Txid of the old Bitcoin transaction.
        #[serde(rename = "old_txid")]
        old_txid: Txid,
        /// The Txid of the new Bitcoin transaction.
        #[serde(rename = "new_txid")]
        new_txid: Txid,
        /// The output with the customs's change.
        #[serde(rename = "runes_change_output")]
        runes_change_output: RunesChangeOutput,
        #[serde(rename = "btc_change_output")]
        btc_change_output: BtcChangeOutput,
        /// The IC time at which the customs submitted the transaction.
        #[serde(rename = "submitted_at")]
        submitted_at: u64,
        /// The fee per vbyte (in millisatoshi) that we used for the transaction.
        #[serde(rename = "fee")]
        fee_per_vbyte: u64,
    },

    /// Indicates that the customs received enough confirmations for a bitcoin
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

/// Reconstructs the customs state from an event log.
pub fn replay(mut events: impl Iterator<Item = Event>) -> Result<CustomsState, ReplayLogError> {
    let mut state = match events.next() {
        Some(Event::Init(args)) => CustomsState::from(args),
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
            Event::AddedChain(chain) => {
                state.counterparties.insert(chain.chain_id.clone(), chain);
            }
            Event::AddedToken { rune_id, token } => {
                state
                    .tokens
                    .insert(token.token_id.clone(), (rune_id, token));
            }
            Event::ToggleChainState(toggle) => {
                if toggle.chain_id == state.chain_id {
                    state.chain_state = toggle.action.into();
                } else if let Some(chain) = state.counterparties.get_mut(&toggle.chain_id) {
                    chain.chain_state = toggle.action.into();
                }
            }
            Event::UpdateNextDirectiveSeq(next_seq) => {
                assert!(next_seq > state.next_directive_seq);
                state.next_directive_seq = next_seq;
            }
            Event::UpdateNextTicketSeq(next_seq) => {
                assert!(next_seq > state.next_ticket_seq);
                state.next_ticket_seq = next_seq;
            }
            Event::ReceivedUtxos {
                destination,
                utxos,
                is_runes,
            } => state.add_utxos(destination, utxos, is_runes),
            Event::UpdatedRunesBalance { txid, balance } => {
                state.update_runes_balance(txid, balance);
            }
            Event::AcceptedGenTicketRequest(req) => {
                // There is no need to add utxos here, because in previous versions,
                // A ReceivedUtxos Event will be emitted at the same time.
                state
                    .pending_gen_ticket_requests
                    .insert(req.txid, req.into());
            }
            Event::AcceptedGenTicketRequestV2(req) => {
                let new_utxos = req.new_utxos.clone();
                let dest = Destination {
                    target_chain_id: req.target_chain_id.clone(),
                    receiver: req.receiver.clone(),
                    token: Some(RUNES_TOKEN.into()),
                };
                state.pending_gen_ticket_requests.insert(req.txid, req);
                state.add_utxos(dest, new_utxos, true);
            }
            Event::RemovedTicketRequest { txid } => {
                let req = state
                    .pending_gen_ticket_requests
                    .remove(&txid)
                    .ok_or_else(|| {
                        ReplayLogError::InconsistentLog(format!(
                            "Attempted to remove a non-pending generate ticket request {}",
                            txid
                        ))
                    })?;
                for utxo in &req.new_utxos {
                    state.forget_utxo(utxo);
                }
            }
            Event::FinalizedTicketRequest { txid, balances } => {
                let request = state
                    .pending_gen_ticket_requests
                    .remove(&txid)
                    .ok_or_else(|| {
                        ReplayLogError::InconsistentLog(format!(
                            "Attempted to remove a non-pending generate ticket request {}",
                            txid
                        ))
                    })?;
                for balance in balances {
                    state.update_runes_balance(txid, balance);
                }
                state.push_finalized_ticket(request);
            }
            Event::AcceptedReleaseTokenRequest(req) => {
                state.push_back_pending_request(req.into());
            }
            Event::AcceptedRuneTxRequest(req) => {
                state.push_back_pending_request(req);
            }
            Event::SentBtcTransaction {
                rune_id,
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
                    let request = state
                        .remove_pending_request(release_id.clone())
                        .ok_or_else(|| {
                            ReplayLogError::InconsistentLog(format!(
                                "Attempted to send a non-pending release_token request {:?}",
                                release_id
                            ))
                        })?;
                    release_token_requests.push(request);
                }
                for utxo in runes_utxos.iter() {
                    state.available_runes_utxos.remove(utxo);
                }
                for utxo in btc_utxos.iter() {
                    state.available_fee_utxos.remove(utxo);
                }
                state.push_submitted_transaction(SubmittedBtcTransactionV2 {
                    rune_id,
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
                    SubmittedBtcTransactionV2 {
                        rune_id: runes_change_output.rune_id.clone(),
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
