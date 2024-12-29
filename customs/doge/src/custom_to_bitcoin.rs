use std::str::FromStr;

use bitcoin::secp256k1::{ecdsa::Signature, PublicKey};
use bitcoin::ecdsa::Signature as SighashSignature;
use bitcoin::EcdsaSighashType;
use candid::CandidType;

use ic_canister_log::log;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::constants::{
    FINALIZE_UNLOCK_TICKET_NAME, SUBMIT_UNLOCK_TICKETS_NAME,
};
use crate::doge::ecdsa::sign_with;
use crate::doge::fee::{fee_by_size, DUST_LIMIT};
use crate::doge::rpc::DogeRpc;
use crate::doge::script;
use crate::doge::sighash::SighashCache;
use crate::doge::transaction::{OutPoint, Transaction, TxIn, TxOut};
use crate::types::{Destination, Txid, Utxo};
use omnity_types::ic_log::{CRITICAL, ERROR, INFO};
use omnity_types::Seq;

use crate::custom_to_bitcoin::CustomToBitcoinError::{
    ArgumentError, SignFailed, SendTransactionFailed,
};

use crate::hub::update_tx_hash;

use crate::state::{
    finalization_time_estimate, mutate_state,
    read_state,
};

#[derive(Error, Debug, CandidType)]
pub enum CustomToBitcoinError {
    #[error("bitcoin sign error: {0}")]
    SignFailed(String),
    // #[error("build a brc20 transfer error: {0}")]
    // BuildTransactionFailed(String),
    #[error("ArgumentError: {0}")]
    ArgumentError(String),
    #[error("InsufficientFunds")]
    InsufficientFunds,
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

pub async fn send_tickets_to_bitcoin() {
    let (from, to ,fee_rate) = read_state(|s| (s.next_consume_ticket_seq, s.next_ticket_seq, s.doge_fee_rate));
    // let to = read_state(|s| s.next_ticket_seq);
    if from < to {
        log!(INFO, "submit unlock tx: from {} to {}", from, to);
        // let fee_rate = estimate_fee_per_vbyte().await / 1000;
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

pub async fn process_unlock_ticket(seq: Seq, fee_rate: Option<u64>) -> Result<(), CustomToBitcoinError> {
    let res = submit_unlock_ticket(seq, fee_rate).await;
    if res.is_err() {
        let err = res.err().unwrap();
        log!(CRITICAL, "send ticket to bitcoin failed {}, {}", seq, &err);
        return Err(err);
    } else {
        let r = res.ok().unwrap();
        match r {
            None => {}
            Some(info) => {
                // let reveal_utxo_index = format!("{}:0", info.txs[1].txid());
                mutate_state(|s| {
                    s.flight_unlock_ticket_map.insert(seq, info);
                    // s.reveal_utxo_index.insert(reveal_utxo_index);
                });
            }
        }
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
    let min_confirmations = read_state(|s|
            s.min_confirmations
    );

    let doge_rpc: DogeRpc = read_state(|s| s.default_rpc_config.clone()).into();
    for (seq, send_result) in can_check_finalizations.clone() {
        let need_check_txid = send_result.txid.clone();
        let transfer_txid = need_check_txid.to_string();
        // let tx = query_transaction((transfer_txid.clone(), "".to_string())).await;
        let tx = doge_rpc.get_tx_out(&transfer_txid).await;
        match tx {
            Ok(t) => {
                if t.confirmations >= min_confirmations {
                    mutate_state(|s| {
                        let r = s.flight_unlock_ticket_map.remove(&seq).unwrap();
                        // let reveal_utxo_index = format!("{}:0", r.tx[1].txid());
                        // s.reveal_utxo_index.remove(&reveal_utxo_index);
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

pub async fn submit_unlock_ticket(
    seq: Seq,
    fee_rate: Option<u64>,
) -> Result<Option<SendTicketResult>, CustomToBitcoinError> {
    match read_state(|s| s.tickets_queue.get(&seq)) {
        None => Ok(None),
        Some(ticket) => {
            // check ticket
            if read_state(|s| s.finalized_unlock_ticket_map.contains_key(&seq)) {
                return Ok(None);
            }
            if read_state(|s| s.flight_unlock_ticket_map.contains_key(&seq)) {
                return Ok(None);
            }

            let amount = ticket.amount.parse::<u64>().map_err(|e| ArgumentError(e.to_string()))?;
            if amount < DUST_LIMIT * 10 {
                return Err(CustomToBitcoinError::AmountTooSmall);
            }
            let chain_params = read_state(|s| s.chain_params());
            let receiver = script::Address::from_str(&ticket.receiver).map_err(|e| ArgumentError(e.to_string()))?;
            if !receiver.is_p2pkh(chain_params) {
                return Err(CustomToBitcoinError::ArgumentError("receiver address is not p2pkh".to_string()));
            }

            // 1. select utxos
            let (utxos, total) = select_utxos(amount)?;

            // 2. build transaction
            let (
                key_name, 
                // _ecdsa_public_key_opt,
                change_address_ret
            ) = read_state(|s| 
                (
                    s.ecdsa_key_name.clone(), 
                    // s.ecdsa_public_key.clone(),
                    s.get_address(Destination::change_address())
            ));
            let (change_address, _ ) = change_address_ret.map_err(|e| 
                ArgumentError(e.to_string())
            )?;
            // let ecdsa_public_key = ecdsa_public_key_opt.ok_or(ArgumentError("ecdsa_public_key is None".to_string()))?;
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
                .collect(),
                output: vec![
                    TxOut {
                        value: amount,
                        script_pubkey: receiver.to_script(chain_params),
                    },
                    TxOut {
                        value: total.saturating_sub(amount),
                        script_pubkey: change_address.to_script(chain_params),
                    },
                ],
            };
            let fee = fee_by_size(send_tx.estimate_size() as u64, fee_rate);
            send_tx.output[0].value = amount.saturating_sub(fee);
            if send_tx.output[1].value <= DUST_LIMIT {
                send_tx.output.pop();
            }

            // 3. sign transaction

            let mut sighasher = SighashCache::new(&mut send_tx);
            for (i, (_utxo, destination)) in utxos.iter().enumerate() {
                // let account = 
                let (address, pk) = read_state(|s| s.get_address(destination.clone())).map_err(|e| ArgumentError(e.to_string()))?;
                let hash = sighasher.signature_hash(i, &address.to_script(chain_params), EcdsaSighashType::All).map_err(
                    |e| SignFailed(e.to_string())
                )?;
                let sig = sign_with(&key_name, destination.derivation_path(), *hash).await.map_err(
                    |e| SignFailed(e.to_string())
                )?;
                let signature = Signature::from_compact(&sig).map_err(|e| SignFailed(e.to_string())  )?;
                sighasher
                    .set_input_script(
                        i,
                        &SighashSignature {
                            signature,
                            sighash_type: EcdsaSighashType::All,
                        },
                        &PublicKey::from_slice(&pk)
                        .map_err(|e| 
                            SignFailed(e.to_string())
                         )?,
                    )
                    .map_err(|e| SignFailed(e.to_string()) )?;
            }

            // 4. send transaction
        
            let doge_rpc: DogeRpc = read_state(|s| s.default_rpc_config.clone()).into();
            let txid = doge_rpc.send_transaction(&send_tx).await.map_err(|e| SendTransactionFailed(e.to_string()))?;

            mutate_state(|s| {
                if send_tx.output.len() == 2 {
                    s.deposited_utxo.push((Utxo {
                        txid: crate::types::Txid::from(txid).into(),
                        vout: 1,
                        value: send_tx.output[1].value,
                    }, Destination::change_address()));
                }
            });
            
            Ok(Some(SendTicketResult { 
                txid: send_tx.compute_txid().into(), 
                success: true, 
                time_at: ic_cdk::api::time() 
            }))
        }
    }
}

pub fn select_utxos(amount: u64) -> CustomToBitcoinResult<(Vec<(Utxo, Destination)>, u64)> {
    let mut selected_utxos: Vec<(Utxo, Destination)> = vec![];
    let mut total = 0u64;
    mutate_state(|s| {
        while total < amount && s.deposited_utxo.len() > 0 {
            let (utxo, d) = s.deposited_utxo.pop().ok_or(CustomToBitcoinError::InsufficientFunds)?;
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