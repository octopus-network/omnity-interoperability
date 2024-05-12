use crate::route_to_cdk::{send_directives_to_cdk, send_tickets_to_cdk};
use crate::state::{mutate_state, read_state};
use crate::types::Ticket;
use crate::*;
use alloy_primitives::hex::ToHexExt;
use alloy_primitives::{LogData, B256};
use alloy_sol_types::{abi::token::WordToken, sol, SolEvent};
use anyhow::anyhow;
use cketh_common::{eth_rpc::LogEntry, eth_rpc_client::RpcConfig, numeric::BlockNumber};
use evm_rpc::{
    candid_types::{self, BlockTag},
    MultiRpcResult, RpcServices,
};
use log::{error, info};

const MAX_SCAN_BLOCKS: u64 = 20;

sol! {

    #[derive(Default, Debug)]
    event TokenMinted(
        string tokenId,
        address receiver,
        uint256 amount,
        uint256 ticketId,
        string memo
    );

    #[derive(Default, Debug)]
    event TokenTransportRequested(
        string dstChainId,
        string tokenId,
        string receiver,
        uint256 amount,
        string channelId,
        string memo
    );

    #[derive(Default, Debug)]
    event TokenBurned(
        string tokenId,
        string receiver,
        uint256 amount,
        string channelId
    );

    #[derive(Default, Debug)]
    event DirectiveExecuted(
        uint256 seq
    );
}

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
    let contract_addr = read_state(|s| s.omnity_port_contract.encode_hex());
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
        let raw_log = LogData::new(
            l.topics.iter().map(|topic| topic.0.into()).collect_vec(),
            alloy_primitives::Bytes::from(l.data.0.clone()),
        )
        .expect("topics lenght > 4");
        if topic1 == TokenBurned::SIGNATURE_HASH.0 {
            let token_burned = TokenBurned::decode_log_data(&raw_log, false)
                .map_err(|e| super::Error::ParseEventError(e.to_string()))?;
            handle_token_burn(&l, token_burned).await?;
        } else if topic1 == TokenMinted::SIGNATURE_HASH.0 {
            let token_mint = TokenMinted::decode_log_data(&raw_log, false)
                .map_err(|e| super::Error::ParseEventError(e.to_string()))?;
            handle_token_mint(token_mint);
        } else if topic1 == TokenTransportRequested::SIGNATURE_HASH.0 {
            let token_transport = TokenTransportRequested::decode_log_data(&raw_log, false)
                .map_err(|e| super::Error::ParseEventError(e.to_string()))?;
            handle_token_transport(&l, token_transport).await?;
        }
        mutate_state(|s| s.handled_cdk_event.insert(log_key));
    }
    mutate_state(|s| s.scan_start_height = to);
    Ok(())
}

pub async fn handle_token_burn(log_entry: &LogEntry, event: TokenBurned) -> anyhow::Result<()> {
    let ticket = Ticket::from_burn_event(&log_entry, event);
    ic_cdk::call(crate::state::hub_addr(), "send_ticket", (ticket,))
        .await
        .map_err(|(_, s)| Error::HubError(s))?;

    Ok(())
}

pub fn handle_token_mint(event: TokenMinted) {
    let tid = event.ticketId.to_string();
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

pub async fn get_cdk_finalized_height() -> anyhow::Result<u64> {
    let json_rpc_payload = r#"{"method":"eth_blockNumber","params":[],"id":1,"jsonrpc":"2.0"}"#;
    let (result,): (u64,) = ic_cdk::api::call::call(
        crate::state::rpc_addr(),
        "request",
        (
            RpcServices::Custom {
                chain_id: crate::state::target_chain_id(),
                services: crate::state::rpc_providers(),
            },
            json_rpc_payload,
            100000,
        ),
    )
    .await
    .map_err(|err| Error::IcCallError(err.0, err.1))?;
    info!("received get cdk finalized height: {}", result);
    Ok(result - 12)
}

pub async fn fetch_logs(
    from_height: u64,
    to_height: u64,
    address: String,
) -> std::result::Result<Vec<LogEntry>, Error> {
    let (rpc_result,): (MultiRpcResult<Vec<LogEntry>>,) = ic_cdk::api::call::call(
        crate::state::rpc_addr(),
        "eth_getLogs",
        (
            RpcServices::Custom {
                chain_id: crate::state::target_chain_id(),
                services: crate::state::rpc_providers(),
            },
            None::<RpcConfig>,
            candid_types::GetLogsArgs {
                from_block: Some(BlockTag::Number(BlockNumber::from(from_height))),
                to_block: Some(BlockTag::Number(BlockNumber::from(to_height))),
                addresses: vec![address],
                // todo check if correct
                topics: Some(vec![
                    vec![TokenBurned::SIGNATURE_HASH.encode_hex()],
                    vec![TokenMinted::SIGNATURE_HASH.encode_hex()],
                    vec![TokenTransportRequested::SIGNATURE_HASH.encode_hex()],
                ]),
            },
        ),
    )
    .await
    .map_err(|err| Error::IcCallError(err.0, err.1))?;

    match rpc_result {
        MultiRpcResult::Consistent(result) => result.map_err(|e| {
            error!("fetch logs rpc error: {:?}", e.clone());
            Error::EvmRpcError(format!("{:?}", e))
        }),
        MultiRpcResult::Inconsistent(_) => {
            return Result::Err(super::Error::EvmRpcError("Inconsistent result".to_string()))
        }
    }
}
