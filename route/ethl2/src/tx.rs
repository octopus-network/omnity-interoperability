use super::Error;
use rlp::{Encodable, RlpStream};
use secp256k1::{
    ecdsa::{RecoverableSignature, RecoveryId},
    Message, PublicKey,
};
use tiny_keccak::{Hasher, Keccak};

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

const EIP_1559_TYPE: u8 = 0x02;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct EIP1559Transaction {
    /// Chain ID of EIP155
    pub chain: u64,
    /// Nonce
    pub nonce: u128,
    /// Gas price
    pub max_priority_fee_per_gas: u128,
    pub max_fee_per_gas: u128,
    /// Gas limit
    pub gas: u128,
    /// Recipient
    pub to: Option<[u8; 20]>,
    /// Transfered value
    pub value: u128,
    /// Input data
    pub data: Vec<u8>,
    /// List of addresses and storage keys the transaction plans to access
    pub access_list: AccessList,
}

impl EIP1559Transaction {
    fn compose(&self) -> Vec<u8> {
        let mut rlp_stream = RlpStream::new();
        rlp_stream.begin_unbounded_list();
        rlp_stream.append(&self.chain);
        rlp_stream.append(&self.nonce);
        rlp_stream.append(&self.max_priority_fee_per_gas);
        rlp_stream.append(&self.max_fee_per_gas);
        rlp_stream.append(&self.gas);
        rlp_stream.append(&self.to.map(|x| x.to_vec()).unwrap_or_default());
        rlp_stream.append(&self.value);
        rlp_stream.append(&self.data);
        rlp_stream.append(&self.access_list);
        rlp_stream.finalize_unbounded_list();
        let mut rlp_bytes = vec![EIP_1559_TYPE];
        rlp_bytes.extend_from_slice(&rlp_stream.out());
        rlp_bytes
    }

    pub(crate) fn digest(&self) -> [u8; 32] {
        let mut hasher = Keccak::v256();
        hasher.update(self.compose().as_slice());
        let mut digest: [u8; 32] = Default::default();
        hasher.finalize(&mut digest);
        digest
    }

    pub(crate) fn finalize(&self, sig: EthereumSignature) -> Vec<u8> {
        let mut data = self.compose();
        let EthereumSignature { r, s, v } = sig;
        let mut rlp_stream = RlpStream::new();
        rlp_stream.append(&v);
        rlp_stream.append(&r);
        rlp_stream.append(&s);
        rlp_stream.finalize_unbounded_list();
        data.extend_from_slice(&rlp_stream.out());
        data
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Access {
    pub address: [u8; 20],
    pub storage_keys: Vec<[u8; 32]>,
}

/// EIP-2930
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct AccessList(Vec<Access>);

impl Encodable for AccessList {
    fn rlp_append(&self, rlp_stream: &mut RlpStream) {
        rlp_stream.begin_unbounded_list();
        for access in self.0.iter() {
            let address_bytes: Vec<u8> = access.address.to_vec();
            rlp_stream.begin_unbounded_list();
            rlp_stream.append(&address_bytes);
            {
                rlp_stream.begin_unbounded_list();
                for storage_key in access.storage_keys.iter() {
                    let storage_key_bytes: Vec<u8> = storage_key.to_vec();
                    rlp_stream.append(&storage_key_bytes);
                }
                rlp_stream.finalize_unbounded_list();
            }
            rlp_stream.finalize_unbounded_list();
        }
        rlp_stream.finalize_unbounded_list();
    }
}
