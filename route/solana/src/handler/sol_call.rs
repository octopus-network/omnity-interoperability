use std::str::FromStr;

use crate::call_error::Reason;

use crate::{call_error::CallError, state::read_state};

use ic_canister_log::log;
use ic_solana::logs::DEBUG;
use ic_solana::rpc_client::RpcResult;
use ic_solana::token::{SolanaClient, TokenInfo};
use ic_solana::types::{Pubkey, TransactionStatus};
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;
use serde_json::Value;
pub async fn solana_client() -> SolanaClient {
    let (chain_id, schnorr_key_name, schnorr_canister, sol_canister) = read_state(|s| {
        (
            s.chain_id.to_owned(),
            s.schnorr_key_name.to_owned(),
            s.schnorr_canister,
            s.sol_canister,
        )
    });

    let payer = eddsa_public_key()
        .await
        .map_err(|message| CallError {
            method: "eddsa_public_key".to_string(),
            reason: Reason::CanisterError(message),
        })
        .unwrap();

    // log!(
    //     DEBUG,
    //     "[tickets::solana_client] payer pub key: {:?} ",
    //     payer.to_string()
    // );

    let derived_path = vec![ByteBuf::from(chain_id.as_bytes())];
    SolanaClient {
        sol_canister_id: sol_canister,
        payer: payer,
        payer_derive_path: derived_path,
        chainkey_name: schnorr_key_name,
        schnorr_canister: schnorr_canister,
    }
}

// get account info
pub async fn get_account_info(account: String) -> Result<Option<String>, CallError> {
    let sol_client = solana_client().await;
    let account_info = sol_client
        .get_account_info(account.to_string())
        .await
        .map_err(|e| CallError {
            method: "[sol_call::get_account_info] get_account_info".to_string(),
            reason: Reason::CanisterError(e.to_string()),
        })?;
    log!(
        DEBUG,
        "[sol_call::get_account_info] account({}) info : {:?}",
        account,
        account_info,
    );

    Ok(account_info)
}

// create mint token account with token metadata
pub async fn create_mint_account(token_mint: Pubkey, req: TokenInfo) -> Result<String, CallError> {
    let sol_client = solana_client().await;

    let signature = sol_client
        .create_mint_with_metadata(token_mint, req)
        .await
        .map_err(|e| CallError {
            method: "[sol_call::create_mint_account] create_mint_with_metadata".to_string(),
            reason: Reason::CanisterError(e.to_string()),
        })?;

    log!(
        DEBUG,
        "[sol_call::create_mint_account] mint account signature: {:?}",
        signature.to_string()
    );

    Ok(signature.to_string())
}

// get or create associated account
pub async fn create_ata(to_account: String, token_mint: String) -> Result<String, CallError> {
    let to_account = Pubkey::from_str(to_account.as_str()).expect("Invalid to_account address");
    let token_mint = Pubkey::from_str(token_mint.as_str()).expect("Invalid token_mint address");

    let sol_client = solana_client().await;
    let associated_token_account = sol_client
        .create_associated_token_account(&to_account, &token_mint)
        .await
        .map_err(|e| CallError {
            method: "create_associated_token_account".to_string(),
            reason: Reason::CanisterError(e.to_string()),
        })?;

    log!(DEBUG,
        "[solana_client::get_or_create_ata] wallet address: {:?}, token_mint: {:?}, and the associated token account: {:?} ",
        to_account.to_string(),
        token_mint.to_string(),
        associated_token_account.to_string()
    );
    Ok(associated_token_account.to_string())
}

pub async fn mint_to(
    associated_account: String,
    amount: u64,
    token_mint: String,
) -> Result<String, CallError> {
    let sol_client = solana_client().await;
    let associated_account =
        Pubkey::try_from(associated_account.as_str()).expect("Invalid receiver address");
    let token_mint = Pubkey::try_from(token_mint.as_str()).expect("Invalid receiver address");
    let signature = sol_client
        .mint_to(associated_account, amount, token_mint)
        .await
        .map_err(|e| CallError {
            method: "mint_to".to_string(),
            reason: Reason::CanisterError(e.to_string()),
        })?;

    log!(
        DEBUG,
        "[tickets::mint_to] mint successful and the signature is : {}",
        signature
    );
    Ok(signature)
}

