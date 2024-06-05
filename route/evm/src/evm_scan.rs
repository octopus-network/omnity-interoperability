use crate::const_args::{MAX_SCAN_BLOCKS, SCAN_EVM_TASK_NAME};
use crate::contract_types::{
    AbiSignature, DecodeLog, DirectiveExecuted, TokenBurned, TokenMinted, TokenTransportRequested,
};
use crate::eth_common::get_evm_finalized_height;
use crate::state::{mutate_state, read_state};
use crate::types::{ChainState, Directive, Ticket};
use crate::*;
use anyhow::anyhow;
use cketh_common::{eth_rpc::LogEntry, eth_rpc_client::RpcConfig, numeric::BlockNumber};
use cketh_common::eth_rpc::Hash;
use ethers_core::abi::{AbiEncode, RawLog};
use ethers_core::utils::keccak256;
use evm_rpc::{
    candid_types::{self, BlockTag},
    MultiRpcResult, RpcServices,
};
use itertools::Itertools;
use log::{error, info};

pub fn scan_evm_task() {
    ic_cdk::spawn(async {
        let _guard = match crate::guard::TimerLogicGuard::new(SCAN_EVM_TASK_NAME.to_string()) {
            Some(guard) => guard,
            None => return,
        };
        let r = handle_port_events().await;
        match r {
            Ok(_) => {}
            Err(e) => {
                error!("[evm route] handle evm logs error: {}", e.to_string());
            }
        }
    });
}

pub async fn handle_port_events() -> anyhow::Result<()> {
    let (from, to) = determine_from_to().await?;
    let contract_addr = read_state(|s| s.omnity_port_contract.to_hex());
    let logs = fetch_logs(from, to, contract_addr).await?;
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
            handle_token_burn(&l, token_burned).await?;
        } else if topic1 == TokenMinted::signature_hash() {
            let token_mint = TokenMinted::decode_log(&raw_log)
                .map_err(|e| super::Error::ParseEventError(e.to_string()))?;
            mutate_state(|s| s.pending_tickets_map.remove(&token_mint.ticket_id));
            mutate_state(|s| {
                s.finalized_mint_token_requests
                    .insert(token_mint.ticket_id.clone(), tx_hash)
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
            }else {
                let tx_hash = l.transaction_hash.clone().unwrap_or(Hash([0u8;32])).to_string();
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
                Directive::AddChain(_) => {
                    //the directive need not send to port, it had been processed in fetch hub task.
                }
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
            }
        }
        mutate_state(|s| s.handled_evm_event.insert(log_key));
    }
    mutate_state(|s| s.scan_start_height = to);
    Ok(())
}

pub async fn handle_token_burn(log_entry: &LogEntry, event: TokenBurned) -> anyhow::Result<()> {
    let ticket = Ticket::from_burn_event(log_entry, event);
    ic_cdk::call(crate::state::hub_addr(), "send_ticket", (ticket,))
        .await
        .map_err(|(_, s)| Error::HubError(s))?;
    Ok(())
}

pub async fn handle_token_transport(
    log_entry: &LogEntry,
    event: TokenTransportRequested,
) -> anyhow::Result<()> {
    let ticket = Ticket::from_transport_event(log_entry, event);
    ic_cdk::call(crate::state::hub_addr(), "send_ticket", (ticket,))
        .await
        .map_err(|(_, s)| Error::HubError(s))?;
    Ok(())
}

async fn determine_from_to() -> anyhow::Result<(u64, u64)> {
    let from_height = read_state(|s| s.scan_start_height);
    let to_height = get_evm_finalized_height().await.map_err(|e| {
        error!("query evm block height error: {:?}", e.to_string());
        e
    })?;
    Ok((from_height, to_height.min(from_height + MAX_SCAN_BLOCKS)))
}

pub async fn fetch_logs(
    from_height: u64,
    to_height: u64,
    address: String,
) -> std::result::Result<Vec<LogEntry>, Error> {
    let cycles = 1_000_000_000;
    let (rpc_result,): (MultiRpcResult<Vec<LogEntry>>,) = ic_cdk::api::call::call_with_payment128(
        crate::state::rpc_addr(),
        "eth_getLogs",
        (
            RpcServices::Custom {
                chain_id: crate::state::evm_chain_id(),
                services: crate::state::rpc_providers(),
            },
            None::<RpcConfig>,
            candid_types::GetLogsArgs {
                from_block: Some(BlockTag::Number(BlockNumber::from(from_height))),
                to_block: Some(BlockTag::Number(BlockNumber::from(to_height))),
                addresses: vec![address],
                topics: Some(vec![vec![
                    keccak256(TokenBurned::abi_signature().as_bytes()).encode_hex(),
                    keccak256(TokenMinted::abi_signature().as_bytes()).encode_hex(),
                    keccak256(TokenTransportRequested::abi_signature().as_bytes()).encode_hex(),
                    keccak256(DirectiveExecuted::abi_signature().as_bytes()).encode_hex(),
                ]]),
            },
        ),
        cycles,
    )
    .await
    .map_err(|err| Error::IcCallError(err.0, err.1))?;

    match rpc_result {
        MultiRpcResult::Consistent(result) => result.map_err(|e| {
            error!("fetch logs rpc error: {:?}", e.clone());
            Error::EvmRpcError(format!("{:?}", e))
        }),
        MultiRpcResult::Inconsistent(_) => {
            Err(super::Error::EvmRpcError("Inconsistent result".to_string()))
        }
    }
}
