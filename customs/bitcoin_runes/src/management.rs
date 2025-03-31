//! This module contains async functions for interacting with the management canister.

use omnity_types::call_error::{CallError, Reason};
use crate::state::read_state;
use crate::tx;
use crate::ECDSAPublicKey;
use bitcoin::Transaction;
use candid::{CandidType, Principal};
use ic_btc_interface::{
    Address, GetBalanceRequest, GetCurrentFeePercentilesRequest, GetUtxosRequest, GetUtxosResponse,
    MillisatoshiPerByte, Network, Satoshi, UtxosFilterInRequest,
};
use ic_canister_log::log;
use ic_cdk::api::call::CallResult;
use ic_ic00_types::{
    DerivationPath, ECDSAPublicKeyArgs, ECDSAPublicKeyResponse, EcdsaCurve, EcdsaKeyId,
    SignWithECDSAArgs, SignWithECDSAReply,
};
use omnity_types::ic_log::CRITICAL;
use serde::de::DeserializeOwned;

async fn call<I, O>(method: &str, payment: u64, input: &I) -> Result<O, CallError>
where
    I: CandidType,
    O: CandidType + DeserializeOwned,
{
    let balance = ic_cdk::api::canister_balance128();
    if balance < payment as u128 {
        log!(
            CRITICAL,
            "Failed to call {}: need {} cycles, the balance is only {}",
            method,
            payment,
            balance
        );

        return Err(CallError {
            method: method.to_string(),
            reason: Reason::OutOfCycles,
        });
    }

    let res: Result<(O,), _> = ic_cdk::api::call::call_with_payment(
        Principal::management_canister(),
        method,
        (input,),
        payment,
    )
    .await;

    match res {
        Ok((output,)) => Ok(output),
        Err((code, msg)) => Err(CallError {
            method: method.to_string(),
            reason: Reason::from_reject(code, msg),
        }),
    }
}

#[derive(Clone, Copy)]
pub enum CallSource {
    /// The client initiated the call.
    Client,
    /// The custom initiated the call for internal bookkeeping.
    Custom,
}

pub async fn get_bitcoin_balance(
    network: Network,
    address: &Address,
    min_confirmations: u32,
    call_source: CallSource,
) -> Result<Satoshi, CallError> {
    // NB. The minimum number of cycles that need to be sent with the call is 10B (4B) for
    // Bitcoin mainnet (Bitcoin testnet):
    // https://internetcomputer.org/docs/current/developer-docs/integrations/bitcoin/bitcoin-how-it-works#api-fees--pricing
    let get_balance_cost_cycles = match network {
        Network::Mainnet => 10_000_000_000,
        Network::Testnet | Network::Regtest => 4_000_000_000,
    };

    // Calls "bitcoin_get_utxos" method with the specified argument on the
    // management canister.
    async fn bitcoin_get_balance(
        req: &GetBalanceRequest,
        cycles: u64,
        source: CallSource,
    ) -> Result<Satoshi, CallError> {
        match source {
            CallSource::Client => &crate::metrics::GET_UTXOS_CLIENT_CALLS,
            CallSource::Custom => &crate::metrics::GET_UTXOS_CUSTOM_CALLS,
        }
        .with(|cell| cell.set(cell.get() + 1));
        call("bitcoin_get_balance", cycles, req).await
    }
    bitcoin_get_balance(
        &GetBalanceRequest {
            address: address.to_string(),
            network: network.into(),
            min_confirmations: Some(min_confirmations),
        },
        get_balance_cost_cycles,
        call_source,
    )
    .await
}
/// Fetches the full list of UTXOs for the specified address.
pub async fn get_utxos(
    network: Network,
    address: &Address,
    min_confirmations: u32,
    source: CallSource,
) -> Result<GetUtxosResponse, CallError> {
    // NB. The minimum number of cycles that need to be sent with the call is 10B (4B) for
    // Bitcoin mainnet (Bitcoin testnet):
    // https://internetcomputer.org/docs/current/developer-docs/integrations/bitcoin/bitcoin-how-it-works#api-fees--pricing
    let get_utxos_cost_cycles = match network {
        Network::Mainnet => 10_000_000_000,
        Network::Testnet | Network::Regtest => 4_000_000_000,
    };

    // Calls "bitcoin_get_utxos" method with the specified argument on the
    // management canister.
    async fn bitcoin_get_utxos(
        req: &GetUtxosRequest,
        cycles: u64,
        source: CallSource,
    ) -> Result<GetUtxosResponse, CallError> {
        match source {
            CallSource::Client => &crate::metrics::GET_UTXOS_CLIENT_CALLS,
            CallSource::Custom => &crate::metrics::GET_UTXOS_CUSTOM_CALLS,
        }
        .with(|cell| cell.set(cell.get() + 1));
        call("bitcoin_get_utxos", cycles, req).await
    }

    let mut response = bitcoin_get_utxos(
        &GetUtxosRequest {
            address: address.to_string(),
            network: network.into(),
            filter: Some(UtxosFilterInRequest::MinConfirmations(min_confirmations)),
        },
        get_utxos_cost_cycles,
        source,
    )
    .await?;

    let mut utxos = std::mem::take(&mut response.utxos);

    // Continue fetching until there are no more pages.
    while let Some(page) = response.next_page {
        response = bitcoin_get_utxos(
            &GetUtxosRequest {
                address: address.to_string(),
                network: network.into(),
                filter: Some(UtxosFilterInRequest::Page(page)),
            },
            get_utxos_cost_cycles,
            source,
        )
        .await?;

        utxos.append(&mut response.utxos);
    }

    response.utxos = utxos;

    Ok(response)
}

