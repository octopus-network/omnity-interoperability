use anyhow::anyhow;
use cketh_common::{eth_rpc::LogEntry, eth_rpc_client::RpcConfig};
use cketh_common::eth_rpc::Hash;
use ethers_core::abi::RawLog;
use ethers_core::utils::hex::ToHexExt;
use evm_rpc::{
    MultiRpcResult, RpcServices,
};
use evm_rpc::candid_types::TransactionReceipt;
use itertools::Itertools;
use log::{error, info};

use crate::*;
use crate::const_args::{SCAN_EVM_CYCLES, SCAN_EVM_TASK_NAME};
use crate::contract_types::{
    AbiSignature, DecodeLog, DirectiveExecuted, RunesMintRequested, TokenAdded, TokenBurned,
    TokenMinted, TokenTransportRequested,
};
use crate::state::{mutate_state, read_state};
use crate::types::{ChainState, Directive, Ticket};

pub fn scan_evm_task() {
    ic_cdk::spawn(async {
        let _guard = match crate::guard::TimerLogicGuard::new(SCAN_EVM_TASK_NAME.to_string()) {
            Some(guard) => guard,
            None => return,
        };
        let events = read_state(|s| s.pending_events_on_chain.clone());
        let interval =
            read_state(|s| s.block_interval_secs) * crate::const_args::EVM_FINALIZED_CONFIRM_HEIGHT;
        for (hash, time) in events {
            if read_state(|s| s.handled_evm_event.contains(&hash.to_lowercase())) {
                continue;
            }
            let now = get_time_secs();
            if now - time < interval || now - time > interval * 5 {
                continue;
            }
            let receipt = crate::evm_scan::get_transaction_receipt(&hash)
                .await
                .map_err(|e| {
                    log::error!("user query transaction receipt error: {:?}", e);
                    "rpc".to_string()
                });
            if let Ok(Some(tr)) = receipt {
                if tr.status == 0 {
                    mutate_state(|s| s.pending_events_on_chain.remove(&hash));
                    continue;
                }
                let res = handle_port_events(tr.logs.clone()).await;
                match res {
                    Ok(_) => {
                        mutate_state(|s| s.handled_evm_event.insert(hash.clone().to_lowercase()));
                        mutate_state(|s| s.pending_events_on_chain.remove(&hash));
                    }
                    Err(e) => {
                        error!("[evm route] handle evm logs error: {}", e.to_string());
                    }
                }
            }
        }
    });
}

