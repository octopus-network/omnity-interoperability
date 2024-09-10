use std::future::Future;
use std::str::FromStr;

use candid::{CandidType, Nat};
use cketh_common::eth_rpc::{Hash, HttpOutcallError, RpcError};
use cketh_common::eth_rpc_client::providers::{RpcApi, RpcService};
use cketh_common::eth_rpc_client::RpcConfig;
use cketh_common::numeric::BlockNumber;
use ethereum_types::Address;
use ethers_core::abi::ethereum_types;
use ethers_core::types::{Eip1559TransactionRequest, TransactionRequest, U256};
use ethers_core::utils::keccak256;
use evm_rpc::{MultiRpcResult, RpcServices};
use evm_rpc::candid_types::{BlockTag, GetTransactionCountArgs, SendRawTransactionStatus};
use ic_cdk::api::management_canister::ecdsa::{sign_with_ecdsa, SignWithEcdsaArgument};
use log::{error, info};
use num_traits::ToPrimitive;
use serde_derive::{Deserialize, Serialize};

use crate::{const_args, Error, eth_common, state};
use crate::const_args::{
    BROADCAST_TX_CYCLES, EVM_ADDR_BYTES_LEN, GET_ACCOUNT_NONCE_CYCLES, SCAN_EVM_CYCLES,
};
use crate::eth_common::EvmAddressError::LengthError;
use crate::state::{evm_transfer_gas_factor, rpc_providers};

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct TransactionReceipt {
    #[serde(rename = "blockHash")]
    pub block_hash: String,
    #[serde(rename = "blockNumber")]
    pub block_number: String,
    #[serde(rename = "gasUsed")]
    pub gas_used: String,
    pub status: String,
    #[serde(rename = "transactionHash")]
    pub transaction_hash: String,
    #[serde(rename = "contractAddress")]
    pub contract_address: Option<String>,
    pub from: String,
    pub logs: Vec<cketh_common::eth_rpc::LogEntry>,
    #[serde(rename = "logsBloom")]
    pub logs_bloom: String,
    pub to: String,
    #[serde(rename = "transactionIndex")]
    pub transaction_index: String,
    pub r#type: String,
}

impl Into<evm_rpc::candid_types::TransactionReceipt> for TransactionReceipt {
    fn into(self) -> evm_rpc::candid_types::TransactionReceipt {
        evm_rpc::candid_types::TransactionReceipt {
            block_hash: self.block_hash,
            block_number: BlockNumber::new(hex_to_u64(&self.block_number) as u128),
            effective_gas_price: Default::default(),
            gas_used: hex_to_u64(&self.gas_used).into(),
            status: hex_to_u64(&self.status).into(),
            transaction_hash: self.transaction_hash,
            contract_address: self.contract_address,
            from: self.from,
            logs: self.logs,
            logs_bloom: self.logs_bloom,
            to: self.to,
            transaction_index: hex_to_u64(&self.transaction_index).into(),
            r#type: self.r#type,
        }
    }
}

pub fn hex_to_u64(hex_str: &String) -> u64 {
    u64::from_str_radix(hex_str.strip_prefix("0x").unwrap(), 16).unwrap()
}

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

pub async fn sign_transaction(evm_tx_request: EvmTxRequest) -> anyhow::Result<Vec<u8>> {
    match evm_tx_request {
        EvmTxRequest::Legacy(tx) => sign_transaction_legacy(tx).await,
        EvmTxRequest::Eip1559(tx) => sign_transaction_eip1559(tx).await,
    }
}

pub async fn sign_transaction_eip1559(tx: Eip1559TransactionRequest) -> anyhow::Result<Vec<u8>> {
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

pub async fn sign_transaction_legacy(tx: TransactionRequest) -> anyhow::Result<Vec<u8>> {
    use ethers_core::types::Signature;
    let unsigned_tx_bytes = tx.rlp().to_vec();
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
        v: y_parity(&txhash, &r.signature, crate::state::public_key().as_ref())
            + tx.chain_id.unwrap().as_u64() * 2
            + 35,
        r: U256::from_big_endian(&r.signature[0..32]),
        s: U256::from_big_endian(&r.signature[32..64]),
    };
    let signed_tx_bytes = tx.rlp_signed(&signature).to_vec();
    Ok(signed_tx_bytes)
}

