//! State modifications that should end up in the event log.

use super::{
    eventlog::Event, CustomState, FinalizedTicket,
    FinalizedTicketStatus, GenTicketRequest, ReleaseTokenRequest, RunesBalance,
    SubmittedBtcTransaction,
};
use crate::destination::Destination;
use crate::storage::record_event;
use ic_btc_interface::{OutPoint, Txid, Utxo};

pub fn accept_release_token_request(state: &mut CustomState, request: ReleaseTokenRequest) {
    record_event(&Event::AcceptedReleaseTokenRequest(request.clone()));
    state.push_back_pending_request(request);
}

pub fn accept_generate_ticket_request(state: &mut CustomState, request: GenTicketRequest) {
    record_event(&&Event::AcceptedGenTicketRequest(request.clone()));
    state
        .pending_gen_ticket_requests
        .insert(request.tx_id, request);
}

pub fn add_utxos(
    state: &mut CustomState,
    destination: Destination,
    utxos: Vec<Utxo>,
    is_runes: bool,
) {
    record_event(&Event::ReceivedUtxos {
        destination: destination.clone(),
        utxos: utxos.clone(),
        is_runes,
    });

    state.add_utxos(destination, utxos, is_runes);
}

pub fn update_runes_balance(state: &mut CustomState, outpoint: OutPoint, balance: RunesBalance) {
    record_event(&Event::ReceivedRunesToken {
        outpoint: outpoint.clone(),
        balance: balance.clone(),
    });

    state.update_runes_balance(outpoint, balance);
}

pub fn finalize_ticket_request(
    state: &mut CustomState,
    request: &GenTicketRequest,
    status: FinalizedTicketStatus,
) {
    record_event(&Event::FinalizedTicketRequest {
        tx_id: request.tx_id,
        status: status.clone(),
    });

    state.push_finalized_boarding_pass(FinalizedTicket {
        request: request.clone(),
        status,
    });
}

pub fn sent_transaction(state: &mut CustomState, tx: SubmittedBtcTransaction) {
    record_event(&Event::SentBtcTransaction {
        runes_id: tx.runes_id.clone(),
        request_release_ids: tx.requests.iter().map(|r| r.release_id.clone()).collect(),
        txid: tx.txid,
        runes_utxos: tx.runes_utxos.clone(),
        btc_utxos: tx.btc_utxos.clone(),
        runes_change_output: tx.runes_change_output.clone(),
        btc_change_output: tx.btc_change_output.clone(),
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
        runes_change_output: new_tx.runes_change_output.clone(),
        btc_change_output: new_tx.btc_change_output.clone(),
        submitted_at: new_tx.submitted_at,
        fee_per_vbyte: new_tx
            .fee_per_vbyte
            .expect("bug: all replacement transactions must have the fee"),
    });
    state.replace_transaction(&old_txid, new_tx);
}
