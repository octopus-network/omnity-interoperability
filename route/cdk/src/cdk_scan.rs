use crate::contract_types::{
    AbiSignature, DecodeLog, DirectiveExecuted, TokenBurned, TokenMinted, TokenTransportRequested,
};
use crate::state::{mutate_state, read_state};
use crate::types::Ticket;
use crate::*;
use anyhow::anyhow;
use cketh_common::eth_rpc::RpcError;
use cketh_common::eth_rpc_client::providers::RpcService;
use cketh_common::{eth_rpc::LogEntry, eth_rpc_client::RpcConfig, numeric::BlockNumber};
use ethers_core::abi::{AbiEncode, RawLog};
use ethers_core::types::U256;
use ethers_core::utils::keccak256;
use evm_rpc::{
    candid_types::{self, BlockTag},
    MultiRpcResult, RpcServices,
};
use itertools::Itertools;
use log::{error};
use serde_derive::{Deserialize, Serialize};
use crate::eth_common::get_cdk_finalized_height;

const MAX_SCAN_BLOCKS: u64 = 20;

pub fn scan_cdk_task() {
    ic_cdk::spawn(async {
        let _guard = match crate::guard::TimerLogicGuard::new() {
            Some(guard) => guard,
            None => return,
        };
        let _ = handle_port_events().await;
    });
}

pub async fn handle_port_events() -> anyhow::Result<()> {
    let (from, to) = determine_from_to().await?;
    let contract_addr = read_state(|s| s.omnity_port_contract.0.encode_hex());
    let logs = fetch_logs(from, to, contract_addr).await?;
    for l in logs {
        if l.removed {
            return Err(anyhow!("log is removed"));
        }
        let block = l.block_number.ok_or(anyhow!("block is pending"))?;
        let log_index = l.log_index.ok_or(anyhow!("log is pending"))?;
        let log_key = std::format!("{}-{}", block, log_index);
        if read_state(|s| s.handled_cdk_event.contains(&log_key)) {
            continue;
        }
        let topic1 = l.topics.first().ok_or(anyhow!("topic is none"))?.0.clone();
        let raw_log: RawLog = RawLog {
            topics: l.topics.iter().map(|topic| topic.0.into()).collect_vec(),
            data: l.data.0.clone(),
        };
        if topic1 == keccak256(TokenBurned::abi_signature().as_bytes()) {
            let token_burned = TokenBurned::decode_log(&raw_log)
                .map_err(|e| super::Error::ParseEventError(e.to_string()))?;
            handle_token_burn(&l, token_burned).await?;
        } else if topic1 == keccak256(TokenMinted::abi_signature().as_bytes()) {
            let token_mint = TokenMinted::decode_log(&raw_log)
                .map_err(|e| super::Error::ParseEventError(e.to_string()))?;
            handle_token_mint(token_mint);
        } else if topic1 == keccak256(TokenTransportRequested::abi_signature().as_bytes()) {
            let token_transport = TokenTransportRequested::decode_log(&raw_log)
                .map_err(|e| super::Error::ParseEventError(e.to_string()))?;
            handle_token_transport(&l, token_transport).await?;
        }
        mutate_state(|s| s.handled_cdk_event.insert(log_key));
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

pub fn handle_token_mint(event: TokenMinted) {
    let tid = event.ticket_id.to_string();
    mutate_state(|s| s.pending_tickets_map.remove(&tid));
}

pub async fn handle_token_transport(
    log_entry: &LogEntry,
    event: TokenTransportRequested,
) -> anyhow::Result<()> {
    let ticket = Ticket::from_transport_event(&log_entry, event);
    ic_cdk::call(crate::state::hub_addr(), "send_ticket", (ticket,))
        .await
        .map_err(|(_, s)| Error::HubError(s))?;
    Ok(())
}

async fn determine_from_to() -> anyhow::Result<(u64, u64)> {
    let from_height = read_state(|s| s.scan_start_height);
    let to_height = get_cdk_finalized_height().await.map_err(|e| {
        error!("query cdk block height error: {:?}", e.to_string());
        e
    })?;
    Ok((from_height, to_height.min(from_height + MAX_SCAN_BLOCKS)))
}

pub async fn fetch_logs(
    from_height: u64,
    to_height: u64,
    address: String,
) -> std::result::Result<Vec<LogEntry>, Error> {
    let cycles = 100_000_000_000; //TODO
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
                // todo check if correct
                topics: Some(vec![
                    vec![keccak256(TokenBurned::abi_signature().as_bytes()).encode_hex()],
                    vec![keccak256(TokenMinted::abi_signature().as_bytes()).encode_hex()],
                    vec![
                        keccak256(TokenTransportRequested::abi_signature().as_bytes()).encode_hex(),
                    ],
                    vec![keccak256(DirectiveExecuted::abi_signature().as_bytes()).encode_hex()],
                ]),
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