pub async fn broadcast(tx: Vec<u8>, rpcs: Vec<RpcApi>) -> Result<String, super::Error> {
    let raw = format!("0x{}", hex::encode(tx));
    info!("[evm route] preparing to send tx: {}", raw);
    let (r,): (MultiRpcResult<SendRawTransactionStatus>,) =
        ic_cdk::api::call::call_with_payment128(
            crate::state::rpc_addr(),
            "eth_sendRawTransaction",
            (
                RpcServices::Custom {
                    chain_id: crate::state::evm_chain_id(),
                    services: rpcs,
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
        MultiRpcResult::Consistent(res) => {
            match res {
                Ok(s) => match s {
                    SendRawTransactionStatus::Ok(hash) => {
                        Ok(hex::encode(hash.unwrap_or(Hash([0u8; 32])).0))
                    }
                    SendRawTransactionStatus::InsufficientFunds => {
                        Err(Error::Fatal("InsufficientFunds".to_string()))
                    }
                    SendRawTransactionStatus::NonceTooLow => {
                        Err(Error::Custom("NonceTooLow".to_string()))
                    }
                    SendRawTransactionStatus::NonceTooHigh => {
                        Err(Error::Custom("NonceToohigh".to_string()))
                    }
                },
                Err(r) => {
                    if let RpcError::JsonRpcError(ref jerr) = r {
                        if (jerr.code == -32603 && jerr.message == "already known")
                        || (jerr.code == -32010 && jerr.message == "pending transaction with same hash already exists")
                        || (jerr.code == -32015 && jerr.message == "transaction pool error transaction already exists in the pool") {
                        return Ok(hex::encode([1u8; 32]));
                    }
                        if jerr.code == -32015
                            && jerr
                                .message
                                .contains("transaction pool error invalid transaction nonce")
                        {
                            return Err(Error::Temporary);
                        }
                    }
                    Err(Error::EvmRpcError(format!("{:?}", r)))
                }
            }
        }
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

pub async fn get_account_nonce(addr: String, rpcs: Vec<RpcApi>) -> Result<u64, super::Error> {
    let (r,): (MultiRpcResult<Nat>,) = ic_cdk::api::call::call_with_payment128(
        crate::state::rpc_addr(),
        "eth_getTransactionCount",
        (
            RpcServices::Custom {
                chain_id: crate::state::evm_chain_id(),
                services: rpcs,
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

pub async fn get_gasprice(_v: (), rpcs: Vec<RpcApi>) -> Result<U256, Error> {
    // Define request parameters
    let params = (
        RpcService::Custom(rpcs.clone().pop().unwrap()), // Ethereum mainnet
        serde_json::to_string(&EvmJsonRpcRequest {
            method: "eth_gasPrice".to_string(),
            params: vec![],
            id: 1,
            jsonrpc: "2.0".to_string(),
        })
        .unwrap(),
        1000u64,
    );
    // Get cycles cost
    let (cycles_result,): (std::result::Result<u128, RpcError>,) =
        ic_cdk::api::call::call(state::rpc_addr(), "requestCost", params.clone())
            .await
            .unwrap();
    let cycles = cycles_result.map_err(|e| {
        error!("[evm route] evm request error: {:?}", e);
        Error::Custom(format!("error in `request_cost`: {:?}", e))
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
        Error::Custom(format!("[evm route]query gas price error: {:?}", &e))
    })?;
    let r: BlockNumberResult =
        serde_json::from_str(r.as_str()).map_err(|e| Error::Fatal(e.to_string()))?;
    let r = r.result.strip_prefix("0x").unwrap_or(r.result.as_str());
    let r = u64::from_str_radix(r, 16).map_err(|e| Error::Fatal(e.to_string()))?;
    Ok(U256::from(r * evm_transfer_gas_factor() / 100))
}

pub async fn get_balance(addr: String, rpcs: Vec<RpcApi>) -> Result<U256, Error> {
    let params = (
        RpcService::Custom(rpcs.clone().pop().unwrap()), // Ethereum mainnet
        serde_json::to_string(&EvmJsonRpcRequest {
            method: "eth_getBalance".to_string(),
            params: vec![addr, "latest".to_string()],
            id: 1,
            jsonrpc: "2.0".to_string(),
        })
        .unwrap(),
        1000u64,
    );
    // Get cycles cost
    let (cycles_result,): (std::result::Result<u128, RpcError>,) =
        ic_cdk::api::call::call(state::rpc_addr(), "requestCost", params.clone())
            .await
            .unwrap();
    let cycles = cycles_result.map_err(|e| {
        error!("[evm route] evm request error: {:?}", e);
        Error::Custom(format!("error in `request_cost`: {:?}", e))
    })?;
    // Call with expected number of cycles
    let (result,): (std::result::Result<String, RpcError>,) =
        ic_cdk::api::call::call_with_payment128(state::rpc_addr(), "request", params, cycles)
            .await
            .map_err(|err| Error::IcCallError(err.0, err.1))?;
    #[derive(Serialize, Deserialize, Debug)]
    struct BalanceResult {
        pub id: u32,
        pub jsonrpc: String,
        pub result: String,
    }
    let r = result.map_err(|e| {
        error!(
            "[evm route]query chainkey address evm balance error: {:?}",
            &e
        );
        Error::Custom(format!(
            "[evm route]query chainkey address evm balance error: {:?}",
            &e
        ))
    })?;
    let r: BalanceResult =
        serde_json::from_str(r.as_str()).map_err(|e| Error::Fatal(e.to_string()))?;
    let r = r.result.strip_prefix("0x").unwrap_or(r.result.as_str());
    let r = u64::from_str_radix(r, 16).map_err(|e| Error::Fatal(e.to_string()))?;
    Ok(U256::from(r))
}

pub async fn get_transaction_receipt(
    hash: &String,
    rpcs: Vec<RpcApi>,
) -> std::result::Result<Option<evm_rpc::candid_types::TransactionReceipt>, Error> {
    let rpc_size = rpcs.len() as u128;
    let (rpc_result,): (MultiRpcResult<Option<evm_rpc::candid_types::TransactionReceipt>>,) =
        ic_cdk::api::call::call_with_payment128(
            crate::state::rpc_addr(),
            "eth_getTransactionReceipt",
            (
                RpcServices::Custom {
                    chain_id: crate::state::evm_chain_id(),
                    services: rpcs,
                },
                Some(RpcConfig {
                    response_size_estimate: Some(10000),
                }),
                hash,
            ),
            SCAN_EVM_CYCLES * rpc_size,
        )
        .await
        .map_err(|err| Error::IcCallError(err.0, err.1))?;
    match rpc_result {
        MultiRpcResult::Consistent(result) => match result {
            Ok(info) => {
                return Ok(info);
            }
            Err(e) => {
                if let RpcError::HttpOutcallError(ee) = e.clone() {
                    match ee {
                        HttpOutcallError::IcError { .. } => {}
                        HttpOutcallError::InvalidHttpJsonRpcResponse { status, body, .. } => {
                            if status == 200 {
                                info!("content: {}", &body);
                                let json_rpc: JsonRpcResponse<eth_common::TransactionReceipt> =
                                    serde_json::from_str(&body).map_err(|e| {
                                        Error::EvmRpcError(format!(
                                            "local deserialize error: {}",
                                            e.to_string()
                                        ))
                                    })?;
                                return Ok(Some(json_rpc.result.into()));
                            }
                        }
                    }
                }
                error!("query transaction receipt error: {:?}", e.clone());
                Err(Error::EvmRpcError(format!("{:?}", e)))
            }
        },
        MultiRpcResult::Inconsistent(_) => {
            Err(super::Error::EvmRpcError("Inconsistent result".to_string()))
        }
    }
}

#[derive(
    CandidType, Serialize, Deserialize, Default, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord,
)]
pub enum EvmTxType {
    Legacy,
    #[default]
    Eip1559,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum EvmTxRequest {
    Legacy(TransactionRequest),
    Eip1559(Eip1559TransactionRequest),
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

pub async fn call_rpc_with_retry<P: Clone, T, R: Future<Output = Result<T, Error>>>(
    params: P,
    call_rpc: fn(params: P, rpc_api: Vec<RpcApi>) -> R,
) -> Result<T, Error> {
    let rpcs = rpc_providers();
    let mut rs = Err(Error::RouteNotInitialized);
    if rpcs.is_empty() {
        return rs;
    }
    for i in 0..const_args::RPC_RETRY_TIMES {
        let r = rpcs[i % rpcs.len()].clone();
        log::info!("[evm route]request rpc request times: {}, rpc_url: {}", i+1, r.url.clone());
        let call_res = call_rpc(params.clone(), vec![r]).await;
        if call_res.is_ok() {
            rs = call_res;
            break;
        } else {
            let err = call_res.err().unwrap();
            log::error!("[evm route]call  rpc error: {}", err.clone().to_string());
            rs = Err(err);
        }
        if let Err(Error::Fatal(_)) = rs {
            break;
        }
    }
    rs
}