// create mint token account with token metadata
pub async fn update_token_metadata(
    token_mint: String,
    req: TokenInfo,
) -> Result<String, CallError> {
    let sol_client = solana_client().await;
    let token_mint = Pubkey::from_str(&token_mint).expect("Invalid token mint address");

    let signature = sol_client
        .update_metadata(token_mint, req)
        .await
        .map_err(|e| CallError {
            method: "[sol_call::update_token_metadata] update_token_metadata".to_string(),
            reason: Reason::CanisterError(e.to_string()),
        })?;

    log!(
        DEBUG,
        "[sol_call::update_token_metadata] signature: {:?}",
        signature
    );

    Ok(signature.to_string())
}

// transfer from signer or payer
pub async fn transfer_to(to_account: String, amount: u64) -> Result<String, CallError> {
    let sol_client = solana_client().await;
    let token_mint = Pubkey::from_str(&to_account).expect("Invalid token mint address");

    let signature = sol_client
        .transfer_to(token_mint, amount)
        .await
        .map_err(|e| CallError {
            method: "[sol_call::transfer_to] transfer_to".to_string(),
            reason: Reason::CanisterError(e.to_string()),
        })?;

    log!(DEBUG, "[sol_call::transfer_to] signature: {:?}", signature);

    Ok(signature.to_string())
}

// query solana tx signature status
pub async fn get_signature_status(
    signatures: Vec<String>,
) -> Result<Vec<TransactionStatus>, CallError> {
    let sol_canister = read_state(|s| s.sol_canister);

    let response: Result<(RpcResult<String>,), _> =
        ic_cdk::call(sol_canister, "sol_getSignatureStatuses", (signatures,)).await;
    let tx_status = response
        .map_err(|(code, message)| CallError {
            method: "sol_getSignatureStatuses".to_string(),
            reason: Reason::from_reject(code, message),
        })?
        .0
        .map_err(|rpc_error| CallError {
            method: "sol_getSignatureStatuses".to_string(),
            reason: Reason::CanisterError(rpc_error.to_string()),
        })?;

    log!(DEBUG, "call sol_getSignatureStatuses resp: {:?}", tx_status);

    let status =
        serde_json::from_str::<Vec<TransactionStatus>>(&tx_status).map_err(|err| CallError {
            method: "sol_getSignatureStatuses".to_string(),
            reason: Reason::CanisterError(err.to_string()),
        })?;
    log!(DEBUG, "call sol_getSignatureStatuses staus: {:?}", status);
    Ok(status)
}

pub async fn eddsa_public_key() -> Result<Pubkey, String> {
    let (chain_id, schnorr_key_name, schnorr_canister) = read_state(|s| {
        (
            s.chain_id.to_owned(),
            s.schnorr_key_name.to_owned(),
            s.schnorr_canister,
        )
    });
    let derived_path = vec![ByteBuf::from(chain_id.as_bytes())];

    let pk =
        ic_solana::eddsa::eddsa_public_key(schnorr_canister, schnorr_key_name, derived_path).await;
    Pubkey::try_from(pk.as_slice()).map_err(|e| e.to_string())
}

