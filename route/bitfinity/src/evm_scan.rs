use anyhow::anyhow;
use did::{TransactionReceipt, transaction::TransactionReceiptLog};
use ethers_core::abi::RawLog;
use ethers_core::utils::hex::ToHexExt;
use ic_canister_log::log;
use itertools::Itertools;

use omnity_types::{ChainState, Directive, Ticket, ChainId, Fee};
use omnity_types::ic_log::{CRITICAL, ERROR, INFO};

use crate::*;
use crate::const_args::SCAN_EVM_TASK_NAME;
use crate::contract_types::{
    AbiSignature, DecodeLog, DirectiveExecuted, RunesMintRequested, TokenAdded,
    TokenBurned, TokenMinted, TokenTransportRequested,
};
use crate::convert::{ticket_from_burn_event, ticket_from_runes_mint_event, ticket_from_transport_event};
use crate::state::{mutate_state, read_state, bitfinity_get_redeem_fee};

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
            if read_state(|s| s.handled_evm_event.contains(&hash)) {
                log!(INFO, "[Consolidation] bitfinity route finalized hash: {:?}", &hash);
                mutate_state(|s| s.pending_events_on_chain.remove(&hash));
                continue;
            }
            let now = get_time_secs();
            if now - time < interval || now - time > interval * 5 {
                continue;
            }
            let receipt = crate::eth_common::get_transaction_receipt(&hash)
                .await
                .map_err(|e| {
                    log!(ERROR, "user query transaction receipt error: {:?}", e);
                    "rpc".to_string()
                });           
            if let Ok(Some(tr)) = receipt {
                match tr.status {
                    None => { continue }
                    Some(s) => {
                        if s == did::U64::zero() {
                            log!(INFO, "[Consolidation] bitfinity route finalized hash: {:?}", &hash);
                            mutate_state(|s| s.pending_events_on_chain.remove(&hash));
                            continue;
                        }
                    }
                }
                let res = handle_port_events(tr.logs.clone()).await;
                match res {
                    Ok(_) => {
                        log!(INFO, "[Consolidation] bitfinity route finalized hash: {:?}", &hash);
                        mutate_state(|s| s.pending_events_on_chain.remove(&hash));
                        mutate_state(|s| s.handled_evm_event.insert(hash));
                    }
                    Err(e) => {
                        log!(ERROR, "[bitfinity route] handle evm logs error: {}", e.to_string());
                    }
                }
            }
        }
    });
}

