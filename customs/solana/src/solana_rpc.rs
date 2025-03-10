use std::str::FromStr;

use crate::{
    address::payer_address_path,
    port_native::{self, instruction::InstSerialize, port_address, vault_address},
    state::read_state,
    SYSTEM_PROGRAM_ID,
};
use ic_canister_log::log;
use ic_cdk::api::{call::call_with_payment, management_canister::http_request::HttpHeader};
use ic_solana::{
    eddsa::{sign_with_eddsa, KeyType},
    logs::DEBUG,
    rpc_client::{RpcApi, RpcConfig, RpcResult, RpcServices},
    types::{
        tagged::{EncodedConfirmedTransactionWithStatusMeta, EncodedTransaction, UiTransaction},
        AccountMeta, BlockHash, Instruction, Message, Pubkey, RpcBlockhash, RpcContextConfig,
        RpcSendTransactionConfig, RpcSignatureStatusConfig, RpcTransactionConfig, Signature,
        Transaction, TransactionStatus, UiTransactionEncoding,
    },
};
use serde_bytes::ByteBuf;

const CYCLE_COST: u64 = 10_000_000_000;

pub async fn query_transaction(signature: String) -> Result<UiTransaction, String> {
    let (sol_canister, providers) = read_state(|s| (s.sol_canister, s.providers.clone()));
    // We can directly access normal RPC because query_transaction does not have consensus issues.
    let source = RpcServices::Custom(
        providers
            .iter()
            .map(|p| RpcApi {
                network: p.rpc_url(),
                headers: None,
            })
            .collect(),
    );
    let tx_config = RpcTransactionConfig {
        encoding: Some(UiTransactionEncoding::Json),
        commitment: None,
        max_supported_transaction_version: None,
    };
    let result =
        call_with_payment::<_, (RpcResult<Option<EncodedConfirmedTransactionWithStatusMeta>>,)>(
            sol_canister,
            "sol_getTransaction",
            (
                source,
                None::<Option<RpcConfig>>,
                signature,
                Some(tx_config),
            ),
            CYCLE_COST,
        )
        .await
        .map_err(|(_, err)| err)?
        .0
        .map_err(|err| err.to_string())?;

    match result {
        None => Err("result of query_transaction is None".into()),
        Some(tx) => match tx.transaction.transaction {
            EncodedTransaction::Json(tx) => Ok(tx),
            _ => Err("invalid type of query_transaction".into()),
        },
    }
}

pub async fn send_transaction(
    instructions: &[Instruction],
    paths: Vec<Vec<ByteBuf>>,
) -> Result<String, String> {
    let blockhash = get_latest_block_hash().await?;
    log!(
        DEBUG,
        "[solana_client::send_raw_transaction] get_latest_blockhash : {:?}",
        blockhash
    );

    let message = Message::new_with_blockhash(
        instructions.iter().as_ref(),
        None,
        &BlockHash::from_str(&blockhash).unwrap(),
    );
    let mut tx = Transaction::new_unsigned(message);

    let (sol_canister, key_name) = read_state(|s| (s.sol_canister, s.schnorr_key_name.clone()));
    for i in 0..paths.len() {
        let signature = sign_with_eddsa(
            &KeyType::ChainKey,
            key_name.clone(),
            paths[i].clone(),
            tx.message_data(),
        )
        .await;
        tx.add_signature(i, Signature::try_from(signature).unwrap());
    }

    log!(
        DEBUG,
        "[solana_client::get_compute_units] signed_tx : {:?} and string : {:?}",
        tx,
        tx.to_string()
    );

    let rpc_list = proxy_rpc_api_list();
    let signature = call_with_payment::<_, (RpcResult<String>,)>(
        sol_canister,
        "sol_sendTransaction",
        (
            // Use idempotent-proxy to avoid sending transactions multiple times
            RpcServices::Custom(vec![rpc_list[0].clone()]),
            None::<Option<RpcConfig>>,
            tx.to_string(),
            None::<Option<RpcSendTransactionConfig>>,
        ),
        CYCLE_COST,
    )
    .await
    .map_err(|(_, err)| err)?
    .0
    .map_err(|err| err.to_string())?;

    Ok(signature)
}

