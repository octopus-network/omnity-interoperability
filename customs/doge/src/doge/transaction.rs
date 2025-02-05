// SPDX-License-Identifier: CC0-1.0

//! Dogecoin transactions.

use bitcoin::consensus::{encode, Decodable, Encodable};
use bitcoin::hashes::{hash_newtype, sha256d, Hash};
use bitcoin::{ScriptBuf, VarInt};
use bitcoin_io::{BufRead, Error, Write};
use core::cmp;
use serde::{Deserialize, Serialize};
use std::ops::Deref;

use crate::errors::CustomsError;
use crate::types::deserialize_hex;

use super::chainparams::DOGE_MAIN_NET_CHAIN;
use super::script::classify_script;

pub fn consensus_encode_vec<T, W>(vv: &[T], w: &mut W) -> Result<usize, Error>
where
    T: Encodable,
    W: Write + ?Sized,
{
    let mut len = 0;
    VarInt::from(vv.len()).consensus_encode(w)?;
    for v in vv.iter() {
        len += v.consensus_encode(w)?;
    }
    Ok(len)
}

pub fn consensus_decode_from_vec<T, R>(r: &mut R) -> Result<Vec<T>, encode::Error>
where
    T: Decodable,
    R: BufRead + ?Sized,
{
    let cap: VarInt = Decodable::consensus_decode(r)?;
    let cap = cap.0 as usize;
    let mut vv = Vec::with_capacity(cap);
    for _ in 0..cap {
        vv.push(Decodable::consensus_decode_from_finite_reader(r)?);
    }
    Ok(vv)
}

pub fn err_string(err: impl std::fmt::Display) -> String {
    err.to_string()
}

hash_newtype! {
    pub struct Txid(sha256d::Hash);
}

impl Default for Txid {
    fn default() -> Txid {
        Txid(sha256d::Hash::all_zeros())
    }
}

impl Deref for Txid {
    type Target = [u8; 32];

    fn deref(&self) -> &[u8; 32] {
        self.0.as_byte_array()
    }
}

impl Encodable for Txid {
    fn consensus_encode<W: Write + ?Sized>(&self, w: &mut W) -> Result<usize, Error> {
        let mut len = 0;
        len += self.0.consensus_encode(w)?;
        Ok(len)
    }
}

