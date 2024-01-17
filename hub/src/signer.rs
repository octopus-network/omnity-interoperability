use crate::auth::auth;
use crate::utils::Network;
use crate::Error;
use candid::CandidType;
use candid::Principal;
use ic_cdk::update;

use k256::ecdsa::{signature::Signer, Signature, SigningKey};
use log::debug;
use serde::{Deserialize, Serialize};
use sha2::Digest;
use std::convert::TryFrom;
use std::str::FromStr; // requires 'getrandom' feature
pub const CYCLES_NUMBER: u64 = 27_000_000_000;

// signing key for testnet
// const SIGNING_KEY: &str = "A77EE070FDCFD9E8670ED2AF6934263D43220B0926B4849479FA054156745389";
const SIGNING_KEY: [u8; 32] = [
    167, 126, 224, 112, 253, 207, 217, 232, 103, 14, 210, 175, 105, 52, 38, 61, 67, 34, 11, 9, 38,
    180, 132, 148, 121, 250, 5, 65, 86, 116, 83, 137,
];
// verifying key for testnet
// const VERIFYING_KEY: &str = "02B0BDD0434C4D3580383BC369F18E5E1CDD90923E0B7F65DF967DB857C56BDB2A";
const VERIFYING_KEY: [u8; 33] = [
    2, 176, 189, 208, 67, 76, 77, 53, 128, 56, 59, 195, 105, 241, 142, 94, 28, 221, 144, 146, 62,
    11, 127, 101, 223, 150, 125, 184, 87, 197, 107, 219, 42,
];
lazy_static::lazy_static! {

    static ref SIGING_KEY:SigningKey = SigningKey::from_bytes(&SIGNING_KEY).expect("Faile to init siging key");

}

#[derive(CandidType, Serialize, Debug)]
pub struct PublicKeyReply {
    pub public_key: Vec<u8>,
}

impl From<Vec<u8>> for PublicKeyReply {
    fn from(public_key: Vec<u8>) -> Self {
        Self { public_key }
    }
}

#[derive(CandidType, Serialize, Debug)]
pub struct SignatureReply {
    pub signature: Vec<u8>,
}

impl From<Vec<u8>> for SignatureReply {
    fn from(signature: Vec<u8>) -> Self {
        Self { signature }
    }
}

#[derive(CandidType, Serialize, Debug)]
pub struct SignatureVerificationReply {
    pub is_signature_valid: bool,
}

impl From<bool> for SignatureVerificationReply {
    fn from(is_signature_valid: bool) -> Self {
        Self { is_signature_valid }
    }
}

type CanisterId = Principal;

#[derive(CandidType, Serialize, Debug)]
struct ECDSAPublicKey {
    pub canister_id: Option<CanisterId>,
    pub derivation_path: Vec<Vec<u8>>,
    pub key_id: EcdsaKeyId,
}

#[derive(CandidType, Deserialize, Debug)]
struct ECDSAPublicKeyReply {
    pub public_key: Vec<u8>,
    pub chain_code: Vec<u8>,
}

#[derive(CandidType, Serialize, Debug)]
struct SignWithECDSA {
    pub message_hash: Vec<u8>,
    pub derivation_path: Vec<Vec<u8>>,
    pub key_id: EcdsaKeyId,
}

#[derive(CandidType, Deserialize, Debug)]
struct SignWithECDSAReply {
    pub signature: Vec<u8>,
}

#[derive(CandidType, Serialize, Debug, Clone)]
pub struct EcdsaKeyId {
    pub curve: EcdsaCurve,
    pub name: String,
}

#[derive(CandidType, Serialize, Debug, Clone)]
pub enum EcdsaCurve {
    #[serde(rename = "secp256k1")]
    Secp256k1,
}

#[update(guard = "auth")]
pub async fn get_pub_key(network: Network) -> Result<PublicKeyReply, Error> {
    match network {
        Network::Testnet => {
            // let verifying_key: Vec<u8> = hex::decode(VERIFYING_KEY).unwrap();
            let pk_reply = PublicKeyReply {
                public_key: VERIFYING_KEY.into(),
            };

            debug!("public_key(): {network:?}, PublicKeyReply: {:?}", pk_reply);
            Ok(pk_reply)
        }
        Network::Local | Network::Mainnet => {
            let request = ECDSAPublicKey {
                canister_id: None,
                derivation_path: vec![],
                key_id: network.key_id(),
            };

            let (res,): (ECDSAPublicKeyReply,) = ic_cdk::call(
                Principal::management_canister(),
                "ecdsa_public_key",
                (request,),
            )
            .await
            .map_err(|(_, e)| {
                Error::CustomError(format!(
                    "ecdsa_public_key failed Error:({e}) \n {}",
                    std::panic::Location::caller()
                ))
            })?;
            debug!(
                "public_key(): {network:?}, PublicKeyReply: {:?}, PublicKey: {:?}",
                res,
                hex::encode(res.public_key.clone())
            );
            Ok(res.public_key.into())
        }
    }
}

