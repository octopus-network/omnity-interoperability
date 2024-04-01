use candid::Principal;

use crate::PublicKeyReply;
use crate::SignatureReply;
use crate::{ECDSAPublicKey, ECDSAPublicKeyReply, SignWithECDSA, SignWithECDSAReply};
use k256::ecdsa::{signature::Signer, Signature, SigningKey};
use log::debug;

use sha2::Digest;

use crate::{Error, Network};

// requires 'getrandom' feature
// use std::str::FromStr;

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
                Error::EcdsaPublicKeyError(format!("{e} \n {}", std::panic::Location::caller()))
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
            // let signature: Signature = SIGING_KEY.sign(&message);
            let signature: Signature = SIGING_KEY.sign(&sha256(&message).to_vec());
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
                Error::SighWithEcdsaError(format!("{e} \n {}", std::panic::Location::caller()))
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
        let message = b"Hi,Omnity";
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
        let message = b"Hi,Omnity";
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
    fn test_sig4() {
        let message = b"Hi,Omnity";
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
