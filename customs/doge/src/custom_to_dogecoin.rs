use std::str::FromStr;

use bitcoin::ecdsa::Signature as SighashSignature;
use bitcoin::secp256k1::{ecdsa::Signature, PublicKey};
use bitcoin::EcdsaSighashType;
use candid::CandidType;

use ic_canister_log::log;

use ic_stable_structures::storable::Bound;
use ic_stable_structures::Storable;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::constants::{FINALIZE_UNLOCK_TICKET_NAME, SUBMIT_UNLOCK_TICKETS_NAME};
use crate::doge::ecdsa::sign_with;
use crate::doge::fee::{fee_by_size, DOGE_AMOUNT, DUST_LIMIT, FEE_CAP};
use crate::doge::rpc::DogeRpc;
use crate::doge::script;
use crate::doge::sighash::SighashCache;
use crate::doge::transaction::{OutPoint, Transaction, TxIn, TxOut};
use crate::types::{Destination, Txid, Utxo};
use omnity_types::ic_log::{CRITICAL, ERROR, INFO};
use omnity_types::Seq;

use crate::custom_to_dogecoin::CustomToBitcoinError::{
    ArgumentError, SendTransactionFailed, SignFailed,
};

use crate::hub::update_tx_hash;

use crate::state::{finalization_time_estimate, mutate_state, read_state};

#[derive(Error, Debug, CandidType)]
pub enum CustomToBitcoinError {
    #[error("bitcoin sign error: {0}")]
    SignFailed(String),
    #[error("ArgumentError: {0}")]
    ArgumentError(String),
    #[error("InsufficientFunds")]
    InsufficientFunds,
    #[error("InsufficientFee, need: {0}, able to pay: {1}")]
    InsufficientFee(u64, u64),
    #[error("AmountTooSmall")]
    AmountTooSmall,
    #[error("SendTransactionFailed: {0}")]
    SendTransactionFailed(String),
}
pub type CustomToBitcoinResult<T> = Result<T, CustomToBitcoinError>;

#[derive(Serialize, Deserialize, Clone, CandidType)]
pub struct SendTicketResult {
    pub txid: Txid,
    pub success: bool,
    pub time_at: u64,
}

impl Storable for SendTicketResult {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        bincode::serialize(self).unwrap().into()
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        bincode::deserialize(bytes.as_ref()).unwrap()
    }

    const BOUND: Bound = Bound::Unbounded;
}

pub async fn send_tickets_to_bitcoin() {
    let (from, to, fee_rate) = read_state(|s| {
        (
            s.next_consume_ticket_seq,
            s.next_ticket_seq,
            s.doge_fee_rate,
        )
    });
    if from < to {
        log!(INFO, "submit unlock tx: from {} to {}", from, to);
        for seq in from..to {
            let r = process_unlock_ticket(seq, fee_rate).await;
            match r {
                Ok(_) => {
                    mutate_state(|s| s.next_consume_ticket_seq = seq + 1);
                }
                Err(e) => {
                    log!(ERROR, "send unlock error: ticket seq: {}, error{}", seq, e);
                    break;
                }
            }
        }
    }
}

pub async fn process_unlock_ticket(
    seq: Seq,
    fee_rate: Option<u64>,
) -> Result<(), CustomToBitcoinError> {
    let res = submit_unlock_ticket(seq, fee_rate).await;
    if res.is_err() {
        let err = res.err().unwrap();
        log!(
            CRITICAL,
            "send ticket to bitcoin failed, ticket seq: {}, {}",
            seq,
            &err
        );
        return Err(err);
    } else {
        let r = res.ok().unwrap();
        log!(
            INFO,
            "process ticket to bitcoin success, ticket seq: {}, txid: {}",
            seq,
            r.txid
        );
        mutate_state(|s| {
            s.flight_unlock_ticket_map.insert(seq, r);
        });
    }
    Ok(())
}

