use crate::tx::*;
use candid::{CandidType, Principal};
use ic_cdk::api::management_canister::ecdsa::{sign_with_ecdsa, SignWithEcdsaArgument};
use omnity_types::*;
use std::collections::{BTreeMap, HashMap};

const PULL_BATCH_SIZE: u64 = 100;

pub(crate) async fn pull(
    hub: Principal,
    target: ChainId,
    seq: u64,
) -> Result<BTreeMap<u64, Ticket>, super::Error> {
    let (r,): (BTreeMap<u64, Ticket>,) =
        ic_cdk::call(hub, "query_tickets", (target, seq, seq + PULL_BATCH_SIZE))
            .await
            .map_err(|(_, e)| super::Error::HubOffline(e))?;
    Ok(r)
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
pub(crate) async fn approve(ticket: Ticket) -> Result<Vec<u8>, super::Error> {
    // TODO serialize ticket then digest it
    // let tx = EIP1559Transaction::default();
    let prehash = vec![];
    let arg = SignWithEcdsaArgument {
        message_hash: prehash.clone(),
        derivation_path: super::key_derivation_path(),
        key_id: super::try_key_id()?,
    };
    let (r,) = sign_with_ecdsa(arg)
        .await
        .map_err(|(_, e)| super::Error::ChainKeyError(e))?;
    let chain_id = super::target_chain_id();
    // The signatures are encoded as the concatenation of the 32-byte big endian encodings of the two values r and s.
    let signature = EthereumSignature::try_from_ecdsa(
        &r.signature,
        &prehash,
        chain_id,
        super::try_public_key()?.as_ref(),
    )?;
    // TODO let raw = tx.finalize(signature);
    Ok(vec![])
}

pub(crate) async fn broadcast(tx: Vec<u8>) -> Result<(), super::Error> {
    Ok(())
}
