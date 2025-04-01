use crate::lightclient::rpc_types::receipt::{encode_receipt, TransactionReceipt};
use anyhow::anyhow;
use const_hex::traits::ToHexExt;
use ethereum_common::convert::{
    ticket_from_burn_event, ticket_from_runes_mint_event, ticket_from_transport_event,
};
use ethers_core::abi::RawLog;
use ic_canister_log::log;
use itertools::Itertools;
use tree_hash::fixed_bytes::B256;

use crate::const_args::SCAN_EVM_TASK_NAME;
use crate::eth_common::{call_rpc_with_retry, checked_get_receipt, get_receipt};
use crate::lightclient::TicketVerifyRecord;
use crate::state::{get_redeem_fee, mutate_state, read_state};
use crate::state_provider::EthereumStateProvider;
use crate::*;
use ethereum_common::contract_types::{
    AbiSignature, DecodeLog, DirectiveExecuted, RunesMintRequested, TokenAdded, TokenBurned,
    TokenMinted, TokenTransportRequested,
};
use ethereum_common::error::Error;
use ethereum_common::evm_log::LogEntry;
use omnity_types::hub;
use omnity_types::ic_log::{INFO, WARNING};
use omnity_types::{ChainId, ChainState, Directive, Memo, Ticket};

pub fn scan_evm_task() {
    ic_cdk::spawn(async {
        let _guard = match crate::guard::TimerLogicGuard::new(SCAN_EVM_TASK_NAME.to_string()) {
            Some(guard) => guard,
            None => return,
        };
        let events = read_state(|s| s.pending_events_on_chain.clone());
        let finality_blocks = const_args::EVM_FINALIZED_CONFIRM_HEIGHT;
        let interval = read_state(|s| s.block_interval_secs) * finality_blocks;
        for (hash, time) in events {
            if read_state(|s| s.handled_evm_event.contains(&hash)) {
                mutate_state(|s| s.pending_events_on_chain.remove(&hash));
                continue;
            }
            let now = get_time_secs();
            if now - time < interval {
                continue;
            }
            if now - time > interval * 5 {
                mutate_state(|s| s.pending_events_on_chain.remove(&hash));
                continue;
            }
            sync_mint_status(hash).await;
        }
    });
}

pub async fn sync_mint_status(hash: String) {
    let min_resp_count = read_state(|s| s.minimum_response_count);
    let receipt = if min_resp_count > 1 {
        checked_get_receipt(&hash).await.map_err(|e| {
            log!(WARNING, "user query transaction receipt error: {:?}", e);
            "rpc".to_string()
        })
    } else {
        call_rpc_with_retry(hash.as_str(), get_receipt).await.map_err(|e| {
            log!(WARNING, "user query transaction receipt error: {:?}", e);
            "rpc".to_string()
        })
    };
    let port_address = read_state(|s| s.omnity_port_contract.clone());
    if let Ok(Some(tr)) = receipt {
        if tr.status.is_none_or(|status| status == 0u64) {
            mutate_state(|s| s.pending_events_on_chain.remove(&hash));
            return;
        }

        if tr.to != port_address.encode_hex_with_prefix() {
            mutate_state(|s| s.pending_events_on_chain.remove(&hash));
            return;
        }

        let res = handle_port_events(tr.logs.clone(), encode_receipt(&tr)).await;
        match res {
            Ok(_) => {
                mutate_state(|s| {
                    s.pending_events_on_chain.remove(&hash);
                    s.handled_evm_event.insert(hash)
                });
            }
            Err(e) => {
                log!(
                    WARNING,
                    "[evm route] handle evm logs error: {}",
                    e.to_string()
                );
            }
        }
    }
}

