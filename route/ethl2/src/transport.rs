use crate::tx::*;
use candid::{CandidType, Deserialize, Principal};
use cketh_common::eth_rpc_client::providers::{
    EthMainnetService, EthSepoliaService, RpcApi, RpcService,
};
use ic_cdk::api::management_canister::ecdsa::{sign_with_ecdsa, SignWithEcdsaArgument};
use omnity_types::*;
use std::collections::{BTreeMap, HashMap};

const PULL_BATCH_SIZE: u64 = 100;

/// pull tickets from hub, the return value indicates if there are tickets pulled
pub(crate) async fn transport() -> bool {
    if !super::is_active() {
        return;
    }
    let hub = super::hub_addr();
    let target_chain = super::target_chain();
    let max_seq = super::max_ticket_id();
    match pull(hub, target_chain, max_seq + 1).await {
        Ok(r) => {
            for (seq, ticket) in r.into_iter() {
                match sign_and_broadcast(ticket.clone()) {
                    Ok(hash) => {
                        // TODO save the hash
                        // TODO save the ticket
                    }
                    Err(e) => {
                        ic_cdk::println!("Error handling ticket: {:?}", e);
                        break;
                    }
                }
            }
            !r.is_empty()
        }
        Err(e) => {
            ic_cdk::println!("Error pulling tickets: {:?}", e);
            false
        }
    }
}

async fn pull(
    hub: Principal,
    target: ChainId,
    seq: u64,
) -> Result<BTreeMap<u64, Ticket>, super::Error> {
    let (r,): (BTreeMap<u64, Ticket>,) =
        ic_cdk::call(hub, "query_tickets", (target, seq, seq + PULL_BATCH_SIZE))
            .await
            .map_err(|(_, e)| super::Error::HubError(e))?;
    Ok(r)
}

/// peridoically check if there are tickets to be delivered
async fn sign_and_broadcast(ticket: Ticket) -> Result<String, super::Error> {
    let tx = approve(&ticket).await?;
    let hash = broadcast(tx).await?;
    Ok(hash)
}

/// transform ticket to a signed transaction:
///
/// function transportToken(
///     bytes32 dstChainId,
///     bytes32 tokenId,
///     string memory receiver,
///     uint256 amount,
///     string memory channelId,
///     string memory memo
/// )
async fn approve(ticket: &Ticket) -> Result<Vec<u8>, super::Error> {
    // TODO serialize ticket then digest it, **NOTICE** do we need to transform the decimals?
    // let tx = EIP1559Transaction {};
    let nonce = super::fetch_and_incr_nonce();
    let prehash = vec![];
    let arg = SignWithEcdsaArgument {
        message_hash: prehash.clone(),
        derivation_path: super::key_derivation_path(),
        key_id: super::key_id(),
    };
    // The signatures are encoded as the concatenation of the 32-byte big endian encodings of the two values r and s.
    let (r,) = sign_with_ecdsa(arg)
        .await
        .map_err(|(_, e)| super::Error::ChainKeyError(e))?;
    let chain_id = super::target_chain_id();
    let signature = EthereumSignature::try_from_ecdsa(
        &r.signature,
        &prehash,
        chain_id,
        super::try_public_key()?.as_ref(),
    )?;
    // TODO let raw = tx.finalize(signature);
    Ok(vec![])
}

async fn broadcast(tx: Vec<u8>) -> Result<String, super::Error> {
    let raw = hex::encode(tx);
    // see https://github.com/internet-computer-protocol/evm-rpc-canister/blob/main/src/main.rs#L87
    let (r,): (SendRawTransactionStatus,) = ic_cdk::call(
        super::rpc_addr().ok_or(super::Error::RouteNotInitialized)?,
        "eth_sendRawTransaction",
        (
            RpcServices::Custom {
                chain_id: super::target_chain_id(),
                services: super::rpc_providers(),
            },
            None,
            raw,
        ),
    )
    .await
    .map_err(|(_, e)| super::Error::EthRpcError(e))?;
    match r {
        SendRawTransactionStatus::Ok(hash) => hash.ok_or(super::Error::EthRpcError(
            "A transaction hash is expected".to_string(),
        )),
        _ => Err(super::Error::EthRpcError(format!("{:?}", r))),
    }
}

/// trace the broadcasted transaction
async fn trace(tx_hash: String) -> Result<bool, super::Error> {
    Ok(true)
}

// copy from evm-rpc-canister because we can't compile it with the current version of the candid crate
#[derive(Clone, CandidType, Deserialize)]
pub enum RpcServices {
    EthMainnet(Option<Vec<EthMainnetService>>),
    EthSepolia(Option<Vec<EthSepoliaService>>),
    Custom {
        #[serde(rename = "chainId")]
        chain_id: u64,
        services: Vec<RpcApi>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, CandidType, Deserialize)]
pub enum SendRawTransactionStatus {
    Ok(Option<Hash>),
    InsufficientFunds,
    NonceTooLow,
    NonceTooHigh,
}
