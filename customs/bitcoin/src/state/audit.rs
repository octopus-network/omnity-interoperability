//! State modifications that should end up in the event log.

use super::{
    eventlog::Event, CustomState, FinalizedBtcRetrieval, FinalizedStatus, ReleaseTokenRequest,
    SubmittedBtcTransaction,
};
use crate::destination::Destination;
use crate::storage::record_event;
use ic_btc_interface::{Txid, Utxo};

pub fn accept_release_token_request(state: &mut CustomState, request: ReleaseTokenRequest) {
    record_event(&Event::AcceptedReleaseTokenRequest(request.clone()));
    state.pending_release_token_requests.push(request.clone());
}

pub fn add_utxos(
    state: &mut CustomState,
    destination: Destination,
    utxos: Vec<Utxo>,
) {
    record_event(&Event::ReceivedUtxos {
        destination: destination,
        utxos: utxos.clone(),
    });

    state.add_utxos(destination, utxos);
}

pub fn remove_retrieve_btc_request(state: &mut CustomState, request: ReleaseTokenRequest) {
    record_event(&Event::RemovedRetrieveBtcRequest {
        block_index: request.block_index,
    });

    state.push_finalized_request(FinalizedBtcRetrieval {
        request,
        state: FinalizedStatus::AmountTooLow,
    });
}

pub fn sent_transaction(state: &mut CustomState, tx: SubmittedBtcTransaction) {
    record_event(&Event::SentBtcTransaction {
        request_block_indices: tx.requests.iter().map(|r| r.block_index).collect(),
        txid: tx.txid,
        utxos: tx.used_utxos.clone(),
        change_output: tx.change_output.clone(),
        submitted_at: tx.submitted_at,
        fee_per_vbyte: tx.fee_per_vbyte,
    });

    state.push_submitted_transaction(tx);
}

pub fn confirm_transaction(state: &mut CustomState, txid: &Txid) {
    record_event(&Event::ConfirmedBtcTransaction { txid: *txid });
    state.finalize_transaction(txid);
}

pub fn replace_transaction(
    state: &mut CustomState,
    old_txid: Txid,
    new_tx: SubmittedBtcTransaction,
) {
    record_event(&Event::ReplacedBtcTransaction {
        old_txid,
        new_txid: new_tx.txid,
        change_output: new_tx
            .change_output
            .clone()
            .expect("bug: all replacement transactions must have the change output"),
        submitted_at: new_tx.submitted_at,
        fee_per_vbyte: new_tx
            .fee_per_vbyte
            .expect("bug: all replacement transactions must have the fee"),
    });
    state.replace_transaction(&old_txid, new_tx);
}