pub async fn handle_port_events(logs: Vec<LogEntry>) -> anyhow::Result<()> {
    for l in logs {
        if l.removed {
            return Err(anyhow!("log is removed"));
        }
        let block = l.block_number.ok_or(anyhow!("block is pending"))?;
        let log_index = l.log_index.ok_or(anyhow!("log is pending"))?;
        let log_key = std::format!("{}-{}", block, log_index);
        let tx_hash = l
            .transaction_hash
            .unwrap_or(cketh_common::eth_rpc::Hash([0u8; 32]))
            .to_string();
        let topic1 = l.topics.first().ok_or(anyhow!("topic is none"))?.0;
        let raw_log: RawLog = RawLog {
            topics: l.topics.iter().map(|topic| topic.0.into()).collect_vec(),
            data: l.data.0.clone(),
        };
        if read_state(|s| s.handled_evm_event.contains(&log_key)) {
            continue;
        }

        if topic1 == TokenBurned::signature_hash() {
            let token_burned = TokenBurned::decode_log(&raw_log)
                .map_err(|e| super::Error::ParseEventError(e.to_string()))?;
            handle_token_burn(&l, token_burned.clone()).await?;
        } else if topic1 == TokenMinted::signature_hash() {
            let token_mint = TokenMinted::decode_log(&raw_log)
                .map_err(|e| super::Error::ParseEventError(e.to_string()))?;
            mutate_state(|s| s.pending_tickets_map.remove(&token_mint.ticket_id));
            mutate_state(|s| {
                s.finalized_mint_token_requests
                    .insert(token_mint.ticket_id.clone(), tx_hash.clone())
            });
        } else if topic1 == TokenTransportRequested::signature_hash() {
            let token_transport = TokenTransportRequested::decode_log(&raw_log)
                .map_err(|e| super::Error::ParseEventError(e.to_string()))?;
            let dst_check_result = read_state(|s| {
                let r = s.counterparties.get(&token_transport.dst_chain_id);
                match r {
                    None => false,
                    Some(c) => c.chain_state == ChainState::Active,
                }
            });
            if dst_check_result {
                handle_token_transport(&l, token_transport).await?;
            } else {
                let tx_hash = l
                    .transaction_hash
                    .unwrap_or(Hash([0u8; 32]))
                    .to_string();
                info!("[evm route] received a transport ticket with a unknown or deactived dst chain, ignore, txhash={}" ,tx_hash );
            }
        } else if topic1 == DirectiveExecuted::signature_hash() {
            let directive_executed = DirectiveExecuted::decode_log(&raw_log)
                .map_err(|e| Error::ParseEventError(e.to_string()))?;
            mutate_state(|s| s.pending_directive_map.remove(&directive_executed.seq.0[0]));
            let directive =
                read_state(|s| s.directives_queue.get(&directive_executed.seq.0[0]).clone())
                    .expect("directive not found");
            match directive.clone() {
                Directive::AddToken(token) => {
                    match crate::updates::add_new_token(token.clone()).await {
                        Ok(_) => {
                            log::info!(
                                "[process directives] add token successful, token id: {}",
                                token.token_id
                            );
                        }
                        Err(err) => {
                            log::error!(
                                "[process directives] failed to add token: token id: {}, err: {:?}",
                                token.token_id,
                                err
                            );
                        }
                    }
                }
                Directive::ToggleChainState(toggle) => {
                    mutate_state(|s| audit::toggle_chain_state(s, toggle.clone()));
                }
                Directive::UpdateFee(fee) => {
                    mutate_state(|s| audit::update_fee(s, fee.clone()));
                    info!("[process_directives] success to update fee, fee: {}", fee);
                }
                Directive::UpdateChain(_) | Directive::UpdateToken(_) | Directive::AddChain(_) => {
                    //the directive need not send to port, it had been processed in fetch hub task.
                }
            }
        } else if topic1 == TokenAdded::signature_hash() {
            let token_added = TokenAdded::decode_log(&raw_log)
                .map_err(|e| Error::ParseEventError(e.to_string()))?;
            mutate_state(|s| {
                s.token_contracts.insert(
                    token_added.token_id,
                    token_added.token_address.encode_hex_with_prefix(),
                )
            });
        } else if topic1 == RunesMintRequested::signature_hash() {
            let runes_mint = RunesMintRequested::decode_log(&raw_log)
                .map_err(|e| Error::ParseEventError(e.to_string()))?;
            handle_runes_mint(&l, runes_mint).await?;
        }
    }
    Ok(())
}

pub async fn handle_runes_mint(
    log_entry: &LogEntry,
    event: RunesMintRequested,
) -> anyhow::Result<()> {
    let ticket = Ticket::from_runes_mint_event(log_entry, event);
    ic_cdk::call(crate::state::hub_addr(), "send_ticket", (ticket.clone(), ))
        .await
        .map_err(|(_, s)| Error::HubError(s))?;
    info!("[evm_route] rune_mint_ticket sent to hub success: {:?}", ticket);
    Ok(())
}

pub async fn handle_token_burn(log_entry: &LogEntry, event: TokenBurned) -> anyhow::Result<()> {
    let ticket = Ticket::from_burn_event(log_entry, event);
    ic_cdk::call(crate::state::hub_addr(), "send_ticket", (ticket.clone(), ))
        .await
        .map_err(|(_, s)| Error::HubError(s))?;
    info!("[evm_route] burn_ticket sent to hub success: {:?}", ticket);
    Ok(())
}

