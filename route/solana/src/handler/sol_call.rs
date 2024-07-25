use crate::call_error::Reason;

use crate::types::Token;
use crate::{call_error::CallError, state::read_state};

use ic_solana::rpc_client::RpcResult;
use ic_solana::token::{SolanaClient, TokenCreateInfo};
use ic_solana::types::{Pubkey, TransactionStatus};

use serde_bytes::ByteBuf;

// create mint token account and init token metadata
pub async fn create_mint_account(token: Token) -> Result<String, CallError> {
    let (sol_canister, schnorr_key_name, schnorr_canister) = read_state(|s| {
        (
            s.sol_canister,
            s.schnorr_key_name.to_owned(),
            s.schnorr_canister,
        )
    });

    let cur_canister_id = ic_cdk::api::id();
    let derived_path = vec![ByteBuf::from(cur_canister_id.as_slice())];

    let payer = cur_pub_key().await.map_err(|message| CallError {
        method: "cur_pub_key".to_string(),
        reason: Reason::CanisterError(message),
    })?;

    let sol_client = SolanaClient {
        sol_canister_id: sol_canister,
        payer: payer,
        payer_derive_path: derived_path,
        chainkey_name: schnorr_key_name,
        schnorr_canister: schnorr_canister,
    };
    let req = TokenCreateInfo {
        name: token.name.to_owned(),
        symbol: token.symbol.to_owned(),
        decimals: token.decimals,
        uri: token.icon.unwrap_or_default(),
    };

    let token_pub_key = sol_client
        .create_mint_with_metadata(req)
        .await
        .map_err(|e| CallError {
            method: "create_mint".to_string(),
            reason: Reason::CanisterError(e.to_string()),
        })?;

    ic_cdk::println!("create_mint result: {}", token_pub_key);

  
    Ok(token_pub_key.to_string())
}

//first, check receiver ATA ,create it if not exites
//then, mint token to receiver ATA
pub async fn mint_to(
    sol_token_address: String,
    receiver: String,
    amount: u64,
) -> Result<String, CallError> {
    let (sol_canister, schnorr_key_name, schnorr_canister) = read_state(|s| {
        (
            s.sol_canister,
            s.schnorr_key_name.to_owned(),
            s.schnorr_canister,
        )
    });

    let cur_canister_id = ic_cdk::api::id();
    let derived_path = vec![ByteBuf::from(cur_canister_id.as_slice())];

    let payer = cur_pub_key().await.map_err(|message| CallError {
        method: "cur_pub_key".to_string(),
        reason: Reason::CanisterError(message),
    })?;

    let sol_client = SolanaClient {
        sol_canister_id: sol_canister,
        payer: payer,
        payer_derive_path: derived_path,
        chainkey_name: schnorr_key_name,
        schnorr_canister: schnorr_canister,
    };
    let receiver = Pubkey::try_from(receiver.as_str()).expect("Invalid receiver address");
    let token_mint =
        Pubkey::try_from(sol_token_address.as_str()).expect("Invalid receiver address");
    let signature = sol_client
        .mint_to(receiver, amount, token_mint)
        .await
        .map_err(|e| CallError {
            method: "sol_mint_to".to_string(),
            reason: Reason::CanisterError(e.to_string()),
        })?;

    ic_cdk::println!("mint_to signature: {}", signature);
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

pub async fn cur_pub_key() -> Result<Pubkey, String> {
    let cur_canister_id = ic_cdk::api::id();
    let derived_path = vec![ByteBuf::from(cur_canister_id.as_slice())];
    let (schnorr_canister, key_name) =
        read_state(|s| (s.schnorr_canister, s.schnorr_key_name.to_owned()));
    let pk = ic_solana::eddsa::eddsa_public_key(schnorr_canister, key_name, derived_path).await;
    Pubkey::try_from(pk.as_slice()).map_err(|e| e.to_string())
}
