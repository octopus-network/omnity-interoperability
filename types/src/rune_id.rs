use candid::Deserialize;
use serde::Serialize;
use std::{
    error::Error,
    fmt::{self, Display, Formatter},
    str::FromStr,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseRuneIdError;

impl fmt::Display for ParseRuneIdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        "provided rune_id was not valid".fmt(f)
    }
}

impl Error for ParseRuneIdError {
    fn description(&self) -> &str {
        "failed to parse rune_id"
    }
}

#[derive(
    candid::CandidType,
    Clone,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Copy,
    Default,
    Serialize,
    Deserialize,
)]
pub struct RuneId {
    pub block: u64,
    pub tx: u32,
}

impl RuneId {
    pub fn delta(self, next: RuneId) -> Option<(u128, u128)> {
        let block = next.block.checked_sub(self.block)?;

        let tx = if block == 0 {
            next.tx.checked_sub(self.tx)?
        } else {
            next.tx
        };

        Some((block.into(), tx.into()))
    }
}

impl Display for RuneId {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.block, self.tx,)
    }
}

impl FromStr for RuneId {
    type Err = ParseRuneIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (height, index) = s.split_once(':').ok_or(ParseRuneIdError)?;

        Ok(Self {
            block: height.parse().map_err(|_| ParseRuneIdError)?,
            tx: index.parse().map_err(|_| ParseRuneIdError)?,
        })
    }
}
