use candid::{CandidType, Principal};
use ic_ic00_types::{
    DerivationPath, ECDSAPublicKeyArgs, ECDSAPublicKeyResponse, EcdsaCurve, EcdsaKeyId,
};
use serde::de::DeserializeOwned;

use crate::call_error::{CallError, Reason};
use crate::types::ECDSAPublicKey;

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

// pub async fn sign_with(
//     key_name: &str,
//     derivation_path: Vec<Vec<u8>>,
//     message_hash: [u8; 32],
// ) -> Result<Vec<u8>, String> {
//     let args = ecdsa::SignWithEcdsaArgument {
//         message_hash: message_hash.to_vec(),
//         derivation_path,
//         key_id: ecdsa::EcdsaKeyId {
//             curve: ecdsa::EcdsaCurve::Secp256k1,
//             name: key_name.to_string(),
//         },
//     };

//     let (response,): (ecdsa::SignWithEcdsaResponse,) = ecdsa::sign_with_ecdsa(args)
//         .await
//         .map_err(|err| format!("sign_with_ecdsa failed {:?}", err))?;

//     Ok(response.signature)
// }