/// Returns the current fee percentiles on the bitcoin network.
pub async fn get_current_fees(network: Network) -> Result<Vec<MillisatoshiPerByte>, CallError> {
    let cost_cycles = match network {
        Network::Mainnet => 100_000_000,
        Network::Testnet => 40_000_000,
        Network::Regtest => 0,
    };

    call(
        "bitcoin_get_current_fee_percentiles",
        cost_cycles,
        &GetCurrentFeePercentilesRequest {
            network: network.into(),
        },
    )
    .await
}

/// Sends the transaction to the network the management canister interacts with.
pub async fn send_etching(transaction: &Transaction) -> Result<(), CallError> {
    use ic_cdk::api::management_canister::bitcoin::BitcoinNetwork;
    let network = read_state(|s| s.btc_network);
    let cdk_network = match network {
        Network::Mainnet => BitcoinNetwork::Mainnet,
        Network::Testnet => BitcoinNetwork::Testnet,
        Network::Regtest => BitcoinNetwork::Regtest,
    };
    let tx_bytes = bitcoin::consensus::serialize(&transaction);
    ic_cdk::api::management_canister::bitcoin::bitcoin_send_transaction(
        ic_cdk::api::management_canister::bitcoin::SendTransactionRequest {
            transaction: tx_bytes,
            network: cdk_network,
        },
    )
    .await
    .map_err(|(code, msg)| CallError {
        method: "bitcoin_send_transaction".to_string(),
        reason: Reason::from_reject(code, msg),
    })
}

pub async fn send_transaction(
    transaction: &tx::SignedTransaction,
    network: Network,
) -> Result<(), CallError> {
    use ic_cdk::api::management_canister::bitcoin::BitcoinNetwork;

    let cdk_network = match network {
        Network::Mainnet => BitcoinNetwork::Mainnet,
        Network::Testnet => BitcoinNetwork::Testnet,
        Network::Regtest => BitcoinNetwork::Regtest,
    };
    let tx_bytes = transaction.serialize();
    ic_cdk::api::management_canister::bitcoin::bitcoin_send_transaction(
        ic_cdk::api::management_canister::bitcoin::SendTransactionRequest {
            transaction: tx_bytes,
            network: cdk_network,
        },
    )
    .await
    .map_err(|(code, msg)| CallError {
        method: "bitcoin_send_transaction".to_string(),
        reason: Reason::from_reject(code, msg),
    })
}

/// Fetches the ECDSA public key of the canister.
pub async fn ecdsa_public_key(
    key_name: String,
    derivation_path: DerivationPath,
) -> Result<ECDSAPublicKey, CallError> {
    // Retrieve the public key of this canister at the given derivation path
    // from the ECDSA API.
    call(
        "ecdsa_public_key",
        /*payment=*/ 0,
        &ECDSAPublicKeyArgs {
            canister_id: None,
            derivation_path,
            key_id: EcdsaKeyId {
                curve: EcdsaCurve::Secp256k1,
                name: key_name,
            },
        },
    )
    .await
    .map(|response: ECDSAPublicKeyResponse| ECDSAPublicKey {
        public_key: response.public_key,
        chain_code: response.chain_code,
    })
}

/// Signs a message hash using the tECDSA API.
pub async fn sign_with_ecdsa(
    key_name: String,
    derivation_path: DerivationPath,
    message_hash: [u8; 32],
) -> Result<Vec<u8>, CallError> {
    // The cost of a single tECDSA signature is 26_153_846_153.
    // ref: https://internetcomputer.org/docs/current/references/t-sigs-how-it-works#fees-for-the-t-ecdsa-production-key
    const CYCLES_PER_SIGNATURE: u64 = 30_000_000_000;

    let reply: SignWithECDSAReply = call(
        "sign_with_ecdsa",
        CYCLES_PER_SIGNATURE,
        &SignWithECDSAArgs {
            message_hash,
            derivation_path,
            key_id: EcdsaKeyId {
                curve: EcdsaCurve::Secp256k1,
                name: key_name.clone(),
            },
        },
    )
    .await?;
    Ok(reply.signature)
}

pub async fn raw_rand() -> CallResult<[u8; 32]> {
    let (random_bytes,): (Vec<u8>,) =
        ic_cdk::api::call::call(Principal::management_canister(), "raw_rand", ()).await?;
    let mut v = [0u8; 32];
    v.copy_from_slice(random_bytes.as_slice());
    Ok(v)
}
