use crate::management;
use crate::ord::result::{OrdError, OrdResult};
use crate::state::read_state;
use bitcoin::secp256k1::ecdsa::Signature;
use bitcoin::secp256k1::{Message};
use bitcoin::{Address, PublicKey};
use ic_canister_log::log;
use ic_ic00_types::DerivationPath;
use omnity_types::ic_log::ERROR;

#[derive(Clone)]
pub struct MixSigner {
    pub pubkey: PublicKey,
    pub signer_addr: Address,
}

impl MixSigner {
    pub fn new(public_key: PublicKey, addr: Address) -> Self {
        Self {
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
                .map_err(|e| {
                    log!(ERROR, "call management signature error: {:?}", e);
                    OrdError::UnexpectedSignature
                })?;
        Signature::from_compact(sec1_signature.as_slice()).map_err(OrdError::Signature)
    }
}
