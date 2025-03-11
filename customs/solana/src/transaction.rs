use candid::CandidType;
use ic_solana::types::{Slot, TransactionError, TransactionResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;

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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TransactionDetail {
    pub block_time: Option<u64>,
    pub meta: Meta,
    pub slot: u64,
    pub transaction: Transaction,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Meta {
    pub compute_units_consumed: u64,
    pub err: Option<Value>,
    pub fee: u64,
    pub inner_instructions: Vec<Value>,
    pub log_messages: Vec<Value>,
    pub post_balances: Vec<u64>,
    pub post_token_balances: Vec<Value>,
    pub pre_balances: Vec<u64>,
    pub pre_token_balances: Vec<Value>,
    pub rewards: Vec<Value>,
    pub status: Status,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Status {
    #[serde(rename = "Ok")]
    pub ok: Option<Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Transaction {
    pub message: Message,
    pub signatures: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub account_keys: Vec<AccountKey>,
    pub instructions: Vec<Instruction>,
    pub recent_blockhash: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct AccountKey {
    pub pubkey: String,
    pub signer: bool,
    pub source: String,
    pub writable: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Instruction {
    #[serde(flatten)]
    pub parsed: Option<Value>,
    pub program: Option<String>,
    pub program_id: String,
    pub stack_height: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ParsedValue {
    pub parsed: Value,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ParsedInstruction {
    pub parsed: Value,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum InstructionEnum {
    Transfer(ParsedIns),
    Burn(ParsedBurnChecked),
    Memo(String),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ParsedIns {
    pub info: Value,
    #[serde(rename = "type")]
    pub instr_type: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Transfer {
    pub destination: String,
    pub lamports: u64,
    pub source: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ParsedBurnChecked {
    pub info: BurnChecked,
    #[serde(rename = "type")]
    pub instr_type: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BurnChecked {
    pub account: String,
    pub authority: String,
    pub mint: String,
    pub token_amount: TokenAmount,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ParsedBurn {
    pub info: Burn,
    #[serde(rename = "type")]
    pub instr_type: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Burn {
    pub account: String,
    pub amount: String,
    pub authority: String,
    pub mint: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TokenAmount {
    pub amount: String,
    pub decimals: u8,
    pub ui_amount: f64,
    pub ui_amount_string: String,
}