pub async fn get_signature_status(
    signatures: Vec<String>,
) -> Result<Vec<Option<TransactionStatus>>, String> {
    let sol_canister = read_state(|s| s.sol_canister);
    let result = call_with_payment::<_, (RpcResult<Vec<Option<TransactionStatus>>>,)>(
        sol_canister,
        "sol_getSignatureStatuses",
        (
            // Using normal RPC will not allow consensus to be reached
            // due to inconsistent slots.
            RpcServices::Custom(proxy_rpc_api_list()),
            None::<Option<RpcConfig>>,
            signatures,
            Some(RpcSignatureStatusConfig {
                search_transaction_history: true,
            }),
        ),
        CYCLE_COST,
    )
    .await
    .map_err(|(_, err)| err)?
    .0
    .map_err(|err| err.to_string())?;

    Ok(result)
}

pub async fn init_port() -> Result<String, String> {
    let port_program_id = read_state(|s| s.port_program_id.clone());
    let payer = eddsa_public_key(payer_address_path()).await;

    let (port, _) = port_address();
    let (_, vault_bump) = vault_address();

    let initialize = port_native::instruction::Initialize { vault_bump };
    let instruction = Instruction::new_with_bytes(
        port_program_id,
        &initialize.data(),
        vec![
            AccountMeta::new(port, false),
            AccountMeta::new(payer, true),
            AccountMeta::new_readonly(Pubkey::from_str(SYSTEM_PROGRAM_ID).unwrap(), false),
        ],
    );

    let signature = send_transaction(&vec![instruction], vec![payer_address_path()])
        .await
        .map_err(|err| err.to_string())?;
    Ok(signature)
}

pub async fn redeem(ticket_id: String, receiver: Pubkey, amount: u64) -> Result<String, String> {
    let port_program_id = read_state(|s| s.port_program_id.clone());
    let payer = eddsa_public_key(payer_address_path()).await;

    let (port, _) = port_address();
    let (vault, _) = vault_address();
    let (redeem_record, _) = Pubkey::find_program_address(
        &[&b"redeem"[..], port.as_ref(), ticket_id.as_bytes()],
        &port_program_id,
    );

    let initialize = port_native::instruction::Redeem {
        ticket_id: ticket_id.clone(),
        amount,
    };
    let instruction = Instruction::new_with_bytes(
        port_program_id,
        &initialize.data(),
        vec![
            AccountMeta::new(port, false),
            AccountMeta::new(vault, false),
            AccountMeta::new(redeem_record, false),
            AccountMeta::new(payer, true),
            AccountMeta::new(receiver, false),
            AccountMeta::new_readonly(Pubkey::from_str(SYSTEM_PROGRAM_ID).unwrap(), false),
        ],
    );

    let signature = send_transaction(&vec![instruction], vec![payer_address_path()])
        .await
        .map_err(|err| err.to_string())?;
    log!(
        DEBUG,
        "[solana_custom] send raw transaction, ticket id: {}",
        ticket_id
    );
    Ok(signature)
}

pub async fn eddsa_public_key(derived_path: Vec<ByteBuf>) -> Pubkey {
    let schnorr_key_name = read_state(|s| s.schnorr_key_name.to_owned());

    let pk =
        ic_solana::eddsa::eddsa_public_key(KeyType::ChainKey, schnorr_key_name, derived_path).await;
    Pubkey::try_from(pk.as_slice()).unwrap()
}

async fn get_latest_block_hash() -> Result<String, String> {
    let sol_canister = read_state(|s| s.sol_canister);
    let rpc_list = proxy_rpc_api_list();
    let result = call_with_payment::<_, (RpcResult<RpcBlockhash>,)>(
        sol_canister,
        "sol_getLatestBlockhash",
        (
            RpcServices::Custom(vec![rpc_list[0].clone()]),
            None::<Option<RpcConfig>>,
            None::<Option<RpcContextConfig>>,
        ),
        CYCLE_COST,
    )
    .await
    .map_err(|(_, err)| err)?
    .0
    .map_err(|err| err.to_string())?;

    Ok(result.blockhash)
}

fn proxy_rpc_api_list() -> Vec<RpcApi> {
    let (proxy_rpc, providers) = read_state(|s| (s.proxy_rpc.clone(), s.providers.clone()));
    providers
        .iter()
        .map(|p| RpcApi {
            network: format!(
                "{}{}",
                proxy_rpc,
                p.api_key_param
                    .clone()
                    .map_or("".into(), |param| format!("/?{}", param))
            ),
            headers: Some(vec![HttpHeader {
                name: "x-forwarded-host".into(),
                value: p.host.clone(),
            }]),
        })
        .collect()
}