pub async fn sign(network: Network, message: Vec<u8>) -> Result<SignatureReply, Error> {
    match network {
        Network::Testnet => {
            let signature: Signature = SIGING_KEY.sign(&message);
            let sig_reply = SignatureReply {
                signature: signature.to_vec(),
            };
            debug!(
                "sign(): {network:?}, SignatureReply: {:?}, signature:{:?}",
                sig_reply,
                hex::encode(sig_reply.signature.clone())
            );
            Ok(sig_reply)
        }
        Network::Local | Network::Mainnet => {
            let request = SignWithECDSA {
                message_hash: sha256(&message).to_vec(),
                derivation_path: vec![],
                key_id: network.key_id(),
            };

            let (response,): (SignWithECDSAReply,) = ic_cdk::api::call::call_with_payment(
                Principal::management_canister(),
                "sign_with_ecdsa",
                (request,),
                CYCLES_NUMBER,
            )
            .await
            .map_err(|(_, e)| {
                Error::CustomError(format!(
                    "sign_with_ecdsa failed Error({e}) \n {}",
                    std::panic::Location::caller()
                ))
            })?;
            debug!(
                "sign(): {network:?}, SignatureReply: {:?}, signature:{:?}",
                response,
                hex::encode(response.signature.clone())
            );
            Ok(response.signature.into())
        }
    }
}

fn sha256(input: &[u8]) -> [u8; 32] {
    let mut hasher = sha2::Sha256::new();
    hasher.update(input);
    hasher.finalize().into()
}

pub enum EcdsaKeyIds {
    #[allow(unused)]
    TestKeyLocalDevelopment,
    #[allow(unused)]
    TestKey1,
    #[allow(unused)]
    ProductionKey1,
}

