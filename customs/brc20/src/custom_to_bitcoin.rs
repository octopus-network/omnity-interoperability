use std::ops::Div;
use std::str::FromStr;

use bitcoin::hashes::Hash;
use bitcoin::{Address, Amount, PublicKey, Transaction, Txid};
use ic_btc_interface::{MillisatoshiPerByte, Network};

use ic_canister_log::log;
use rust_decimal::Decimal;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::bitcoin_to_custom::query_transaction;
use crate::call_error::CallError;
use crate::constants::{FINALIZE_UNLOCK_TICKET_NAME, SUBMIT_UNLOCK_TICKETS_NAME};
use omnity_types::ic_log::{CRITICAL, ERROR};
use omnity_types::{Seq, Ticket};

use crate::custom_to_bitcoin::CustomToBitcoinError::{
    ArgumentError, BuildTransactionFailed, SignFailed,
};

use crate::hub::update_tx_hash;
use crate::management::get_fee_utxos;
use crate::ord::builder::fees::{calc_fees, Fees};
use crate::ord::builder::signer::MixSigner;
use crate::ord::builder::spend_transaction::spend_utxo_transaction;
use crate::ord::builder::{
    CreateCommitTransactionArgsV2, OrdTransactionBuilder, RevealTransactionArgs,
    SignCommitTransactionArgs, Utxo,
};
use crate::ord::inscription::brc20::Brc20;
use crate::{management, state};

use crate::ord::parser::POSTAGE;
use crate::state::{
    bitcoin_network, deposit_addr, deposit_pubkey, finalization_time_estimate, mutate_state,
    read_state,
};

#[derive(Error, Debug)]
pub enum CustomToBitcoinError {
    #[error("bitcoin sign error: {0}")]
    SignFailed(String),
    #[error("build a brc20 transfer error: {0}")]
    BuildTransactionFailed(String),
    #[error("ArgumentError: {0}")]
    ArgumentError(String),
    #[error("InsufficientFunds")]
    InsufficientFunds,
}
pub type CustomToBitcoinResult<T> = Result<T, CustomToBitcoinError>;

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct SendTicketResult {
    pub txs: Vec<Transaction>,
    pub success: bool,
    pub err_step: Option<u8>,
    pub err_info: Option<CallError>,
    pub time_at: u64,
}

pub async fn send_tickets_to_bitcoin() {
    let from = read_state(|s| s.next_consume_ticket_seq);
    let to = read_state(|s| s.next_ticket_seq);
    if from < to {
        let (nw, deposit_addr) = read_state(|s| (s.btc_network, s.deposit_addr.clone().unwrap()));
        let utxos = get_fee_utxos(nw, &deposit_addr, 0u32).await;
        match utxos {
            Ok(r) => {
                let v = r
                    .utxos
                    .into_iter()
                    .map(|u| Utxo {
                        id: Txid::from_slice(u.outpoint.txid.as_ref()).unwrap(),
                        //u.outpoint.txid.into(),
                        index: u.outpoint.vout,
                        amount: Amount::from_sat(u.value),
                    })
                    .collect();

                mutate_state(|s| s.deposit_addr_utxo = v);
            }
            Err(_) => {
                return;
            }
        }
        let fees = calc_fees(bitcoin_network()).await;
        for seq in from..to {
            if process_unlock_ticket(seq, &fees).await.is_err() {
                break;
            }
            mutate_state(|s| s.next_consume_ticket_seq = seq + 1);
        }
    }
}

pub async fn process_unlock_ticket(seq: Seq, fees: &Fees) -> Result<(), CustomToBitcoinError> {
    let res = send_ticket_to_bitcoin(seq, fees).await;
    if res.is_err() {
        let err = res.err().unwrap();
        log!(CRITICAL, "send ticket to bitcoin failed {}, {}", seq, &err);
        return Err(err);
    } else {
        let r = res.ok().unwrap();
        match r {
            None => {}
            Some(info) => {
                let reveal_utxo_index = format!("{}:0", info.txs[1].txid());
                mutate_state(|s| {
                    s.flight_unlock_ticket_map.insert(seq, info);
                    s.reveal_utxo_index.insert(reveal_utxo_index);
                });
            }
        }
    }
    Ok(())
}

