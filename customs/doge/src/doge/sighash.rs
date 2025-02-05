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
