// SPDX-License-Identifier: CC0-1.0

//! Dogecoin transactions.

use bitcoin::consensus::{encode, Decodable, Encodable};
use bitcoin::hashes::{hash_newtype, sha256d, Hash};
use bitcoin::{ScriptBuf, VarInt};
use bitcoin_io::{BufRead, Error, Write};
use serde::{Deserialize, Serialize};
use core::cmp;
use std::ops::Deref;

use crate::errors::CustomsError;

use super::chainparams::DOGE_MAIN_NET_CHAIN;
use super::script::classify_script;

// use crate::{consensus_decode_from_vec, consensus_encode_vec, err_string};

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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RawTransaction {
    pub result: String,
    pub error: Option<String>,
    pub id: u32,
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
    pub fn unwrap_result(self) -> Result<T, CustomsError> {
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

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use hex::test_hex_unwrap as hex;
//     use std::str::FromStr;

//     use crate::chainparams::DOGE_MAIN_NET_CHAIN;
//     use crate::script::{classify_script, ScriptType};

//     #[test]
//     fn test_transaction() {
//         // https://github.com/dogecoin/dogecoin/blob/master/src/test/data/tt-delin1-out.json
//         let data = hex!("0100000014fd5c23522d31761c50175453daa6edaabe47a602a592d39ce933d8271a1a87274c0100006c493046022100b4251ecd63778a3dde0155abe4cd162947620ae9ee45a874353551092325b116022100db307baf4ff3781ec520bd18f387948cedd15dc27bafe17c894b0fe6ffffcafa012103091137f3ef23f4acfc19a5953a68b2074fae942ad3563ef28c33b0cac9a93adcffffffffc1b37ae964f605978022f94ce2f3f676d66a46d1aef7c2c17d6315b9697f2f75010000006a473044022079bd62ee09621a3be96b760c39e8ef78170101d46313923c6b07ae60a95c90670220238e51ea29fc70b04b65508450523caedbb11cb4dd5aa608c81487de798925ba0121027a759be8df971a6a04fafcb4f6babf75dc811c5cdaa0734cddbe9b942ce75b34ffffffffedd005dc7790ef65c206abd1ab718e75252a40f4b1310e4102cd692eca9cacb0d10000006b48304502207722d6f9038673c86a1019b1c4de2d687ae246477cd4ca7002762be0299de385022100e594a11e3a313942595f7666dcf7078bcb14f1330f4206b95c917e7ec0e82fac012103091137f3ef23f4acfc19a5953a68b2074fae942ad3563ef28c33b0cac9a93adcffffffffdf28d6e26fb7a85a1e6a229b972c1bae0edc1c11cb9ca51e4caf5e59fbea35a1000000006b483045022100a63a4788027b79b65c6f9d9e054f68cf3b4eed19efd82a2d53f70dcbe64683390220526f243671425b2bd05745fcf2729361f985cfe84ea80c7cfc817b93d8134374012103a621f08be22d1bbdcbe4e527ee4927006aa555fc65e2aafa767d4ea2fe9dfa52ffffffffae2a2320a1582faa24469eff3024a6b98bfe00eb4f554d8a0b1421ba53bfd6a5010000006c493046022100b200ac6db16842f76dab9abe807ce423c992805879bc50abd46ed8275a59d9cf022100c0d518e85dd345b3c29dd4dc47b9a420d3ce817b18720e94966d2fe23413a408012103091137f3ef23f4acfc19a5953a68b2074fae942ad3563ef28c33b0cac9a93adcffffffffb3cc5a12548aa1794b4d2bbf076838cfd7fbafb7716da51ee8221a4ff19c291b000000006b483045022100ededc441c3103a6f2bd6cab7639421af0f6ec5e60503bce1e603cf34f00aee1c02205cb75f3f519a13fb348783b21db3085cb5ec7552c59e394fdbc3e1feea43f967012103a621f08be22d1bbdcbe4e527ee4927006aa555fc65e2aafa767d4ea2fe9dfa52ffffffff85145367313888d2cf2747274a32e20b2df074027bafd6f970003fcbcdf11d07150000006b483045022100d9eed5413d2a4b4b98625aa6e3169edc4fb4663e7862316d69224454e70cd8ca022061e506521d5ced51dd0ea36496e75904d756a4c4f9fb111568555075d5f68d9a012103f1575d6124ac78be398c25b31146d08313c6072d23a4d7df5ac6a9f87346c64cffffffff8292c11f6d35abab5bac3ebb627a4ff949e8ecd62d33ed137adf7aeb00e512b0090000006b48304502207e84b27139c4c19c828cb1e30c349bba88e4d9b59be97286960793b5ddc0a2af0221008cdc7a951e7f31c20953ed5635fbabf228e80b7047f32faaa0313e7693005177012103f1575d6124ac78be398c25b31146d08313c6072d23a4d7df5ac6a9f87346c64cffffffff883dcf9a86063db088ad064d0953258d4b0ff3425857402d2f3f839cee0f84581e0000006a4730440220426540dfed9c4ab5812e5f06df705b8bcf307dd7d20f7fa6512298b2a6314f420220064055096e3ca62f6c7352c66a5447767c53f946acdf35025ab3807ddb2fa404012103f1575d6124ac78be398c25b31146d08313c6072d23a4d7df5ac6a9f87346c64cffffffff6697dbb3ed98afe481b568459fa67e503f8a4254532465a670e54669d19c9fe6720000006a47304402200a5e673996f2fc88e21cc8613611f08a650bc0370338803591d85d0ec5663764022040b6664a0d1ec83a7f01975b8fde5232992b8ca58bf48af6725d2f92a936ab2e012103f1575d6124ac78be398c25b31146d08313c6072d23a4d7df5ac6a9f87346c64cffffffff023ffc2182517e1d3fa0896c5b0bd7b4d2ef8a1e42655abe2ced54f657125d59670000006c493046022100d93b30219c5735f673be5c3b4688366d96f545561c74cb62c6958c00f6960806022100ec8200adcb028f2184fa2a4f6faac7f8bb57cb4503bb7584ac11051fece31b3d012103091137f3ef23f4acfc19a5953a68b2074fae942ad3563ef28c33b0cac9a93adcffffffff16f8c77166b0df3d7cc8b5b2ce825afbea9309ad7acd8e2461a255958f81fc06010000006b483045022100a13934e68d3f5b22b130c4cb33f4da468cffc52323a47fbfbe06b64858162246022047081e0a70ff770e64a2e2d31e5d520d9102268b57a47009a72fe73ec766901801210234b9d9413f247bb78cd3293b7b65a2c38018ba5621ea9ee737f3a6a3523fb4cdffffffff197b96f3c87a3adfaa17f63fddc2a738a690ca665439f9431dbbd655816c41fb000000006c49304602210097f1f35d5bdc1a3a60390a1b015b8e7c4f916aa3847aafd969e04975e15bbe70022100a9052eb25517d481f1fda1b129eb1b534da50ea1a51f3ee012dca3601c11b86a0121027a759be8df971a6a04fafcb4f6babf75dc811c5cdaa0734cddbe9b942ce75b34ffffffff20d9a261ee27aa1bd92e7db2fdca935909a40b648e974cd24a10d63b68b94039dd0000006b483045022012b3138c591bf7154b6fef457f2c4a3c7162225003788ac0024a99355865ff13022100b71b125ae1ffb2e1d1571f580cd3ebc8cd049a2d7a8a41f138ba94aeb982106f012103091137f3ef23f4acfc19a5953a68b2074fae942ad3563ef28c33b0cac9a93adcffffffff50f179d5d16cd872f9a63c26c448464ae9bd95cd9421c0476113b5d314571b71010000006b483045022100f834ccc8b22ee72712a3e5e6ef4acb8b2fb791b5385b70e2cd4332674d6667f4022024fbda0a997e0c253503f217501f508a4d56edce2c813ecdd9ad796dbeba907401210234b9d9413f247bb78cd3293b7b65a2c38018ba5621ea9ee737f3a6a3523fb4cdffffffff551b865d1568ac0a305e5f9c5dae6c540982334efbe789074318e0efc5b564631b0000006b48304502203b2fd1e39ae0e469d7a15768f262661b0de41470daf0fe8c4fd0c26542a0870002210081c57e331f9a2d214457d953e3542904727ee412c63028113635d7224da3dccc012103f1575d6124ac78be398c25b31146d08313c6072d23a4d7df5ac6a9f87346c64cffffffff57503e5a016189d407a721791459280875264f908ca2c5d4862c01386e7fb50b470400006b48304502206947a9c54f0664ece4430fd4ae999891dc50bb6126bc36b6a15a3189f29d25e9022100a86cfc4e2fdd9e39a20e305cfd1b76509c67b3e313e0f118229105caa0e823c9012103f1575d6124ac78be398c25b31146d08313c6072d23a4d7df5ac6a9f87346c64cffffffff3f16c1fb9d3e1a26d872933e955df85ee7f3f817711062b00b54a2144827349b250000006b483045022100c7128fe10b2d38744ae8177776054c29fc8ec13f07207723e70766ab7164847402201d2cf09009b9596de74c0183d1ab832e5edddb7a9965880bb400097e850850f8012103f1575d6124ac78be398c25b31146d08313c6072d23a4d7df5ac6a9f87346c64cffffffff4142a69d85b8498af214f0dd427b6ab29c240a0b8577e2944d37a7d8c05c6bb8140000006b48304502203b89a71628a28cc3703d170ca3be77786cff6b867e38a18b719705f8a326578f022100b2a9879e1acf621faa6466c207746a7f3eb4c8514c1482969aba3f2a957f1321012103f1575d6124ac78be398c25b31146d08313c6072d23a4d7df5ac6a9f87346c64cffffffff36e2feecc0a4bff7480015d42c12121932db389025ed0ac1d344ecee53230a3df20000006c493046022100ef794a8ef7fd6752d2a183c18866ff6e8dc0f5bd889a63e2c21cf303a6302461022100c1b09662d9e92988c3f9fcf17d1bcc79b5403647095d7212b9f8a1278a532d68012103091137f3ef23f4acfc19a5953a68b2074fae942ad3563ef28c33b0cac9a93adcffffffff0260f73608000000001976a9148fd139bb39ced713f231c58a4d07bf6954d1c20188ac41420f00000000001976a9146c772e9cf96371bba3da8cb733da70a2fcf2007888ac00000000");
//         let mut rd = &data[..];
//         let tx = Transaction::consensus_decode_from_finite_reader(&mut rd).unwrap();
//         println!("input: {}, output: {}", tx.input.len(), tx.output.len());

//         let mut buf = Vec::new();
//         tx.consensus_encode(&mut buf).unwrap();
//         assert_eq!(buf, data);

//         assert_eq!(
//             tx.compute_txid().to_string(),
//             "81b2035be1da1abe745c6141174a73d151009ec17b3d5ebffa2e177408c50dfd"
//         );

//         assert_eq!(tx.version, 1);
//         assert_eq!(tx.lock_time, 0);
//         let input = vec![TxIn{
//                 prevout: OutPoint {
//                     txid: Txid::from_str(
//                         "27871a1a27d833e99cd392a502a647beaaeda6da535417501c76312d52235cfd"
//                     ).unwrap(),
//                     vout: 332,
//                 },
//                 script: ScriptBuf::from_hex("493046022100b4251ecd63778a3dde0155abe4cd162947620ae9ee45a874353551092325b116022100db307baf4ff3781ec520bd18f387948cedd15dc27bafe17c894b0fe6ffffcafa012103091137f3ef23f4acfc19a5953a68b2074fae942ad3563ef28c33b0cac9a93adc").unwrap(),
//                 sequence: 4294967295,
//                 witness: Witness::default(),
//             },
//             TxIn{
//                 prevout: OutPoint {
//                     txid: Txid::from_str(
//                         "752f7f69b915637dc1c2f7aed1466ad676f6f3e24cf922809705f664e97ab3c1"
//                     ).unwrap(),
//                     vout: 1,
//                 },
//                 script: ScriptBuf::from_hex("473044022079bd62ee09621a3be96b760c39e8ef78170101d46313923c6b07ae60a95c90670220238e51ea29fc70b04b65508450523caedbb11cb4dd5aa608c81487de798925ba0121027a759be8df971a6a04fafcb4f6babf75dc811c5cdaa0734cddbe9b942ce75b34").unwrap(),
//                 sequence: 4294967295,
//                 witness: Witness::default(),
//             },
//             TxIn{
//                 prevout: OutPoint {
//                     txid: Txid::from_str(
//                         "b0ac9cca2e69cd02410e31b1f4402a25758e71abd1ab06c265ef9077dc05d0ed"
//                     ).unwrap(),
//                     vout: 209,
//                 },
//                 script: ScriptBuf::from_hex("48304502207722d6f9038673c86a1019b1c4de2d687ae246477cd4ca7002762be0299de385022100e594a11e3a313942595f7666dcf7078bcb14f1330f4206b95c917e7ec0e82fac012103091137f3ef23f4acfc19a5953a68b2074fae942ad3563ef28c33b0cac9a93adc").unwrap(),
//                 sequence: 4294967295,
//                 witness: Witness::default(),
//             },
//             TxIn{
//                 prevout: OutPoint {
//                     txid: Txid::from_str(
//                         "a135eafb595eaf4c1ea59ccb111cdc0eae1b2c979b226a1e5aa8b76fe2d628df"
//                     ).unwrap(),
//                     vout: 0,
//                 },
//                 script: ScriptBuf::from_hex("483045022100a63a4788027b79b65c6f9d9e054f68cf3b4eed19efd82a2d53f70dcbe64683390220526f243671425b2bd05745fcf2729361f985cfe84ea80c7cfc817b93d8134374012103a621f08be22d1bbdcbe4e527ee4927006aa555fc65e2aafa767d4ea2fe9dfa52").unwrap(),
//                 sequence: 4294967295,
//                 witness: Witness::default(),
//             },
//             TxIn{
//                 prevout: OutPoint {
//                     txid: Txid::from_str(
//                         "a5d6bf53ba21140b8a4d554feb00fe8bb9a62430ff9e4624aa2f58a120232aae"
//                     ).unwrap(),
//                     vout: 1,
//                 },
//                 script: ScriptBuf::from_hex("493046022100b200ac6db16842f76dab9abe807ce423c992805879bc50abd46ed8275a59d9cf022100c0d518e85dd345b3c29dd4dc47b9a420d3ce817b18720e94966d2fe23413a408012103091137f3ef23f4acfc19a5953a68b2074fae942ad3563ef28c33b0cac9a93adc").unwrap(),
//                 sequence: 4294967295,
//                 witness: Witness::default(),
//             },
//             TxIn{
//                 prevout: OutPoint {
//                     txid: Txid::from_str(
//                         "1b299cf14f1a22e81ea56d71b7affbd7cf386807bf2b4d4b79a18a54125accb3"
//                     ).unwrap(),
//                     vout: 0,
//                 },
//                 script: ScriptBuf::from_hex("483045022100ededc441c3103a6f2bd6cab7639421af0f6ec5e60503bce1e603cf34f00aee1c02205cb75f3f519a13fb348783b21db3085cb5ec7552c59e394fdbc3e1feea43f967012103a621f08be22d1bbdcbe4e527ee4927006aa555fc65e2aafa767d4ea2fe9dfa52").unwrap(),
//                 sequence: 4294967295,
//                 witness: Witness::default(),
//             },
//             TxIn{
//                 prevout: OutPoint {
//                     txid: Txid::from_str(
//                         "071df1cdcb3f0070f9d6af7b0274f02d0be2324a274727cfd288383167531485"
//                     ).unwrap(),
//                     vout: 21,
//                 },
//                 script: ScriptBuf::from_hex("483045022100d9eed5413d2a4b4b98625aa6e3169edc4fb4663e7862316d69224454e70cd8ca022061e506521d5ced51dd0ea36496e75904d756a4c4f9fb111568555075d5f68d9a012103f1575d6124ac78be398c25b31146d08313c6072d23a4d7df5ac6a9f87346c64c").unwrap(),
//                 sequence: 4294967295,
//                 witness: Witness::default(),
//             },
//             TxIn{
//                 prevout: OutPoint {
//                     txid: Txid::from_str(
//                         "b012e500eb7adf7a13ed332dd6ece849f94f7a62bb3eac5babab356d1fc19282"
//                     ).unwrap(),
//                     vout: 9,
//                 },
//                 script: ScriptBuf::from_hex("48304502207e84b27139c4c19c828cb1e30c349bba88e4d9b59be97286960793b5ddc0a2af0221008cdc7a951e7f31c20953ed5635fbabf228e80b7047f32faaa0313e7693005177012103f1575d6124ac78be398c25b31146d08313c6072d23a4d7df5ac6a9f87346c64c").unwrap(),
//                 sequence: 4294967295,
//                 witness: Witness::default(),
//             },
//             TxIn{
//                 prevout: OutPoint {
//                     txid: Txid::from_str(
//                         "58840fee9c833f2f2d40575842f30f4b8d2553094d06ad88b03d06869acf3d88"
//                     ).unwrap(),
//                     vout: 30,
//                 },
//                 script: ScriptBuf::from_hex("4730440220426540dfed9c4ab5812e5f06df705b8bcf307dd7d20f7fa6512298b2a6314f420220064055096e3ca62f6c7352c66a5447767c53f946acdf35025ab3807ddb2fa404012103f1575d6124ac78be398c25b31146d08313c6072d23a4d7df5ac6a9f87346c64c").unwrap(),
//                 sequence: 4294967295,
//                 witness: Witness::default(),
//             },
//             TxIn{
//                 prevout: OutPoint {
//                     txid: Txid::from_str(
//                         "e69f9cd16946e570a665245354428a3f507ea69f4568b581e4af98edb3db9766"
//                     ).unwrap(),
//                     vout: 114,
//                 },
//                 script: ScriptBuf::from_hex("47304402200a5e673996f2fc88e21cc8613611f08a650bc0370338803591d85d0ec5663764022040b6664a0d1ec83a7f01975b8fde5232992b8ca58bf48af6725d2f92a936ab2e012103f1575d6124ac78be398c25b31146d08313c6072d23a4d7df5ac6a9f87346c64c").unwrap(),
//                 sequence: 4294967295,
//                 witness: Witness::default(),
//             },
//             TxIn{
//                 prevout: OutPoint {
//                     txid: Txid::from_str(
//                         "595d1257f654ed2cbe5a65421e8aefd2b4d70b5b6c89a03f1d7e518221fc3f02"
//                     ).unwrap(),
//                     vout: 103,
//                 },
//                 script: ScriptBuf::from_hex("493046022100d93b30219c5735f673be5c3b4688366d96f545561c74cb62c6958c00f6960806022100ec8200adcb028f2184fa2a4f6faac7f8bb57cb4503bb7584ac11051fece31b3d012103091137f3ef23f4acfc19a5953a68b2074fae942ad3563ef28c33b0cac9a93adc").unwrap(),
//                 sequence: 4294967295,
//                 witness: Witness::default(),
//             },
//             TxIn{
//                 prevout: OutPoint {
//                     txid: Txid::from_str(
//                         "06fc818f9555a261248ecd7aad0993eafb5a82ceb2b5c87c3ddfb06671c7f816"
//                     ).unwrap(),
//                     vout: 1,
//                 },
//                 script: ScriptBuf::from_hex("483045022100a13934e68d3f5b22b130c4cb33f4da468cffc52323a47fbfbe06b64858162246022047081e0a70ff770e64a2e2d31e5d520d9102268b57a47009a72fe73ec766901801210234b9d9413f247bb78cd3293b7b65a2c38018ba5621ea9ee737f3a6a3523fb4cd").unwrap(),
//                 sequence: 4294967295,
//                 witness: Witness::default(),
//             },
//             TxIn{
//                 prevout: OutPoint {
//                     txid: Txid::from_str(
//                         "fb416c8155d6bb1d43f9395466ca90a638a7c2dd3ff617aadf3a7ac8f3967b19"
//                     ).unwrap(),
//                     vout: 0,
//                 },
//                 script: ScriptBuf::from_hex("49304602210097f1f35d5bdc1a3a60390a1b015b8e7c4f916aa3847aafd969e04975e15bbe70022100a9052eb25517d481f1fda1b129eb1b534da50ea1a51f3ee012dca3601c11b86a0121027a759be8df971a6a04fafcb4f6babf75dc811c5cdaa0734cddbe9b942ce75b34").unwrap(),
//                 sequence: 4294967295,
//                 witness: Witness::default(),
//             },
//             TxIn{
//                 prevout: OutPoint {
//                     txid: Txid::from_str(
//                         "3940b9683bd6104ad24c978e640ba4095993cafdb27d2ed91baa27ee61a2d920"
//                     ).unwrap(),
//                     vout: 221,
//                 },
//                 script: ScriptBuf::from_hex("483045022012b3138c591bf7154b6fef457f2c4a3c7162225003788ac0024a99355865ff13022100b71b125ae1ffb2e1d1571f580cd3ebc8cd049a2d7a8a41f138ba94aeb982106f012103091137f3ef23f4acfc19a5953a68b2074fae942ad3563ef28c33b0cac9a93adc").unwrap(),
//                 sequence: 4294967295,
//                 witness: Witness::default(),
//             },
//             TxIn{
//                 prevout: OutPoint {
//                     txid: Txid::from_str(
//                         "711b5714d3b5136147c02194cd95bde94a4648c4263ca6f972d86cd1d579f150"
//                     ).unwrap(),
//                     vout: 1,
//                 },
//                 script: ScriptBuf::from_hex("483045022100f834ccc8b22ee72712a3e5e6ef4acb8b2fb791b5385b70e2cd4332674d6667f4022024fbda0a997e0c253503f217501f508a4d56edce2c813ecdd9ad796dbeba907401210234b9d9413f247bb78cd3293b7b65a2c38018ba5621ea9ee737f3a6a3523fb4cd").unwrap(),
//                 sequence: 4294967295,
//                 witness: Witness::default(),
//             },
//             TxIn{
//                 prevout: OutPoint {
//                     txid: Txid::from_str(
//                         "6364b5c5efe018430789e7fb4e338209546cae5d9c5f5e300aac68155d861b55"
//                     ).unwrap(),
//                     vout: 27,
//                 },
//                 script: ScriptBuf::from_hex("48304502203b2fd1e39ae0e469d7a15768f262661b0de41470daf0fe8c4fd0c26542a0870002210081c57e331f9a2d214457d953e3542904727ee412c63028113635d7224da3dccc012103f1575d6124ac78be398c25b31146d08313c6072d23a4d7df5ac6a9f87346c64c").unwrap(),
//                 sequence: 4294967295,
//                 witness: Witness::default(),
//             },
//             TxIn{
//                 prevout: OutPoint {
//                     txid: Txid::from_str(
//                         "0bb57f6e38012c86d4c5a28c904f2675082859147921a707d48961015a3e5057"
//                     ).unwrap(),
//                     vout: 1095,
//                 },
//                 script: ScriptBuf::from_hex("48304502206947a9c54f0664ece4430fd4ae999891dc50bb6126bc36b6a15a3189f29d25e9022100a86cfc4e2fdd9e39a20e305cfd1b76509c67b3e313e0f118229105caa0e823c9012103f1575d6124ac78be398c25b31146d08313c6072d23a4d7df5ac6a9f87346c64c").unwrap(),
//                 sequence: 4294967295,
//                 witness: Witness::default(),
//             },
//             TxIn{
//                 prevout: OutPoint {
//                     txid: Txid::from_str(
//                         "9b34274814a2540bb062107117f8f3e75ef85d953e9372d8261a3e9dfbc1163f"
//                     ).unwrap(),
//                     vout: 37,
//                 },
//                 script: ScriptBuf::from_hex("483045022100c7128fe10b2d38744ae8177776054c29fc8ec13f07207723e70766ab7164847402201d2cf09009b9596de74c0183d1ab832e5edddb7a9965880bb400097e850850f8012103f1575d6124ac78be398c25b31146d08313c6072d23a4d7df5ac6a9f87346c64c").unwrap(),
//                 sequence: 4294967295,
//                 witness: Witness::default(),
//             },
//             TxIn{
//                 prevout: OutPoint {
//                     txid: Txid::from_str(
//                         "b86b5cc0d8a7374d94e277850b0a249cb26a7b42ddf014f28a49b8859da64241"
//                     ).unwrap(),
//                     vout: 20,
//                 },
//                 script: ScriptBuf::from_hex("48304502203b89a71628a28cc3703d170ca3be77786cff6b867e38a18b719705f8a326578f022100b2a9879e1acf621faa6466c207746a7f3eb4c8514c1482969aba3f2a957f1321012103f1575d6124ac78be398c25b31146d08313c6072d23a4d7df5ac6a9f87346c64c").unwrap(),
//                 sequence: 4294967295,
//                 witness: Witness::default(),
//             },
//             TxIn{
//                 prevout: OutPoint {
//                     txid: Txid::from_str(
//                         "3d0a2353eeec44d3c10aed259038db321912122cd4150048f7bfa4c0ecfee236"
//                     ).unwrap(),
//                     vout: 242,
//                 },
//                 script: ScriptBuf::from_hex("493046022100ef794a8ef7fd6752d2a183c18866ff6e8dc0f5bd889a63e2c21cf303a6302461022100c1b09662d9e92988c3f9fcf17d1bcc79b5403647095d7212b9f8a1278a532d68012103091137f3ef23f4acfc19a5953a68b2074fae942ad3563ef28c33b0cac9a93adc").unwrap(),
//                 sequence: 4294967295,
//                 witness: Witness::default(),
//             }];

//         assert_eq!(input.len(), tx.input.len());
//         for (i, v) in tx.input.iter().enumerate() {
//             assert_eq!(v, &input[i], "input[{}] mismatch: {}", i, v.prevout.txid);
//         }

//         let output = [
//             TxOut {
//                 value: 137820000,
//                 script_pubkey: ScriptBuf::from_hex(
//                     "76a9148fd139bb39ced713f231c58a4d07bf6954d1c20188ac",
//                 )
//                 .unwrap(),
//             },
//             TxOut {
//                 value: 1000001,
//                 script_pubkey: ScriptBuf::from_hex(
//                     "76a9146c772e9cf96371bba3da8cb733da70a2fcf2007888ac",
//                 )
//                 .unwrap(),
//             },
//         ];

//         // {
//         //     "value": 1.3782,
//         //     "n": 0,
//         //     "scriptPubKey": {
//         //         "asm": "OP_DUP OP_HASH160 8fd139bb39ced713f231c58a4d07bf6954d1c201 OP_EQUALVERIFY OP_CHECKSIG",
//         //         "hex": "76a9148fd139bb39ced713f231c58a4d07bf6954d1c20188ac",
//         //         "reqSigs": 1,
//         //         "type": "pubkeyhash",
//         //         "addresses": [
//         //             "DJFXow7CYcBWKVjwe1VH4or5f6YetLH1hw"
//         //         ]
//         //     }
//         // },
//         // {
//         //     "value": 0.01000001,
//         //     "n": 1,
//         //     "scriptPubKey": {
//         //         "asm": "OP_DUP OP_HASH160 6c772e9cf96371bba3da8cb733da70a2fcf20078 OP_EQUALVERIFY OP_CHECKSIG",
//         //         "hex": "76a9146c772e9cf96371bba3da8cb733da70a2fcf2007888ac",
//         //         "reqSigs": 1,
//         //         "type": "pubkeyhash",
//         //         "addresses": [
//         //             "DF2cHtiK4xeXPUBhMdK7XWU5UNYSg2KFvt"
//         //         ]
//         //     }
//         // }
//         let s = ScriptBuf::from(hex!("76a9148fd139bb39ced713f231c58a4d07bf6954d1c20188ac"));
//         println!("script: {:?}", s);

//         let (script_type, addr) = classify_script(s.as_bytes(), &DOGE_MAIN_NET_CHAIN);
//         assert_eq!(script_type, ScriptType::PubKeyHash);
//         assert_eq!(
//             addr.unwrap().to_string(),
//             "DJFXow7CYcBWKVjwe1VH4or5f6YetLH1hw"
//         );

//         let (script_type, addr) = classify_script(
//             &hex!("76a9146c772e9cf96371bba3da8cb733da70a2fcf2007888ac"),
//             &DOGE_MAIN_NET_CHAIN,
//         );
//         assert_eq!(script_type, ScriptType::PubKeyHash);
//         assert_eq!(
//             addr.unwrap().to_string(),
//             "DF2cHtiK4xeXPUBhMdK7XWU5UNYSg2KFvt"
//         );

//         assert_eq!(output.len(), tx.output.len());
//         for (i, v) in tx.output.iter().enumerate() {
//             assert_eq!(
//                 v,
//                 &output[i],
//                 "output[{}] mismatch: {}",
//                 i,
//                 v.script_pubkey.to_hex_string()
//             );
//         }
//     }
// }
