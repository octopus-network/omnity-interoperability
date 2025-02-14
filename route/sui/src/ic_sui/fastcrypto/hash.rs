//! This module contains a selection of cryptographic hash functions implementing a common [HashFunction] trait.
//!
//! # Example
//! ```
//! # use fastcrypto::hash::*;
//! let digest1 = Sha256::digest(b"Hello, world!");
//!
//! let mut hash_function = Sha256::default();
//! hash_function.update(b"Hello, ");
//! hash_function.update(b"world!");
//! let digest2 = hash_function.finalize();
//!
//! assert_eq!(digest1, digest2);
//! ```

use core::fmt::Debug;
// use digest::OutputSizeUser;
use generic_array::GenericArray;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::fmt;

use super::encoding::{Base64, Encoding};

/// Represents a digest of `DIGEST_LEN` bytes.
#[serde_as]
#[derive(Hash, PartialEq, Eq, Clone, Serialize, Deserialize, Ord, PartialOrd, Copy)]
pub struct Digest<const DIGEST_LEN: usize> {
    #[serde_as(as = "[_; DIGEST_LEN]")]
    pub digest: [u8; DIGEST_LEN],
}

impl<const DIGEST_LEN: usize> Digest<DIGEST_LEN> {
    /// Create a new digest containing the given bytes
    pub fn new(digest: [u8; DIGEST_LEN]) -> Self {
        Digest { digest }
    }

    /// Copy the digest into a new vector.
    pub fn to_vec(&self) -> Vec<u8> {
        self.digest.to_vec()
    }

    /// The size of this digest in bytes.
    pub fn size(&self) -> usize {
        DIGEST_LEN
    }
}

impl<const DIGEST_LEN: usize> fmt::Debug for Digest<DIGEST_LEN> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{}", Base64::encode(self.digest))
    }
}

impl<const DIGEST_LEN: usize> fmt::Display for Digest<DIGEST_LEN> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{}", Base64::encode(self.digest))
    }
}

impl<const DIGEST_LEN: usize> AsRef<[u8]> for Digest<DIGEST_LEN> {
    fn as_ref(&self) -> &[u8] {
        self.digest.as_ref()
    }
}

impl<const DIGEST_LEN: usize> From<Digest<DIGEST_LEN>> for [u8; DIGEST_LEN] {
    fn from(digest: Digest<DIGEST_LEN>) -> Self {
        digest.digest
    }
}

/// Trait implemented by hash functions providing a output of fixed length
pub trait HashFunction<const DIGEST_LENGTH: usize>: Default {
    /// The length of this hash functions digests in bytes.
    const OUTPUT_SIZE: usize = DIGEST_LENGTH;

    /// Create a new hash function of the given type
    fn new() -> Self {
        Self::default()
    }

    /// Process the given data, and update the internal of the hash function.
    fn update<Data: AsRef<[u8]>>(&mut self, data: Data);

    /// Retrieve result and consume hash function.
    fn finalize(self) -> Digest<DIGEST_LENGTH>;

    /// Compute the digest of the given data and consume the hash function.
    fn digest<Data: AsRef<[u8]>>(data: Data) -> Digest<DIGEST_LENGTH> {
        let mut h = Self::default();
        h.update(data);
        h.finalize()
    }

    /// Compute a single digest from all slices in the iterator in order and consume the hash function.
    fn digest_iterator<K: AsRef<[u8]>, I: Iterator<Item = K>>(iter: I) -> Digest<DIGEST_LENGTH> {
        let mut h = Self::default();
        iter.for_each(|item| h.update(item));
        h.finalize()
    }
}

/// This trait is implemented by all messages that can be hashed.
pub trait Hash<const DIGEST_LEN: usize> {
    /// The type of the digest when this is hashed.
    type TypedDigest: Into<Digest<DIGEST_LEN>> + Eq + std::hash::Hash + Copy + Debug;

    fn digest(&self) -> Self::TypedDigest;
}

/// This wraps a [digest::Digest] as a [HashFunction].
#[derive(Default)]
pub struct HashFunctionWrapper<Variant, const DIGEST_LEN: usize>(Variant);

impl<Variant: digest::Digest + Default, const DIGEST_LEN: usize> HashFunction<DIGEST_LEN>
    for HashFunctionWrapper<Variant, DIGEST_LEN>
{
    fn update<Data: AsRef<[u8]>>(&mut self, data: Data) {
        self.0.update(data);
    }

    fn finalize(self) -> Digest<DIGEST_LEN> {
        let mut digest = [0u8; DIGEST_LEN];
        self.0
            .finalize_into(GenericArray::from_mut_slice(&mut digest));
        Digest { digest }
    }
}

pub type Blake2b256 = HashFunctionWrapper<blake2::Blake2b<typenum::U32>, 32>;

/// A Multiset Hash is a homomorphic hash function, which hashes arbitrary multisets of objects such
/// that the hash of the union of two multisets is easy to compute from the hashes of the two multisets.
///
/// The hash may be computed incrementally, adding items one at a time, and the order does not affect the
/// result. The hash of two multisets can be compared by using the Eq trait impl'd for the given hash function,
/// and the hash function should be collision resistant. Items may also be removed again.
///
/// See ["Incremental Multiset Hash Functions and Their Application to Memory Integrity Checking" by D. Clarke
/// et al.](https://link.springer.com/chapter/10.1007/978-3-540-40061-5_12) for a discussion of this type of hash
/// functions.
///
/// # Example
/// ```
/// use fastcrypto::hash::{EllipticCurveMultisetHash, MultisetHash};
///
/// let mut hash1 = EllipticCurveMultisetHash::default();
/// hash1.insert(b"Hello");
/// hash1.insert(b"World");
///
/// let mut hash2 = EllipticCurveMultisetHash::default();
/// hash2.insert(b"World");
/// hash2.insert(b"Hello");
///
/// assert_eq!(hash1, hash2);
/// assert_eq!(hash1.digest(), hash2.digest());
/// ```
pub trait MultisetHash<const DIGEST_LENGTH: usize>: Eq {
    /// Insert an item into this hash function.
    fn insert<Data: AsRef<[u8]>>(&mut self, item: Data);

    /// Insert multiple items into this hash function.
    fn insert_all<It, Data>(&mut self, items: It)
    where
        It: IntoIterator<Item = Data>,
        Data: AsRef<[u8]>;

    /// Add all the elements of another hash function into this hash function.
    fn union(&mut self, other: &Self);

    // Note that the "remove" operation is safe even if an item has been removed
    // more times than it has been inserted. To see why, consider the following
    // example: Suppose an adversary has performed two sets of "insert(x)" and
    // "remove(x)" operations resulting in the same hash, i.e., the sum of each set
    // is \sum_x m_x H(x), where m_x is the difference between the number of times
    // "x" was inserted and removed.
    // Then, one can create two new sets with the same hash by taking the original
    // sets and subtracting m_x H(x) from both sets for every item "x" such that m_x
    // was negative in any of the original sets. Since we "subtract" (or actually
    // insert) the same elements from both sets, the resulting hash will remain the
    // same. Moreover, since none of the values of m_x in the new sets are negative,
    // we can conclude that no item was removed more times than it was inserted in
    // the new sets.
    /// Remove an element from this hash function.
    fn remove<Data: AsRef<[u8]>>(&mut self, item: Data);

    /// Remove multiple items from this hash function.
    fn remove_all<It, Data>(&mut self, items: It)
    where
        It: IntoIterator<Item = Data>,
        Data: AsRef<[u8]>;

    /// Generate a digest of the current state of this hash function.
    fn digest(&self) -> Digest<DIGEST_LENGTH>;
}