impl Decodable for Txid {
    #[inline]
    fn consensus_decode_from_finite_reader<R: BufRead + ?Sized>(
        r: &mut R,
    ) -> Result<Self, encode::Error> {
        let hash: sha256d::Hash = Decodable::consensus_decode_from_finite_reader(r)?;
        Ok(Txid(hash))
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct OutPoint {
    /// The referenced transaction's txid.
    pub txid: Txid,
    /// The index of the referenced output in its transaction's vout.
    pub vout: u32,
}

impl OutPoint {
    pub const SIZE: usize = 36;
    pub fn is_null(&self) -> bool {
        self.vout == u32::MAX && self.txid == Txid::default()
    }
}

impl Default for OutPoint {
    fn default() -> OutPoint {
        OutPoint {
            txid: Txid::default(),
            vout: u32::MAX,
        }
    }
}

impl Encodable for OutPoint {
    fn consensus_encode<W: Write + ?Sized>(&self, w: &mut W) -> Result<usize, Error> {
        let mut len = 0;
        len += self.txid.consensus_encode(w)?;
        len += self.vout.consensus_encode(w)?;
        Ok(len)
    }
}

impl Decodable for OutPoint {
    #[inline]
    fn consensus_decode_from_finite_reader<R: BufRead + ?Sized>(
        r: &mut R,
    ) -> Result<Self, encode::Error> {
        Ok(OutPoint {
            txid: Decodable::consensus_decode_from_finite_reader(r)?,
            vout: Decodable::consensus_decode_from_finite_reader(r)?,
        })
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash, Default, Serialize, Deserialize)]
pub struct Witness {
    pub stack: Vec<u8>,
}

impl Encodable for Witness {
    fn consensus_encode<W: Write + ?Sized>(&self, w: &mut W) -> Result<usize, Error> {
        let mut len = 0;
        len += self.stack.consensus_encode(w)?;
        Ok(len)
    }
}

impl Decodable for Witness {
    #[inline]
    fn consensus_decode_from_finite_reader<R: BufRead + ?Sized>(
        r: &mut R,
    ) -> Result<Self, encode::Error> {
        Ok(Witness {
            stack: Decodable::consensus_decode_from_finite_reader(r)?,
        })
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct TxIn {
    pub prevout: OutPoint,
    pub script: ScriptBuf,
    pub sequence: u32,
    pub witness: Witness, // Only Serialize & Deserialize through Transaction
}

impl TxIn {
    pub const SEQUENCE_LOCKTIME_DISABLE_FLAG: u32 = 1 << 31;
    pub const SEQUENCE_LOCKTIME_TYPE_FLAG: u32 = 1 << 22;
    pub const SEQUENCE_LOCKTIME_MASK: u32 = 0x0000ffff;
    pub const SEQUENCE_LOCKTIME_GRANULARITY: u32 = 9;

    pub fn with_outpoint(prevout: OutPoint) -> TxIn {
        TxIn {
            prevout,
            script: ScriptBuf::new(),
            sequence: u32::MAX,
            witness: Witness::default(),
        }
    }

    /// Returns the base size of this input.
    ///
    /// Base size excludes the witness data (see [`Self::total_size`]).
    pub fn size(&self) -> usize {
        let mut size = OutPoint::SIZE;

        // 106 is the common size of a scriptSig
        // if script is empty, we set the size to 106 to compute the fee size
        let len = self.script.len();
        size += VarInt::from(len).size();
        size += len;

        size + 4 // Sequence::SIZE
    }

    /// Returns the estimate number of bytes that this input contributes to a transaction.
    pub fn estimate_size(&self) -> usize {
        let mut size = OutPoint::SIZE;

        // 106 is the common size of a scriptSig
        // if script is empty, we set the size to 106 to compute the fee size
        let len = self.script.len().max(106);
        size += VarInt::from(len).size();
        size += len;
        size += 4; // Sequence::SIZE
        size + self.witness.stack.len()
    }
}

impl Default for TxIn {
    fn default() -> TxIn {
        TxIn {
            prevout: OutPoint::default(),
            script: ScriptBuf::new(),
            sequence: u32::MAX,
            witness: Witness::default(),
        }
    }
}

impl TryFrom<&[u8]> for TxIn {
    type Error = String;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        let mut rd = data;
        Self::consensus_decode_from_finite_reader(&mut rd).map_err(err_string)
    }
}

impl Encodable for TxIn {
    fn consensus_encode<W: Write + ?Sized>(&self, w: &mut W) -> Result<usize, Error> {
        let mut len = 0;
        len += self.prevout.consensus_encode(w)?;
        len += self.script.consensus_encode(w)?;
        len += self.sequence.consensus_encode(w)?;
        Ok(len)
    }
}

impl Decodable for TxIn {
    #[inline]
    fn consensus_decode_from_finite_reader<R: BufRead + ?Sized>(
        r: &mut R,
    ) -> Result<Self, encode::Error> {
        Ok(TxIn {
            prevout: Decodable::consensus_decode_from_finite_reader(r)?,
            script: Decodable::consensus_decode_from_finite_reader(r)?,
            sequence: Decodable::consensus_decode_from_finite_reader(r)?,
            witness: Witness::default(),
        })
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct TxOut {
    pub value: u64,
    pub script_pubkey: ScriptBuf,
}

impl TxOut {
    /// Returns the total number of bytes that this output contributes to a transaction.
    pub fn size(&self) -> usize {
        let len = self.script_pubkey.len();
        VarInt::from(len).size() + len + 8 // value size
    }

    pub fn estimate_size(&self) -> usize {
        self.size()
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        self.consensus_encode(&mut buf).unwrap();
        buf
    }

    pub fn get_mainnet_address(&self) -> Option<String> {
        let (_, addr_opt) = classify_script(&self.script_pubkey.as_bytes(), &DOGE_MAIN_NET_CHAIN);
        addr_opt.map(|addr| addr.to_string())
    }
}

impl Default for TxOut {
    fn default() -> TxOut {
        TxOut {
            value: u64::MAX,
            script_pubkey: ScriptBuf::new(),
        }
    }
}

impl TryFrom<&[u8]> for TxOut {
    type Error = String;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        let mut rd = data;
        Self::consensus_decode_from_finite_reader(&mut rd).map_err(err_string)
    }
}

impl Encodable for TxOut {
    fn consensus_encode<W: Write + ?Sized>(&self, w: &mut W) -> Result<usize, Error> {
        let mut len = 0;
        len += self.value.consensus_encode(w)?;
        len += self.script_pubkey.consensus_encode(w)?;
        Ok(len)
    }
}

impl Decodable for TxOut {
    #[inline]
    fn consensus_decode_from_finite_reader<R: BufRead + ?Sized>(
        r: &mut R,
    ) -> Result<Self, encode::Error> {
        Ok(TxOut {
            value: Decodable::consensus_decode_from_finite_reader(r)?,
            script_pubkey: Decodable::consensus_decode_from_finite_reader(r)?,
        })
    }
}

/**
 * Basic transaction serialization format:
 * - int32_t nVersion
 * - std::vector<CTxIn> vin
 * - std::vector<CTxOut> vout
 * - uint32_t nLockTime
 *
 * TODO: Extended transaction serialization format:
 * - int32_t nVersion
 * - unsigned char dummy = 0x00
 * - unsigned char flags (!= 0)
 * - std::vector<CTxIn> vin
 * - std::vector<CTxOut> vout
 * - if (flags & 1):
 *   - CTxWitness wit;
 * - uint32_t nLockTime
 */
#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct Transaction {
    pub version: u32,
    pub lock_time: u32,
    pub input: Vec<TxIn>,
    pub output: Vec<TxOut>,
}

#[derive(Clone, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
pub struct TransactionJsonResult {
    pub hex: String,
    pub txid: String,
    pub blockhash: String,
    pub confirmations: u32,
    pub time: u32,
    pub blocktime: u32,
}

impl TryFrom<TransactionJsonResult> for Transaction {
    type Error = String;

    fn try_from(value: TransactionJsonResult) -> Result<Self, Self::Error> {
        deserialize_hex(&value.hex)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RpcTxOut {
    pub bestblock: String,
    pub confirmations: u32,
    pub value: f64,
    pub version: u32,
    pub coinbase: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DogeRpcResponse<T> {
    pub result: T,
    pub error: Option<String>,
    pub id: u32,
}

impl<T> DogeRpcResponse<T> {
    pub fn try_result(self) -> Result<T, CustomsError> {
        self.error
            .map_or(Ok(self.result), |e| Err(CustomsError::RpcError(e)))
    }
}

impl cmp::PartialOrd for Transaction {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl cmp::Ord for Transaction {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.version
            .cmp(&other.version)
            .then(self.lock_time.cmp(&other.lock_time))
            .then(self.input.cmp(&other.input))
            .then(self.output.cmp(&other.output))
    }
}

impl Transaction {
    pub const CURRENT_VERSION: u32 = 1;
    pub const MAX_STANDARD_VERSION: u32 = 2;
    pub const SERIALIZE_TRANSACTION_NO_WITNESS: u32 = 0x40000000;

    /// Computes the [`Txid`].
    pub fn compute_txid(&self) -> Txid {
        let mut enc = Txid::engine();
        self.version
            .consensus_encode(&mut enc)
            .expect("engines don't error");
        consensus_encode_vec(&self.input, &mut enc).expect("engines don't error");
        consensus_encode_vec(&self.output, &mut enc).expect("engines don't error");
        self.lock_time
            .consensus_encode(&mut enc)
            .expect("engines don't error");
        Txid::from_engine(enc)
    }

    pub fn is_coinbase(&self) -> bool {
        self.input.len() == 1 && self.input[0].prevout.is_null()
    }

    /// Returns the base transaction size.
    ///
    /// > Base transaction size is the size of the transaction serialised with the witness data stripped.
    pub fn size(&self) -> usize {
        let mut size: usize = 4; // Serialized length of a u32 for the version number.

        size += VarInt::from(self.input.len()).size();
        size += self.input.iter().map(|input| input.size()).sum::<usize>();

        size += VarInt::from(self.output.len()).size();
        size += self
            .output
            .iter()
            .map(|output| output.size())
            .sum::<usize>();

        size + 4 // LockTime::SIZE
    }

    pub fn estimate_size(&self) -> usize {
        let mut size: usize = 4; // Serialized length of a u32 for the version number.

        size += VarInt::from(self.input.len()).size();
        size += self
            .input
            .iter()
            .map(|input| input.estimate_size())
            .sum::<usize>();

        size += VarInt::from(self.output.len()).size();
        size += self
            .output
            .iter()
            .map(|output| output.estimate_size())
            .sum::<usize>();

        size + 4 // LockTime::SIZE
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        self.consensus_encode(&mut buf).unwrap();
        buf
    }
}

impl Encodable for Transaction {
    fn consensus_encode<W: Write + ?Sized>(&self, w: &mut W) -> Result<usize, Error> {
        let mut len = 0;
        len += self.version.consensus_encode(w)?;
        len += consensus_encode_vec(&self.input, w)?;
        len += consensus_encode_vec(&self.output, w)?;
        len += self.lock_time.consensus_encode(w)?;
        Ok(len)
    }
}

impl Decodable for Transaction {
    fn consensus_decode_from_finite_reader<R: BufRead + ?Sized>(
        r: &mut R,
    ) -> Result<Self, encode::Error> {
        Ok(Transaction {
            version: Decodable::consensus_decode_from_finite_reader(r)?,
            input: consensus_decode_from_vec(r)?,
            output: consensus_decode_from_vec(r)?,
            lock_time: Decodable::consensus_decode_from_finite_reader(r)?,
        })
    }
}

impl TryFrom<&[u8]> for Transaction {
    type Error = String;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        let mut rd = data;
        Self::consensus_decode_from_finite_reader(&mut rd).map_err(err_string)
    }
}

impl From<Transaction> for Txid {
    fn from(tx: Transaction) -> Txid {
        tx.compute_txid()
    }
}

impl From<&Transaction> for Txid {
    fn from(tx: &Transaction) -> Txid {
        tx.compute_txid()
    }
}
