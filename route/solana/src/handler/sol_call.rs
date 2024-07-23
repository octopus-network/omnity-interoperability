use crate::call_error::Reason;

use crate::types::Token;
use crate::{call_error::CallError, state::read_state};
use candid::CandidType;

use ic_solana::rpc_client::RpcResult;
use ic_solana::types::{Pubkey, TransactionStatus};
use serde::{Deserialize, Serialize};

#[derive(CandidType, Deserialize, Serialize, Debug)]
pub struct CreateMintAccountRequest {
    pub token_pub_key: Pubkey,
    pub token: Token,
}

// create mint token account and init token metadata
pub async fn create_mint_account(token: Token) -> Result<String, CallError> {
    let sol_canister = read_state(|s| s.sol_canister);

    // derive pub key for sol token
    // let pub_key_reply = sol_token_address(token.token_id.to_string())
    //     .await
    //     .map_err(|message| CallError {
    //         method: "sol_token_address".to_string(),
    //         reason: Reason::CanisterError(message),
    //     })?;
    let pub_key_reply = vec![0; 32];
    let token_pub_key = Pubkey::try_from(pub_key_reply).expect("Invalid public key");

    let req = CreateMintAccountRequest {
        token_pub_key: token_pub_key.clone(),
        token: token.clone(),
    };
    // send tx(sol_create_mint_account) to solana
    let response: Result<(RpcResult<String>,), _> =
        ic_cdk::call(sol_canister, "sol_create_mint_account", (req,)).await;

    let signature = response
        .map_err(|(code, message)| CallError {
            method: "sol_create_mint_account".to_string(),
            reason: Reason::from_reject(code, message),
        })?
        .0
        .map_err(|rpc_error| CallError {
            method: "sol_create_mint_account".to_string(),
            reason: Reason::CanisterError(rpc_error.to_string()),
        })?;
    ic_cdk::println!("tx(sol_create_mint_account) signature: {}", signature);
    // mutate_state(|s| s.sol_token_address.inser)t(token.token_id, token_pub_key));
    Ok(token_pub_key.to_string())
}

#[derive(CandidType, Deserialize, Serialize, Debug)]
pub struct MintToRequest {
    pub sol_token_address: String,
    pub receiver: String,
    pub amount: u128,
}

//first, check receiver ATA ,create it if not exites
//then, mint token to receiver ATA
pub async fn mint_to(
    sol_token_address: String,
    receiver: String,
    amount: u128,
) -> Result<String, CallError> {
    let sol_canister = read_state(|s| s.sol_canister);
    let req = MintToRequest {
        sol_token_address,
        receiver,
        amount,
    };

    // send tx(sol_create_mint_account) to solana
    let response: Result<(RpcResult<String>,), _> =
        ic_cdk::call(sol_canister, "sol_mint_to", (req,)).await;
    let signature = response
        .map_err(|(code, message)| CallError {
            method: "sol_mint_to".to_string(),
            reason: Reason::from_reject(code, message),
        })?
        .0
        .map_err(|rpc_error| CallError {
            method: "sol_mint_to".to_string(),
            reason: Reason::CanisterError(rpc_error.to_string()),
        })?;
    ic_cdk::println!("tx(sol_mint_to) signature: {}", signature);
    // mutate_state(|s| s.sol_token_address.inser)t(token.token_id, token_pub_key));
    Ok(signature)
}

// query solana tx signature status and update txhash to hub
pub async fn get_signature_status(
    signatures: Vec<String>,
) -> Result<Vec<TransactionStatus>, CallError> {
    let sol_canister = read_state(|s| s.sol_canister);

    // send tx(sol_create_mint_account) to solana
    let response: Result<(RpcResult<Vec<TransactionStatus>>,), _> =
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

    ic_cdk::println!("sol_getSignatureStatuses result: {:?}", tx_status);
    // mutate_state(|s| s.sol_token_address.inser)t(token.token_id, token_pub_key));
    Ok(tx_status)
}
