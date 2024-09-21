use std::str::FromStr;
use bitcoin::bip32::ChainCode;
use bitcoin::key::Secp256k1;
use bitcoin::{Address, PublicKey};
use bitcoin::secp256k1::{All, Error, Message};
use bitcoin::secp256k1::ecdsa::Signature;
use ic_ic00_types::DerivationPath;
use log::{error, info};
use crate::custom_to_bitcoin::CustomToBitcoinError;
use crate::management;
use crate::ord::result::{OrdError, OrdResult};
use crate::state::read_state;


#[derive(Clone)]
pub struct MixSigner {
    pub key_id: String,
    pub derive_path: DerivationPath,
    pub secp: Secp256k1<All>,
    pub pubkey: PublicKey,
    pub signer_addr: Address
}

impl MixSigner {

    pub fn chain_code() -> ChainCode {
        ChainCode::from([0; 32])
    }

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

    pub fn ecdsa_public_key(&self) -> bitcoin::PublicKey {
        self.pubkey
    }

    pub async fn sign_with_ecdsa(&self, message: Message) -> OrdResult<Signature> {
        let key_name = read_state(|s|s.ecdsa_key_name.clone());
        let sighash = message.as_ref().clone();
        let sec1_signature =
            management::sign_with_ecdsa(key_name, DerivationPath::new(vec![]), sighash)
                .await.map_err(|e| OrdError::UnexpectedSignature)?;
        info!("len: {} content: {:?}", sec1_signature.len(), sec1_signature.clone() );
       Signature::from_compact(sec1_signature.as_slice()).map_err(|e|OrdError::Signature(e))
    }

}
