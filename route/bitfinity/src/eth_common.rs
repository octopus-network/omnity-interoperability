use std::ops::{Div, Mul};
use std::str::FromStr;

use anyhow::anyhow;
use candid::{CandidType};
use did::{BlockNumber, H256};
use did::error::{EvmError, TransactionPoolError};

use ethereum_types::Address;
use ethers_core::abi::ethereum_types;
use ethers_core::types::{Eip1559TransactionRequest, U256, U64};
use ethers_core::utils::keccak256;
use evm_canister_client::{EvmCanisterClient, IcCanisterClient};
use ic_canister_log::log;
use ic_cdk::api::management_canister::ecdsa::{sign_with_ecdsa, SignWithEcdsaArgument};
use serde_derive::{Deserialize, Serialize};

use crate::{BitfinityRouteError, EvmAddressError};
use crate::BitfinityRouteError::{EvmRpcError, Temporary};
use crate::const_args::{EVM_ADDR_BYTES_LEN};
use crate::eth_common::EvmAddressError::LengthError;
use crate::logs::P0;
use crate::state::{minter_addr, read_state};

pub fn hex_to_u64(hex_str: &str) -> u64 {
    u64::from_str_radix(hex_str.strip_prefix("0x").unwrap(), 16).unwrap()
}

#[derive(Deserialize, CandidType, Serialize, Default, Clone, Eq, PartialEq)]
pub struct EvmAddress(pub(crate) [u8; EVM_ADDR_BYTES_LEN]);

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

pub async fn sign_transaction(tx: Eip1559TransactionRequest) -> anyhow::Result<did::Transaction> {
    sign_transaction_eip1559(tx).await
}

pub async fn sign_transaction_eip1559(tx: Eip1559TransactionRequest) -> anyhow::Result<did::Transaction> {
    use crate::const_args::EIP1559_TX_ID;
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
        .map_err(|(_, e)| super::BitfinityRouteError::ChainKeyError(e))?;
    let signature = Signature {
        v: y_parity(&txhash, &r.signature, crate::state::public_key().as_ref()),
        r: U256::from_big_endian(&r.signature[0..32]),
        s: U256::from_big_endian(&r.signature[32..64]),
    };
    let port_addr = read_state(|s|s.omnity_port_contract.clone());
    let mut  transaction = ethers_core::types::Transaction {
        from: did::H160::from_hex_str(&minter_addr()).unwrap().0,
        to: Some(did::H160::from(port_addr.0).0),
        nonce: did::U256::from(tx.nonce.unwrap_or_default()).0,
        value: 0.into(),
        gas: tx.gas.unwrap_or_default(),
        input: tx.data.unwrap(),
        v: U64::from(signature.v),
        r: U256::from(signature.r),
        s: U256::from(signature.s),
        hash: H256::from(txhash).0,
        chain_id: Some(U256::from(tx.chain_id.unwrap().0[0])),
        max_priority_fee_per_gas: tx.max_priority_fee_per_gas,
        max_fee_per_gas: tx.max_fee_per_gas,
        transaction_type: Some(U64::from(EIP1559_TX_ID as u64)),
        ..Default::default()
    };
    transaction.hash = transaction.hash();
    Ok(transaction.into())
}

