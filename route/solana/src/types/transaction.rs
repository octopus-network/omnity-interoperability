use super::account::{ParsedAccount, UiTokenAmount};
use super::message::{Message, UiMessage};
use super::pubkey::Pubkey;
use super::reward::Rewards;
use super::short_vec;
use super::signature::Signature;
use super::transaction_error::TransactionError;
use super::{BlockHash, CompiledInstruction, Slot, UnixTimestamp};
use candid::CandidType;

use serde::de::Error;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::Display;
use std::str::FromStr;

pub type TransactionResult<T> = Result<T, TransactionError>;

#[derive(Debug, PartialEq, Default, Eq, Clone, Serialize, Deserialize)]
pub struct Transaction {
    #[serde(with = "short_vec")]
    pub signatures: Vec<Signature>,
    pub message: Message,
}

impl Transaction {
    pub fn new_unsigned(message: Message) -> Self {
        Self {
            signatures: vec![Signature::default(); message.header.num_required_signatures as usize],
            message,
        }
    }

    pub fn data(&self, instruction_index: usize) -> &[u8] {
        &self.message.instructions[instruction_index].data
    }

    pub fn key(&self, instruction_index: usize, accounts_index: usize) -> Option<&Pubkey> {
        self.key_index(instruction_index, accounts_index)
            .and_then(|account_keys_index| self.message.account_keys.get(account_keys_index))
    }

    pub fn signer_key(&self, instruction_index: usize, accounts_index: usize) -> Option<&Pubkey> {
        match self.key_index(instruction_index, accounts_index) {
            None => None,
            Some(signature_index) => {
                if signature_index >= self.signatures.len() {
                    return None;
                }
                self.message.account_keys.get(signature_index)
            }
        }
    }

    pub fn set_latest_blockhash(&mut self, blockhash: &BlockHash) {
        self.message.recent_blockhash = *blockhash;
    }

    /// Return the message containing all data that should be signed.
    pub fn message(&self) -> &Message {
        &self.message
    }

    /// Return the serialized message data to sign.
    pub fn message_data(&self) -> Vec<u8> {
        self.message().serialize()
    }

    pub fn is_signed(&self) -> bool {
        self.signatures
            .iter()
            .all(|signature| *signature != Signature::default())
    }

    pub fn add_signature(&mut self, position: usize, signature: Signature) {
        self.signatures[position] = signature;
    }

    fn key_index(&self, instruction_index: usize, accounts_index: usize) -> Option<usize> {
        self.message
            .instructions
            .get(instruction_index)
            .and_then(|instruction| instruction.accounts.get(accounts_index))
            .map(|&account_keys_index| account_keys_index as usize)
    }

    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(self).expect("Transaction serialization failed")
    }
}

impl Display for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", bs58::encode(self.serialize()).into_string())
    }
}

impl FromStr for Transaction {
    type Err = bincode::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = bs58::decode(s)
            .into_vec()
            .map_err(|_| bincode::Error::custom("Transaction deserialization failed"))?;
        bincode::deserialize(&bytes)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase", untagged)]
pub enum TransactionVersion {
    Legacy,
    Number(u8),
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Eq, Hash, PartialEq, CandidType)]
#[serde(rename_all = "camelCase")]
pub enum TransactionBinaryEncoding {
    Base58,
    Base64,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum UiTransactionEncoding {
    Binary, // Legacy. Retained for RPC backwards compatibility
    Base64,
    Base58,
    Json,
    JsonParsed,
}

impl Display for UiTransactionEncoding {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let v = serde_json::to_value(self).map_err(|_| fmt::Error)?;
        let s = v.as_str().ok_or(fmt::Error)?;
        write!(f, "{s}")
    }
}

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TransactionDetails {
    Full,
    Signatures,
    None,
    Accounts,
}

impl Default for TransactionDetails {
    fn default() -> Self {
        Self::Full
    }
}

/// A duplicate representation of a Transaction for pretty JSON serialization
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase")]
pub struct UiTransaction {
    pub signatures: Vec<String>,
    pub message: UiMessage,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase")]
pub struct UiAccountsList {
    pub signatures: Vec<String>,
    pub account_keys: Vec<ParsedAccount>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase", untagged)]
pub enum EncodedTransaction {
    LegacyBinary(String), // Old way of expressing base-58, retained for RPC backwards compatibility
    Binary(String, TransactionBinaryEncoding),
    Json(UiTransaction),
    Accounts(UiAccountsList),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase")]
pub enum TransactionConfirmationStatus {
    Processed,
    Confirmed,
    Finalized,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase")]
pub struct TransactionStatus {
    pub slot: Slot,
    pub confirmations: Option<usize>,  // None = rooted
    pub status: TransactionResult<()>, // legacy field
    pub err: Option<TransactionError>,
    pub confirmation_status: Option<TransactionConfirmationStatus>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase")]
pub struct TransactionStatusMeta {
    pub status: TransactionResult<()>,
    pub fee: u64,
    pub pre_balances: Vec<u64>,
    pub post_balances: Vec<u64>,
    pub inner_instructions: Option<Vec<InnerInstructions>>,
    pub log_messages: Option<Vec<String>>,
    pub pre_token_balances: Option<Vec<TransactionTokenBalance>>,
    pub post_token_balances: Option<Vec<TransactionTokenBalance>>,
    pub rewards: Option<Rewards>,
    pub loaded_addresses: LoadedAddresses,
    pub return_data: Option<TransactionReturnData>,
    pub compute_units_consumed: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase")]
pub struct TransactionTokenBalance {
    pub account_index: u8,
    pub mint: String,
    pub ui_token_amount: UiTokenAmount,
    pub owner: String,
    pub program_id: String,
}

/// Return data at the end of a transaction
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, CandidType)]
pub struct TransactionReturnData {
    pub program_id: Pubkey,
    pub data: Vec<u8>,
}

/// Collection of addresses loaded from on-chain lookup tables, split
/// by readonly and writable.
#[derive(Clone, Default, Debug, PartialEq, Eq, Serialize, Deserialize, CandidType)]
pub struct LoadedAddresses {
    /// List of addresses for writable loaded accounts
    pub writable: Vec<Pubkey>,
    /// List of addresses for read-only loaded accounts
    pub readonly: Vec<Pubkey>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase")]
pub struct EncodedTransactionWithStatusMeta {
    pub transaction: EncodedTransaction,
    pub meta: Option<TransactionStatusMeta>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<TransactionVersion>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase")]
pub struct EncodedConfirmedTransactionWithStatusMeta {
    pub slot: Slot,
    #[serde(flatten)]
    pub transaction: EncodedTransactionWithStatusMeta,
    pub block_time: Option<UnixTimestamp>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase")]
pub struct InnerInstructions {
    /// Transaction instruction index
    pub index: u8,
    /// List of inner instructions
    pub instructions: Vec<InnerInstruction>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, CandidType)]
pub struct InnerInstruction {
    /// Compiled instruction
    pub instruction: CompiledInstruction,
    /// Invocation stack height of the instruction,
    pub stack_height: Option<u32>,
}
