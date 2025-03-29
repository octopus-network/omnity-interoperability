use bitcoin::{
    block::{Header, Version},
    consensus::{encode, Decodable, Encodable},
    hashes::{sha256d, Hash, HashEngine},
    BlockHash, CompactTarget, Target, Transaction, TxMerkleNode,
};
use bitcoin_io::BufRead;
use scrypt::{scrypt, Params};

use crate::errors::ValidationError;

#[derive(Clone, Debug)]
pub struct PureHeader(pub Header);

impl PureHeader {
    /// Returns the chain id of the block.
    pub fn get_chain_id(&self) -> i32 {
        self.0.version.to_consensus() >> 16
    }

    /// Returns the block hash.
    pub fn block_pow_hash(&self) -> BlockHash {
        // Serialize Header into a byte vector
        let mut header_data = Vec::new();
        self.0
            .consensus_encode(&mut header_data)
            .expect("Failed to serialize Header");

        // Scrypt requires a salt, which is the header data itself
        let salt = &header_data;

        // Set up Scrypt parameters (N=2^10, r=1, p=1, dk_len=32)
        let params = Params::new(10, 1, 1, 32).unwrap(); // dk_len=32, output is a 32-byte hash

        // Calculate hash using scrypt
        let mut result = vec![0u8; 32]; // Allocate a 32-byte Vec to store the result
        scrypt(&header_data, salt, &params, &mut result).expect("Scrypt computation failed");

        BlockHash::from_slice(&result).unwrap()
    }

    /// Checks that the proof-of-work for the block is valid, returning the block hash.
    pub fn validate_doge_pow(&self, required_target: Target) -> Result<BlockHash, ValidationError> {
        let pow_hash = self.block_pow_hash();
        if required_target.is_met_by(pow_hash) {
            Ok(self.0.block_hash())
        } else {
            Err(ValidationError::BadProofOfWork)
        }
    }
}

impl From<PureHeader> for Header {
    fn from(value: PureHeader) -> Self {
        value.0
    }
}

impl Into<PureHeader> for Header {
    fn into(self) -> PureHeader {
        PureHeader(self)
    }
}

#[derive(Clone, Debug)]
pub struct DogecoinHeader {
    /// Block version, now repurposed for soft fork signalling.
    pub version: Version,
    /// Reference to the previous block in the chain.
    pub prev_blockhash: BlockHash,
    /// The root hash of the merkle tree of transactions in the block.
    pub merkle_root: TxMerkleNode,
    /// The timestamp of the block, as claimed by the miner.
    pub time: u32,
    /// The target value below which the blockhash must lie.
    pub bits: CompactTarget,
    /// The nonce, selected to obtain a low enough blockhash.
    pub nonce: u32,
    /// The auxpow info
    pub auxpow: Option<AuxPow>,
}

#[derive(Clone, Debug)]
pub struct AuxPow {
    /// The parent block's coinbase transaction.
    pub coinbase_tx: Transaction,
    /// Block hash
    pub block_hash: BlockHash,
    /// The Merkle branch of the coinbase tx to the parent block's root.
    pub coinbase_branch: Vec<TxMerkleNode>,
    /// Coinbase index
    pub coinbase_index: i32,
    /// The merkle branch connecting the aux block to our coinbase.
    pub blockchain_branch: Vec<TxMerkleNode>,
    /// Merkle tree index of the aux block header in the coinbase.
    pub chain_index: i32,
    /// Parent block header (on which the real PoW is done).
    pub parent_block_header: Header,
}

impl From<DogecoinHeader> for Header {
    fn from(value: DogecoinHeader) -> Self {
        Header {
            version: value.version,
            prev_blockhash: value.prev_blockhash,
            merkle_root: value.merkle_root,
            time: value.time,
            bits: value.bits,
            nonce: value.nonce,
        }
    }
}

impl DogecoinHeader {
    pub fn block_hash(&self) -> BlockHash {
        let pure_header: Header = self.clone().into();
        pure_header.block_hash()
    }

    /// Returns whether the block is a legacy block.
    pub fn is_legacy(&self) -> bool {
        self.version.to_consensus() == 1
        // Dogecoin: We have a random v2 block with no AuxPoW, treat as legacy
        || (self.version.to_consensus() == 2 && self.get_chain_id() == 0)
    }

    /// Returns the chain id of the block.
    pub fn get_chain_id(&self) -> i32 {
        self.version.to_consensus() >> 16
    }