pub async fn handle_token_transport(
    log_entry: &LogEntry,
    event: TokenTransportRequested,
) -> anyhow::Result<()> {
    let ticket = Ticket::from_transport_event(log_entry, event);
    ic_cdk::call(crate::state::hub_addr(), "send_ticket", (ticket.clone(), ))
        .await
        .map_err(|(_, s)| Error::HubError(s))?;
    info!("[evm_route] transport_ticket sent to hub success: {:?}", ticket);
    Ok(())
}

pub async fn get_transaction_receipt(
    hash: &String,
) -> std::result::Result<Option<TransactionReceipt>, Error> {
    let rpc_size = read_state(|s| s.rpc_providers.len() as u128);
    let (rpc_result,): (MultiRpcResult<Option<TransactionReceipt>>,) =
        ic_cdk::api::call::call_with_payment128(
            crate::state::rpc_addr(),
            "eth_getTransactionReceipt",
            (
                RpcServices::Custom {
                    chain_id: crate::state::evm_chain_id(),
                    services: crate::state::rpc_providers(),
                },
                None::<RpcConfig>,
                hash,
            ),
            SCAN_EVM_CYCLES * rpc_size,
        )
        .await
        .map_err(|err| Error::IcCallError(err.0, err.1))?;
    match rpc_result {
        MultiRpcResult::Consistent(result) => result.map_err(|e| {
            error!("query transaction receipt error: {:?}", e.clone());
            Error::EvmRpcError(format!("{:?}", e))
        }),
        MultiRpcResult::Inconsistent(_) => {
            Err(super::Error::EvmRpcError("Inconsistent result".to_string()))
        }
    }
}

pub async fn create_ticket_by_tx(tx_hash: &String) -> Result<(Ticket, TransactionReceipt), String> {
    let receipt = crate::evm_scan::get_transaction_receipt(tx_hash)
        .await
        .map_err(|e| {
            log::error!("user query transaction receipt error: {:?}", e);
            "rpc".to_string()
        })?;
    match receipt {
        None => {
            Err("not find".to_string())
        }
        Some(tr) => {
            let return_tr = tr.clone();
            assert_eq!(tr.status, 1, "transaction failed");
            let ticket = generate_ticket_by_logs(tr.logs);
            let t = ticket.map_err(|e| e.to_string())?;
            Ok((t, return_tr))
        }
    }
}

pub fn generate_ticket_by_logs(logs: Vec<LogEntry>) -> anyhow::Result<Ticket> {
    for l in logs {
        if l.removed {
            return Err(anyhow!("log is removed"));
        }
        let topic1 = l.topics.first().ok_or(anyhow!("topic is none"))?.0;
        let raw_log: RawLog = RawLog {
            topics: l.topics.iter().map(|topic| topic.0.into()).collect_vec(),
            data: l.data.0.clone(),
        };
        if topic1 == TokenBurned::signature_hash() {
            let token_burned = TokenBurned::decode_log(&raw_log)
                .map_err(|e| super::Error::ParseEventError(e.to_string()))?;
            return Ok(Ticket::from_burn_event(&l, token_burned));
        } else if topic1 == TokenTransportRequested::signature_hash() {
            let token_transport = TokenTransportRequested::decode_log(&raw_log)
                .map_err(|e| super::Error::ParseEventError(e.to_string()))?;
            let dst_check_result = read_state(|s| {
                let r = s.counterparties.get(&token_transport.dst_chain_id);
                match r {
                    None => false,
                    Some(c) => c.chain_state == ChainState::Active,
                }
            });
            if dst_check_result {
                return Ok(Ticket::from_transport_event(&l, token_transport));
            } else {
                let tx_hash = l
                    .transaction_hash
                    .unwrap_or(Hash([0u8; 32]))
                    .to_string();
                info!("[evm route] received a transport ticket with a unknown or deactived dst chain, ignore, txhash={}" ,tx_hash);
            }
        } else if topic1 == RunesMintRequested::signature_hash() {
            let runes_mint = RunesMintRequested::decode_log(&raw_log)
                .map_err(|e| Error::ParseEventError(e.to_string()))?;
            return Ok(Ticket::from_runes_mint_event(&l, runes_mint));
        }
    }
    Err(anyhow!("not found ticket"))
}