pub async fn finalize_flight_unlock_tickets() {
    let now = ic_cdk::api::time();
    let can_check_finalizations = read_state(|s| {
        let wait_time = finalization_time_estimate(s.min_confirmations, s.btc_network);
        s.flight_unlock_ticket_map
            .iter()
            .filter(|&req| (req.1.time_at + (wait_time.as_nanos() as u64) < now))
            .map(|req| (*req.0, req.1.clone()))
            .collect::<Vec<(Seq, SendTicketResult)>>()
    });
    let (_network, _deposit_addr, _min_confirmations) = read_state(|s| {
        (
            s.btc_network,
            s.deposit_addr.clone().unwrap(),
            s.min_confirmations as u32,
        )
    });
    for (seq, send_result) in can_check_finalizations.clone() {
        let need_check_tx = send_result.txs.last().cloned().unwrap();
        let transfer_txid = need_check_tx.txid().to_string();
        let tx = query_transaction(&transfer_txid).await;
        match tx {
            Ok(t) => {
                if t.status.confirmed {
                    mutate_state(|s| {
                        let r = s.flight_unlock_ticket_map.remove(&seq).unwrap();
                        let reveal_utxo_index = format!("{}:0", r.txs[1].txid());
                        s.reveal_utxo_index.remove(&reveal_utxo_index);
                        s.finalized_unlock_ticket_map.insert(seq, r);
                    });
                    let (hub_principal, ticket) =
                        read_state(|s| (s.hub_principal, s.tickets_queue.get(&seq).unwrap()));
                    if let Err(err) =
                        update_tx_hash(hub_principal, ticket.ticket_id, transfer_txid).await
                    {
                        log!(
                            CRITICAL,
                            "[rewrite tx_hash] failed to write brc20 release tx hash, reason: {}",
                            err
                        );
                    }
                }
            }
            Err(e) => {
                log!(ERROR, "confirm flight ticket error: {:?}", e);
            }
        }
    }
}

pub async fn send_ticket_to_bitcoin(
    seq: Seq,
    fees: &Fees,
) -> Result<Option<SendTicketResult>, CustomToBitcoinError> {
    let ticket = read_state(|s| s.tickets_queue.get(&seq));
    match ticket {
        None => Ok(None),
        Some(t) => {
            if read_state(|s| s.finalized_unlock_ticket_map.contains_key(&seq)) {
                return Ok(None);
            }
            if read_state(|s| s.flight_unlock_ticket_map.get(&seq).is_some()) {
                return Ok(None);
            }
            let token = read_state(|s| s.tokens.get(&t.token).cloned().unwrap());
            let vins = select_inscribe_txins(fees)?;
            let key_id = read_state(|s| s.ecdsa_key_name.clone());
            let mut builder = OrdTransactionBuilder::p2tr(
                PublicKey::from_str(deposit_pubkey().as_str()).unwrap(),
                key_id,
                deposit_addr(),
            );
            let amount: u128 = t.amount.parse().unwrap();
            let amt =
                Decimal::from(amount).div(Decimal::from(10u128.pow(token.decimals as u32)));
            let commit_tx = builder
                .build_commit_transaction_with_fixed_fees(
                    bitcoin_network(),
                    CreateCommitTransactionArgsV2 {
                        inputs: vins.clone(),
                        inscription: Brc20::transfer(token.name.clone(), amt),
                        txin_script_pubkey: deposit_addr().script_pubkey(),
                        leftovers_recipient: deposit_addr().clone(),
                        commit_fee: fees.commit_fee,
                        reveal_fee: fees.reveal_fee,
                        spend_fee: fees.utxo_fee,
                    },
                )
                .await
                .map_err(|e| BuildTransactionFailed(e.to_string()))?;

            let signed_commit_tx = builder
                .sign_commit_transaction(
                    commit_tx.unsigned_tx,
                    SignCommitTransactionArgs {
                        inputs: vins,
                        txin_script_pubkey: deposit_addr().script_pubkey(),
                    },
                )
                .await
                .map_err(|e| SignFailed(e.to_string()))?;

            let reveal_transaction = builder
                .build_reveal_transaction(RevealTransactionArgs {
                    input: Utxo {
                        id: signed_commit_tx.txid(),
                        index: 0,
                        amount: commit_tx.reveal_balance,
                    },
                    spend_fee: fees.utxo_fee,
                    recipient_address: deposit_addr(), // NOTE: it's correct, see README.md to read about how transfer works
                    redeem_script: commit_tx.redeem_script,
                })
                .await
                .map_err(|e| BuildTransactionFailed(e.to_string()))?;

            let real_utxo = Utxo {
                id: reveal_transaction.txid(),
                index: 0,
                amount: Amount::from_sat(POSTAGE+fees.utxo_fee.to_sat()),
            };

            let commit_remain_fee = None; //find_commit_remain_fee(&signed_commit_tx);
            let transfer_trasaction =
                build_transfer_transfer(&t, fees, real_utxo, &builder.signer(), commit_remain_fee)
                    .await?;
            let network = read_state(|s| s.btc_network);
            let tx_vec = vec![signed_commit_tx, reveal_transaction, transfer_trasaction];

            let mut send_res = SendTicketResult {
                txs: tx_vec.clone(),
                success: true,
                err_step: None,
                err_info: None,
                time_at: ic_cdk::api::time(),
            };
            for (index, tx) in tx_vec.into_iter().enumerate() {
                let r = crate::management::send_transaction(&tx, network).await;
                if r.is_err() {
                    send_res.success = false;
                    send_res.err_step = Some(index as u8);
                    send_res.err_info = r.err();
                    break;
                }
            }
           /* if send_res.success {
                //insert_utxo
                let transfer_transaction = send_res.txs.last().cloned().unwrap();
                let txid = transfer_transaction.txid();
                let index = transfer_transaction.output.len() as u32 - 1;
                let value = transfer_transaction.output.last().unwrap().value;
                let utxo = Utxo {
                    id: txid,
                    index,
                    amount: value,
                };
                mutate_state(|s| s.deposit_addr_utxo.push(utxo));
            }*/
            Ok(Some(send_res))
        }
    }
}

