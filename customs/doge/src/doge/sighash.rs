#![allow(unused)]

use bitcoin::consensus::Encodable;
use bitcoin::hashes::{hash_newtype, sha256d, Hash};
use bitcoin_io::Write;
use std::borrow::{Borrow, BorrowMut};
use std::ops::Deref;

pub use bitcoin::ecdsa::Signature as SighashSignature;
pub use bitcoin::secp256k1::{Message, PublicKey};
pub use bitcoin::EcdsaSighashType;

use crate::doge::script::ScriptBuf;

use super::transaction::*;
// use crate::{err_string, transaction::*};

hash_newtype! {
    /// Hash of a transaction according to the legacy signature algorithm.
    #[hash_newtype(forward)]
    pub struct Sighash(sha256d::Hash);
}

impl Deref for Sighash {
    type Target = [u8; 32];

    fn deref(&self) -> &[u8; 32] {
        self.0.as_byte_array()
    }
}

impl From<Sighash> for Message {
    fn from(hash: Sighash) -> Self {
        Message::from_digest(hash.to_byte_array())
    }
}

/// Used for signature hash for invalid use of SIGHASH_SINGLE.
pub(crate) const UINT256_ONE: [u8; 32] = [
    1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

/// Efficiently calculates signature hash message for legacy, segwit and taproot inputs.
#[derive(Debug)]
pub struct SighashCache<T: Borrow<Transaction>> {
    tx: T,
}

impl<R: Borrow<Transaction>> SighashCache<R> {
    /// Constructs a new `SighashCache` from an unsigned transaction.
    pub fn new(tx: R) -> Self {
        SighashCache { tx }
    }

    /// Returns the reference to the cached transaction.
    pub fn transaction(&self) -> &Transaction {
        self.tx.borrow()
    }

    /// Destroys the cache and recovers the stored transaction.
    pub fn into_transaction(self) -> R {
        self.tx
    }

    pub fn encode_signing_data_to<W: Write + ?Sized>(
        &self,
        writer: &mut W,
        input_index: usize,
        script_pubkey: &ScriptBuf,
        sighash_type: EcdsaSighashType,
    ) -> Result<bool, String> {
        // Validate input_index.
        if input_index >= self.tx.borrow().input.len() {
            return Err(format!("input index {} out of range", input_index));
        }

        if is_invalid_use_of_sighash_single(
            sighash_type,
            input_index,
            self.tx.borrow().output.len(),
        ) {
            // We cannot correctly handle the SIGHASH_SINGLE bug here because usage of this function
            // will result in the data written to the writer being hashed, however the correct
            // handling of the SIGHASH_SINGLE bug is to return the 'one array' - either implement
            // this behaviour manually or use `signature_hash()`.
            return Ok(false);
        }

        let (sighash, anyone_can_pay) = split_anyonecanpay_flag(sighash_type);
        let stx = self.tx.borrow();
        // Build tx to sign
        let mut tx = Transaction {
            version: stx.version,
            lock_time: stx.lock_time,
            input: vec![],
            output: vec![],
        };
        // Add all inputs necessary..
        if anyone_can_pay {
            tx.input = vec![TxIn {
                prevout: stx.input[input_index].prevout,
                script: script_pubkey.clone(),
                sequence: stx.input[input_index].sequence,
                witness: Witness::default(),
            }];
        } else {
            tx.input = Vec::with_capacity(stx.input.len());
            for (n, input) in stx.input.iter().enumerate() {
                tx.input.push(TxIn {
                    prevout: input.prevout,
                    script: if n == input_index {
                        script_pubkey.clone()
                    } else {
                        ScriptBuf::new()
                    },
                    sequence: if n != input_index
                        && (sighash == EcdsaSighashType::Single
                            || sighash == EcdsaSighashType::None)
                    {
                        0
                    } else {
                        input.sequence
                    },
                    witness: Witness::default(),
                });
            }
        }
        // ..then all outputs
        tx.output = match sighash {
            EcdsaSighashType::All => stx.output.clone(),
            EcdsaSighashType::Single => {
                let output_iter = stx
                    .output
                    .iter()
                    .take(input_index + 1) // sign all outputs up to and including this one, but erase
                    .enumerate() // all of them except for this one
                    .map(|(n, out)| {
                        if n == input_index {
                            out.clone()
                        } else {
                            TxOut::default()
                        }
                    });
                output_iter.collect()
            }
            EcdsaSighashType::None => vec![],
            _ => unreachable!(),
        };
        // hash the result
        tx.consensus_encode(writer).map_err(|err| err.to_string())?;
        sighash_type
            .to_u32()
            .to_le_bytes()
            .consensus_encode(writer)
            .map_err(|err| err.to_string())?;
        Ok(true)
    }

    /// Computes a signature hash for a given input index with a given sighash flag.
    pub fn signature_hash(
        &self,
        input_index: usize,
        script_pubkey: &ScriptBuf,
        sighash_type: EcdsaSighashType,
    ) -> Result<Sighash, String> {
        let mut engine = Sighash::engine();
        match self.encode_signing_data_to(&mut engine, input_index, script_pubkey, sighash_type) {
            Ok(true) => Ok(Sighash::from_engine(engine)),
            Ok(false) => Ok(Sighash::from_byte_array(UINT256_ONE)),
            Err(e) => Err(e),
        }
    }
}

impl<R: BorrowMut<Transaction>> SighashCache<R> {
    /// Allows modification of script.
    ///
    /// This method allows doing exactly that if the transaction is owned by the `SighashCache` or
    /// borrowed mutably.
    pub fn set_input_script(
        &mut self,
        input_index: usize,
        signature: &SighashSignature,
        pubkey: &PublicKey,
    ) -> Result<(), String> {
        self.tx
            .borrow_mut()
            .input
            .get_mut(input_index)
            .map(|i| {
                let mut buf = ScriptBuf::new();
                buf.push_slice(signature.serialize());
                buf.push_slice(pubkey.serialize());
                i.script = buf;
            })
            .ok_or("input index out of range".to_string())
    }
}

// // https://en.bitcoin.it/wiki/Wallet_import_format
// pub fn decode_secretkey_wif(sk: &str) -> Result<SecretKey, String> {
//     match base58::decode_check(sk) {
//         Ok(data) => {
//             let chain = chain_from_wif(sk);
//             if data[0] != chain.pkey_prefix {
//                 return Err("wrong key prefix".to_string());
//             }
//             let key = SecretKey::from_slice(&data[1..33]).map_err(err_string)?;
//             Ok(key)
//         }
//         Err(_) => Err("invalid base58 secret key".to_string()),
//     }
// }

fn split_anyonecanpay_flag(st: EcdsaSighashType) -> (EcdsaSighashType, bool) {
    use EcdsaSighashType::*;
    match st {
        All => (All, false),
        None => (None, false),
        Single => (Single, false),
        AllPlusAnyoneCanPay => (All, true),
        NonePlusAnyoneCanPay => (None, true),
        SinglePlusAnyoneCanPay => (Single, true),
    }
}

fn is_invalid_use_of_sighash_single(
    ty: EcdsaSighashType,
    input_index: usize,
    outputs_len: usize,
) -> bool {
    ty == EcdsaSighashType::Single && input_index >= outputs_len
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use bitcoin::script::ScriptBuf;
//     use bitcoin::PubkeyHash;
//     use hex::test_hex_unwrap as hex;

//     #[test]
//     fn test_sighash() {
//         // https://dogechain.info/tx/f875c9959d2013caedf7b3acce278d1bed7be0e7d45aa266db1c262e6743cb99
//         // https://sochain.com/tx/DOGE/f875c9959d2013caedf7b3acce278d1bed7be0e7d45aa266db1c262e6743cb99
//         let tx_data = hex!("0100000003607ee3f9c2c8eb4db9297d38c3ed1493fbdd257283eabf5a89999ebd5676f13f000000006a473044022045228179cc4fed581b5d9e6411402fd34c40866397094f0d97843ee257b30abc0220310c33c4452d47113c51257883c780d40928ba6b9e2bf14c234f1bd5991607e6012102ca3e8f77965b91cbe15203817cce0170bc066d5d7a9acb2070e9b3e4b2077bb8feffffff538cde51d37d84ec955410476c18c7dddaf5531a7aa07e60b75dd106ca50b447000000006b483045022100b58824fe1e036320a5cc267fe65fec100988df2a0d0ea594b945088c4945765b0220091e3dead2cec11ce5c4139f126452437b8ba8d957ae7dcee0fba60d854d46df0121026079f574275f68fd88fbd01c0af3364524b3071419dc24bddbf60003be974288feffffff7e24f4f31a3a6eaf109c3368352975c1e0ef10d175905e991553e4040b049e8c010000006b483045022100f3d369af38220916afb0c805581cd14b08ff0ee83b093c71fdeccb9b8fea197f022053e648c6a0ed5e1b42042d0e4654be8e039082c8106f147726e420cf1c986d03012103a0e805a231331c414b0423adef1ddedb23cc801acfd90ba7bd95907048a3b908feffffff02a4700106000000001976a91467f5672ce989470f4dcba16c1e930f80c03c887488acf23b03c3360000001976a91489248eee4e9d99729ebebbca00efb75ceb1ed01888acf9ed4900");

//         let tx = Transaction::try_from(&tx_data[..]).unwrap();
//         assert_eq!(
//             tx.compute_txid().to_string(),
//             "f875c9959d2013caedf7b3acce278d1bed7be0e7d45aa266db1c262e6743cb99"
//         );
//         assert_eq!(tx.input.len(), 3);
//         // https://sochain.com/tx/DOGE/3ff17656bd9e99895abfea837225ddfb9314edc3387d29b94debc8c2f9e37e60
//         // output 0: value: 27188437677
//         let script_pubkey = ScriptBuf::new_p2pkh(
//             &PubkeyHash::from_slice(&hex!("e1209b366bebd881861c9aaeed0b780f531c6435")).unwrap(),
//         );
//         println!("script: {:?}", tx.input[0].script);
//         let mut ins = tx.input[0].script.instructions();
//         let sig = ins.next().unwrap().unwrap();
//         let sig = sig.push_bytes().unwrap();
//         let sig = SighashSignature::from_slice(sig.as_bytes()).unwrap();

//         let pubkey = ins.next().unwrap().unwrap();
//         let pubkey = pubkey.push_bytes().unwrap();
//         let pubkey = PublicKey::from_slice(pubkey.as_bytes()).unwrap();

//         let sighasher = SighashCache::new(&tx);
//         let sighash = sighasher
//             .signature_hash(0, &script_pubkey, EcdsaSighashType::All)
//             .unwrap();
//         let msg = Message::from_digest(sighash.to_byte_array());
//         let secp = Secp256k1::verification_only();
//         assert!(secp.verify_ecdsa(&msg, &sig.signature, &pubkey).is_ok());

//         let mut some_tx = tx.clone();
//         some_tx.input[0].script = ScriptBuf::new();

//         let mut sighasher = SighashCache::new(&mut some_tx);
//         assert!(sighasher.set_input_script(0, &sig, &pubkey).is_ok());
//         assert!(sighasher.set_input_script(3, &sig, &pubkey).is_err());
//         assert_eq!(some_tx.to_bytes(), tx_data);
//     }
// }
