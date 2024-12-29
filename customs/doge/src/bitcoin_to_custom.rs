use crate::constants::FINALIZE_LOCK_TICKET_NAME;
use crate::doge::chainparams::DOGE_MAIN_NET_CHAIN;
use crate::doge::rpc::DogeRpc;
use crate::doge::script::classify_script;
use crate::doge::transaction::Transaction ;
use crate::errors::CustomsError;
use crate::generate_ticket::GenerateTicketArgs;
use crate::hub;
use crate::state::{finalization_time_estimate, mutate_state, read_state};
use crate::types::{Destination, LockTicketRequest, Txid};
use ic_canister_log::log;
use omnity_types::ic_log::{ERROR, INFO};


pub async fn check_transaction(
    req: GenerateTicketArgs,
) -> Result<(Transaction, u64, Option<String> ), CustomsError> {
    read_state(|s| s.tokens.get(&req.token_id).cloned())
        .ok_or(CustomsError::InvalidArgs(serde_json::to_string(&req).unwrap()))?;
    read_state(|s| s.counterparties.get(&req.target_chain_id).cloned())
        .ok_or(CustomsError::InvalidArgs(serde_json::to_string(&req).unwrap()))?;
 
    let doge_rpc: DogeRpc = read_state(|s| s.default_rpc_config.clone()).into();

    let transaction = doge_rpc.get_raw_transaction(&req.txid).await?;
    
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

    // let first_txout = transaction.output.first().cloned().ok_or(CustomsError::CustomError("transaction output is empty".to_string()))?;

    // let receiver = first_txout.get_mainnet_address().ok_or(CustomsError::CustomError("first output receiver address is empty".to_string()))?;
    // let amount = first_txout.value;

    // receiver should be destination address
    let destination = Destination::new(req.target_chain_id.clone(), req.receiver.clone(), None);

    let (destination_to_address, _ ) = read_state(|s| s.get_address(destination.clone()))?;

    let mut amount = 0;
    let sender = transaction.input.first().and_then(|input| {
        let (_, addr_opt) = classify_script(input.script.clone().as_bytes(), &DOGE_MAIN_NET_CHAIN);
        addr_opt.map(|e| e.to_string().clone())
    });
    for tx_out in transaction.output.clone() {
        let (_, addr_opt) = classify_script(tx_out.script_pubkey.as_bytes(), &DOGE_MAIN_NET_CHAIN);
        if let Some(addr) = addr_opt {
            if addr.to_string() == destination_to_address.to_string() {
                amount += tx_out.value;
            }
        }
    }

    if amount == 0 {
        return Err(CustomsError::DepositUtxoNotFound(req.txid.clone(), destination));
    }
    
    Ok((transaction, amount, sender))
}

// pub async fn query_transaction(
//     (txid, url): (String, String),
// ) -> Result<Transaction, CustomsError> {
//     const MAX_CYCLES: u128 = 60_000_000_000;

//     let request = CanisterHttpRequestArgument {
//         url: url.clone(),
//         method: HttpMethod::POST,
//         body: Some(json!({
//             "jsonrpc": "2.0",
//             "method": "getrawtransaction",
//             "params": [txid.clone()],
//             "id": 1
//         }).to_string().into_bytes()),
//         max_response_bytes: Some(KB100),
//         transform: Some(TransformContext {
//             function: TransformFunc(candid::Func {
//                 principal: ic_cdk::api::id(),
//                 method: "transform".to_string(),
//             }),
//             context: vec![],
//         }),
//         headers: vec![HttpHeader {
//             name: "Content-Type".to_string(),
//             value: "application/json".to_string(),
//         }],
//     };

//     match http_request(request, MAX_CYCLES).await {
//         Ok((response,)) => {
//             let status = response.status;
//             if status == 200_u32 {
//                 let body = String::from_utf8(response.body).map_err(|_| {
//                     CustomsError::RpcError(
//                         "Transformed response is not UTF-8 encoded".to_string(),
//                     )
//                 })?;
//                 log!(INFO, "tx content: {}", &body);
//                 let raw_tx: RawTransaction = serde_json::from_str(&body).map_err(|e| {
//                     log!(CRITICAL, "json error {:?}", e);
//                     CustomsError::RpcError(
//                         "failed to decode transaction from json".to_string(),
//                     )
//                 })?;
//                 if raw_tx.error.is_some() {
//                     return Err(CustomsError::RpcError(
//                         format!("failed to get transaction: {:?}", raw_tx.error.unwrap()),
//                     ));
//                 }
//                 let tx: Transaction = deserialize_hex(&raw_tx.result).map_err(wrap_to_customs_error)?;
//                 Ok(tx)
//             } else {
//                 Err(CustomsError::RpcError(
//                     "http response not 200".to_string(),
//                 ))
//             }
//         }
//         Err((_, m)) => Err(CustomsError::RpcError(m)),
//     }
// }

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
                (req.1.received_at + wait_time < now) && (req.1.received_at + wait_time * 6 > now)
            })
            .map(|req| (req.0.clone(), req.1.clone()))
            .collect::<Vec<(Txid, LockTicketRequest)>>()
    });
    for (txid, _) in should_check_finalizations.clone() {
        match check_tx_confirmation(txid.clone()).await {
            Ok(can_finalize) => {
                if can_finalize {
                    match finalize_ticket(txid.clone().into()).await {
                        Ok(_) => {
                            log!(INFO, "finalize lock success: {:?}", txid);
                        },
                        Err(e) => {
                            log!(ERROR, "finalize lock error: {:?}", e);
                        },
                    }

                }
            },
            Err(e) => {
                log!(ERROR, "finalize lock error: {:?}", e);
            },
        }
    }
}

pub async fn check_tx_confirmation(
    txid: Txid,
)-> Result<bool, CustomsError> {
     
    let doge_rpc: DogeRpc = read_state(|s| s.default_rpc_config.clone()).into();
    let tx_out = doge_rpc.get_tx_out(txid.to_string().as_str()).await?;
    let min_confirmations = read_state(|s| s.min_confirmations);
    return Ok(tx_out.confirmations >= min_confirmations);
}

async fn finalize_ticket(txid: Txid)-> Result<(), CustomsError> {
    let hub_principal = read_state(|s| s.hub_principal);
    hub::finalize_ticket(hub_principal, txid.to_string())
    .await
    .map_err(|e| {
        CustomsError::CallError(
            hub_principal,
            e.method,
            e.reason.to_string()
        )
    })?;

    mutate_state(|s| {
        let v = s.pending_lock_ticket_requests.remove(&txid).ok_or(
            CustomsError::CustomError("pending lock ticket request not found".to_string())
        )?;
        s.finalized_lock_ticket_requests.insert(txid, v.clone());
        s.save_utxo(v)?;

        Ok(())
    })?;

    Ok(())
}