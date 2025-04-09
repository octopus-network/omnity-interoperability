use std::ops::{Div, Mul};

use did::error::{EvmError, TransactionPoolError};
use did::{BlockNumber, H256};

use ethereum_common::const_args::EIP1559_TX_ID;
use ethereum_common::error::Error;
use ethereum_common::signs::y_parity;
use ethers_core::types::{Eip1559TransactionRequest, U256, U64};
use ethers_core::utils::keccak256;
use evm_canister_client::{EvmCanisterClient, IcCanisterClient};
use ic_canister_log::log;
use ic_cdk::api::management_canister::ecdsa::{sign_with_ecdsa, SignWithEcdsaArgument};
use omnity_types::ic_log::ERROR;

use crate::state::{minter_addr, read_state};
use ethereum_common::error::Error::{EvmRpcError, Temporary};

pub async fn sign_transaction(tx: Eip1559TransactionRequest) -> anyhow::Result<did::Transaction> {
    sign_transaction_eip1559(tx).await
}

pub async fn sign_transaction_eip1559(
    tx: Eip1559TransactionRequest,
) -> anyhow::Result<did::Transaction> {
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
        .map_err(|(_, e)| Error::ChainKeyError(e))?;
    let signature = Signature {
        v: y_parity(&txhash, &r.signature, crate::state::public_key().as_ref()),
        r: U256::from_big_endian(&r.signature[0..32]),
        s: U256::from_big_endian(&r.signature[32..64]),
    };
    let port_addr = read_state(|s| s.omnity_port_contract.clone());
    let mut transaction = ethers_core::types::Transaction {
        from: did::H160::from_hex_str(&minter_addr())?.0,
        to: Some(did::H160::from(port_addr.0).0),
        nonce: did::U256::from(tx.nonce.unwrap_or_default()).0,
        value: 0.into(),
        gas: tx.gas.unwrap_or_default(),
        input: tx.data.unwrap(),
        v: U64::from(signature.v),
        r: signature.r,
        s: signature.s,
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

pub async fn broadcast(tx: did::Transaction) -> Result<String, Error> {
    let client = bitfinity_evm_canister_client();
    let r = client.send_raw_transaction(tx.clone()).await.map_err(|e| {
        log!(
            ERROR,
            "[bitfinity route]broadcasts canister client error: {}",
            e.to_string()
        );
        Error::EvmRpcError(e.to_string())
    })?;
    match r {
        Ok(r) => Ok(r.to_hex_str()),
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
                EvmError::TransactionPool(e) => match e {
                    TransactionPoolError::TooManyTransactions => {}
                    TransactionPoolError::TxReplacementUnderpriced => {}
                    TransactionPoolError::TransactionAlreadyExists => {
                        return Ok(tx.hash.to_hex_str());
                    }
                    TransactionPoolError::InvalidNonce { .. } => {
                        return Err(Temporary);
                    }
                },
                EvmError::NoHistoryDataForBlock(_) => {}
                EvmError::BlockDoesNotExist(_) => {}
                EvmError::TransactionSignature(_) => {}
                EvmError::GasTooLow { .. } => {}
                EvmError::AnonymousPrincipal => {}
                EvmError::BadRequest(_) => {}
                EvmError::TransactionReverted(_) => {}
                EvmError::Precompile(_) => {}
            }
            Err(Error::Custom(format!("broadcast error: {}", e.to_string())))
        }
    }
}

pub async fn get_account_nonce(addr: String) -> Result<did::U256, Error> {
    let client = bitfinity_evm_canister_client();
    let r = client
        .eth_get_transaction_count(
            did::H160::from_hex_str(addr.as_str()).unwrap(),
            BlockNumber::Pending,
        )
        .await;
    let r = r
        .map_err(|e| {
            log!(
                ERROR,
                "[bitfinity route]query chainkey account nonce client error: {:?}",
                &e
            );
            EvmRpcError(e.to_string())
        })?
        .map_err(|e| {
            log!(
                ERROR,
                "[bitfinity route]query chainkey account nonce evm error: {:?}",
                &e
            );
            EvmRpcError(e.to_string())
        })?;
    Ok(r)
}

pub async fn get_gasprice() -> anyhow::Result<U256> {
    let client = bitfinity_evm_canister_client();
    let r = client.get_min_gas_price().await;
    let r = r
        .map_err(|e| {
            log!(ERROR, "[bitfinity route]query gas price error: {:?}", &e);
            Error::Custom(format!("[bitfinity route]query gas price error: {:?}", &e))
        })?
        .0;
    Ok(r.mul(11i32).div(10i32))
}

pub async fn get_balance(addr: String) -> anyhow::Result<U256> {
    let client = bitfinity_evm_canister_client();
    let r = client
        .eth_get_balance(did::H160::from_hex_str(addr.as_str())?, BlockNumber::Latest)
        .await;
    let r = r
        .map_err(|e| {
            log!(
                ERROR,
                "[bitfinity route]query chainkey address evm balance error: {:?}",
                &e
            );
            Error::Custom(format!(
                "[bitfinity route]query chainkey address evm balance error: {:?}",
                &e
            ))
        })?
        .map_err(|e| {
            log!(
                ERROR,
                "[bitfinity route]query chainkey address evm balance evm error: {:?}",
                &e
            );
            Error::EvmRpcError(format!(
                "[bitfinity route]query chainkey address evm balance evm error: {:?}",
                &e
            ))
        })?;
    Ok(r.0)
}

pub async fn get_transaction_receipt(
    hash: &String,
) -> std::result::Result<Option<did::TransactionReceipt>, Error> {
    let client = bitfinity_evm_canister_client();
    let h = did::H256::from_hex_str(hash).map_err(|e| {
        log!(ERROR, "[bitfinity route] decode tx hash error: {:?}", &e);
        Error::Custom(format!("[bitfinity route] decode tx hash error: {:?}", &e))
    })?;
    let r = client.eth_get_transaction_receipt(h).await;
    let r = r
        .map_err(|e| {
            log!(
                ERROR,
                "[bitfinity route]query transaction receipt client error: hash: {}, error: {:?}",
                hash,
                &e
            );
            Error::Custom(format!(
                "[bitfinity route]query transaction receipt client error: {:?}",
                &e
            ))
        })?
        .map_err(|e| {
            log!(
                ERROR,
                "[bitfinity route]query transaction receipt evm error: hash:{}, error: {:?}",
                hash,
                &e
            );
            Error::EvmRpcError(format!(
                "[bitfinity route]query transaction receipt evm error: {:?}",
                &e
            ))
        })?;
    Ok(r)
}

fn bitfinity_evm_canister_client() -> EvmCanisterClient<IcCanisterClient> {
    let p = read_state(|s| s.bitfinity_canister);
    EvmCanisterClient::new(IcCanisterClient::new(p))
}
