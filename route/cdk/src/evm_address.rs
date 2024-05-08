use std::str::FromStr;
use candid::CandidType;
use ethers_core::abi::ethereum_types;
use serde_derive::{Deserialize, Serialize};
use thiserror::Error;
use ethereum_types::Address;
use hex::ToHex;

const EVM_ADDR_BYTES_LEN: usize = 20;

#[derive(Deserialize, CandidType,Serialize, Default, Clone, Eq, PartialEq)]
pub struct EvmAddress(pub(crate) [u8;EVM_ADDR_BYTES_LEN]);

#[derive(Error, Clone,Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum EvmAddressError {
    #[error("Bytes is longer than 29 bytes.")]
    LengthError,
    #[error("Bytes is longer than 29 bytes.")]
    FormatError,
}

impl Into<Address> for EvmAddress {
    fn into(self) -> Address {
        Address::from(self.0)
    }
}
impl AsRef<[u8]> for EvmAddress {
    fn as_ref(&self) -> &[u8] {
        self.0.as_slice()
    }
}

impl FromStr for EvmAddress {
    type Err = EvmAddressError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        EvmAddress::from_text(s)
    }
}

impl EvmAddress {
    pub fn from_text<S: AsRef<str>>(text: S) -> Result<Self, EvmAddressError>{
        let t = if text.as_ref().starts_with("0x") {
            text.as_ref().strip_prefix("0x").unwrap()
        }else {
            text.as_ref()
        };
        let r =  hex::decode(t).map_err(|e| EvmAddressError::FormatError)?;
        if r.len() != EVM_ADDR_BYTES_LEN {
            return Err(EvmAddressError::LengthError);
        }
        let mut v = [0u8; 20];
        v.copy_from_slice(r.as_slice());
        Ok(EvmAddress(v))
    }

}