    /// Returns whether the block is an auxpow block.
    pub fn is_auxpow(&self) -> bool {
        self.version.to_consensus() & (1 << 8) != 0
    }

    /// Checks that the proof-of-work for the block is valid, returning the block hash.
    pub fn validate_doge_pow(
        &self,
        is_strict_chain_id: bool,
    ) -> Result<BlockHash, ValidationError> {
        // Dogecoin main fStrictChainId set true
        // Dogecoin testnet fStrictChainId set false
        // Dogecoin main/testnet nAuxpowChainId set 0x0062
        if !self.is_legacy() && is_strict_chain_id && self.get_chain_id() != 98 {
            return Err(ValidationError::BadVersion);
        }

        let header: Header = self.clone().into();
        let pure_header: PureHeader = header.into();

        // Check that the proof-of-work is correct
        if self.auxpow.is_none() {
            if self.is_auxpow() {
                return Err(ValidationError::BadVersion);
            }
            return pure_header.validate_doge_pow(header.target());
        }

        // Check auxpow
        if !self.is_auxpow() {
            return Err(ValidationError::BadVersion);
        }
        let auxpow = self.auxpow.as_ref().expect("auxpow");
        auxpow.check(is_strict_chain_id, header.block_hash())?;
        // Check that the parent block's proof-of-work is correct
        Into::<PureHeader>::into(auxpow.parent_block_header).validate_doge_pow(header.target())?;

        Ok(self.block_hash())
    }
}

impl Decodable for AuxPow {
    #[inline]
    fn consensus_decode_from_finite_reader<R: BufRead + ?Sized>(
        r: &mut R,
    ) -> Result<AuxPow, encode::Error> {
        Ok(AuxPow {
            coinbase_tx: Decodable::consensus_decode_from_finite_reader(r)?,
            block_hash: Decodable::consensus_decode_from_finite_reader(r)?,
            coinbase_branch: Decodable::consensus_decode_from_finite_reader(r)?,
            coinbase_index: Decodable::consensus_decode_from_finite_reader(r)?,
            blockchain_branch: Decodable::consensus_decode_from_finite_reader(r)?,
            chain_index: Decodable::consensus_decode_from_finite_reader(r)?,
            parent_block_header: Decodable::consensus_decode_from_finite_reader(r)?,
        })
    }
}

impl Decodable for DogecoinHeader {
    #[inline]
    fn consensus_decode_from_finite_reader<R: bitcoin_io::BufRead + ?Sized>(
        d: &mut R,
    ) -> Result<Self, bitcoin::consensus::encode::Error> {
        let version: Version = Decodable::consensus_decode_from_finite_reader(d)?;
        let prev_blockhash = Decodable::consensus_decode_from_finite_reader(d)?;
        let merkle_root = Decodable::consensus_decode_from_finite_reader(d)?;
        let time = Decodable::consensus_decode_from_finite_reader(d)?;
        let bits = Decodable::consensus_decode_from_finite_reader(d)?;
        let nonce = Decodable::consensus_decode_from_finite_reader(d)?;

        let auxpow = if version.to_consensus() & (1 << 8) != 0 {
            Some(Decodable::consensus_decode_from_finite_reader(d)?)
        } else {
            None
        };

        Ok(Self {
            bits,
            merkle_root,
            nonce,
            time,
            prev_blockhash,
            version,
            auxpow,
        })
    }

    // #[inline]
    // fn consensus_decode<R: bitcoin_io::BufRead + ?Sized>(reader: &mut R) -> Result<Self, bitcoin::consensus::encode::Error> {
    //     Self::consensus_decode_from_finite_reader(&mut reader.take(bitcoin::consensus::encode::MAX_VEC_SIZE as u64))
    // }
}

