use candid::Principal;
use ic_management_canister_types::{
    SchnorrAlgorithm, SchnorrKeyId, SchnorrPublicKeyArgs, SchnorrPublicKeyResult,
    SignWithSchnorrArgs, SignWithSchnorrResult,
};
use serde_bytes::ByteBuf;

const ECDSA_SIGN_COST: u64 = 26_153_846_153;

pub async fn sign_with_eddsa(
    key_name: String,
    derivation_path: Vec<ByteBuf>,
    message: Vec<u8>,
) -> Vec<u8> {
    let res: Result<(SignWithSchnorrResult,), _> = ic_cdk::api::call::call_with_payment(
        Principal::management_canister(),
        "sign_with_schnorr",
        (SignWithSchnorrArgs {
            message,
            derivation_path: derivation_path
                .iter()
                .map(|p| p.clone().into_vec())
                .collect(),
            key_id: SchnorrKeyId {
                name: key_name,
                algorithm: SchnorrAlgorithm::Ed25519,
            },
        },),
        // https://internetcomputer.org/docs/current/references/t-sigs-how-it-works/#fees-for-the-t-schnorr-production-key
        // 26_153_846_153,
        ECDSA_SIGN_COST,
    )
    .await;

    res.unwrap().0.signature
}

pub async fn eddsa_public_key(key_name: String, derivation_path: Vec<ByteBuf>) -> Vec<u8> {
    let res: Result<(SchnorrPublicKeyResult,), _> = ic_cdk::call(
        Principal::management_canister(),
        "schnorr_public_key",
        (SchnorrPublicKeyArgs {
            canister_id: None,
            derivation_path: derivation_path
                .iter()
                .map(|p| p.clone().into_vec())
                .collect(),
            key_id: SchnorrKeyId {
                algorithm: SchnorrAlgorithm::Ed25519,
                name: key_name,
            },
        },),
    )
    .await;

    res.unwrap().0.public_key
}
