//! Implements `InscriptionId`

use std::str::FromStr;

use bitcoin::hashes::Hash;
use bitcoin::Txid;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::ord::result::InscriptionParseError;

/// Represents an Ordinal/BRC20 inscription identifier,
/// derived from the transaction ID and the associated `vout` (index) of the UTXO
/// in the format `("{}i{}", self.txid, self.index)`.
#[derive(Debug, PartialEq, Copy, Clone, Hash, Eq, PartialOrd, Ord)]
pub struct InscriptionId {
    pub txid: Txid,
    pub index: u32,
}

impl Default for InscriptionId {
    fn default() -> Self {
        Self {
            txid: Txid::all_zeros(),
            index: 0,
        }
    }
}

impl std::fmt::Display for InscriptionId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}i{}", self.txid, self.index)
    }
}

impl Serialize for InscriptionId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for InscriptionId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        DeserializeFromStr::with(deserializer)
    }
}

struct DeserializeFromStr<T: FromStr>(pub T);

impl<'de, T: FromStr> DeserializeFromStr<T>
where
    T::Err: std::fmt::Display,
{
    pub fn with<D>(deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(DeserializeFromStr::<T>::deserialize(deserializer)?.0)
    }
}

impl<'de, T: FromStr> Deserialize<'de> for DeserializeFromStr<T>
where
    T::Err: std::fmt::Display,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Self(
            FromStr::from_str(&String::deserialize(deserializer)?)
                .map_err(serde::de::Error::custom)?,
        ))
    }
}

impl FromStr for InscriptionId {
    type Err = InscriptionParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(char) = s.chars().find(|char| !char.is_ascii()) {
            return Err(InscriptionParseError::Character(char));
        }

        const TXID_LEN: usize = 64;
        const MIN_LEN: usize = TXID_LEN + 2;

        if s.len() < MIN_LEN {
            return Err(InscriptionParseError::InscriptionIdLength(s.len()));
        }

        let txid = &s[..TXID_LEN];

        let separator = s.chars().nth(TXID_LEN).unwrap();

        if separator != 'i' {
            return Err(InscriptionParseError::CharacterSeparator(separator));
        }

        let vout = &s[TXID_LEN + 1..];

        Ok(Self {
            txid: txid.parse().map_err(InscriptionParseError::Txid)?,
            index: vout.parse().map_err(InscriptionParseError::Index)?,
        })
    }
}
