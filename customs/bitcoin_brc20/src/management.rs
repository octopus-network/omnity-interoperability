use bitcoin::Transaction;
use candid::{CandidType, Principal};
use ic_btc_interface::{GetCurrentFeePercentilesRequest, MillisatoshiPerByte, Network};
use ic_cdk::api::call::CallResult;
use ic_ic00_types::{
    DerivationPath, ECDSAPublicKeyArgs, ECDSAPublicKeyResponse, EcdsaCurve, EcdsaKeyId,
    SignWithECDSAArgs, SignWithECDSAReply,
};
use serde::de::DeserializeOwned;

use crate::bitcoin::ECDSAPublicKey;
use omnity_types::call_error::{CallError, Reason};
use crate::state::read_state;

async fn call<I, O>(method: &str, payment: u64, input: &I) -> Result<O, CallError>
where
    I: CandidType,
    O: CandidType + DeserializeOwned,
{
    let balance = ic_cdk::api::canister_balance128();
    if balance < payment as u128 {
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
pub async fn send_transaction(transaction: &Transaction) -> Result<(), CallError> {
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

pub async fn raw_rand() -> CallResult<[u8; 32]> {
    let (random_bytes,): (Vec<u8>,) =
        ic_cdk::api::call::call(Principal::management_canister(), "raw_rand", ()).await?;
    let mut v = [0u8; 32];
    v.copy_from_slice(random_bytes.as_slice());
    Ok(v)
}

/// Signs a message hash using the tECDSA API.
pub async fn sign_with_ecdsa(
    key_name: String,
    derivation_path: DerivationPath,
    message_hash: [u8; 32],
) -> Result<Vec<u8>, CallError> {
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
