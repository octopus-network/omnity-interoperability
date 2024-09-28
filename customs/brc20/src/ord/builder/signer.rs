use crate::management;
use crate::ord::result::{OrdError, OrdResult};
use crate::state::read_state;
use bitcoin::key::Secp256k1;
use bitcoin::secp256k1::ecdsa::Signature;
use bitcoin::secp256k1::{All, Message};
use bitcoin::{Address, PublicKey};
use ic_ic00_types::DerivationPath;
use log::{ info};

#[derive(Clone)]
pub struct MixSigner {
    pub key_id: String,
    pub derive_path: DerivationPath,
    pub secp: Secp256k1<All>,
    pub pubkey: PublicKey,
    pub signer_addr: Address,
}

impl MixSigner {
    pub fn new(key_id: String, public_key: PublicKey, addr: Address) -> Self {
        // Network is only used for encoding and decoding the private key and is not important for
        // signing. So we can use any value here.
        Self {
            key_id,
            derive_path: DerivationPath::new(vec![]),
            secp: Secp256k1::new(),
            pubkey: public_key,
            signer_addr: addr,
        }
    }

    pub async fn sign_with_ecdsa(&self, message: Message) -> OrdResult<Signature> {
        let key_name = read_state(|s| s.ecdsa_key_name.clone());
        let sighash = *message.as_ref();
        let sec1_signature =
            management::sign_with_ecdsa(key_name, DerivationPath::new(vec![]), sighash)
                .await
                .map_err(|_| OrdError::UnexpectedSignature)?;
        info!(
            "len: {} content: {:?}",
            sec1_signature.len(),
            sec1_signature.clone()
        );
        Signature::from_compact(sec1_signature.as_slice()).map_err(|e| OrdError::Signature(e))
    }
}
