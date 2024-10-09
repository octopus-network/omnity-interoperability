use crate::call_error::{CallError, Reason};
use crate::constants::FINALIZE_LOCK_TICKET_NAME;
use crate::generate_ticket::GenerateTicketError::InvalidArgs;
use crate::generate_ticket::{GenerateTicketArgs, GenerateTicketError};
use crate::hub;
use crate::ord::inscription::brc20::{Brc20, Brc20Transfer201};
use crate::ord::mempool_rpc_types::TxInfo;
use crate::ord::parser::OrdParser;
use crate::retry::call_rpc_with_retry;
use crate::state::{deposit_addr, finalization_time_estimate, mutate_state, read_state};
use crate::types::{create_query_brc20_transfer_args, LockTicketRequest};
use bitcoin::Transaction;
use ic_btc_interface::{Network, Txid};
use ic_canister_log::log;
use ic_cdk::api::management_canister::http_request::{
    http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod, TransformContext,
    TransformFunc,
};
use num_traits::Zero;
use omnity_types::brc20::{Brc20TransferEvent, QueryBrc20TransferArgs};
use omnity_types::ic_log::{CRITICAL, ERROR, INFO, WARNING};
use std::str::FromStr;

pub async fn check_transaction(
    req: GenerateTicketArgs,
) -> Result<Brc20Transfer201, GenerateTicketError> {
    let token = read_state(|s| s.tokens.get(&req.token_id).cloned())
        .ok_or(InvalidArgs(serde_json::to_string(&req).unwrap()))?;
    let chain = read_state(|s| s.counterparties.get(&req.target_chain_id).cloned())
        .ok_or(InvalidArgs(serde_json::to_string(&req).unwrap()))?;
    let transfer_transfer = call_rpc_with_retry(&req.txid, query_transaction).await?; //query_transaction(&req.txid).await?;
    let receiver = transfer_transfer
        .vout
        .first()
        .cloned()
        .unwrap()
        .scriptpubkey_address
        .unwrap();
    if receiver != deposit_addr().to_string() {
        return Err(GenerateTicketError::InvalidTxId);
    }
    let inscribe_txid = transfer_transfer.vin.first().cloned().unwrap().txid;
    let inscribe_transfer: Transaction = call_rpc_with_retry(&inscribe_txid, query_transaction)
        .await?
        .try_into()
        .map_err(|e: anyhow::Error| GenerateTicketError::RpcError(e.to_string()))?;
    let (_inscription_id, parsed_inscription) = OrdParser::parse_one(&inscribe_transfer, 0)
        .map_err(|e| GenerateTicketError::OrdTxError(e.to_string()))?;
    let brc20 = Brc20::try_from(parsed_inscription)
        .map_err(|e| GenerateTicketError::OrdTxError(e.to_string()))?;
    log!(INFO, "brc20 info:{:?}", serde_json::to_string(&brc20));
    match brc20 {
        Brc20::Brc201Transfer(t) => {
            if t.amt != req.amount
                || t.tick != token.name
                || t.refx.to_lowercase() != req.receiver.to_lowercase()
                || t.chain != chain.chain_id
            {
                Err(InvalidArgs(serde_json::to_string(&t).unwrap()))
            } else {
                Ok(t)
            }
        }
        _ => Err(GenerateTicketError::NotBridgeTx),
    }
}

pub async fn query_transaction(txid: &String) -> Result<TxInfo, GenerateTicketError> {
    let nw = read_state(|s| s.btc_network);
    let network_str = match nw {
        Network::Mainnet => "".to_string(),
        Network::Testnet => "testnet".to_string(),
        Network::Regtest => {
            panic!("unsupported network")
        }
    };
    const MAX_CYCLES: u128 = 30_000_000_000;
    let url = format!("https://mempool.space/{}/api/tx/{}", network_str, txid);

    let request = CanisterHttpRequestArgument {
        url: url.to_string(),
        method: HttpMethod::GET,
        body: None,
        max_response_bytes: None,
        transform: Some(TransformContext {
            function: TransformFunc(candid::Func {
                principal: ic_cdk::api::id(),
                method: "transform".to_string(),
            }),
            context: vec![],
        }),
        headers: vec![HttpHeader {
            name: "Content-Type".to_string(),
            value: "application/json".to_string(),
        }],
    };

    match http_request(request, MAX_CYCLES).await {
        Ok((response,)) => {
            let status = response.status;
            if status == 200_u32 {
                let body = String::from_utf8(response.body).map_err(|_| {
                    GenerateTicketError::RpcError(
                        "Transformed response is not UTF-8 encoded".to_string(),
                    )
                })?;
                let tx: TxInfo = serde_json::from_str(&body).map_err(|_| {
                    GenerateTicketError::RpcError(
                        "failed to decode transaction from json".to_string(),
                    )
                })?;
                Ok(tx)
            } else {
                Err(GenerateTicketError::RpcError(
                    "http response not 200".to_string(),
                ))
            }
        }
        Err((_, m)) => Err(GenerateTicketError::RpcError(m)),
    }
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
    let can_check_finalizations = read_state(|s| {
        let wait_time = finalization_time_estimate(s.min_confirmations, s.btc_network);
        s.pending_lock_ticket_requests
            .iter()
            .filter(|&req| (req.1.received_at + (wait_time.as_nanos() as u64) < now))
            .map(|req| (*req.0, req.1.clone()))
            .collect::<Vec<(Txid, LockTicketRequest)>>()
    });
    let deposit_addr = read_state(|s| s.deposit_addr.clone().unwrap());
    for (seq, gen_ticket_request) in can_check_finalizations.clone() {
        let token = read_state(|s| s.tokens.get(&gen_ticket_request.token_id).cloned());
        match token {
            None => {
                log!(
                    WARNING,
                    "don't found a token named {}",
                    &gen_ticket_request.token_id
                );
            }
            Some(token) => {
                let args = create_query_brc20_transfer_args(
                    gen_ticket_request.clone(),
                    deposit_addr.clone(),
                    token.decimals,
                );
                let query = query_indexed_transfer(args).await;
                if let Ok(Some(_t)) = query {
                    //Check success
                    //FINALIZED TO HUB:
                    let hub_principal = read_state(|s| s.hub_principal);
                    let _r =
                        hub::finalize_ticket(hub_principal, gen_ticket_request.txid.to_string())
                            .await
                            .map_err(|e| {
                                log!(CRITICAL, "finalize gen ticket to hub error: {:?}", &e);
                            });
                    mutate_state(|s| {
                        let v = s.pending_lock_ticket_requests.remove(&seq);
                        s.finalized_lock_ticket_requests.insert(seq, v.unwrap());
                    });
                } else {
                    log!(
                        WARNING,
                        "query indexer failed, will retry. {}",
                        serde_json::to_string(&gen_ticket_request).unwrap()
                    );
                }
            }
        }
    }
}

pub async fn query_indexed_transfer(
    args: QueryBrc20TransferArgs,
) -> Result<Option<Brc20TransferEvent>, CallError> {
    let indexer_principal = read_state(|s| s.indexer_principal);
    let method = "get_indexed_transfer";
    let resp: (Option<Brc20TransferEvent>,) =
        ic_cdk::api::call::call(indexer_principal, method, (args,))
            .await
            .map_err(|(code, message)| {
                log!(ERROR, "query brc20 index error: {:?}, {}", &code, &message);
                CallError {
                    method: method.to_string(),
                    reason: Reason::from_reject(code, message),
                }
            })?;
    Ok(resp.0)
}