use crate::traits::StateProvider;
use crate::tx_types::EvmTxRequest;
use ethers_core::types::{Eip1559TransactionRequest, TransactionRequest, U256};
use ethers_core::utils::keccak256;
use ic_cdk::api::management_canister::ecdsa::{sign_with_ecdsa, SignWithEcdsaArgument};

pub async fn sign_transaction<P: StateProvider>(
    evm_tx_request: EvmTxRequest,
) -> anyhow::Result<Vec<u8>> {
    match evm_tx_request {
        EvmTxRequest::Legacy(tx) => sign_transaction_legacy::<P>(tx).await,
        EvmTxRequest::Eip1559(tx) => sign_transaction_eip1559::<P>(tx).await,
    }
}

pub async fn sign_transaction_eip1559<P: StateProvider>(
    tx: Eip1559TransactionRequest,
) -> anyhow::Result<Vec<u8>> {
    let signature_base = P::get_signature_base();
    use crate::const_args::EIP1559_TX_ID;
    use ethers_core::types::Signature;
    let mut unsigned_tx_bytes = tx.rlp().to_vec();
    unsigned_tx_bytes.insert(0, EIP1559_TX_ID);
    let txhash = keccak256(&unsigned_tx_bytes);
    let arg = SignWithEcdsaArgument {
        message_hash: txhash.clone().to_vec(),
        derivation_path: signature_base.key_derivation_path,
        key_id: signature_base.key_id,
    };
    // The signatures are encoded as the concatenation of the 32-byte big endian encodings of the two values r and s.
    let (r,) = sign_with_ecdsa(arg)
        .await
        .map_err(|(_, e)| crate::error::Error::ChainKeyError(e))?;
    let signature = Signature {
        v: y_parity(&txhash, &r.signature, signature_base.public_key.as_ref()),
        r: U256::from_big_endian(&r.signature[0..32]),
        s: U256::from_big_endian(&r.signature[32..64]),
    };
    let mut signed_tx_bytes = tx.rlp_signed(&signature).to_vec();
    signed_tx_bytes.insert(0, EIP1559_TX_ID);
    Ok(signed_tx_bytes)
}

pub async fn sign_transaction_legacy<P: StateProvider>(
    tx: TransactionRequest,
) -> anyhow::Result<Vec<u8>> {
    let signature_base = P::get_signature_base();
    use ethers_core::types::Signature;
    let unsigned_tx_bytes = tx.rlp().to_vec();
    let txhash = keccak256(&unsigned_tx_bytes);
    let arg = SignWithEcdsaArgument {
        message_hash: txhash.clone().to_vec(),
        derivation_path: signature_base.key_derivation_path,
        key_id: signature_base.key_id,
    };
    // The signatures are encoded as the concatenation of the 32-byte big endian encodings of the two values r and s.
    let (r,) = sign_with_ecdsa(arg)
        .await
        .map_err(|(_, e)| crate::error::Error::ChainKeyError(e))?;
    let signature = Signature {
        v: y_parity(&txhash, &r.signature, signature_base.public_key.as_ref())
            + tx.chain_id.unwrap().as_u64() * 2
            + 35,
        r: U256::from_big_endian(&r.signature[0..32]),
        s: U256::from_big_endian(&r.signature[32..64]),
    };
    let signed_tx_bytes = tx.rlp_signed(&signature).to_vec();
    Ok(signed_tx_bytes)
}

pub fn y_parity(prehash: &[u8], sig: &[u8], pubkey: &[u8]) -> u64 {
    use k256::ecdsa::{RecoveryId, Signature, VerifyingKey};
    let orig_key = VerifyingKey::from_sec1_bytes(pubkey).expect("failed to parse the pubkey");
    let signature = Signature::try_from(sig).unwrap();
    for parity in [0u8, 1] {
        let recid = RecoveryId::try_from(parity).unwrap();
        let recovered_key = VerifyingKey::recover_from_prehash(prehash, &signature, recid)
            .expect("failed to recover key");
        if recovered_key == orig_key {
            return parity as u64;
        }
    }
    panic!(
        "failed to recover the parity bit from a signature; sig: {}, pubkey: {}",
        hex::encode(sig),
        hex::encode(pubkey)
    )
}
