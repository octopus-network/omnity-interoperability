use std::str::FromStr;

use crate::call_error::Reason;

use crate::state::mutate_state;
use crate::{call_error::CallError, state::read_state};

use crate::state::AtaKey;
use crate::state::TxStatus;
// use anyhow::Ok;
use ic_canister_log::log;
use ic_solana::types::tagged::UiAccount;
use solana_rpc::{SolanaClient, TokenInfo};

use crate::eddsa::KeyType;

use crate::logs::DEBUG;

use ic_solana::types::{Pubkey, TransactionStatus};
use ic_spl::token::constants::token_program_id;
use serde_bytes::ByteBuf;
pub mod solana_rpc;
use crate::handler::mint_token::MintTokenRequest;

pub async fn solana_client() -> SolanaClient {
    let (chain_id, schnorr_key_name, sol_canister, priority, key_type) = read_state(|s| {
        (
            s.chain_id.to_owned(),
            s.schnorr_key_name.to_owned(),
            s.sol_canister,
            s.priority.to_owned(),
            s.key_type.to_owned(),
        )
    });

    if let Some((_, client)) = read_state(|s| s.solana_client_cache.clone())
        .filter(|(cache_key_type, _)| *cache_key_type == key_type)
    {
        return client;
    }

    let payer = eddsa_public_key(key_type.to_owned())
        .await
        .map_err(|message| CallError {
            method: "eddsa_public_key".to_string(),
            reason: Reason::CanisterError(message),
        })
        .unwrap();

    let derived_path = vec![ByteBuf::from(chain_id.as_bytes())];

    let solana_client = SolanaClient {
        sol_canister_id: sol_canister,
        payer,
        payer_derive_path: derived_path,
        chainkey_name: schnorr_key_name,
        priority,
        key_type: key_type.clone(),
    };
    mutate_state(|s| {
        s.solana_client_cache = Some((key_type, solana_client.clone()));
    });
    solana_client
}

// get account info
pub async fn get_account_info(account: String) -> Result<Option<UiAccount>, CallError> {
    let sol_client = solana_client().await;

    let account_info = sol_client
        .get_account_info(account.to_string())
        .await
        .map_err(|e| CallError {
            method: "[solana_rpc::get_account_info] get_account_info".to_string(),
            reason: Reason::CanisterError(e.to_string()),
        })?;
    log!(
        DEBUG,
        "[solana_rpc::get_account_info] account({}) info : {:?}",
        account,
        account_info,
    );

    Ok(account_info)
}

// create mint token account with token metadata
pub async fn create_mint_account(token_mint: Pubkey, req: TokenInfo) -> Result<String, CallError> {
    let sol_client = solana_client().await;
    // update account.status to pending
    mutate_state(|s| {
        if let Some(account) = s.token_mint_accounts.get(&req.token_id).as_mut() {
            account.status = TxStatus::Pending;
            s.token_mint_accounts
                .insert(req.token_id.to_string(), account.to_owned());
        }
    });
    let signature: String = sol_client
        .create_mint_with_metaplex(token_mint, req)
        .await
        .map_err(|e| CallError {
            method: "[solana_rpc::create_mint_account] create_mint_with_metaplex".to_string(),
            reason: Reason::TxError(e.try_into().unwrap()),
        })?;

    log!(
        DEBUG,
        "[solana_rpc::create_mint_account] mint account signature: {:?}",
        signature.to_string()
    );

    Ok(signature.to_string())
}

// get or create associated account
pub async fn create_ata(to_account: String, token_mint: String) -> Result<String, CallError> {
    let to_account = Pubkey::from_str(to_account.as_str()).expect("Invalid to_account address");
    let token_mint = Pubkey::from_str(token_mint.as_str()).expect("Invalid token_mint address");
    let sol_client = solana_client().await;
    // update status to pending
    mutate_state(|s| {
        let ata_key = AtaKey {
            owner: to_account.to_string(),
            token_mint: token_mint.to_string(),
        };
        if let Some(account) = s.associated_accounts.get(&ata_key).as_mut() {
            account.status = TxStatus::Pending;
            s.associated_accounts.insert(ata_key, account.to_owned());
        }
    });

    let signature = sol_client
        .create_associated_token_account(&to_account, &token_mint, &token_program_id())
        .await
        .map_err(|e| CallError {
            method: "create_associated_token_account".to_string(),
            reason: Reason::TxError(e.try_into().unwrap()),
        })?;

    log!(DEBUG,
        "[solana_rpc::get_or_create_ata] wallet address: {:?}, token_mint: {:?}, and tx signature: {:?} ",
        to_account.to_string(),
        token_mint.to_string(),
        signature.to_string()
    );
    Ok(signature.to_string())
}