pub async fn handle_port_events(logs: Vec<TransactionReceiptLog>) -> anyhow::Result<()> {
    for l in logs {
        if l.removed {
            return Err(anyhow!("log is removed"));
        }
        let tx_hash = l
            .transaction_hash.to_hex_str();
        let topic1 = l.topics.first().ok_or(anyhow!("topic is none"))?.0.0;
        let raw_log: RawLog = RawLog {
            topics: l.topics.iter().map(|topic| topic.0).collect_vec(),
            data: l.data.clone().into(),
        };
        if read_state(|s| s.handled_evm_event.contains(&l.transaction_hash.to_hex_str())) {
            continue;
        }
        if topic1 == TokenBurned::signature_hash() {
            let token_burned = TokenBurned::decode_log(&raw_log)
                .map_err(|e| super::BitfinityRouteError::ParseEventError(e.to_string()))?;
            handle_token_burn(&l, token_burned.clone()).await?;
        } else if topic1 == TokenMinted::signature_hash() {
            let token_mint = TokenMinted::decode_log(&raw_log)
                .map_err(|e| super::BitfinityRouteError::ParseEventError(e.to_string()))?;
            mutate_state(|s| s.pending_tickets_map.remove(&token_mint.ticket_id));
            mutate_state(|s| {
                s.finalized_mint_token_requests
                    .insert(token_mint.ticket_id.clone(), tx_hash.clone())
            });
        } else if topic1 == TokenTransportRequested::signature_hash() {
            let token_transport = TokenTransportRequested::decode_log(&raw_log)
                .map_err(|e| super::BitfinityRouteError::ParseEventError(e.to_string()))?;
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
                log!(INFO, "[bitfinity route] received a transport ticket with a unknown or deactived dst chain, ignore, txhash={}" ,&tx_hash);
            }
        } else if topic1 == DirectiveExecuted::signature_hash() {
            let directive_executed = DirectiveExecuted::decode_log(&raw_log)
                .map_err(|e| BitfinityRouteError::ParseEventError(e.to_string()))?;
            mutate_state(|s| s.pending_directive_map.remove(&directive_executed.seq.0[0]));
            let directive =
                read_state(|s| s.directives_queue.get(&directive_executed.seq.0[0]).clone())
                    .expect("directive not found");
            match directive.clone() {
                Directive::AddToken(token) => {
                    match crate::updates::add_new_token(token.clone()).await {
                        Ok(_) => {
                            log!(INFO,
                                "[process directives] add token successful, token id: {}",
                                token.token_id
                            );
                        }
                        Err(err) => {
                            log!(ERROR,
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
                    log!(INFO, "[process_directives] success to update fee, fee: {}", fee);
                }
                Directive::UpdateChain(_) | Directive::UpdateToken(_) | Directive::AddChain(_) => {
                    //the directive need not send to port, it had been processed in fetch hub task.
                }
            }
        } else if topic1 == TokenAdded::signature_hash() {
            let token_added = TokenAdded::decode_log(&raw_log)
                .map_err(|e| BitfinityRouteError::ParseEventError(e.to_string()))?;
            mutate_state(|s| {
                s.token_contracts.insert(
                    token_added.token_id,
                    token_added.token_address.encode_hex_with_prefix(),
                )
            });
        } else if topic1 == RunesMintRequested::signature_hash() {
            let runes_mint = RunesMintRequested::decode_log(&raw_log)
                .map_err(|e| BitfinityRouteError::ParseEventError(e.to_string()))?;
            handle_runes_mint(&l, runes_mint).await?;
        }
    }
    Ok(())
}

pub async fn handle_runes_mint(
    log_entry: &TransactionReceiptLog,
    event: RunesMintRequested,
) -> anyhow::Result<()> {
    let ticket = ticket_from_runes_mint_event(log_entry, event, false);
    hub::finalize_ticket(crate::state::hub_addr(), ticket.ticket_id.clone())
        .await
        .map_err(|e| BitfinityRouteError::HubError(e.to_string()))?;
    log!(INFO,
        "[bitfinity route] rune_mint_ticket sent to hub success: {:?}",
        ticket
    );
    Ok(())
}

pub async fn handle_token_burn(log_entry: &TransactionReceiptLog, event: TokenBurned) -> anyhow::Result<()> {
    let ticket = ticket_from_burn_event(log_entry, event, false);
    hub::finalize_ticket(crate::state::hub_addr(), ticket.ticket_id.clone())
        .await
        .map_err(|e| BitfinityRouteError::HubError(e.to_string()))?;
    log!(INFO, "[bitfinity route] burn_ticket sent to hub success: {:?}", ticket);
    Ok(())
}

pub async fn handle_token_transport(
    log_entry: &TransactionReceiptLog,
    event: TokenTransportRequested,
) -> anyhow::Result<()> {
    let ticket = ticket_from_transport_event(log_entry, event, false);
    hub::finalize_ticket(crate::state::hub_addr(), ticket.ticket_id.clone())
        .await
        .map_err(|e| BitfinityRouteError::HubError(e.to_string()))?;
    log!(INFO,
        "[bitfinity route] transport_ticket sent to hub success: {:?}",
        ticket
    );
    Ok(())
}

pub async fn create_ticket_by_tx(tx_hash: &String) -> Result<(Ticket, TransactionReceipt), String> {
    let receipt = crate::eth_common::get_transaction_receipt(tx_hash)
        .await
        .map_err(|e| {
            log!(CRITICAL, "user query transaction receipt error: {:?}", e);
            "rpc".to_string()
        })?;
    match receipt {
        None => Err("not find".to_string()),
        Some(tr) => {
            let return_tr = tr.clone();
            assert_eq!(tr.status, Some(did::U64::one()), "transaction failed");         
            let ticket = generate_ticket_by_logs(tr.logs);
            let t = ticket.map_err(|e| e.to_string())?;
            Ok((t, return_tr))
        }
    }
}

pub fn generate_ticket_by_logs(logs: Vec<TransactionReceiptLog>) -> anyhow::Result<Ticket> {
    for l in logs {
        if l.removed {
            return Err(anyhow!("log is removed"));
        }
        let topic1 = l.topics.first().ok_or(anyhow!("topic is none"))?.0.0;
        let raw_log: RawLog = RawLog {
            topics: l.topics.iter().map(|topic| topic.0).collect_vec(),
            data: l.data.clone().into(),
        };
        if topic1 == TokenBurned::signature_hash() {
            let token_burned = TokenBurned::decode_log(&raw_log)
                .map_err(|e| super::BitfinityRouteError::ParseEventError(e.to_string()))?;
            return Ok(ticket_from_burn_event(&l, token_burned, true));
        } else if topic1 == TokenTransportRequested::signature_hash() {
            let token_transport = TokenTransportRequested::decode_log(&raw_log)
                .map_err(|e| super::BitfinityRouteError::ParseEventError(e.to_string()))?;
            let dst_check_result = read_state(|s| {
                let r = s.counterparties.get(&token_transport.dst_chain_id);
                match r {
                    None => false,
                    Some(c) => c.chain_state == ChainState::Active,
                }
            });
            if dst_check_result {
                return Ok(ticket_from_transport_event(&l, token_transport, true));
            } else {
                let tx_hash = l.transaction_hash.to_hex_str();
                log!(INFO, "[bitfinity route] received a transport ticket with a unknown or deactived dst chain, ignore, txhash={}" ,tx_hash);
            }
        } else if topic1 == RunesMintRequested::signature_hash() {
            let runes_mint = RunesMintRequested::decode_log(&raw_log)
                .map_err(|e| BitfinityRouteError::ParseEventError(e.to_string()))?;
            return Ok(ticket_from_runes_mint_event(&l, runes_mint, true));
        }
    }
    Err(anyhow!("not found ticket"))
}

pub fn get_memo(memo: Option<String>, dst_chain: ChainId) -> Option<String> {
    let fee = bitfinity_get_redeem_fee(dst_chain);
    let bridge_fee = Fee {bridge_fee: fee.unwrap_or_default() as u128};
    bridge_fee.add_to_memo(memo).unwrap_or_default()
}