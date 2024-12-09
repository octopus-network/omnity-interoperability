use candid::{Deserialize, CandidType};
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
   CandidType,
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


#[derive(Default, Serialize, Debug, PartialEq, Clone)]
pub struct Etching {
    pub divisibility: Option<u8>,
    pub premine: Option<u128>,
    pub rune: Option<Rune>,
    pub spacers: Option<u32>,
    pub symbol: Option<char>,
    pub terms: Option<Terms>,
    pub turbo: bool,
}
#[derive(CandidType, Default, Serialize, Deserialize, Debug, PartialEq, Eq, Copy, Clone)]
pub struct Rune(pub u128);

impl Rune {
    pub fn commitment(self) -> Vec<u8> {
        let bytes = self.0.to_le_bytes();
        let mut end = bytes.len();
        while end > 0 && bytes[end - 1] == 0 {
          end -= 1;
        }
        bytes[..end].into()
      }
}

impl Etching {
    pub const MAX_DIVISIBILITY: u8 = 38;
    pub const MAX_SPACERS: u32 = 0b00000111_11111111_11111111_11111111;
  
    pub fn supply(&self) -> Option<u128> {
      let premine = self.premine.unwrap_or_default();
      let cap = self.terms.and_then(|terms| terms.cap).unwrap_or_default();
      let amount = self
        .terms
        .and_then(|terms| terms.amount)
        .unwrap_or_default();
      premine.checked_add(cap.checked_mul(amount)?)
    }
}

#[derive(CandidType, Default, Serialize, Deserialize, Debug, PartialEq, Eq, Copy, Clone)]
pub struct Terms {
  pub amount: Option<u128>,
  pub cap: Option<u128>,
  pub height: (Option<u64>, Option<u64>),
  pub offset: (Option<u64>, Option<u64>),
}

#[derive(CandidType, Default, Serialize, Deserialize, Debug, PartialEq, Eq, Copy, Clone)]
pub struct SpacedRune {
    pub rune: Rune,
    pub spacers: u32,
}