pub async fn mint_to_with_req(req: MintTokenRequest) -> Result<String, CallError> {
    let sol_client = solana_client().await;
    log!(
        DEBUG,
        "[Consolidation]solana_rpc::mint_to_with_req build solana client, ticket id: {}",
        req.ticket_id
    );
    let associated_account =
        Pubkey::try_from(req.associated_account.as_str()).expect("Invalid receiver address");
    let token_mint = Pubkey::try_from(req.token_mint.as_str()).expect("Invalid receiver address");

    // update status to pending
    mutate_state(|s| {
        let new_req = MintTokenRequest {
            ticket_id: req.ticket_id.to_owned(),
            associated_account: req.associated_account,
            amount: req.amount,
            token_mint: req.token_mint,
            status: TxStatus::Pending,
            signature: req.signature,
            retry_4_building: req.retry_4_building,
            retry_4_status: req.retry_4_status,
        };
        s.mint_token_requests
            .insert(req.ticket_id.to_owned(), new_req);
    });

    let signature = sol_client
        .mint_to(
            associated_account,
            req.amount,
            token_mint,
            token_program_id(),
        )
        .await
        .map_err(|e| CallError {
            method: "[solana_rpc::mint_to_with_req]".to_string(),
            reason: Reason::TxError(e.try_into().unwrap()),
        })?;
    log!(
        DEBUG,
        "[Consolidation]solana_rpc::mint_to_with_req client mint success, ticket id: {}",
        req.ticket_id
    );
    Ok(signature)
}

// create mint token account with token metadata
pub async fn update_with_metaplex(token_mint: String, req: TokenInfo) -> Result<String, CallError> {
    let sol_client = solana_client().await;
    let token_mint = Pubkey::from_str(&token_mint).expect("Invalid token mint address");

    let signature = sol_client
        .update_with_metaplex(token_mint, req)
        .await
        .map_err(|e| CallError {
            method: "[solana_rpc::update_with_metaplex] update_with_metaplex".to_string(),
            reason: Reason::TxError(e.try_into().unwrap()),
        })?;

    log!(
        DEBUG,
        "[solana_rpc::update_with_metaplex] signature: {:?}",
        signature
    );

    Ok(signature.to_string())
}

// transfer from signer or payer
pub async fn transfer_to(to_account: String, amount: u64) -> Result<String, CallError> {
    let sol_client = solana_client().await;
    let wallet_account = Pubkey::from_str(&to_account).expect("Invalid token mint address");

    let signature = sol_client
        .transfer_to(wallet_account, amount)
        .await
        .map_err(|e| CallError {
            method: "[solana_rpc::transfer_to] transfer_to".to_string(),
            reason: Reason::CanisterError(e.to_string()),
        })?;

    log!(
        DEBUG,
        "[solana_rpc::transfer_to] signature: {:?}",
        signature
    );

    Ok(signature.to_string())
}

// query solana tx signatures status
pub async fn get_signature_status(
    signatures: Vec<String>,
) -> Result<Vec<Option<TransactionStatus>>, CallError> {
    let sol_client = solana_client().await;

    let status = sol_client
        .get_signature_status(signatures.to_owned())
        .await
        .map_err(|e| CallError {
            method: "[solana_rpc::get_signature_status] get_signature_status".to_string(),
            reason: Reason::CanisterError(e.to_string()),
        })?;
    log!(
        DEBUG,
        "[solana_rpc::get_signature_status] signatures ({:?}) status: {:?}",
        signatures,
        status,
    );

    Ok(status)
}

pub async fn eddsa_public_key(key_type: KeyType) -> Result<Pubkey, String> {
    let (chain_id, schnorr_key_name) =
        read_state(|s| (s.chain_id.to_owned(), s.schnorr_key_name.to_owned()));
    let derived_path = vec![ByteBuf::from(chain_id.as_bytes())];

    let pk = crate::eddsa::eddsa_public_key(key_type, schnorr_key_name, derived_path).await;
    Pubkey::try_from(pk.as_slice()).map_err(|e| e.to_string())
}

pub async fn sign(msg: String, key_type: KeyType) -> Result<Vec<u8>, String> {
    let (chain_id, schnorr_key_name) =
        read_state(|s| (s.chain_id.to_owned(), s.schnorr_key_name.to_owned()));
    let derived_path = vec![ByteBuf::from(chain_id.as_bytes())];
    let msg = msg.as_bytes().to_vec();
    let signature =
        crate::eddsa::sign_with_eddsa(&key_type, schnorr_key_name, derived_path, msg).await;
    // let sig = String::from_utf8_lossy(&signature).to_string();
    Ok(signature)
}
