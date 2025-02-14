use crate::ic_sui::constants::EDDSA_SIGN_COST;
use bip32::Seed;
use candid::Principal;
use candid::{CandidType, Deserialize};
use ic_management_canister_types::{
    DerivationPath, SchnorrAlgorithm, SchnorrKeyId, SchnorrPublicKeyArgs, SchnorrPublicKeyResponse,
    SignWithSchnorrArgs, SignWithSchnorrReply,
};
use ic_stable_structures::storable::Bound;
use ic_stable_structures::Storable;
use serde::Serialize;
use serde_bytes::ByteBuf;
use sha2::Digest;
use std::borrow::Cow;
use std::vec;

#[derive(
    Default, Hash, Eq, Ord, PartialEq, PartialOrd, CandidType, Deserialize, Serialize, Debug, Clone,
)]
pub enum KeyType {
    #[default]
    ChainKey,
    Native(Vec<u8>),
}

impl Storable for KeyType {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        let bytes = bincode::serialize(&self).expect("failed to serialize KeyType");
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        bincode::deserialize(bytes.as_ref()).expect("failed to deserialize KeyType")
    }

    const BOUND: Bound = Bound::Unbounded;
}

/// Fetches the ed25519 public key from the schnorr canister.
pub async fn public_key_ed25519(
    key_type: KeyType,
    key_name: String,
    derivation_path: Vec<ByteBuf>,
) -> Vec<u8> {
    match key_type {
        KeyType::ChainKey => {
            let res: Result<(SchnorrPublicKeyResponse,), _> = ic_cdk::call(
                Principal::management_canister(),
                "schnorr_public_key",
                (SchnorrPublicKeyArgs {
                    canister_id: None,
                    derivation_path: DerivationPath::new(derivation_path),
                    key_id: SchnorrKeyId {
                        algorithm: SchnorrAlgorithm::Ed25519,
                        name: key_name,
                    },
                },),
            )
            .await;

            res.unwrap().0.public_key
        }
        KeyType::Native(seed) => {
            let derivation_path = derivation_path_ed25519(&ic_cdk::api::id(), &derivation_path);
            native_public_key_ed25519(Seed::new(seed.try_into().unwrap()), derivation_path)
        }
    }
}

fn native_public_key_ed25519(
    seed: Seed,
    derivation_path: ic_crypto_ed25519::DerivationPath,
) -> Vec<u8> {
    let seed_32_bytes =
        <[u8; 32]>::try_from(&seed.as_bytes()[0..32]).expect("seed should be >= 32 bytes");
    let master_secret = ic_crypto_ed25519::PrivateKey::deserialize_raw_32(&seed_32_bytes);
    let (derived_secret, _) = master_secret.derive_subkey(&derivation_path);
    let public_key = derived_secret.public_key();
    public_key.serialize_raw().to_vec()
}

fn derivation_path_ed25519(
    canister_id: &Principal,
    derivation_path: &Vec<ByteBuf>,
) -> ic_crypto_ed25519::DerivationPath {
    let mut path = vec![];
    let derivation_index = ic_crypto_ed25519::DerivationIndex(canister_id.as_slice().to_vec());
    path.push(derivation_index);

    for index in derivation_path {
        path.push(ic_crypto_ed25519::DerivationIndex(index.to_vec()));
    }
    ic_crypto_ed25519::DerivationPath::new(path)
}

/// Signs a message with an ed25519 key.
pub async fn sign_with_eddsa(
    key_type: &KeyType,
    key_name: String,
    derivation_path: Vec<ByteBuf>,
    message: Vec<u8>,
) -> Vec<u8> {
    match key_type {
        KeyType::ChainKey => {
            let res: Result<(SignWithSchnorrReply,), _> = ic_cdk::api::call::call_with_payment(
                Principal::management_canister(),
                "sign_with_schnorr",
                (SignWithSchnorrArgs {
                    message,
                    derivation_path: DerivationPath::new(derivation_path),
                    key_id: SchnorrKeyId {
                        name: key_name,
                        algorithm: SchnorrAlgorithm::Ed25519,
                    },
                },),
                // https://internetcomputer.org/docs/current/references/t-sigs-how-it-works/#fees-for-the-t-schnorr-production-key
                // 26_153_846_153,
                EDDSA_SIGN_COST as u64,
            )
            .await;

            res.unwrap().0.signature
        }
        KeyType::Native(seed) => {
            let derivation_path = derivation_path_ed25519(&ic_cdk::api::id(), &derivation_path);
            sign_with_native_ed25519(
                &Seed::new(seed.as_slice().try_into().unwrap()),
                derivation_path,
                ByteBuf::from(message),
            )
        }
    }
}

fn sign_with_native_ed25519(
    seed: &Seed,
    derivation_path: ic_crypto_ed25519::DerivationPath,
    message: ByteBuf,
) -> Vec<u8> {
    let seed_32_bytes =
        <[u8; 32]>::try_from(&seed.as_bytes()[0..32]).expect("seed should be >= 32 bytes");
    let master_secret = ic_crypto_ed25519::PrivateKey::deserialize_raw_32(&seed_32_bytes);
    let (derived_secret, _chain_code) = master_secret.derive_subkey(&derivation_path);
    derived_secret.sign_message(&message).to_vec()
}

pub fn sha256(input: &[u8]) -> [u8; 32] {
    let mut hasher = sha2::Sha256::new();
    hasher.update(input);
    hasher.finalize().into()
}

pub fn hash_with_sha256(input: &str) -> String {
    let value = sha256(input.as_bytes());
    hex::encode(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    // use crate::types::Pubkey;

    #[test]
    fn test_sign_and_verify_native_schnorr_ed25519() {
        use ed25519_dalek::{Signature, Verifier, VerifyingKey};

        // Setup for signing
        let test_seed = [1u8; 64];
        // Example derivation path for signing
        let derivation_path = [vec![1u8; 4]]
            .iter()
            .map(|v| ByteBuf::from(v.clone()))
            .collect();
        let derivation_path = derivation_path_ed25519(&Principal::anonymous(), &derivation_path);

        let message = b"Test message";

        // Call the sign function
        let sign_reply = sign_with_native_ed25519(
            &Seed::new(test_seed),
            derivation_path.clone(),
            ByteBuf::from(message.to_vec()),
        );

        // Setup for verification
        let signature = Signature::from_slice(&sign_reply).expect("Invalid signature format");
        println!("signature: {:?}", signature);
        let public_key_reply = native_public_key_ed25519(Seed::new(test_seed), derivation_path);
        // let pk = Pubkey::try_from(public_key_reply.to_owned().as_slice())
        //     .map_err(|e| e.to_string())
        //     .unwrap();
        // println!("public_key: {:?}", pk.to_string());

        let raw_public_key = public_key_reply.as_slice();
        assert_eq!(raw_public_key.len(), 32);
        let mut public_key = [0u8; 32];
        public_key.copy_from_slice(raw_public_key);

        let public_key = VerifyingKey::from_bytes(&public_key).unwrap();

        // Verify the signature
        assert!(public_key.verify(message, &signature).is_ok());
    }
}
