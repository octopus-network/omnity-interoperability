//! State modifications that should end up in the event log.

use super::{
    eventlog::Event, CustomsState, FinalizedTicket, FinalizedTicketStatus, GenTicketRequest,
    ReleaseTokenRequest, RunesBalance, SubmittedBtcTransaction,
};
use crate::destination::Destination;
use crate::storage::record_event;
use ic_btc_interface::{Txid, Utxo};

pub fn accept_release_token_request(state: &mut CustomsState, request: ReleaseTokenRequest) {
    record_event(&Event::AcceptedReleaseTokenRequest(request.clone()));
    state.push_back_pending_request(request);
}

pub fn accept_generate_ticket_request(state: &mut CustomsState, request: GenTicketRequest) {
    record_event(&&Event::AcceptedGenTicketRequest(request.clone()));
    state
        .pending_gen_ticket_requests
        .insert(request.txid, request);
}

pub fn add_utxos(
    state: &mut CustomsState,
    destination: Destination,
    utxos: Vec<Utxo>,
    is_runes: bool,
) {
    if utxos.is_empty() {
        return;
    }
    record_event(&Event::ReceivedUtxos {
        destination: destination.clone(),
        utxos: utxos.clone(),
        is_runes,
    });

    state.add_utxos(destination, utxos, is_runes);
}

pub fn update_runes_balance(state: &mut CustomsState, txid: Txid, balance: RunesBalance) {
    record_event(&Event::UpdatedRunesBalance {
        txid,
        balance: balance.clone(),
    });

    state.update_runes_balance(txid, balance);
}

pub fn finalize_ticket_request(
    state: &mut CustomsState,
    request: &GenTicketRequest,
    balances: Vec<RunesBalance>,
) {
    record_event(&Event::FinalizedTicketRequest {
        txid: request.txid,
        balances: balances.clone(),
    });

    state.pending_gen_ticket_requests.remove(&request.txid);
    for balance in balances {
        state.update_runes_balance(request.txid, balance);
    }
    state.push_finalized_ticket(FinalizedTicket {
        request: request.clone(),
        status: FinalizedTicketStatus::Finalized,
    });
}

pub fn remove_ticket_request(
    state: &mut CustomsState,
    request: &GenTicketRequest,
    status: FinalizedTicketStatus,
) {
    record_event(&Event::RemovedTicketRequest {
        txid: request.txid,
        status: status.clone(),
    });
    state.pending_gen_ticket_requests.remove(&request.txid);
    state.push_finalized_ticket(FinalizedTicket {
        request: request.clone(),
        status,
    });
}

pub fn sent_transaction(state: &mut CustomsState, tx: SubmittedBtcTransaction) {
    record_event(&Event::SentBtcTransaction {
        rune_id: tx.rune_id,
        request_release_ids: tx.requests.iter().map(|r| r.ticket_id.clone()).collect(),
        txid: tx.txid,
        runes_utxos: tx.runes_utxos.clone(),
        btc_utxos: tx.btc_utxos.clone(),
        runes_change_output: tx.runes_change_output.clone(),
        btc_change_output: tx.btc_change_output.clone(),
        submitted_at: tx.submitted_at,
        fee_per_vbyte: tx.fee_per_vbyte,
        raw_tx: tx.raw_tx.clone(),
    });

    state.push_submitted_transaction(tx);
}

pub fn confirm_transaction(state: &mut CustomsState, txid: &Txid) {
    record_event(&Event::ConfirmedBtcTransaction { txid: *txid });
    state.finalize_transaction(txid);
}

pub fn replace_transaction(
    state: &mut CustomsState,
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
        raw_tx: new_tx.raw_tx.clone(),
    });
    state.replace_transaction(&old_txid, new_tx);
}