pub async fn broadcast(tx: did::Transaction) -> Result<String, super::BitfinityRouteError> {
    let client = bitfinity_evm_canister_client();
    let r  =  client.send_raw_transaction(tx.clone()).await.map_err(|e|{
       log!(P0, "[bitfinity route]broadcats canister client error: {}", e.to_string());
        BitfinityRouteError::EvmRpcError(e.to_string())
    })?;
    match r {
        Ok(r) => {
            Ok(r.to_hex_str())
        }
        Err(e) => {
            match e.clone() {
                EvmError::Internal(_) => {}
                EvmError::InsufficientBalance { .. } => {}
                EvmError::NotProcessableTransactionError(_) => {}
                EvmError::FatalEvmExecutorError(_) => {}
                EvmError::InvalidGasPrice(_) => {}
                EvmError::NotAuthorized => {}
                EvmError::ReservationFailed(_) => {}
                EvmError::StableStorageError(_) => {}
                EvmError::TransactionPool(e) => {
                    match e {
                        TransactionPoolError::TooManyTransactions => {}
                        TransactionPoolError::TxReplacementUnderpriced => {}
                        TransactionPoolError::TransactionAlreadyExists => {
                            return  Ok(tx.hash.to_hex_str());
                        }
                        TransactionPoolError::InvalidNonce { .. } => {
                            return Err(Temporary);
                        }
                    }
                }
                EvmError::NoHistoryDataForBlock(_) => {}
                EvmError::BlockDoesNotExist(_) => {}
                EvmError::TransactionSignature(_) => {}
                EvmError::GasTooLow { .. } => {}
                EvmError::AnonymousPrincipal => {}
                EvmError::BadRequest(_) => {}
                EvmError::TransactionReverted(_) => {}
                EvmError::Precompile(_) => {}
            }
            Err(BitfinityRouteError::Custom(anyhow::anyhow!("broadcast error: {}", e.to_string())))
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

pub async fn get_account_nonce(addr: String) -> Result<did::U256, super::BitfinityRouteError> {
    let client = bitfinity_evm_canister_client();
    let r = client.eth_get_transaction_count(
        did::H160::from_hex_str(addr.as_str()).unwrap(), BlockNumber::Pending).await;
    let r = r.map_err(|e|{
        log!(P0, "[bitfinity route]query chainkey account nonce client error: {:?}", &e);
            EvmRpcError(e.to_string())
        })?
       .map_err(|e|{
           log!(P0, "[bitfinity route]query chainkey account nonce evm error: {:?}", &e);
           EvmRpcError(e.to_string())
       })?;
    Ok(r)
}

pub async fn get_gasprice() -> anyhow::Result<U256> {
    let client = bitfinity_evm_canister_client();
    let r = client.get_min_gas_price().await;
    let r = r.map_err(|e| {
        log!(P0, "[bitfinity route]query gas price error: {:?}", &e);
        BitfinityRouteError::Custom(anyhow!(format!(
            "[bitfinity route]query gas price error: {:?}",
            &e
        )))
    })?.0;
    Ok(r.mul(11i32).div(10i32))
}

pub async fn get_balance(addr: String) -> anyhow::Result<U256> {
    let client = bitfinity_evm_canister_client();
    let r = client.eth_get_balance(
        did::H160::from_hex_str(addr.as_str()).unwrap(), BlockNumber::Latest).await;
    let r = r.map_err(|e| {
        log!(P0,
            "[bitfinity route]query chainkey address evm balance error: {:?}",
            &e
        );
        BitfinityRouteError::Custom(anyhow!(format!(
            "[bitfinity route]query chainkey address evm balance error: {:?}",
            &e
        )))
    })?.map_err(|e|{
        log!(P0,
            "[bitfinity route]query chainkey address evm balance evm error: {:?}",
            &e
        );
        BitfinityRouteError::EvmRpcError(format!(
            "[bitfinity route]query chainkey address evm balance evm error: {:?}",
            &e
        ))
    })?;
    Ok(r.0)
}


pub async fn get_transaction_receipt(
    hash: &String,
) -> std::result::Result<Option<did::TransactionReceipt>, BitfinityRouteError> {
    let client = bitfinity_evm_canister_client();
    let h = did::H256::from_hex_str(hash).map_err(|e|{
        log!(P0, "[bitfinity route] decode tx hash error: {:?}", &e);
        BitfinityRouteError::Custom(anyhow!("[bitfinity route] decode tx hash error: {:?}",&e))
    })?;
    let r = client.eth_get_transaction_receipt(h).await;
    let r = r.map_err(|e| {
        log!(P0,
            "[bitfinity route]query transaction receipt client error: hash: {}, error: {:?}",
            hash,
            &e
        );
        BitfinityRouteError::Custom(anyhow!(format!(
            "[bitfinity route]query transaction receipt client error: {:?}",
            &e
        )))
    })?.map_err(|e|{
        log!(P0,
            "[bitfinity route]query transaction receipt evm error: hash:{}, error: {:?}",
            hash,
            &e
        );
        BitfinityRouteError::EvmRpcError(format!(
            "[bitfinity route]query transaction receipt evm error: {:?}",
            &e
        ))
    })?;
    Ok(r)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct EvmJsonRpcRequest {
    pub method: String,
    pub params: Vec<String>,
    pub id: u64,
    pub jsonrpc: String,
}


#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct JsonRpcResponse<T> {
    pub jsonrpc: String,
    pub result: T,
    pub id: u32,
}

fn bitfinity_evm_canister_client() -> EvmCanisterClient<IcCanisterClient> {
    let p = read_state(|s|s.bitfinity_canister);
    EvmCanisterClient::new(IcCanisterClient::new(p))
}