impl EcdsaKeyIds {
    pub fn to_key_id(&self) -> EcdsaKeyId {
        EcdsaKeyId {
            curve: EcdsaCurve::Secp256k1,
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
getrandom::register_custom_getrandom!(always_fail);
pub fn always_fail(_buf: &mut [u8]) -> Result<(), getrandom::Error> {
    Err(getrandom::Error::UNSUPPORTED)
}

#[cfg(test)]
mod tests {
    use super::*;
    use k256::ecdsa::{signature::Verifier, VerifyingKey};
    use k256::{
        ecdsa::{signature::Signer, Signature, SigningKey},
        elliptic_curve::rand_core,
    };
    use rand_core::OsRng; // requires 'getrandom' feature
    use tokio;

    #[test]
    fn test_sig() {
        // Signing
        let signing_key = SigningKey::random(&mut OsRng); // Serialize with `::to_bytes()`
        println!("signing_key: {:?}", signing_key.to_bytes());
        let message =
            b"ECDSA proves knowledge of a secret number in the context of a single message";
        println!("message: {:?}", message);
        // Note: The signature type must be annotated or otherwise inferable as
        // `Signer` has many impls of the `Signer` trait (for both regular and
        // recoverable signature types).
        let signature: Signature = signing_key.sign(message);
        println!("signature: {:?}", signature.to_bytes());
        // Verification

        let verifying_key = VerifyingKey::from(&signing_key); // Serialize with `::to_encoded_point()`
        println!(
            "verifying_key: {:?}",
            verifying_key.to_encoded_point(true).as_bytes()
        );

        assert!(verifying_key.verify(message, &signature).is_ok());
    }
    #[tokio::test]
    async fn test_sig2() {
        // Signing
        let network = Network::Testnet;
        let message = b"Hi,Octopus";
        println!("message: {:?}", message);

        let sig_reply = sign(network, message.to_vec()).await.unwrap();
        println!(
            "sig_reply: {:?}, len: {:?}",
            sig_reply,
            sig_reply.signature.len()
        );
        let signature = Signature::try_from(sig_reply.signature.as_slice()).unwrap();
        println!("signature: {:?}", signature);

        // Verification
        let pub_key = get_pub_key(network).await.unwrap();
        let verifying_key = VerifyingKey::from_sec1_bytes(&pub_key.public_key).unwrap();
        println!("verifying_key: {:?}", verifying_key);

        let message_hash = sha256(&message.to_vec()).to_vec();
        let result = verifying_key.verify(&message_hash, &signature).is_ok();
        println!("verifying_key.verify result: {:?}", result);
        assert!(result, "{}", true);
    }

    #[test]
    fn test_sig3() {
        let verifying_key_bytes: Vec<u8> = vec![
            3, 255, 119, 226, 76, 234, 148, 6, 55, 75, 54, 90, 185, 243, 173, 136, 161, 57, 197,
            52, 233, 95, 27, 41, 118, 126, 247, 216, 232, 95, 89, 32, 52,
        ];
        let verifying_key = VerifyingKey::from_sec1_bytes(&verifying_key_bytes).unwrap();

        let message = b"hi,Boern";
        println!("message: {:?}", message);

        let sig_bytes: Vec<u8> = vec![
            173, 237, 168, 39, 92, 189, 117, 220, 120, 143, 181, 193, 154, 108, 172, 31, 250, 87,
            142, 45, 63, 160, 197, 11, 253, 182, 72, 240, 33, 157, 248, 226, 72, 228, 164, 164, 19,
            144, 179, 52, 84, 52, 69, 87, 71, 142, 245, 101, 76, 234, 75, 21, 224, 241, 251, 217,
            137, 35, 85, 82, 237, 74, 36, 72,
        ];
        let signature = Signature::try_from(sig_bytes.as_slice()).unwrap();
        println!("signature: {:?}", signature);

        // Verification
        let result = verifying_key.verify(message, &signature).is_ok();
        println!("verifying_key.verify result: {:?}", result);
        assert!(result, "{}", true);
    }

    #[test]
    fn test_sig4() {
        let message = b"Hi,Boern";
        println!("message: {:?}", message);
        let message_hash = sha256(&message.to_vec()).to_vec();
        println!("message_hash: {:?}", hex::encode(message_hash));

        let siging_key_bytes =
            hex::decode("A77EE070FDCFD9E8670ED2AF6934263D43220B0926B4849479FA054156745389")
                .unwrap();
        println!("siging_key_bytes: {:?}", siging_key_bytes);

        let siging_key = SigningKey::from_bytes(&siging_key_bytes).unwrap();
        // let signature: Signature = siging_key.sign(&message_hash);
        let signature: Signature = siging_key.sign(message);
        println!("signature: {:?}", hex::encode(signature.to_bytes()));

        let verifying_key_bytes: Vec<u8> =
            hex::decode("02B0BDD0434C4D3580383BC369F18E5E1CDD90923E0B7F65DF967DB857C56BDB2A")
                .unwrap();
        println!("verifying_key_bytes: {:?}", verifying_key_bytes);
        let verifying_key = VerifyingKey::from_sec1_bytes(&verifying_key_bytes).unwrap();
        let verifying_key2 = VerifyingKey::from(&siging_key); // Serialize with `::to_encoded_point()`
        println!(
            "verifying_key2: {:?}",
            verifying_key2.to_encoded_point(true).as_bytes()
        );
        assert_eq!(
            verifying_key_bytes,
            verifying_key2.to_encoded_point(true).as_bytes()
        );
        // Verification
        // let result = verifying_key.verify(&message_hash, &signature).is_ok();
        let result = verifying_key.verify(message, &signature).is_ok();
        println!("verifying_key.verify result: {:?}", result);
        assert!(result, "{}", true);
    }

    #[test]
    fn test_sig5() {
        let message = b"Hi,Boern";
        println!("message: {:?}", message);
        let message_hash = sha256(&message.to_vec()).to_vec();
        println!("message_hash: {:?}", hex::encode(message_hash));

        println!("siging_key_bytes: {:?}", SIGNING_KEY);
        println!("verifying_key_bytes: {:?}", VERIFYING_KEY);

        let signature: Signature = SIGING_KEY.sign(message);
        println!("signature: {:?}", hex::encode(signature.to_bytes()));

        let verifying_key = VerifyingKey::from_sec1_bytes(&VERIFYING_KEY).unwrap();

        // Verification
        // let result = verifying_key.verify(&message_hash, &signature).is_ok();
        let result = verifying_key.verify(message, &signature).is_ok();
        println!("verifying_key.verify result: {:?}", result);
        assert!(result, "{}", true);
    }
}
