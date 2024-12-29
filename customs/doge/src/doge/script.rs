#![allow(unused)]
use bitcoin::base58;
use bitcoin::hashes::{hash160, Hash};
use std::str::FromStr;

use crate::errors::CustomsError;

use super::chainparams::ChainParams;
use super::opcodes::*;

pub use bitcoin::key::PubkeyHash;
pub use bitcoin::script::{Bytes, PushBytes, Script, ScriptBuf, ScriptHash};

// Dogecoin Script Types enum.
// Inferred from ScriptPubKey scripts by pattern-matching the code (script templates)
// https://github.com/dogecoin/dogecoin/blob/master/src/script/standard.cpp#L24
#[derive(Clone, PartialEq, Eq, Debug, Hash, Default)]
pub enum ScriptType {
    #[default]
    NonStandard,
    PubKey,
    PubKeyHash,
    ScriptHash,
    MultiSig,
    NullData,
    WitnessV0KeyHash,
    WitnessV0ScriptHash,
}

impl std::fmt::Display for ScriptType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScriptType::NonStandard => write!(f, "nonstandard"),
            ScriptType::PubKey => write!(f, "pubkey"),
            ScriptType::PubKeyHash => write!(f, "pubkeyhash"),
            ScriptType::ScriptHash => write!(f, "scripthash"),
            ScriptType::MultiSig => write!(f, "multisig"),
            ScriptType::NullData => write!(f, "nulldata"),
            ScriptType::WitnessV0KeyHash => write!(f, "witness_v0_keyhash"),
            ScriptType::WitnessV0ScriptHash => write!(f, "witness_v0_scripthash"),
        }
    }
}

pub const ECPRIV_KEY_LEN: usize = 32; // bytes.
pub const ECPUB_KEY_COMPRESSED_LEN: usize = 33; // bytes: [x02/x03][32-X] 2=even 3=odd
pub const ECPUB_KEY_UNCOMPRESSED_LEN: usize = 65; // bytes: [x04][32-X][32-Y]

#[derive(Clone, PartialEq, Eq, Debug, Hash, Default)]
pub struct Address(pub [u8; 21]); // Dogecoin address (base-58 Public Key Hash aka PKH)
impl Address {
    pub fn is_p2pkh(&self, chain: &ChainParams) -> bool {
        self.0[0] == chain.p2pkh_address_prefix
    }

    pub fn is_p2sh(&self, chain: &ChainParams) -> bool {
        self.0[0] == chain.p2sh_address_prefix
    }

    pub fn is_valid(&self, chain: &ChainParams) -> bool {
        self.0[0] == chain.p2pkh_address_prefix || self.0[0] == chain.p2sh_address_prefix
    }

    pub fn to_script(&self, chain: &ChainParams) -> ScriptBuf {
        if self.is_p2pkh(chain) {
            ScriptBuf::new_p2pkh(&PubkeyHash::from_slice(&self.0[1..]).unwrap())
        } else if self.is_p2sh(chain) {
            ScriptBuf::new_p2sh(&ScriptHash::from_slice(&self.0[1..]).unwrap())
        } else {
            ScriptBuf::default()
        }
    }
}

impl std::fmt::Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", base58::encode_check(&self.0))
    }
}

impl FromStr for Address {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match base58::decode_check(s) {
            Ok(key) => {
                let mut addr = [0u8; 21];
                if key.len() != 21 {
                    return Err("invalid address".to_string());
                }

                addr.copy_from_slice(&key);
                Ok(Address(addr))
            }
            Err(_) => Err("invalid address".to_string()),
        }
    }
}

pub fn hash160_to_address(hash: &[u8], prefix: u8) -> Address {
    assert!(
        hash.len() == 20,
        "hash160_to_address: wrong RIPEMD-160 length"
    );
    let mut addr = Address::default();
    addr.0[0] = prefix;
    addr.0[1..21].copy_from_slice(hash);
    addr
}

pub fn p2pkh_address(pubkey: &[u8], chain: &ChainParams) -> Result<Address, CustomsError> {
    if !((pubkey.len() == ECPUB_KEY_UNCOMPRESSED_LEN && pubkey[0] == 0x04)
        || (pubkey.len() == ECPUB_KEY_COMPRESSED_LEN && (pubkey[0] == 0x02 || pubkey[0] == 0x03)))
    {
        return Err(CustomsError::CustomError("p2pkh_address: bad pubkey length".to_string()));
    }
    let payload = hash160::Hash::hash(pubkey);
    Ok(hash160_to_address(
        payload.as_ref(),
        chain.p2pkh_address_prefix,
    ))
}

pub fn p2sh_address(script: &[u8], chain: &ChainParams) -> Result<Address, String> {
    if script.is_empty() {
        return Err("p2sh_address: bad script length".to_string());
    }

    let payload = hash160::Hash::hash(script);
    Ok(hash160_to_address(
        payload.as_ref(),
        chain.p2sh_address_prefix,
    ))
}

pub fn classify_script(script: &[u8], chain: &ChainParams) -> (ScriptType, Option<Address>) {
    let l = script.len();
    // P2PKH: OP_DUP OP_HASH160 <pubKeyHash:20> OP_EQUALVERIFY OP_CHECKSIG (25)
    if l == 25
        && script[0] == OP_DUP
        && script[1] == OP_HASH160
        && script[2] == 20
        && script[23] == OP_EQUALVERIFY
        && script[24] == OP_CHECKSIG
    {
        let addr = hash160_to_address(&script[3..23], chain.p2pkh_address_prefix);
        return (ScriptType::PubKeyHash, Some(addr));
    }

    // P2PK: <compressedPubKey:33> OP_CHECKSIG
    if l == 35 && script[0] == 33 && script[34] == OP_CHECKSIG {
        // no Base58 Address for P2PK.
        return (ScriptType::PubKey, None);
    }

    // P2PK: <uncompressedPubKey:65> OP_CHECKSIG
    if l == 67 && script[0] == 65 && script[66] == OP_CHECKSIG {
        // no Base58 Address for P2PK.
        return (ScriptType::PubKey, None);
    }

    // P2SH: OP_HASH160 0x14 <hash> OP_EQUAL
    if l == 23 && script[0] == OP_HASH160 && script[1] == 20 && script[22] == OP_EQUAL {
        let addr = hash160_to_address(&script[2..22], chain.p2sh_address_prefix);
        return (ScriptType::ScriptHash, Some(addr));
    }

    // OP_m <pubkey*n> OP_n OP_CHECKMULTISIG
    if l >= 3 + 34
        && script[l - 1] == OP_CHECKMULTISIG
        && is_op_n1(script[l - 2])
        && is_op_n1(script[0])
    {
        let mut num_keys = script[l - 2] - (OP_1 - 1);
        let mut ofs = 1;
        let end_keys = l - 2;
        while ofs < end_keys && num_keys > 0 {
            if script[ofs] == 65 && ofs + 66 <= end_keys {
                // no Base58 Address for PubKey.
                ofs += 66
            } else if script[ofs] == 33 && ofs + 34 <= end_keys {
                // no Base58 Address for PubKey.
                ofs += 34
            } else {
                break;
            }
            num_keys -= 1
        }

        if ofs == end_keys && num_keys == 0 {
            return (ScriptType::MultiSig, None);
        }

        return (ScriptType::NonStandard, None);
    }

    // OP_RETURN
    if l > 0 && script[0] == OP_RETURN {
        return (ScriptType::NullData, None);
    }

    (ScriptType::NonStandard, None)
}

fn is_op_n1(op: u8) -> bool {
    (OP_1..=OP_16).contains(&op)
}
