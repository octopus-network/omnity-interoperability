//! An implementation of the [BRC20 Token Standard](https://domo-2.gitbook.io/brc-20-experiment),
//! an experiment to see if [Ordinal Theory](https://docs.ordinals.com/) can facilitate fungibility on Bitcoin.
//!
//! 1. Deployments initialize the BRC-20. Do not affect state.
//! 2. Mints provide a balance to only the first owner of the mint function inscription.
//! 3. Transfers deduct from the sender's balance and add to the receiver's balance,
//!     only upon the first transfer of the transfer function. That is,
//!     - step 1. Sender inscribes the transfer function to sender's (own) address.
//!     - step 2. Sender transfers transfer function to final destination address.

use std::str::FromStr;

use bitcoin::opcodes::all::{OP_CHECKSIG, OP_ENDIF, OP_IF};
use bitcoin::opcodes::{OP_0, OP_FALSE};
use bitcoin::script::{Builder as ScriptBuilder, PushBytesBuf};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use crate::ord::builder::RedeemScriptPubkey;
use crate::ord::inscription::Inscription;
use crate::ord::parser::push_bytes::bytes_to_push_bytes;
use crate::ord::result::{OrdError, OrdResult};
use serde_with::DisplayFromStr;
const PROTOCOL: &str = "brc-20";

/// Represents a BRC-20 operation: (Deploy, Mint, Transfer)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op")]
pub enum Brc20 {
    /// Deploy a BRC-20 token
    #[serde(rename = "deploy")]
    Deploy(Brc20Deploy),
    /// Mint BRC-20 tokens
    #[serde(rename = "mint")]
    Mint(Brc20Mint),
    /// Transfer BRC-20 tokens
    #[serde(rename = "transfer")]
    Transfer(Brc20Transfer),
}

impl Brc20 {
    /// Create a new BRC-20 deploy operation
    pub fn deploy(
        tick: impl ToString,
        max: u64,
        lim: Option<u64>,
        dec: Option<u64>,
        self_mint: Option<bool>,
    ) -> Self {
        Self::Deploy(Brc20Deploy {
            protocol: PROTOCOL.to_string(),
            tick: tick.to_string(),
            max,
            lim,
            dec,
            self_mint,
        })
    }

    /// Create a new BRC-20 mint operation
    pub fn mint(tick: impl ToString, amt: u64, rf: String) -> Self {
        Self::Mint(Brc20Mint {
            protocol: PROTOCOL.to_string(),
            tick: tick.to_string(),
            amt,
            rf,
        })
    }

    /// Create a new BRC-20 transfer operation
    pub fn transfer(tick: impl ToString, amt: u64, receiver: String, chainid: String) -> Self {
        Self::Transfer(Brc20Transfer {
            protocol: PROTOCOL.to_string(),
            tick: tick.to_string(),
            amt,
            refx: receiver,
            chain: chainid,
            ext: "bridge-in".to_string(),
        })
    }

    fn append_reveal_script_to_builder(
        &self,
        builder: ScriptBuilder,
        pubkey: RedeemScriptPubkey,
    ) -> OrdResult<ScriptBuilder> {
        let encoded_pubkey = pubkey.encode()?;

        Ok(builder
            .push_slice(encoded_pubkey.as_push_bytes())
            .push_opcode(OP_CHECKSIG)
            .push_opcode(OP_FALSE)
            .push_opcode(OP_IF)
            .push_slice(b"ord")
            .push_slice(b"\x01")
            .push_slice(bytes_to_push_bytes(self.content_type().as_bytes())?.as_push_bytes())
            .push_opcode(OP_0)
            .push_slice(self.data()?.as_push_bytes())
            .push_opcode(OP_ENDIF))
    }
}

impl FromStr for Brc20 {
    type Err = OrdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s).map_err(OrdError::from)
    }
}

impl Inscription for Brc20 {
    fn generate_redeem_script(
        &self,
        builder: ScriptBuilder,
        pubkey: RedeemScriptPubkey,
    ) -> OrdResult<ScriptBuilder> {
        self.append_reveal_script_to_builder(builder, pubkey)
    }

    fn content_type(&self) -> String {
        "text/plain;charset=utf-8".to_string()
    }

    fn data(&self) -> OrdResult<PushBytesBuf> {
        bytes_to_push_bytes(self.encode()?.as_bytes())
    }
}

/// `deploy` op
#[serde_as]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Brc20Deploy {
    /// Protocol (required): Helps other systems identify and process brc-20 events
    #[serde(rename = "p")]
    protocol: String,
    /// Ticker (required): 4 or 5 letter identifier of the brc-20
    pub tick: String,
    /// Max supply (required): Set max supply of the brc-20
    #[serde_as(as = "DisplayFromStr")]
    pub max: u64,
    /// Mint limit (optional): If letting users mint to themsleves, limit per ordinal
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub lim: Option<u64>,
    /// Decimals (optional): Set decimal precision, default to 18
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub dec: Option<u64>,
    /// Self mint (optional): Set the ticker to be mintable only by the deployment holder
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub self_mint: Option<bool>,
}

/// `mint` op
#[serde_as]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Brc20Mint {
    /// Protocol (required): Helps other systems identify and process brc-20 events
    #[serde(rename = "p")]
    protocol: String,
    /// Ticker (required): 4 or 5 letter identifier of the brc-20
    pub tick: String,
    /// Amount to mint (required): States the amount of the brc-20 to mint.
    /// Has to be less than "lim" of the `deploy` op if stated.
    #[serde_as(as = "DisplayFromStr")]
    pub amt: u64,
    #[serde(rename = "btx")]
    pub rf: String,
}

/// `transfer` op
#[serde_as]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Brc20Transfer {
    /// Protocol (required): Helps other systems identify and process brc-20 events
    #[serde(rename = "p")]
    protocol: String,
    /// Ticker (required): 4 or 5 letter identifier of the brc-20
    pub tick: String,
    /// Amount to transfer (required): States the amount of the brc-20 to transfer.
    #[serde_as(as = "DisplayFromStr")]
    pub amt: u64,
    #[serde(rename = "ref")]
    pub refx: String,
    pub chain: String,
    pub ext: String,
}