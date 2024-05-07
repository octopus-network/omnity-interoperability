use crate::{tx::*, types::*};
use cketh_common::eth_rpc_client::RpcConfig;
use evm_rpc::{candid_types::SendRawTransactionStatus, RpcServices};
use ic_cdk::api::management_canister::ecdsa::{sign_with_ecdsa, SignWithEcdsaArgument};


/*/// peridoically check if there are tickets to be delivered
async fn sign_and_broadcast(ticket: Ticket) -> Result<String, super::Error> {
    let tx = approve(&ticket).await?;
    let hash = broadcast(tx).await?;
    Ok(hash)
}*/

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
    let tx = EIP1559Transaction {
        chain: 0,
        nonce: 0,
        max_priority_fee_per_gas: 0,
        max_fee_per_gas: 0,
        gas: 0,
        to: None,
        value: 0,
        data: vec![],
        access_list: Default::default(),
    };
    let nonce = crate::state::fetch_and_incr_nonce();
    let prehash = vec![];
    let arg = SignWithEcdsaArgument {
        message_hash: prehash.clone(),
        derivation_path: crate::state::key_derivation_path(),
        key_id: crate::state::key_id(),
    };
    // The signatures are encoded as the concatenation of the 32-byte big endian encodings of the two values r and s.
    let (r,) = sign_with_ecdsa(arg)
        .await
        .map_err(|(_, e)| super::Error::ChainKeyError(e))?;
    let chain_id = crate::state::target_chain_id();
    let signature = EthereumSignature::try_from_ecdsa(
        &r.signature,
        &prehash,
        chain_id,
        crate::state::try_public_key()?.as_ref(),
    )?;
    let raw = tx.finalize(signature);
    Ok(vec![])
}


/// trace the broadcasted transaction
async fn trace(tx_hash: String) -> Result<bool, super::Error> {
    Ok(true)
}

// #[derive(Clone, CandidType, Deserialize)]
// pub enum RpcServices {
//     EthMainnet(Option<Vec<EthMainnetService>>),
//     EthSepolia(Option<Vec<EthSepoliaService>>),
//     Custom {
//         #[serde(rename = "chainId")]
//         chain_id: u64,
//         services: Vec<RpcApi>,
//     },
// }

// #[derive(Debug, Clone, PartialEq, Eq, CandidType, Deserialize)]
// pub enum SendRawTransactionStatus {
//     Ok(Option<Hash>),
//     InsufficientFunds,
//     NonceTooLow,
//     NonceTooHigh,
// }