pub async fn sign(msg: String) -> Result<String, String> {
    let (chain_id, schnorr_key_name, schnorr_canister) = read_state(|s| {
        (
            s.chain_id.to_owned(),
            s.schnorr_key_name.to_owned(),
            s.schnorr_canister,
        )
    });
    let derived_path = vec![ByteBuf::from(chain_id.as_bytes())];
    let msg = msg.as_bytes().to_vec();
    let signature =
        ic_solana::eddsa::sign_with_eddsa(schnorr_canister, schnorr_key_name, derived_path, msg)
            .await;
    let sig = String::from_utf8_lossy(&signature).to_string();
    Ok(sig)
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TransactionDetail {
    pub block_time: Option<u64>,
    pub meta: Meta,
    pub slot: u64,
    pub transaction: Transaction,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Meta {
    pub compute_units_consumed: u64,
    pub err: Option<Value>,
    pub fee: u64,
    pub inner_instructions: Vec<Value>,
    pub log_messages: Vec<String>,
    pub post_balances: Vec<u64>,
    pub post_token_balances: Vec<Value>,
    pub pre_balances: Vec<u64>,
    pub pre_token_balances: Vec<Value>,
    pub rewards: Vec<Value>,
    pub status: Status,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Status {
    #[serde(rename = "Ok")]
    pub ok: Option<Value>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Transaction {
    pub message: Message,
    pub signatures: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub account_keys: Vec<AccountKey>,
    pub instructions: Vec<Instruction>,
    pub recent_blockhash: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AccountKey {
    pub pubkey: String,
    pub signer: bool,
    pub source: String,
    pub writable: bool,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Instruction {
    #[serde(flatten)]
    pub parsed: Value,
    pub program: String,
    pub program_id: String,
    pub stack_height: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug)]
// #[serde(untagged)]
pub struct ParsedValue {
    pub parsed: Value,
    // pub parsed: InstructionEnum,
}

#[derive(Serialize, Deserialize, Debug)]
// #[serde(untagged)]
pub struct ParsedInstruction {
    pub parsed: Value,
    // Memo(String),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum InstructionEnum {
    Transfer(ParsedIns),
    Burn(ParsedBurn),
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
pub struct ParsedBurn {
    pub info: Burn,
    #[serde(rename = "type")]
    pub instr_type: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Burn {
    pub account: String,
    pub authority: String,
    pub mint: String,
    pub token_amount: TokenAmount,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TokenAmount {
    pub amount: String,
    pub decimals: u8,
    pub ui_amount: f64,
    pub ui_amount_string: String,
}

#[cfg(test)]
mod test {
    use super::*;
    use candid::Principal;
    use ic_solana::rpc_client::JsonRpcResponse;
    use serde_json::from_value;

    #[test]
    fn test_parse_transfer_with_memo_tx() {
        let json_data = r#"
        {
            "jsonrpc": "2.0",
            "result": {
                "blockTime": 1721963687,
                "meta": {
                    "computeUnitsConsumed": 7350,
                    "err": null,
                    "fee": 5000,
                    "innerInstructions": [],
                    "logMessages": [
                        "Program 11111111111111111111111111111111 invoke [1]",
                        "Program 11111111111111111111111111111111 success",
                        "Program MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr invoke [1]",
                        "Program log: Memo (len 16): \"receiver_address\"",
                        "Program MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr consumed 7200 of 399850 compute units",
                        "Program MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr success"
                    ],
                    "postBalances": [
                        5999995000,
                        12008970000,
                        1,
                        521498880
                    ],
                    "postTokenBalances": [],
                    "preBalances": [
                        8000000000,
                        10008970000,
                        1,
                        521498880
                    ],
                    "preTokenBalances": [],
                    "rewards": [],
                    "status": {
                        "Ok": null
                    }
                },
                "slot": 314272704,
                "transaction": {
                    "message": {
                        "accountKeys": [
                            {
                                "pubkey": "74SqAGc8wHgkwNx2Hqiz1UdKkZL1gCCvsRRwN2tSm8Ny",
                                "signer": true,
                                "source": "transaction",
                                "writable": true
                            },
                            {
                                "pubkey": "3gghk7mHWtFsJcg6EZGK7sbHj3qW6ExUdZLs9q8GRjia",
                                "signer": false,
                                "source": "transaction",
                                "writable": true
                            },
                            {
                                "pubkey": "11111111111111111111111111111111",
                                "signer": false,
                                "source": "transaction",
                                "writable": false
                            },
                            {
                                "pubkey": "MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr",
                                "signer": false,
                                "source": "transaction",
                                "writable": false
                            }
                        ],
                        "instructions": [
                            {
                                "parsed": {
                                    "info": {
                                        "destination": "3gghk7mHWtFsJcg6EZGK7sbHj3qW6ExUdZLs9q8GRjia",
                                        "lamports": 2000000000,
                                        "source": "74SqAGc8wHgkwNx2Hqiz1UdKkZL1gCCvsRRwN2tSm8Ny"
                                    },
                                    "type": "transfer"
                                },
                                "program": "system",
                                "programId": "11111111111111111111111111111111",
                                "stackHeight": null
                            },
                            {
                                "parsed": "receiver_address",
                                "program": "spl-memo",
                                "programId": "MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr",
                                "stackHeight": null
                            }
                        ],
                        "recentBlockhash": "BVoPc2NaRNnGBrFssmapBZTycQGyXzxtFn1Uciy52GTT"
                    },
                    "signatures": [
                        "zPTNV4iYR4xdtMupgkFfBYuL99VpdByNGjahNrMjRfWr2FWCRJeMiq3za5pSWT1Jj8z9bG3fBknWfmdL7XFRxud"
                    ]
                }
            },
            "id": 0
        }
        "#;

        let transaction_response =
            serde_json::from_str::<JsonRpcResponse<TransactionDetail>>(json_data).unwrap();
        // let transaction_response: JsonRpcResponse = serde_json::from_str(json_data).unwrap();

        println!("transaction_response: {:#?}", transaction_response);
        for instruction in &transaction_response
            .result
            .unwrap()
            .transaction
            .message
            .instructions
        {
            if let Ok(parsed_instr) = from_value::<ParsedValue>(instruction.parsed.clone()) {
                println!("Parsed Instruction: {:#?}", parsed_instr);
            } else if let Ok(parsed_str) = from_value::<String>(instruction.parsed.clone()) {
                println!("Parsed String: {:#?}", parsed_str);
            } else {
                println!("Unknown Parsed Value: {:#?}", instruction.parsed);
            }
        }
    }

    #[test]
    fn test_parse_burn_with_memo_tx() {
        let json_data = r#"
        {
            "jsonrpc": "2.0",
            "result": {
                "blockTime": 1722149061,
                "meta": {
                    "computeUnitsConsumed": 36589,
                    "err": null,
                    "fee": 5000,
                    "innerInstructions": [],
                    "logMessages": [
                        "Program MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr invoke [1]",
                        "Program log: Signed by 3gghk7mHWtFsJcg6EZGK7sbHj3qW6ExUdZLs9q8GRjia",
                        "Program log: Memo (len 44): \"3gghk7mHWtFsJcg6EZGK7sbHj3qW6ExUdZLs9q8GRjia\"",
                        "Program MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr consumed 30755 of 400000 compute units",
                        "Program MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr success",
                        "Program TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb invoke [1]",
                        "Program log: Instruction: BurnChecked",
                        "Program TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb consumed 5834 of 369245 compute units",
                        "Program TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb success"
                    ],
                    "postBalances": [
                        12008965000,
                        3883680,
                        2074080,
                        521498880,
                        1141440
                    ],
                    "postTokenBalances": [
                        {
                            "accountIndex": 2,
                            "mint": "AN2n5RYpqH9FfgD5zHFZS2wkezPTAhrukbPYvbx4ZEAj",
                            "owner": "3gghk7mHWtFsJcg6EZGK7sbHj3qW6ExUdZLs9q8GRjia",
                            "programId": "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb",
                            "uiTokenAmount": {
                                "amount": "90000000000",
                                "decimals": 9,
                                "uiAmount": 90.0,
                                "uiAmountString": "90"
                            }
                        }
                    ],
                    "preBalances": [
                        12008970000,
                        3883680,
                        2074080,
                        521498880,
                        1141440
                    ],
                    "preTokenBalances": [
                        {
                            "accountIndex": 2,
                            "mint": "AN2n5RYpqH9FfgD5zHFZS2wkezPTAhrukbPYvbx4ZEAj",
                            "owner": "3gghk7mHWtFsJcg6EZGK7sbHj3qW6ExUdZLs9q8GRjia",
                            "programId": "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb",
                            "uiTokenAmount": {
                                "amount": "100000000000",
                                "decimals": 9,
                                "uiAmount": 100.0,
                                "uiAmountString": "100"
                            }
                        }
                    ],
                    "rewards": [],
                    "status": {
                        "Ok": null
                    }
                },
                "slot": 314771079,
                "transaction": {
                    "message": {
                        "accountKeys": [
                            {
                                "pubkey": "3gghk7mHWtFsJcg6EZGK7sbHj3qW6ExUdZLs9q8GRjia",
                                "signer": true,
                                "source": "transaction",
                                "writable": true
                            },
                            {
                                "pubkey": "AN2n5RYpqH9FfgD5zHFZS2wkezPTAhrukbPYvbx4ZEAj",
                                "signer": false,
                                "source": "transaction",
                                "writable": true
                            },
                            {
                                "pubkey": "D58qMHmDAoEaviG8s9VmGwRhcw2z1apJHt6RnPtgxdVj",
                                "signer": false,
                                "source": "transaction",
                                "writable": true
                            },
                            {
                                "pubkey": "MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr",
                                "signer": false,
                                "source": "transaction",
                                "writable": false
                            },
                            {
                                "pubkey": "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb",
                                "signer": false,
                                "source": "transaction",
                                "writable": false
                            }
                        ],
                        "instructions": [
                            {
                                "parsed": "3gghk7mHWtFsJcg6EZGK7sbHj3qW6ExUdZLs9q8GRjia",
                                "program": "spl-memo",
                                "programId": "MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr",
                                "stackHeight": null
                            },
                            {
                                "parsed": {
                                    "info": {
                                        "account": "D58qMHmDAoEaviG8s9VmGwRhcw2z1apJHt6RnPtgxdVj",
                                        "authority": "3gghk7mHWtFsJcg6EZGK7sbHj3qW6ExUdZLs9q8GRjia",
                                        "mint": "AN2n5RYpqH9FfgD5zHFZS2wkezPTAhrukbPYvbx4ZEAj",
                                        "tokenAmount": {
                                            "amount": "10000000000",
                                            "decimals": 9,
                                            "uiAmount": 10.0,
                                            "uiAmountString": "10"
                                        }
                                    },
                                    "type": "burnChecked"
                                },
                                "program": "spl-token",
                                "programId": "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb",
                                "stackHeight": null
                            }
                        ],
                        "recentBlockhash": "HXnTGc3GHrcAuDkAJKyH7wStMii51vYYuMyBGpkAMt61"
                    },
                    "signatures": [
                        "5FHvSDvAmsUnyBRurtsJ3RjMz45CtqUjBP5FvQBQiXBCHfXwb3xqP7cBXGnuDepeGwCR8cE51NJVZY2GHms4GG1Z"
                    ]
                }
            },
            "id": 1
        }
        "#;

        let transaction_response =
            serde_json::from_str::<JsonRpcResponse<TransactionDetail>>(json_data).unwrap();

        println!("transaction_response: {:#?}", transaction_response);
        for instruction in &transaction_response
            .result
            .unwrap()
            .transaction
            .message
            .instructions
        {
            if let Ok(parsed_instr) = from_value::<ParsedValue>(instruction.parsed.clone()) {
                println!("Parsed Instruction: {:#?}", parsed_instr);
            } else if let Ok(parsed_str) = from_value::<String>(instruction.parsed.clone()) {
                println!("Parsed String: {:#?}", parsed_str);
            } else {
                println!("Unknown Parsed Value: {:#?}", instruction.parsed);
            }
        }
    }

    #[test]
    fn test_parse_transfer_burn_with_memo_tx() {
        let json_data = r#"
        {
            "jsonrpc": "2.0",
            "result": {
                "blockTime": 1722149061,
                "meta": {
                    "computeUnitsConsumed": 36589,
                    "err": null,
                    "fee": 5000,
                    "innerInstructions": [],
                    "logMessages": [
                        "Program MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr invoke [1]",
                        "Program log: Signed by 3gghk7mHWtFsJcg6EZGK7sbHj3qW6ExUdZLs9q8GRjia",
                        "Program log: Memo (len 44): \"3gghk7mHWtFsJcg6EZGK7sbHj3qW6ExUdZLs9q8GRjia\"",
                        "Program MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr consumed 30755 of 400000 compute units",
                        "Program MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr success",
                        "Program TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb invoke [1]",
                        "Program log: Instruction: BurnChecked",
                        "Program TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb consumed 5834 of 369245 compute units",
                        "Program TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb success"
                    ],
                    "postBalances": [
                        12008965000,
                        3883680,
                        2074080,
                        521498880,
                        1141440
                    ],
                    "postTokenBalances": [
                        {
                            "accountIndex": 2,
                            "mint": "AN2n5RYpqH9FfgD5zHFZS2wkezPTAhrukbPYvbx4ZEAj",
                            "owner": "3gghk7mHWtFsJcg6EZGK7sbHj3qW6ExUdZLs9q8GRjia",
                            "programId": "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb",
                            "uiTokenAmount": {
                                "amount": "90000000000",
                                "decimals": 9,
                                "uiAmount": 90.0,
                                "uiAmountString": "90"
                            }
                        }
                    ],
                    "preBalances": [
                        12008970000,
                        3883680,
                        2074080,
                        521498880,
                        1141440
                    ],
                    "preTokenBalances": [
                        {
                            "accountIndex": 2,
                            "mint": "AN2n5RYpqH9FfgD5zHFZS2wkezPTAhrukbPYvbx4ZEAj",
                            "owner": "3gghk7mHWtFsJcg6EZGK7sbHj3qW6ExUdZLs9q8GRjia",
                            "programId": "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb",
                            "uiTokenAmount": {
                                "amount": "100000000000",
                                "decimals": 9,
                                "uiAmount": 100.0,
                                "uiAmountString": "100"
                            }
                        }
                    ],
                    "rewards": [],
                    "status": {
                        "Ok": null
                    }
                },
                "slot": 314771079,
                "transaction": {
                    "message": {
                        "accountKeys": [
                            {
                                "pubkey": "3gghk7mHWtFsJcg6EZGK7sbHj3qW6ExUdZLs9q8GRjia",
                                "signer": true,
                                "source": "transaction",
                                "writable": true
                            },
                            {
                                "pubkey": "AN2n5RYpqH9FfgD5zHFZS2wkezPTAhrukbPYvbx4ZEAj",
                                "signer": false,
                                "source": "transaction",
                                "writable": true
                            },
                            {
                                "pubkey": "D58qMHmDAoEaviG8s9VmGwRhcw2z1apJHt6RnPtgxdVj",
                                "signer": false,
                                "source": "transaction",
                                "writable": true
                            },
                            {
                                "pubkey": "MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr",
                                "signer": false,
                                "source": "transaction",
                                "writable": false
                            },
                            {
                                "pubkey": "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb",
                                "signer": false,
                                "source": "transaction",
                                "writable": false
                            }
                        ],
                        "instructions": [
                            {
                                "parsed": "3gghk7mHWtFsJcg6EZGK7sbHj3qW6ExUdZLs9q8GRjia",
                                "program": "spl-memo",
                                "programId": "MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr",
                                "stackHeight": null
                            },
                            {
                                "parsed": {
                                    "info": {
                                        "account": "D58qMHmDAoEaviG8s9VmGwRhcw2z1apJHt6RnPtgxdVj",
                                        "authority": "3gghk7mHWtFsJcg6EZGK7sbHj3qW6ExUdZLs9q8GRjia",
                                        "mint": "AN2n5RYpqH9FfgD5zHFZS2wkezPTAhrukbPYvbx4ZEAj",
                                        "tokenAmount": {
                                            "amount": "10000000000",
                                            "decimals": 9,
                                            "uiAmount": 10.0,
                                            "uiAmountString": "10"
                                        }
                                    },
                                    "type": "burnChecked"
                                },
                                "program": "spl-token",
                                "programId": "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb",
                                "stackHeight": null
                            },
                            {
                                "parsed": {
                                    "info": {
                                        "destination": "3gghk7mHWtFsJcg6EZGK7sbHj3qW6ExUdZLs9q8GRjia",
                                        "lamports": 2000000000,
                                        "source": "74SqAGc8wHgkwNx2Hqiz1UdKkZL1gCCvsRRwN2tSm8Ny"
                                    },
                                    "type": "transfer"
                                },
                                "program": "system",
                                "programId": "11111111111111111111111111111111",
                                "stackHeight": null
                            }
                        ],
                        "recentBlockhash": "HXnTGc3GHrcAuDkAJKyH7wStMii51vYYuMyBGpkAMt61"
                    },
                    "signatures": [
                        "5FHvSDvAmsUnyBRurtsJ3RjMz45CtqUjBP5FvQBQiXBCHfXwb3xqP7cBXGnuDepeGwCR8cE51NJVZY2GHms4GG1Z"
                    ]
                }
            },
            "id": 1
        }
        "#;

        let transaction_response =
            serde_json::from_str::<JsonRpcResponse<TransactionDetail>>(json_data).unwrap();
        // let transaction_response: JsonRpcResponse = serde_json::from_str(json_data).unwrap();

        println!("transaction_response: {:#?}", transaction_response);
        for instruction in &transaction_response
            .result
            .unwrap()
            .transaction
            .message
            .instructions
        {
            if let Ok(parsed_instr) = from_value::<ParsedValue>(instruction.parsed.clone()) {
                println!("Parsed Instruction: {:#?}", parsed_instr);

                if let Ok(pi) = from_value::<ParsedIns>(parsed_instr.parsed.clone()) {
                    println!("Parsed transfer: {:#?}", pi);
                    if pi.instr_type.eq("transfer") {
                        let transfer = from_value::<Transfer>(pi.info.clone());
                        println!("Parsed transfer: {:#?}", transfer);
                    }
                    if pi.instr_type.eq("burnChecked") {
                        let burn = from_value::<Burn>(pi.info.clone());
                        println!("Parsed burn: {:#?}", burn);
                    }
                } else if let Ok(memo) = from_value::<String>(parsed_instr.parsed.clone()) {
                    println!("Parsed memo: {:?}", memo);
                } else {
                    println!("Unknown Parsed instruction: {:#?}", parsed_instr.parsed);
                }
            } else {
                println!("Unknown Parsed Value: {:#?}", instruction.parsed);
            }
        }
    }

    #[test]
    fn test_management_canister() {
        let principal = Principal::management_canister();
        println!("The management principal value is: {}", principal)
    }
}