pub async fn handle_port_events(
    logs: Vec<lightclient::rpc_types::log::LogEntry>,
    receipt_info: Vec<u8>,
) -> anyhow::Result<()> {
    let port = read_state(|s| s.omnity_port_contract.clone());
    for l in logs {
        let common_log: ethereum_common::evm_log::LogEntry = l.clone().into();
        if l.address.encode_hex_with_prefix() != port.encode_hex_with_prefix() {
            continue;
        }
        if l.removed {
            return Err(anyhow!("log is removed"));
        }
        let block = l.block_number.ok_or(anyhow!("block is pending"))?;
        let log_index = l.log_index.ok_or(anyhow!("log is pending"))?;
        let log_key = std::format!("{}-{}", block, log_index);
        let tx_hash = l
            .transaction_hash
            .unwrap_or(B256::default())
            .encode_hex_with_prefix();
        let topic1 = l.topics.first().ok_or(anyhow!("topic is none"))?.0;
        let raw_log: RawLog = RawLog {
            topics: l.topics.iter().map(|topic| topic.0.into()).collect_vec(),
            data: l.data.0.clone(),
        };
        //just for proventing history mistakes. In history.
        // we use log_key as event identifier instead of transaction's hash,
        // so we putted some log_keys into handled_evm_events
        if read_state(|s| s.handled_evm_event.contains(&log_key)) {
            continue;
        }

        if topic1 == TokenBurned::signature_hash() {
            let token_burned = TokenBurned::decode_log(&raw_log)
                .map_err(|e| Error::ParseEventError(e.to_string()))?;
            handle_token_burn(&common_log, token_burned.clone(), receipt_info.clone()).await?;
        } else if topic1 == TokenMinted::signature_hash() {
            let token_mint = TokenMinted::decode_log(&raw_log)
                .map_err(|e| Error::ParseEventError(e.to_string()))?;
            mutate_state(|s| s.pending_tickets_map.remove(&token_mint.ticket_id));
            mutate_state(|s| {
                s.finalized_mint_token_requests
                    .insert(token_mint.ticket_id.clone(), tx_hash.clone())
            });
        } else if topic1 == TokenTransportRequested::signature_hash() {
            let token_transport = TokenTransportRequested::decode_log(&raw_log)
                .map_err(|e| Error::ParseEventError(e.to_string()))?;
            let dst_check_result = read_state(|s| {
                let r = s.counterparties.get(&token_transport.dst_chain_id);
                match r {
                    None => false,
                    Some(c) => c.chain_state == ChainState::Active,
                }
            });
            if dst_check_result {
                handle_token_transport(&common_log, token_transport).await?;
            } else {
                let tx_hash = l
                    .transaction_hash
                    .unwrap_or(B256::default())
                    .encode_hex_with_prefix();
                log!(INFO, "[evm route] received a transport ticket with a unknown or deactived dst chain, ignore, txhash={}" ,tx_hash );
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
                            log!(
                                INFO,
                                "[process directives] add token successful, token id: {}",
                                token.token_id
                            );
                        }
                        Err(err) => {
                            log!(
                                WARNING,
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
                    log!(
                        INFO,
                        "[process_directives] success to update fee, fee: {}",
                        fee
                    );
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
            handle_runes_mint(&common_log, runes_mint).await?;
        }
    }
    Ok(())
}

pub async fn handle_runes_mint(
    log_entry: &LogEntry,
    event: RunesMintRequested,
) -> anyhow::Result<()> {
    let ticket = ticket_from_runes_mint_event::<EthereumStateProvider>(log_entry, event, false);
    hub::finalize_ticket(crate::state::hub_addr(), ticket.ticket_id.clone())
        .await
        .map_err(|e| Error::HubError(e.to_string()))?;
    log!(
        INFO,
        "[evm_route] rune_mint_ticket sent to hub success: {:?}",
        ticket
    );
    Ok(())
}

pub async fn handle_token_burn(
    log_entry: &LogEntry,
    event: TokenBurned,
    receipt_info: Vec<u8>,
) -> anyhow::Result<()> {
    let ticket = ticket_from_burn_event::<EthereumStateProvider>(log_entry, event, false);
    hub::finalize_ticket(crate::state::hub_addr(), ticket.ticket_id.clone())
        .await
        .map_err(|e| Error::HubError(e.to_string()))?;
    log!(
        INFO,
        "[evm_route] burn_ticket sent to hub success: {:?}",
        ticket
    );
    let ticket_verify_record = TicketVerifyRecord {
        receipt: receipt_info,
        block_number: log_entry.block_number.unwrap_or(u64::MAX),
        block_hash: log_entry.block_hash.unwrap_or_default(),
        tx_hash: log_entry.transaction_hash,
        time: ic_cdk::api::time() / 1000000000,
    };
    mutate_state(|s| {
        s.lightclient_verify_requests.insert(
            ticket_verify_record.tx_hash.encode_hex_with_prefix(),
            ticket_verify_record,
        )
    });
    Ok(())
}

pub async fn handle_token_transport(
    log_entry: &LogEntry,
    event: TokenTransportRequested,
) -> anyhow::Result<()> {
    let ticket = ticket_from_transport_event::<EthereumStateProvider>(log_entry, event, false);
    hub::finalize_ticket(crate::state::hub_addr(), ticket.ticket_id.clone())
        .await
        .map_err(|e| Error::HubError(e.to_string()))?;
    log!(
        INFO,
        "[evm_route] transport_ticket sent to hub success: {:?}",
        ticket
    );
    Ok(())
}

pub async fn create_ticket_by_tx(tx_hash: &str) -> Result<(Ticket, TransactionReceipt), String> {
    let receipt = call_rpc_with_retry(tx_hash, get_receipt)
        .await
        .map_err(|e| {
            log!(WARNING, "user query transaction receipt error: {:?}", e);
            "rpc".to_string()
        })?;
    match receipt {
        None => Err("not find".to_string()),
        Some(tr) => {
            let return_tr = tr.clone();
            assert_eq!(tr.status, Some(1u64), "transaction failed");
            let ticket = generate_ticket_by_logs(tr.logs);
            let t = ticket.map_err(|e| e.to_string())?;
            Ok((t, return_tr))
        }
    }
}

pub fn generate_ticket_by_logs(
    logs: Vec<crate::lightclient::rpc_types::log::LogEntry>,
) -> anyhow::Result<Ticket> {
    for l in logs {
        let common_log: ethereum_common::evm_log::LogEntry = l.clone().into();
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
                .map_err(|e| Error::ParseEventError(e.to_string()))?;
            return Ok(ticket_from_burn_event::<EthereumStateProvider>(
                &common_log,
                token_burned,
                true,
            ));
        } else if topic1 == TokenTransportRequested::signature_hash() {
            let token_transport = TokenTransportRequested::decode_log(&raw_log)
                .map_err(|e| Error::ParseEventError(e.to_string()))?;
            let dst_check_result = read_state(|s| {
                let r = s.counterparties.get(&token_transport.dst_chain_id);
                match r {
                    None => false,
                    Some(c) => c.chain_state == ChainState::Active,
                }
            });
            if dst_check_result {
                return Ok(ticket_from_transport_event::<EthereumStateProvider>(
                    &common_log,
                    token_transport,
                    true,
                ));
            } else {
                let tx_hash = l.transaction_hash;
                log!(INFO, "[evm route] received a transport ticket with a unknown or deactived dst chain, ignore, txhash={:?}" ,tx_hash);
            }
        } else if topic1 == RunesMintRequested::signature_hash() {
            let runes_mint = RunesMintRequested::decode_log(&raw_log)
                .map_err(|e| Error::ParseEventError(e.to_string()))?;
            return Ok(ticket_from_runes_mint_event::<EthereumStateProvider>(
                &common_log,
                runes_mint,
                true,
            ));
        }
    }
    Err(anyhow!("not found ticket"))
}

pub fn get_memo(memo: Option<String>, dst_chain: ChainId) -> Option<String> {
    let fee = get_redeem_fee(dst_chain);
    let memo_json = Memo {
        memo,
        bridge_fee: fee.unwrap_or_default() as u128,
    }
    .convert_to_memo_json()
    .unwrap_or_default();
    Some(memo_json)
}
