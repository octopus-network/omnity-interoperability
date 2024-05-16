use std::str::FromStr;

use candid::CandidType;
use cketh_common::eth_rpc_client::RpcConfig;
use ethereum_types::Address;
use ethers_core::abi::{ethereum_types, AbiEncode};
use ethers_core::types::{Eip1559TransactionRequest, U256};
use ethers_core::utils::keccak256;
use evm_rpc::candid_types::SendRawTransactionStatus;
use evm_rpc::RpcServices;
use ic_cdk::api::management_canister::ecdsa::{sign_with_ecdsa, SignWithEcdsaArgument};
use serde_derive::{Deserialize, Serialize};

use crate::Error;
use crate::eth_common::EvmAddressError::LengthError;

const EVM_ADDR_BYTES_LEN: usize = 20;

#[derive(Deserialize, CandidType, Serialize, Default, Clone, Eq, PartialEq)]
pub struct EvmAddress(pub(crate) [u8; EVM_ADDR_BYTES_LEN]);

#[derive(Error, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum EvmAddressError {
    #[error("Bytes is longer than 29 bytes.")]
    LengthError,
    #[error("Bytes is longer than 29 bytes.")]
    FormatError,
}

impl Into<Address> for EvmAddress {
    fn into(self) -> Address {
        Address::from(self.0)
    }
}
impl AsRef<[u8]> for EvmAddress {
    fn as_ref(&self) -> &[u8] {
        self.0.as_slice()
    }
}

impl FromStr for EvmAddress {
    type Err = EvmAddressError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let t = if text.starts_with("0x") {
            text.strip_prefix("0x").unwrap()
        } else {
            text
        };
        let r = hex::decode(t).map_err(|_e| EvmAddressError::FormatError)?;
        EvmAddress::try_from(r)

    }
}

impl TryFrom<Vec<u8>> for EvmAddress {
    type Error = EvmAddressError;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        if value.len() != EVM_ADDR_BYTES_LEN {
            return Err(LengthError);
        }
        let mut c = [0u8; EVM_ADDR_BYTES_LEN];
        c.copy_from_slice(value.as_slice());
        Ok(EvmAddress(c))
    }
}


pub async fn sign_transaction(tx: Eip1559TransactionRequest) -> anyhow::Result<Vec<u8>> {
    use ethers_core::types::Signature;
    const EIP1559_TX_ID: u8 = 2;
    let mut unsigned_tx_bytes = tx.rlp().to_vec();
    unsigned_tx_bytes.insert(0, EIP1559_TX_ID);
    let txhash = keccak256(&unsigned_tx_bytes);
    let arg = SignWithEcdsaArgument {
        message_hash: txhash.clone().to_vec(),
        derivation_path: crate::state::key_derivation_path(),
        key_id: crate::state::key_id(),
    };
    // The signatures are encoded as the concatenation of the 32-byte big endian encodings of the two values r and s.
    let (r,) = sign_with_ecdsa(arg)
        .await
        .map_err(|(_, e)| super::Error::ChainKeyError(e))?;
    let signature = Signature {
        v: y_parity(
            &txhash,
            &r.signature,
            crate::state::try_public_key()?.as_ref(),
        ),
        r: U256::from_big_endian(&r.signature[0..32]),
        s: U256::from_big_endian(&r.signature[32..64]),
    };
    let mut signed_tx_bytes = tx.rlp_signed(&signature).to_vec();
    signed_tx_bytes.insert(0, EIP1559_TX_ID);
    Ok(signed_tx_bytes)
}

pub async fn broadcast(tx: Vec<u8>) -> Result<String, super::Error> {
    let raw = format!("0x{}", hex::encode(tx));
    let (r,): (SendRawTransactionStatus,) = ic_cdk::call(
        crate::state::rpc_addr(),
        "eth_sendRawTransaction",
        (
            RpcServices::Custom {
                chain_id: crate::state::evm_chain_id(),
                services: crate::state::rpc_providers(),
            },
            None::<RpcConfig>,
            raw,
        ),
    )
    .await
    .map_err(|(_, e)| super::Error::EvmRpcError(e))?;
    match r {
        SendRawTransactionStatus::Ok(hash) => hash.map(|h| h.to_string()).ok_or(
            super::Error::EvmRpcError("A transaction hash is expected".to_string()),
        ),
        _ => Err(super::Error::EvmRpcError(format!("{:?}", r))),
    }
}

fn y_parity(prehash: &[u8], sig: &[u8], pubkey: &[u8]) -> u64 {
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
