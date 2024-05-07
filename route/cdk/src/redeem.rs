use crate::*;
use alloy_primitives::hex::ToHexExt;
use alloy_sol_types::{abi::token::WordToken, sol, SolEvent};
use cketh_common::{eth_rpc::LogEntry, eth_rpc_client::RpcConfig, numeric::BlockNumber};
use ethers_core::{
    abi::RawLog,
};
use evm_rpc::{
    candid_types::{self, BlockTag},
    MultiRpcResult, RpcServices,
};

sol! {
    #[derive(Default, Debug)]
    event TokenBurned(
        bytes32 tokenId,
        string receiver,
        uint256 amount,
        string channelId
    );
}

pub(crate) async fn fetch_event() {}

pub(crate) fn check_event_and_generate_ticket() {}

pub async fn generate_ticket(block_height: u64, address: String) -> Result {
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
                from_block: Some(BlockTag::Number(BlockNumber::from(block_height))),
                to_block: Some(BlockTag::Number(BlockNumber::from(block_height))),
                addresses: vec![address],
                // todo check if correct
                topics: Some(vec![vec![TokenBurned::SIGNATURE_HASH.encode_hex()]]),
            },
        ),
    )
    .await
    .map_err(|err| Error::IcCallError(err.0, err.1))?;

    match rpc_result {
        MultiRpcResult::Consistent(result) => {
            for log_entry in result.expect("Evm rpc error") {
                let raw_log = RawLog {
                    topics: log_entry
                        .topics
                        .iter()
                        .map(|topic| topic.0.into())
                        .collect_vec(),
                    data: log_entry.data.0.clone(),
                };
                let token_burned = TokenBurned::decode_raw_log(
                    vec![WordToken(TokenBurned::SIGNATURE_HASH)],
                    &raw_log.data,
                    false,
                )
                .map_err(|e| super::Error::ParseEventError(e.to_string()))?;
                let ticket = Ticket::from_event(&log_entry, token_burned);
                ic_cdk::call(
                    crate::state::hub_addr(),
                    "send_ticket",
                    (ticket,),
                )
                .await
                .map_err(|(_, s)| Error::HubError(s))?;
            }
        }
        MultiRpcResult::Inconsistent(_) => {
            return Result::Err(super::Error::EvmRpcError("Inconsistent result".to_string()))
        }
    }
    Ok(())
}
