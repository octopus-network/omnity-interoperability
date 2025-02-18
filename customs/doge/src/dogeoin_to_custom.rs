use std::str::FromStr;

use crate::constants::FINALIZE_LOCK_TICKET_NAME;
use crate::doge::chainparams::DOGE_MAIN_NET_CHAIN;
use crate::doge::rpc::DogeRpc;
use crate::doge::script::classify_script;
use crate::doge::transaction::Transaction;
use crate::errors::CustomsError;
use crate::generate_ticket::GenerateTicketWithTxidArgs;
use crate::hub;
use crate::state::{finalization_time_estimate, mutate_state, read_state};
use crate::types::{
    deserialize_hex, wrap_to_customs_error, Destination, LockTicketRequest, Txid, Utxo,
};
use bitcoin::block::Header;
use bitcoin::MerkleBlock;
use ic_canister_log::log;
use omnity_types::ic_log::{ERROR, INFO};

pub async fn query_and_save_utxo_for_payment_address(txid: String) -> Result<u64, CustomsError> {
    if read_state(|s| s.deposit_fee_tx_set.get(&txid).is_some()) {
        Err(CustomsError::CustomError("already saved".to_string()))?;
    }

    let doge_rpc: DogeRpc = read_state(|s| s.default_doge_rpc_config.clone()).into();
    let raw_transaction = doge_rpc.get_raw_transaction(&txid).await?;
    let transaction: Transaction =
        deserialize_hex(&raw_transaction.hex).map_err(wrap_to_customs_error)?;
    let (fee_payment_address, _) =
        read_state(|s| s.get_address(Destination::fee_payment_address()))?;
    let typed_txid = Txid::from_str(&txid).map_err(|_| CustomsError::InvalidTxId)?;
    let mut total = 0;
    for (i, out) in transaction.output.iter().enumerate() {
        let receiver = classify_script(out.script_pubkey.as_bytes(), &DOGE_MAIN_NET_CHAIN)
            .1
            .ok_or(CustomsError::CustomError(
                "failed to get receiver from output".to_string(),
            ))?;
        if receiver.eq(&fee_payment_address) {
            total += out.value;
            mutate_state(|s| {
                s.fee_payment_utxo.push(Utxo {
                    txid: typed_txid.clone(),
                    vout: i as u32,
                    value: out.value,
                });
            })
        }
    }

    if total > 0 {
        mutate_state(|s| {
            s.deposit_fee_tx_set.insert(txid, ());
        })
    }

    Ok(total)
}

pub async fn check_transaction(
    req: GenerateTicketWithTxidArgs,
) -> Result<(Transaction, u64, String), CustomsError> {
    read_state(|s| s.tokens.get(&req.token_id).cloned()).ok_or(CustomsError::InvalidArgs(
        serde_json::to_string(&req).unwrap(),
    ))?;
    read_state(|s| s.counterparties.get(&req.target_chain_id).cloned()).ok_or(
        CustomsError::InvalidArgs(serde_json::to_string(&req).unwrap()),
    )?;

    let default_doge_rpc: DogeRpc = read_state(|s| s.default_doge_rpc_config.clone()).into();
    let multi_rpc_config = read_state(|s| s.multi_rpc_config.clone());
    let transaction_json_result = if multi_rpc_config.rpc_list.len() > 0 {
        multi_rpc_config
            .get_raw_transaction_json_data(&req.txid)
            .await?
    } else {
        default_doge_rpc.get_raw_transaction(&req.txid).await?
    };

    log!(INFO, "get transaction json: {:?}", transaction_json_result);

    let transaction: Transaction = transaction_json_result.try_into().map_err(|e| {
        CustomsError::CustomError(format!(
            "failed to convert transaction json to transaction: {:?}",
            e
        ))
    })?;

    //check whether need to pay fees for transfer. If fee is None, that means paying fees is not need
    let (fee, addr) = read_state(|s| s.get_transfer_fee_info(&req.target_chain_id));
    match fee {
        None => {}
        Some(fee_value) => {
            let mut found_fee_utxo = false;
            let fee_collector = addr.unwrap();
            for out in &transaction.output {
                let (_, addr_opt) =
                    classify_script(out.script_pubkey.as_bytes(), &DOGE_MAIN_NET_CHAIN);
                if let Some(addr) = addr_opt {
                    let addr_str = addr.to_string();
                    if addr_str.eq(&fee_collector) && out.value as u128 == fee_value {
                        found_fee_utxo = true;
                        break;
                    }
                }
            }
            if !found_fee_utxo {
                return Err(CustomsError::NotPayFees);
            }
        }
    }

    // receiver should be destination address
    let destination = Destination::new(req.target_chain_id.clone(), req.receiver.clone(), None);

    let (destination_to_address, _) = read_state(|s| s.get_address(destination.clone()))?;

    let mut amount = 0;
    let first_input = transaction
        .input
        .first()
        .ok_or(CustomsError::DepositUtxoNotFound(
            req.txid.clone(),
            destination.clone(),
        ))?;

    let raw_transaction = default_doge_rpc
        .get_raw_transaction(&first_input.prevout.txid.to_string())
        .await?;
    let transaction_of_input: Transaction =
        deserialize_hex(&raw_transaction.hex).map_err(wrap_to_customs_error)?;
    let output_of_input = transaction_of_input
        .output
        .get(first_input.prevout.vout as usize)
        .ok_or(CustomsError::CustomError("input not found".to_string()))?;

    let sender = classify_script(
        output_of_input.script_pubkey.as_bytes(),
        &DOGE_MAIN_NET_CHAIN,
    )
    .1
    .map(|e| e.to_string().clone())
    .ok_or(CustomsError::CustomError(
        "failed to get sender from output_of_input".to_string(),
    ))?;

    for tx_out in transaction.output.clone() {
        let (_, addr_opt) = classify_script(tx_out.script_pubkey.as_bytes(), &DOGE_MAIN_NET_CHAIN);
        if let Some(addr) = addr_opt {
            if addr.to_string() == destination_to_address.to_string() {
                amount += tx_out.value;
            }
        }
    }

    if amount == 0 {
        return Err(CustomsError::DepositUtxoNotFound(
            req.txid.clone(),
            destination,
        ));
    }

    Ok((transaction, amount, sender))
}