impl AuxPow {
    const MERGED_MINING_HEADER: &[u8] = b"\xfa\xbe\x6d\x6d";
    /// Returns the block hash of the header.
    pub fn check(
        &self,
        is_strict_chain_id: bool,
        block_hash: BlockHash,
    ) -> Result<(), ValidationError> {
        if self.coinbase_index != 0 {
            return Err(ValidationError::BadAuxPow("coinbase_index".to_string()));
        }

        // Aux POW parent cannot has our chain ID 0x0062
        let pure_parent_header: PureHeader = self.parent_block_header.clone().into();
        if is_strict_chain_id && pure_parent_header.get_chain_id() == 98 {
            return Err(ValidationError::BadAuxPow("chain_id".to_string()));
        }

        // Check that the blockchain branch is valid
        if self.blockchain_branch.len() > 30 {
            return Err(ValidationError::BadAuxPow(
                "blockchain_branch length".to_string(),
            ));
        }

        // Check that the chain merkle root is in the coinbase
        let chain_root_hash = check_merkle_branch(
            block_hash.to_raw_hash(),
            &self.blockchain_branch,
            self.chain_index,
        )?;
        let mut reversed_chain_root_hash = chain_root_hash.to_byte_array();
        reversed_chain_root_hash.reverse();

        // Check that we are in the parent block merkle tree
        if self.parent_block_header.merkle_root
            != check_merkle_branch(
                self.coinbase_tx.compute_txid().into(),
                &self.coinbase_branch,
                self.coinbase_index,
            )?
            .into()
        {
            return Err(ValidationError::BadAuxPow("coinbase_branch".to_string()));
        }

        // Extract the coinbase script and ensure it contains the merged mining header and root hash
        let script = self.coinbase_tx.input[0].script_sig.to_bytes();

        // Find merged mining header
        let mm_header_pos = script
            .windows(Self::MERGED_MINING_HEADER.len())
            .position(|window| window == Self::MERGED_MINING_HEADER);

        // Check for chain merkle root in coinbase
        let root_hash_pos = script
            .windows(reversed_chain_root_hash.len())
            .position(|window| window == reversed_chain_root_hash);
        if root_hash_pos.is_none() {
            return Err(ValidationError::BadAuxPow(
                "reversed_chain_root_hash".to_string(),
            ));
        }

        if let Some(header_pos) = mm_header_pos {
            // Enforce only one chain merkle root by checking that a single instance of the merged
            // mining header exists just before.
            let second_mm_header = script
                .windows(Self::MERGED_MINING_HEADER.len())
                .skip(header_pos + 1)
                .position(|window| window == Self::MERGED_MINING_HEADER);

            if second_mm_header.is_some() {
                return Err(ValidationError::BadAuxPow("second_mm_header".to_string()));
            }
            if header_pos + Self::MERGED_MINING_HEADER.len() != root_hash_pos.unwrap() {
                return Err(ValidationError::BadAuxPow("root_hash_pos".to_string()));
            }
        } else {
            // For backward compatibility.
            // Enforce only one chain merkle root by checking that it starts early in the coinbase.
            // 8-12 bytes are enough to encode extraNonce and nBits.
            if root_hash_pos.unwrap() > 20 {
                return Err(ValidationError::BadAuxPow("root_hash_pos > 20".to_string()));
            }
        }

        // Ensure we are at a deterministic point in the merkle leaves by hashing
        // a nonce and our chain ID and comparing to the index.
        let remaining = &script[root_hash_pos.unwrap() + reversed_chain_root_hash.len()..];
        if remaining.len() < 8 {
            return Err(ValidationError::BadAuxPow("remaining".to_string()));
        }
        let merkle_size = u32::from_le_bytes(remaining[0..4].try_into().unwrap());
        let nonce = u32::from_le_bytes(remaining[4..8].try_into().unwrap());
        if merkle_size != (1 << self.blockchain_branch.len()) as u32 {
            return Err(ValidationError::BadAuxPow("merkle_size".to_string()));
        }
        if self.chain_index as u32 != get_expected_index(nonce, 98, self.blockchain_branch.len()) {
            return Err(ValidationError::BadAuxPow("chain_index".to_string()));
        }

        Ok(())
    }
}

/// Merkle branch verification based on the provided `TxMerkleNode` and the index
fn check_merkle_branch(
    hash: sha256d::Hash,
    branch: &[TxMerkleNode],
    index: i32,
) -> Result<sha256d::Hash, ValidationError> {
    if index < 0 {
        return Err(ValidationError::BadAuxPow("index < 0".to_string()));
    }

    let mut current_hash = hash;
    let mut idx = index as usize;
    for merkle_node in branch {
        if idx & 1 == 1 {
            current_hash = hash_internal(merkle_node.to_raw_hash(), current_hash)?;
        } else {
            current_hash = hash_internal(current_hash, merkle_node.to_raw_hash())?;
        }
        idx >>= 1;
    }

    Ok(current_hash.into())
}

/// Helper function to handle hash operation between two nodes in the Merkle branch
fn hash_internal(
    left: sha256d::Hash,
    right: sha256d::Hash,
) -> Result<sha256d::Hash, ValidationError> {
    // Here you would hash the concatenation of the two hashes
    let mut hasher = sha256d::Hash::engine();
    hasher.input(left.as_ref());
    hasher.input(right.as_ref());
    Ok(sha256d::Hash::from_engine(hasher))
}

