use std::str::FromStr;

use anyhow::anyhow;
use candid::{CandidType, Nat};
use cketh_common::eth_rpc::{Hash, RpcError};
use cketh_common::eth_rpc_client::providers::RpcService;
use cketh_common::eth_rpc_client::RpcConfig;
use ethereum_types::Address;
use ethers_core::abi::ethereum_types;
use ethers_core::types::{Eip1559TransactionRequest, U256};
use ethers_core::utils::keccak256;
use evm_rpc::candid_types::{BlockTag, GetTransactionCountArgs, SendRawTransactionStatus};
use evm_rpc::{MultiRpcResult, RpcServices};
use ic_cdk::api::management_canister::ecdsa::{sign_with_ecdsa, SignWithEcdsaArgument};
use log::{error, info};
use num_traits::ToPrimitive;
use serde_derive::{Deserialize, Serialize};

use crate::const_args::{
    BROADCAST_TX_CYCLES, EIP1559_TX_ID, EVM_ADDR_BYTES_LEN, EVM_FINALIZED_CONFIRM_HEIGHT,
    GET_ACCOUNT_NONCE_CYCLES,
};
use crate::eth_common::EvmAddressError::LengthError;
use crate::{state, Error};

#[derive(Deserialize, CandidType, Serialize, Default, Clone, Eq, PartialEq)]
pub struct EvmAddress(pub(crate) [u8; EVM_ADDR_BYTES_LEN]);

#[derive(Error, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum EvmAddressError {
    #[error("Bytes isn't 20 bytes.")]
    LengthError,
    #[error("String is not a hex string.")]
    FormatError,
}

impl EvmAddress {
    pub fn to_hex(&self) -> String {
        format!("0x{}", hex::encode(self.0))
    }
}

impl From<EvmAddress> for Address {
    fn from(value: EvmAddress) -> Self {
        Address::from(value.0)
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
        v: y_parity(&txhash, &r.signature, crate::state::public_key().as_ref()),
        r: U256::from_big_endian(&r.signature[0..32]),
        s: U256::from_big_endian(&r.signature[32..64]),
    };
    let mut signed_tx_bytes = tx.rlp_signed(&signature).to_vec();
    signed_tx_bytes.insert(0, EIP1559_TX_ID);
    Ok(signed_tx_bytes)
}