pub async fn finalize_flight_unlock_tickets() {
    let now = ic_cdk::api::time();
    let can_check_finalizations = read_state(|s| {
        let wait_time = finalization_time_estimate(s.min_confirmations);
        s.flight_unlock_ticket_map
            .iter()
            .filter(|&req| (req.1.time_at + (wait_time.as_nanos() as u64) < now))
            .map(|req| (*req.0, req.1.clone()))
            .collect::<Vec<(Seq, SendTicketResult)>>()
    });
    let min_confirmations = read_state(|s| s.min_confirmations);

    let doge_rpc: DogeRpc = read_state(|s| s.default_doge_rpc_config.clone()).into();
    for (seq, send_result) in can_check_finalizations.clone() {
        let need_check_txid = send_result.txid.clone();
        let transfer_txid = need_check_txid.to_string();
        let tx = doge_rpc.get_tx_out(&transfer_txid).await;
        match tx {
            Ok(t) => {
                if t.confirmations >= min_confirmations {
                    mutate_state(|s| {
                        let r = s.flight_unlock_ticket_map.remove(&seq).unwrap();
                        s.finalized_unlock_ticket_results_map.insert(seq, r);
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
                    } else {
                        log!(INFO, "unlock ticket finalize success! ticket seq: {}", seq);
                    }
                }
            }
            Err(e) => {
                log!(ERROR, "confirm flight ticket error: {:?}", e);
            }
        }
    }
}

pub async fn build_and_send_transaction(
    fee_utxo_list: Vec<Utxo>,
    fee_utxo_total_amount: u64,
    utxos: Vec<(Utxo, Destination)>,
    utxo_total_amount: u64,
    amount: u64,
    fee_rate: Option<u64>,
    receiver: script::Address,
) -> Result<(Transaction, crate::doge::transaction::Txid), CustomToBitcoinError> {

    // build transaction
    let all_utxos: Vec<(Utxo, Destination)> = utxos
        .clone()
        .into_iter()
        .chain(
            fee_utxo_list
                .iter()
                .map(|e| (e.clone(), Destination::fee_payment_address())),
        )
        .collect();
    let chain_params = read_state(|s| s.chain_params());
    let (fee_payment_address, _) =
        read_state(|s| s.get_address(Destination::fee_payment_address()))
            .map_err(|e| ArgumentError(e.to_string()))?;
    let (key_name, change_address_ret) = read_state(|s| {
        (
            s.ecdsa_key_name.clone(),
            s.get_address(Destination::change_address()),
        )
    });
    let (change_address, _) = change_address_ret.map_err(|e| ArgumentError(e.to_string()))?;
    let mut send_tx = Transaction {
        version: Transaction::CURRENT_VERSION,
        lock_time: 0,
        input: utxos
            .iter()
            .map(|(utxo, _)| {
                TxIn::with_outpoint(OutPoint {
                    txid: utxo.txid.clone().into(),
                    vout: utxo.vout,
                })
            })
            .chain(fee_utxo_list.iter().map(|utxo| {
                TxIn::with_outpoint(OutPoint {
                    txid: utxo.txid.clone().into(),
                    vout: utxo.vout,
                })
            }))
            .collect(),
        output: vec![
            TxOut {
                value: amount,
                script_pubkey: receiver.to_script(chain_params),
            },
            TxOut {
                value: utxo_total_amount.saturating_sub(amount),
                script_pubkey: change_address.to_script(chain_params),
            },
            TxOut {
                value: fee_utxo_total_amount,
                script_pubkey: fee_payment_address.to_script(chain_params),
            },
        ],
    };
    let fee = fee_by_size(send_tx.estimate_size() as u64, fee_rate);
    log!(
        INFO,
        "send ticket fee: {}, total_fee_utxo_amount: {}",
        fee,
        fee_utxo_total_amount
    );
    if fee > fee_utxo_total_amount {
        return Err(CustomToBitcoinError::InsufficientFee(fee, fee_utxo_total_amount));
    }
    send_tx.output[2].value = fee_utxo_total_amount.saturating_sub(fee);
    if send_tx.output[2].value <= DUST_LIMIT {
        send_tx.output.pop();
    }

    // sign transaction

    let mut sighasher = SighashCache::new(&mut send_tx);
    for (i, (_utxo, destination)) in all_utxos.iter().enumerate() {
        let (address, pk) = read_state(|s| s.get_address(destination.clone()))
            .map_err(|e| ArgumentError(e.to_string()))?;
        let hash = sighasher
            .signature_hash(i, &address.to_script(chain_params), EcdsaSighashType::All)
            .map_err(|e| SignFailed(e.to_string()))?;
        let sig = sign_with(&key_name, destination.derivation_path(), *hash)
            .await
            .map_err(|e| SignFailed(e.to_string()))?;
        let signature = Signature::from_compact(&sig).map_err(|e| SignFailed(e.to_string()))?;
        sighasher
            .set_input_script(
                i,
                &SighashSignature {
                    signature,
                    sighash_type: EcdsaSighashType::All,
                },
                &PublicKey::from_slice(&pk).map_err(|e| SignFailed(e.to_string()))?,
            )
            .map_err(|e| SignFailed(e.to_string()))?;
    }

    // send transaction
    let doge_rpc: DogeRpc = read_state(|s| s.default_doge_rpc_config.clone()).into();
    let txid = doge_rpc
        .send_transaction(&send_tx)
        .await
        .map_err(|e| SendTransactionFailed(e.to_string()))?;

    Ok((send_tx, txid))
}

pub async fn submit_unlock_ticket(
    seq: Seq,
    fee_rate: Option<u64>,
) -> Result<SendTicketResult, CustomToBitcoinError> {
    match read_state(|s| s.tickets_queue.get(&seq)) {
        None => Err(CustomToBitcoinError::ArgumentError(
            "ticket not found".to_string(),
        )),
        Some(ticket) => {
            // check ticket
            if read_state(|s| s.finalized_unlock_ticket_results_map.contains_key(&seq)) {
                return Err(CustomToBitcoinError::ArgumentError(
                    "ticket already finalized".to_string(),
                ));
            }
            if read_state(|s| s.flight_unlock_ticket_map.contains_key(&seq)) {
                return Err(CustomToBitcoinError::ArgumentError(
                    "ticket already in flight".to_string(),
                ));
            }

            let amount = ticket
                .amount
                .parse::<u64>()
                .map_err(|e| ArgumentError(e.to_string()))?;
            if amount < DUST_LIMIT * 10 {
                return Err(CustomToBitcoinError::AmountTooSmall);
            }
            let chain_params = read_state(|s| s.chain_params());
            let receiver = script::Address::from_str(&ticket.receiver)
                .map_err(|e| ArgumentError(e.to_string()))?;
            if !receiver.is_p2pkh(chain_params) {
                return Err(CustomToBitcoinError::ArgumentError(
                    "receiver address is not p2pkh".to_string(),
                ));
            }

            // select utxos
            let (utxos, total) = select_utxos(amount)?;
            let (fee_utxo_list, fee_utxo_total_amount) = select_fee_utxos(FEE_CAP)?;

            match build_and_send_transaction(
                fee_utxo_list.clone(),
                fee_utxo_total_amount,
                utxos.clone(),
                amount,
                total,
                fee_rate,
                receiver,
            )
            .await
            {
                Ok((send_tx, txid)) => {
                    mutate_state(|s| {
                        if send_tx.output.len() >= 2 {
                            // save change address utxo
                            s.deposited_utxo.push((
                                Utxo {
                                    txid: crate::types::Txid::from(txid).into(),
                                    vout: 1,
                                    value: send_tx.output[1].value,
                                },
                                Destination::change_address(),
                            ));
                        }

                        if send_tx.output.len() >= 3 {
                            // save fee payment utxo
                            s.fee_payment_utxo.push(Utxo {
                                txid: crate::types::Txid::from(txid).into(),
                                vout: 2,
                                value: send_tx.output[2].value,
                            });
                            if s.fee_payment_utxo.iter().map(|u| u.value).sum::<u64>()
                                < 20 * DOGE_AMOUNT || s.fee_payment_utxo.len() < 10
                            {
                                log!(ERROR, "Doge Customs fee_payment_utxo will not enough soon!");
                            }
                        }
                    });
                    Ok(SendTicketResult {
                        txid: send_tx.compute_txid().into(),
                        success: true,
                        time_at: ic_cdk::api::time(),
                    })
                }
                Err(e) => {
                    mutate_state(|s| {
                        s.fee_payment_utxo.append(&mut fee_utxo_list.clone());
                        s.deposited_utxo.append(&mut utxos.clone());
                    });

                    return Err(e);
                }
            }
        }
    }
}

pub fn select_fee_utxos(fee_cap: u64) -> CustomToBitcoinResult<(Vec<Utxo>, u64)> {
    let mut selected_utxos: Vec<Utxo> = vec![];
    let mut total = 0u64;
    mutate_state(|s| {
        while total < fee_cap && s.fee_payment_utxo.len() > 0 {
            let utxo = s
                .fee_payment_utxo
                .pop()
                .ok_or(CustomToBitcoinError::InsufficientFunds)?;
            total += utxo.value;
            selected_utxos.push(utxo);
        }
        Ok(())
    })?;
    if total < fee_cap {
        return Err(CustomToBitcoinError::InsufficientFunds);
    }

    Ok((selected_utxos, total))

}

pub fn select_utxos(amount: u64) -> CustomToBitcoinResult<(Vec<(Utxo, Destination)>, u64)> {
    let mut selected_utxos: Vec<(Utxo, Destination)> = vec![];
    let mut total = 0u64;
    mutate_state(|s| {
        while total < amount && s.deposited_utxo.len() > 0 {
            let (utxo, d) = s
                .deposited_utxo
                .pop()
                .ok_or(CustomToBitcoinError::InsufficientFunds)?;
            total += utxo.value;
            selected_utxos.push((utxo, d));
        }
        Ok(())
    })?;
    if total < amount {
        return Err(CustomToBitcoinError::InsufficientFunds);
    }

    Ok((selected_utxos, total))
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

#[test]
pub fn show_txid_from_vec_u8_data() {
    fn parse_hex_string(input: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let cleaned = input.replace("\\", "").replace(" ", "");
        let bytes: Result<Vec<u8>, _> = (0..cleaned.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&cleaned[i..i + 2], 16))
            .collect();

        Ok(bytes?)
    }
    let txid = r"\a2\44\f5\b0\69\73\4e\5d\15\de\14\8d\50\d5\60\08\21\07\83\09\2a\8d\78\cf\31\a5\77\19\2a\ef\c5\2a";
    let v = parse_hex_string(&txid).unwrap();
    dbg!(&hex::DisplayHex::to_upper_hex_string(&v));
    dbg!(&txid.to_string());
}