/// Chooses a pseudo-random slot in the chain merkle tree,
/// but ensures it is fixed for a given size/nonce/chain combination.
///
/// This prevents the same work from being reused for the same chain
/// and reduces the likelihood of two chains clashing for the same slot.
///
/// Note:
/// - This computation can overflow the `u32` used. However, this is not an issue,
///   since the result is taken modulo a power-of-two, ensuring consistency.
/// - The computation remains consistent even if performed in 64 bits,
///   as it was on some systems in the past.
/// - The `h` parameter is always <= 30, as enforced by the maximum allowed chain
///   merkle branch length, so 32 bits are sufficient for the computation.
fn get_expected_index(n_nonce: u32, n_chain_id: u32, h: usize) -> u32 {
    let mut rand = n_nonce;
    rand = rand.wrapping_mul(1103515245).wrapping_add(12345);
    rand = rand.wrapping_add(n_chain_id);
    rand = rand.wrapping_mul(1103515245).wrapping_add(12345);

    rand % (1 << h)
}

#[test]
pub fn test() {
    use bitcoin::consensus::deserialize;
    // pub use hex::DisplayHex;
    use hex::test_hex_unwrap as hex;
    // Dogecoin header main 5,503,356
    let doge_header_hex = hex!("0401620014f42e78e4cfb6c85e1ed69baaebc7980eab1e562c3102a2313d0d3f60a57f57f8a30d320709365ff18e490ec75fb1eeea21b7f58c2df98d474fc83391dd9786a135a067df89001a0000000001000000010000000000000000000000000000000000000000000000000000000000000000ffffffff6403f0502b2cfabe6d6d091fb22b379c6292e3ea11dfbc5791b846b8710f06c40423f1897d84c99c8de520000000f09f909f092f4632506f6f6c2f620000000000000000000000000000000000000000000000000000000000000000000000050006bbe80000000000027fb64c25000000001976a914f2910ecaf7bb8d18ed71f0904e0e7456f29ce18288ac0000000000000000266a24aa21a9ed71f793873b02234e8f85653bd24b0d4732a03cf982a8557fd8a94a720ba13c0c44802342bbf26d083e6981cd0c8bd8c546017cb3a4d59fd74a8ee40b6f00000000000000093174a22c90c9ba373f6b217103db6fd2e5ee32305412f1a70d6cd518aa37bb585f0157bcf1e917f1a57ca1f25e285ab9a13923b09fbcc067aa11f4df0b8e3f32e95d74c5cb46a0f4b640fc5e266572be748fb44362fa9c8bced497b4d08c335bd0e306a01530abf8cb74c0a30ed18fb64c00c13fffc0eeaf857bb43cc90bc12920e831f5123b3090ddb4735e38a17785c1a122d69ebe2fdfbeba730232f2c7466690e66867453625152d341d01ac6213191c755d1f03c5e7913208a1c29ebd7019d704cd28c2155624a8eb824606d14df9d52c446f9dd23ab5a24d65388125c00b926b648085f6c8c0812fb06131d9db967a8c25d9fd62d51e448ec08f7643f01b557154f42eb947425e4b4268abcd9511f186407ba7e4e0300c6a377306255500000000050900000000000000000000000000000000000000000000000000000000000000710b1b3f2df407a200dd8321454575d9c8a35228a0cb34775aef1d44a656a7c85c12e33ac1828d49a9d4580df8375be9e01265131c99ac1f63ed5f0af1f48eb59f1c3a68d34485e30bc1d754c0bb167db13a6567dd99ae08ad603e718463109fea90a8279fe3b97fc48c854b71443d09e88f2f31040c71a68466fb395cf474410800000014000020250936f4a4a548ac27a19c57718ddd4a0d2f2fa6e7b94e7cfdcf9bd97a51ef6708a07f78e003591bff278d0faccc11d4055054e8ee7f152dabd761aaac71df0fd135a06750bc39190c6c094e");
    let doge_header: DogecoinHeader =
        deserialize(&doge_header_hex).expect("Can't deserialize correct block header");
    let pure_header: Header = doge_header.clone().into();
    assert!(doge_header.is_auxpow());
    assert_eq!(
        doge_header.validate_doge_pow(true).unwrap(),
        doge_header.block_hash()
    );
}