pub async fn broadcast(tx: Vec<u8>) -> Result<String, super::Error> {
    let raw = format!("0x{}", hex::encode(tx));
    info!("[evm route] preparing to send tx: {}", raw);
    let (r,): (MultiRpcResult<SendRawTransactionStatus>,) =
        ic_cdk::api::call::call_with_payment128(
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
            BROADCAST_TX_CYCLES,
        )
        .await
        .map_err(|(_, e)| super::Error::EvmRpcError(e))?;
    info!("broadcast result:{:?}", r.clone());
    match r {
        MultiRpcResult::Consistent(res) => match res {
            Ok(s) => match s {
                SendRawTransactionStatus::Ok(hash) => {
                    Ok(hex::encode(hash.unwrap_or(Hash([0u8; 32])).0))
                }
                SendRawTransactionStatus::InsufficientFunds => {
                    Err(Error::Custom(anyhow!("InsufficientFunds")))
                }
                SendRawTransactionStatus::NonceTooLow => Err(Error::Custom(anyhow!("NonceTooLow"))),
                SendRawTransactionStatus::NonceTooHigh => {
                    Err(Error::Custom(anyhow!("NonceToohigh")))
                }
            },
            Err(r) => {
                if let RpcError::JsonRpcError(ref jerr) = r {
                    if jerr.code == -32603 && jerr.message == "already known" {
                        return Ok(hex::encode([1u8; 32]));
                    }
                }
                Err(Error::EvmRpcError(format!("{:?}", r)))
            }
        },
        MultiRpcResult::Inconsistent(_r) => {
            Err(super::Error::EvmRpcError("Inconsistent result".to_string()))
        }
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

pub async fn get_account_nonce(addr: String) -> Result<u64, super::Error> {
    let (r,): (MultiRpcResult<Nat>,) = ic_cdk::api::call::call_with_payment128(
        crate::state::rpc_addr(),
        "eth_getTransactionCount",
        (
            RpcServices::Custom {
                chain_id: crate::state::evm_chain_id(),
                services: crate::state::rpc_providers(),
            },
            None::<RpcConfig>,
            GetTransactionCountArgs {
                address: addr,
                block: BlockTag::Pending,
            },
        ),
        GET_ACCOUNT_NONCE_CYCLES,
    )
    .await
    .map_err(|(_, e)| super::Error::EvmRpcError(e))?;
    match r {
        MultiRpcResult::Consistent(r) => match r {
            Ok(c) => Ok(c.0.to_u64().unwrap()),
            Err(r) => Err(Error::EvmRpcError(format!("{:?}", r))),
        },
        MultiRpcResult::Inconsistent(_) => {
            Err(super::Error::EvmRpcError("Inconsistent result".to_string()))
        }
    }
}

pub async fn get_gasprice() -> anyhow::Result<U256> {
    // Define request parameters
    let params = (
        RpcService::Custom(state::rpc_providers().clone().pop().unwrap()), // Ethereum mainnet
        r#"{"method":"eth_gasPrice","params":[],"id":1,"jsonrpc":"2.0"}"#.to_string(),
        1000u64,
    );
    // Get cycles cost
    let (cycles_result,): (std::result::Result<u128, RpcError>,) =
        ic_cdk::api::call::call(state::rpc_addr(), "requestCost", params.clone())
            .await
            .unwrap();
    let cycles = cycles_result.map_err(|e| {
        error!("[evm route] evm request error: {:?}", e);
        anyhow!(format!("error in `request_cost`: {:?}", e))
    })?;
    // Call with expected number of cycles
    let (result,): (std::result::Result<String, RpcError>,) =
        ic_cdk::api::call::call_with_payment128(state::rpc_addr(), "request", params, cycles)
            .await
            .map_err(|err| Error::IcCallError(err.0, err.1))?;
    #[derive(Serialize, Deserialize, Debug)]
    struct BlockNumberResult {
        pub id: u32,
        pub jsonrpc: String,
        pub result: String,
    }
    let r = result.map_err(|e| {
        error!("[evm route]query gas price error: {:?}", &e);
        Error::Custom(anyhow!(format!(
            "[evm route]query gas price error: {:?}",
            &e
        )))
    })?;
    let r: BlockNumberResult = serde_json::from_str(r.as_str())?;
    let r = r.result.strip_prefix("0x").unwrap_or(r.result.as_str());
    let r = u64::from_str_radix(r, 16)?;
    Ok(U256::from(r * 11 / 10))
}

pub async fn get_evm_finalized_height() -> anyhow::Result<u64> {
    // Define request parameters
    let params = (
        RpcService::Custom(state::rpc_providers().clone().pop().unwrap()), // Ethereum mainnet
        r#"{"method":"eth_blockNumber","params":[],"id":1,"jsonrpc":"2.0"}"#.to_string(),
        1000u64,
    );
    // Get cycles cost
    let (cycles_result,): (std::result::Result<u128, RpcError>,) =
        ic_cdk::api::call::call(state::rpc_addr(), "requestCost", params.clone())
            .await
            .map_err(|err| Error::IcCallError(err.0, err.1))?;
    let cycles = cycles_result.map_err(|e| {
        error!("[evm route] evm request error: {:?}", e);
        anyhow!(format!("error in `request_cost`: {:?}", e))
    })?;
    // Call with expected number of cycles
    let (result,): (std::result::Result<String, RpcError>,) =
        ic_cdk::api::call::call_with_payment128(state::rpc_addr(), "request", params, cycles)
            .await
            .map_err(|err| Error::IcCallError(err.0, err.1))?;
    #[derive(Serialize, Deserialize, Debug)]
    struct BlockNumberResult {
        pub id: u32,
        pub jsonrpc: String,
        pub result: String,
    }
    let r = result.map_err(|e| {
        error!("[evm route]query block number error: {:?}", &e);
        Error::Custom(anyhow!(format!(
            "[evm route]query block number error: {:?}",
            &e
        )))
    })?;
    let r: BlockNumberResult = serde_json::from_str(r.as_str())?;
    let r = r.result.strip_prefix("0x").unwrap_or(r.result.as_str());
    let r = u64::from_str_radix(r, 16)?;
    Ok(r - EVM_FINALIZED_CONFIRM_HEIGHT)
}
