//! State modifications that should end up in the event log.

use super::{eventlog::Event, CustomsState, GenTicketRequestV2, RuneId, RuneTxRequest, RunesBalance, SubmittedBtcTransactionV2, BitcoinFeeRate, mutate_state};
use crate::storage::record_event;
use crate::{destination::Destination, state::RUNES_TOKEN};
use ic_btc_interface::{Txid, Utxo};
use omnity_types::{Chain, Factor, ToggleState, Token};

pub fn add_chain(state: &mut CustomsState, chain: Chain) {
    record_event(&Event::AddedChain(chain.clone()));
    state.counterparties.insert(chain.chain_id.clone(), chain);
}

pub fn add_token(state: &mut CustomsState, rune_id: RuneId, token: Token) {
    record_event(&Event::AddedToken {
        rune_id,
        token: token.clone(),
    });
    state
        .tokens
        .insert(token.token_id.clone(), (rune_id, token));
}

pub fn toggle_chain_state(state: &mut CustomsState, toggle: ToggleState) {
    record_event(&Event::ToggleChainState(toggle.clone()));
    if toggle.chain_id == state.chain_id {
        state.chain_state = toggle.action.into();
    } else if let Some(chain) = state.counterparties.get_mut(&toggle.chain_id) {
        chain.chain_state = toggle.action.into();
    }
}

pub fn update_next_directive_seq(state: &mut CustomsState, next_seq: u64) {
    if next_seq > state.next_directive_seq {
        record_event(&Event::UpdateNextDirectiveSeq(next_seq));
        state.next_directive_seq = next_seq;
    }
}

pub fn update_next_ticket_seq(state: &mut CustomsState, next_seq: u64) {
    if next_seq > state.next_ticket_seq {
        record_event(&&Event::UpdateNextTicketSeq(next_seq));
        state.next_ticket_seq = next_seq;
    }
}

pub fn accept_rune_tx_request(state: &mut CustomsState, request: RuneTxRequest) {
    record_event(&Event::AcceptedRuneTxRequest(request.clone()));
    state.push_back_pending_request(request);
}

pub fn accept_generate_ticket_request(state: &mut CustomsState, request: GenTicketRequestV2) {
    record_event(&Event::AcceptedGenTicketRequestV3(request.clone()));
    state
        .pending_gen_ticket_requests
        .insert(request.txid, request);
}

pub fn confirm_generate_ticket_request(state: &mut CustomsState, req: GenTicketRequestV2) {
    record_event(&Event::ConfirmedGenTicketRequest(req.clone()));

    assert!(state
        .pending_gen_ticket_requests
        .remove(&req.txid)
        .is_some());

    let new_utxos = req.new_utxos.clone();
    let dest = Destination {
        target_chain_id: req.target_chain_id.clone(),
        receiver: req.receiver.clone(),
        token: Some(RUNES_TOKEN.into()),
    };
    state.confirmed_gen_ticket_requests.insert(req.txid, req);
    state.add_utxos(dest, new_utxos, true);
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

pub fn update_bitcoin_fee_rate(fee_rate: BitcoinFeeRate) {
    record_event(&Event::UpdateBitcoinFeeRate(fee_rate.clone()));
    mutate_state(|s|s.bitcoin_fee_rate = fee_rate);
}
pub fn update_runes_balance(state: &mut CustomsState, txid: Txid, balance: RunesBalance) {
    record_event(&Event::UpdatedRunesBalance {
        txid,
        balance: balance.clone(),
    });

    state.update_runes_balance(txid, balance);
}

pub fn remove_confirmed_request(state: &mut CustomsState, txid: &Txid) {
    record_event(&Event::RemovedTicketRequest { txid: txid.clone() });
    state
        .confirmed_gen_ticket_requests
        .remove(txid)
        .and_then(|req| Some(req.new_utxos.iter().for_each(|u| state.forget_utxo(u))));
}

pub fn finalize_ticket_request(
    state: &mut CustomsState,
    request: &GenTicketRequestV2,
    balances: Vec<RunesBalance>,
) {
    record_event(&Event::FinalizedTicketRequest {
        txid: request.txid,
        balances: balances.clone(),
    });

    state.confirmed_gen_ticket_requests.remove(&request.txid);
    for balance in balances {
        state.update_runes_balance(request.txid, balance);
    }
    state.push_finalized_ticket(request.clone());
}

pub fn sent_transaction(state: &mut CustomsState, tx: SubmittedBtcTransactionV2) {
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
    new_tx: SubmittedBtcTransactionV2,
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

pub fn update_fee(state: &mut CustomsState, fee: Factor) {
    record_event(&Event::UpdatedFee { fee: fee.clone() });
    match fee {
        Factor::UpdateTargetChainFactor(factor) => {
            state
                .target_chain_factor
                .insert(factor.target_chain_id.clone(), factor.target_chain_factor);
        }

        Factor::UpdateFeeTokenFactor(token_factor) => {
            if token_factor.fee_token.eq("BTC") {
                state.fee_token_factor = Some(token_factor.fee_token_factor);
            }
        }
    }
}