pub fn finalize_lock_ticket_task() {
    ic_cdk::spawn(async {
        let _guard = match crate::guard::TimerLogicGuard::new(FINALIZE_LOCK_TICKET_NAME.to_string())
        {
            Some(guard) => guard,
            None => return,
        };
        finalize_lock_ticket_request().await;
    });
}

pub async fn finalize_lock_ticket_request() {
    let now = ic_cdk::api::time();
    let should_check_finalizations = read_state(|s| {
        let wait_time = finalization_time_estimate(s.min_confirmations);
        s.pending_lock_ticket_requests
            .iter()
            .filter(|&req| {
                let wait_time = wait_time.as_nanos() as u64;
                (req.1.received_at + wait_time < now) 
                && (req.1.received_at + wait_time * 6 > now)
            })
            .map(|req| (req.0.clone(), req.1.clone()))
            .collect::<Vec<(Txid, LockTicketRequest)>>()
    });
    for (txid, _) in should_check_finalizations.clone() {
        match check_tx_confirmation_and_verify_by_merkle_root(txid.clone()).await {
            Ok(can_finalize) => {
                if can_finalize {
                    match finalize_ticket(txid.clone().into()).await {
                        Ok(_) => {
                            log!(INFO, "finalize lock success: {:?}", txid);
                        }
                        Err(e) => {
                            log!(ERROR, "finalize lock error: {:?}", e);
                        }
                    }
                }
            }
            Err(e) => {
                log!(ERROR, "finalize lock error: {:?}", e);
            }
        }
    }
}

pub async fn check_tx_confirmation_and_verify_by_merkle_root(
    txid: crate::types::Txid,
) -> Result<bool, CustomsError> {
    use bitcoin::blockdata::transaction::Txid;
    let doge_rpc: DogeRpc = read_state(|s| s.default_doge_rpc_config.clone()).into();
    let raw_transaction = doge_rpc
        .get_raw_transaction(txid.to_string().as_str())
        .await?;

    let min_confirmations = read_state(|s| s.min_confirmations);
    if raw_transaction.confirmations < min_confirmations {
        return Ok(false);
    }

    let block_json = doge_rpc
        .get_block(raw_transaction.blockhash.as_str())
        .await?;
    let mut txids: Vec<Txid> = vec![];
    for hex in &block_json.tx {
        let each_txid = hex
            .parse::<Txid>()
            .map_err(|e| CustomsError::CustomError(format!("failed to parse txid: {:?}", e)))?;
        txids.push(each_txid);
    }

    let block_header: Header = block_json.clone().try_into()?;

    let txid_typed_in_bitcoin = txid
        .to_string()
        .parse::<Txid>()
        .map_err(|e| CustomsError::CustomError(format!("failed to parse txid: {:?}", e)))?;
    let merkle_block =
        MerkleBlock::from_header_txids_with_predicate(&block_header, txids.as_slice(), |t| {
            t.eq(&txid_typed_in_bitcoin)
        });

    let verified_block_json_header = read_state(|s| s.doge_block_headers.get(&block_json.height))
        .ok_or(CustomsError::CustomError(
        "block header not found".to_string(),
    ))?;

    let verified_block_header: Header = verified_block_json_header.clone().try_into()?;

    if verified_block_header.block_hash() != merkle_block.header.block_hash() {
        return Err(CustomsError::MerkleBlockVerifyError(
            merkle_block.header.block_hash().to_string(),
            verified_block_header.block_hash().to_string(),
        ));
    }
    log!(
        INFO,
        "merkle block verify success, txid: {:?}",
        txid
    );

    return Ok(true);
}

async fn finalize_ticket(txid: Txid) -> Result<(), CustomsError> {
    let hub_principal = read_state(|s| s.hub_principal);
    hub::finalize_ticket(hub_principal, txid.to_string())
        .await
        .map_err(|e| CustomsError::CallError(hub_principal, e.method, e.reason.to_string()))?;

    mutate_state(|s| {
        let v = s
            .pending_lock_ticket_requests
            .remove(&txid)
            .ok_or(CustomsError::CustomError(
                "pending lock ticket request not found".to_string(),
            ))?;
        s.finalized_lock_ticket_requests_map
            .insert(txid.clone(), v.clone());
        s.save_utxo(v)
    })?;

    Ok(())
}
