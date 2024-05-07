use secp256k1::{
    ecdsa::{RecoverableSignature, RecoveryId},
    Message, PublicKey,
};

use super::Error;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct EthereumSignature {
    pub r: Vec<u8>,
    pub s: Vec<u8>,
    pub v: u64,
}

impl EthereumSignature {
    pub(crate) fn try_from_ecdsa(
        signature: &[u8],
        prehash: &[u8],
        chain_id: u64,
        pubkey: &[u8],
    ) -> Result<Self, Error> {
        let mut r = signature[..32].to_vec();
        let mut s = signature[32..].to_vec();
        while r[0] == 0 {
            r.remove(0);
        }
        while s[0] == 0 {
            s.remove(0);
        }
        let v = Self::try_derive_recid(signature, prehash, chain_id, pubkey)?;
        Ok(Self { r, s, v })
    }

    fn try_derive_recid(
        signature: &[u8],
        prehash: &[u8],
        chain_id: u64,
        pubkey: &[u8],
    ) -> Result<u64, Error> {
        let pubkey = PublicKey::from_slice(pubkey)
            .map_err(|_| Error::ChainKeyError("invalid public key".to_string()))?;
        let digest = Message::from_digest_slice(prehash)
            .map_err(|_| Error::ChainKeyError("invalid signature".to_string()))?;
        for r in 0..4 {
            let rec_id = RecoveryId::from_i32(r).expect("less than 4;qed");
            let sig = RecoverableSignature::from_compact(signature, rec_id)
                .map_err(|_| Error::ChainKeyError("invalid signature length".to_string()))?;
            if let Ok(pk) = sig.recover(&digest) {
                if pk == pubkey {
                    return Ok(r as u64 + chain_id * 2 + 35);
                }
            }
        }
        Err(Error::ChainKeyError("invalid signature".to_string()))
    }
}