pub fn find_commit_remain_fee(t: &Transaction) -> Option<Utxo> {
    if t.output.len() > 1 {
        let r = t.output.last().cloned().unwrap();
        let utxo = Utxo {
            id: t.txid(),
            index: (t.output.len() - 1) as u32,
            amount: r.value,
        };
        Some(utxo)
    } else {
        None
    }
}

pub async fn build_transfer_transfer(
    ticket: &Ticket,
    fee: &Fees,
    reveal_utxo: Utxo,
    signer: &MixSigner,
    commit_return_fee: Option<Utxo>,
) -> Result<Transaction, CustomToBitcoinError> {
    let fees_inputs =vec![];// determine_transfer_fee_txins(fee, commit_return_fee)?;
    let mut all_inputs = vec![reveal_utxo.clone()];
    all_inputs.extend(fees_inputs);
    let recipient = Address::from_str(&ticket.receiver.to_string())
        .map_err(|e| ArgumentError(e.to_string()))?
        .assume_checked();
    let transfer = spend_utxo_transaction(
        signer,
        recipient,
        Amount::from_sat(POSTAGE),
        all_inputs,
        fee.utxo_fee,
    )
    .await?;
    Ok(transfer)
}

pub fn select_inscribe_txins(fees: &Fees) -> Result<Vec<Utxo>, CustomToBitcoinError> {
    let total_reqiured = POSTAGE + fees.commit_fee.to_sat() + fees.reveal_fee.to_sat() + fees.utxo_fee.to_sat();
    select_utxos(total_reqiured)
}

pub fn determine_transfer_fee_txins(
    fee: &Fees,
    commit_remain_fee: Option<Utxo>,
) -> CustomToBitcoinResult<Vec<Utxo>> {
    let fee_amount = fee.utxo_fee.to_sat();
    match commit_remain_fee {
        None => select_utxos(fee_amount),
        Some(t) => {
            if fee_amount < t.amount.to_sat() {
                Ok(vec![t])
            } else {
                let re_fee = fee_amount - t.amount.to_sat();
                let mut v = select_utxos(re_fee)?;
                v.push(t);
                Ok(v)
            }
        }
    }
}

pub fn select_utxos(fee: u64) -> CustomToBitcoinResult<Vec<Utxo>> {
    let mut selected_utxos: Vec<Utxo> = vec![];
    let mut selected_amount = 0u64;
    mutate_state(|s| loop {
        if selected_amount > fee {
            return Ok(selected_utxos);
        }
        let u = s.deposit_addr_utxo.pop();
        match u {
            None => {
                return Err(CustomToBitcoinError::InsufficientFunds);
            }
            Some(utxo) => {
                selected_amount += utxo.amount.to_sat();
                selected_utxos.push(utxo);
            }
        }
    })
}

pub fn finalize_unlock_tickets_task() {
    ic_cdk::spawn(async {
        let _guard =
            match crate::guard::TimerLogicGuard::new(FINALIZE_UNLOCK_TICKET_NAME.to_string()) {
                Some(guard) => guard,
                None => return,
            };
        finalize_flight_unlock_tickets().await;
    });
}

pub fn submit_unlock_tickets_task() {
    ic_cdk::spawn(async {
        let _guard =
            match crate::guard::TimerLogicGuard::new(SUBMIT_UNLOCK_TICKETS_NAME.to_string()) {
                Some(guard) => guard,
                None => return,
            };
        send_tickets_to_bitcoin().await;
    });
}

/// Returns an estimate for transaction fees in millisatoshi per vbyte. Returns
/// None if the bitcoin canister is unavailable or does not have enough data for
/// an estimate yet.
pub async fn estimate_fee_per_vbyte() -> Option<MillisatoshiPerByte> {
    /// The default fee we use on regtest networks if there are not enough data
    /// to compute the median fee.
    const DEFAULT_FEE: MillisatoshiPerByte = 5_000;

    let btc_network = state::read_state(|s| s.btc_network);
    match management::get_current_fees(btc_network).await {
        Ok(fees) => {
            if btc_network == Network::Regtest {
                return Some(DEFAULT_FEE);
            }
            if fees.len() >= 100 {
                Some(fees[50])
            } else {
                log!(
                    ERROR,
                    "[estimate_fee_per_vbyte]: not enough data points ({}) to compute the fee",
                    fees.len()
                );
                None
            }
        }
        Err(err) => {
            log!(
                ERROR,
                "[estimate_fee_per_vbyte]: failed to get median fee per vbyte: {}",
                err
            );
            None
        }
    }
}