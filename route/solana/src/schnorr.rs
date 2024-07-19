#![allow(unused)]
use candid::CandidType;

use omnity_types::TokenId;
use serde::{Deserialize, Serialize};

use std::convert::TryFrom;
use std::convert::TryInto;

use crate::state::management_canister;
use crate::state::CanisterId;

#[derive(CandidType, Serialize, Debug)]
pub struct ManagementCanisterSchnorrPublicKeyRequest {
    pub canister_id: Option<CanisterId>,
    pub derivation_path: Vec<Vec<u8>>,
    pub key_id: SchnorrKeyId,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct ManagementCanisterSchnorrPublicKeyReply {
    pub public_key: Vec<u8>,
    pub chain_code: Vec<u8>,
}

#[derive(CandidType, Serialize, Debug, Clone)]
pub struct SchnorrKeyId {
    pub algorithm: SchnorrAlgorithm,
    pub name: String,
}

#[derive(CandidType, Serialize, Debug)]
pub struct ManagementCanisterSignatureRequest {
    pub message: Vec<u8>,
    pub derivation_path: Vec<Vec<u8>>,
    pub key_id: SchnorrKeyId,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct ManagementCanisterSignatureReply {
    pub signature: Vec<u8>,
}

#[derive(CandidType, Serialize, Deserialize, Debug, Copy, Clone)]
pub enum SchnorrAlgorithm {
    #[serde(rename = "bip340secp256k1")]
    Bip340Secp256k1,
    #[serde(rename = "ed25519")]
    Ed25519,
}

#[derive(CandidType, Serialize, Deserialize, Debug)]
pub struct PublicKeyReply {
    pub public_key_hex: String,
}

#[derive(CandidType, Serialize, Deserialize, Debug)]
pub struct SignatureReply {
    pub signature_hex: String,
}

#[derive(CandidType, Serialize, Deserialize, Debug)]
pub struct SignatureVerificationReply {
    pub is_signature_valid: bool,
}

pub async fn sol_token_address(token_id: TokenId) -> Result<Vec<u8>, String> {
    let mut derivation_path = ic_cdk::api::caller().as_slice().to_vec();
    derivation_path.extend_from_slice(token_id.as_bytes());
    let request = ManagementCanisterSchnorrPublicKeyRequest {
        canister_id: None,
        derivation_path: vec![derivation_path],
        key_id: SchnorrKeyIds::TestKeyLocalDevelopment.to_key_id(SchnorrAlgorithm::Ed25519),
    };

    let (res,): (ManagementCanisterSchnorrPublicKeyReply,) =
        ic_cdk::call(management_canister(), "schnorr_public_key", (request,))
            .await
            .map_err(|e| format!("schnorr_public_key failed {}", e.1))?;

    Ok(res.public_key)
}

pub async fn public_key(algorithm: SchnorrAlgorithm) -> Result<PublicKeyReply, String> {
    let request = ManagementCanisterSchnorrPublicKeyRequest {
        canister_id: None,
        derivation_path: vec![ic_cdk::api::caller().as_slice().to_vec()],
        key_id: SchnorrKeyIds::TestKeyLocalDevelopment.to_key_id(algorithm),
    };

    let (res,): (ManagementCanisterSchnorrPublicKeyReply,) =
        ic_cdk::call(management_canister(), "schnorr_public_key", (request,))
            .await
            .map_err(|e| format!("schnorr_public_key failed {}", e.1))?;

    Ok(PublicKeyReply {
        public_key_hex: hex::encode(&res.public_key),
    })
}

// #[allow(unused)]
pub async fn sign(message: String, algorithm: SchnorrAlgorithm) -> Result<SignatureReply, String> {
    let internal_request = ManagementCanisterSignatureRequest {
        message: message.as_bytes().to_vec(),
        derivation_path: vec![ic_cdk::api::caller().as_slice().to_vec()],
        key_id: SchnorrKeyIds::TestKeyLocalDevelopment.to_key_id(algorithm),
    };

    let (internal_reply,): (ManagementCanisterSignatureReply,) =
        ic_cdk::api::call::call_with_payment(
            management_canister(),
            "sign_with_schnorr",
            (internal_request,),
            25_000_000_000,
        )
        .await
        .map_err(|e| format!("sign_with_schnorr failed {e:?}"))?;

    Ok(SignatureReply {
        signature_hex: hex::encode(&internal_reply.signature),
    })
}

// #[allow(unused)]
pub async fn verify(
    signature_hex: String,
    message: String,
    public_key_hex: String,
    algorithm: SchnorrAlgorithm,
) -> Result<SignatureVerificationReply, String> {
    let sig_bytes = hex::decode(&signature_hex).expect("failed to hex-decode signature");
    let msg_bytes = message.as_bytes();
    let pk_bytes = hex::decode(&public_key_hex).expect("failed to hex-decode public key");

    match algorithm {
        SchnorrAlgorithm::Bip340Secp256k1 => {
            verify_bip340_secp256k1(&sig_bytes, msg_bytes, &pk_bytes)
        }
        SchnorrAlgorithm::Ed25519 => verify_ed25519(&sig_bytes, &msg_bytes, &pk_bytes),
    }
}

// #[allow(unused)]
pub fn verify_bip340_secp256k1(
    sig_bytes: &[u8],
    msg_bytes: &[u8],
    secp1_pk_bytes: &[u8],
) -> Result<SignatureVerificationReply, String> {
    assert_eq!(secp1_pk_bytes.len(), 33);

    let sig =
        k256::schnorr::Signature::try_from(sig_bytes).expect("failed to deserialize signature");

    let vk = k256::schnorr::VerifyingKey::from_bytes(&secp1_pk_bytes[1..])
        .expect("failed to deserialize BIP340 encoding into public key");

    let is_signature_valid = vk.verify_raw(&msg_bytes, &sig).is_ok();

    Ok(SignatureVerificationReply { is_signature_valid })
}

// #[allow(unused)]
fn verify_ed25519(
    sig_bytes: &[u8],
    msg_bytes: &[u8],
    pk_bytes: &[u8],
) -> Result<SignatureVerificationReply, String> {
    use ed25519_dalek::{Signature, Verifier, VerifyingKey};

    let pk: [u8; 32] = pk_bytes
        .try_into()
        .expect("ed25519 public key incorrect length");
    let vk = VerifyingKey::from_bytes(&pk).expect("failed to parse ed25519 public key");

    let signature = Signature::from_slice(sig_bytes).expect("ed25519 signature incorrect length");

    let is_signature_valid = vk.verify(msg_bytes, &signature).is_ok();

    Ok(SignatureVerificationReply { is_signature_valid })
}

pub enum SchnorrKeyIds {
    #[allow(unused)]
    TestKeyLocalDevelopment,
    #[allow(unused)]
    TestKey1,
    #[allow(unused)]
    ProductionKey1,
}

impl SchnorrKeyIds {
    pub fn to_key_id(&self, algorithm: SchnorrAlgorithm) -> SchnorrKeyId {
        SchnorrKeyId {
            algorithm,
            name: match self {
                Self::TestKeyLocalDevelopment => "dfx_test_key",
                Self::TestKey1 => "test_key_1",
                Self::ProductionKey1 => "key_1",
            }
            .to_string(),
        }
    }
}

// In the following, we register a custom getrandom implementation because
// otherwise getrandom (which is a dependency of k256) fails to compile.
// This is necessary because getrandom by default fails to compile for the
// wasm32-unknown-unknown target (which is required for deploying a canister).
// Our custom implementation always fails, which is sufficient here because
// we only use the k256 crate for verifying secp256k1 signatures, and such
// signature verification does not require any randomness.
// getrandom::register_custom_getrandom!(always_fail);
// pub fn always_fail(_buf: &mut [u8]) -> Result<(), getrandom::Error> {
//     Err(getrandom::Error::UNSUPPORTED)
// }
