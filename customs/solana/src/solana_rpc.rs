use crate::{
    address::fee_address_path,
    call_error::{CallError, Reason},
    port_native::{self, instruction::InstSerialize, port_address, vault_address},
    state::{mutate_state, read_state},
    transaction::{Transaction, TransactionDetail, TransactionStatus},
};
use ic_canister_log::log;
use ic_solana::{
    eddsa::KeyType,
    ic_log::ERROR,
    rpc_client::{JsonRpcResponse, RpcResult},
    token::{constants::system_program_id, SolanaClient},
    types::{AccountMeta, Instruction, Pubkey},
};
use serde_bytes::ByteBuf;

pub async fn query_transaction(signature: String) -> Result<Transaction, String> {
    let (rpc_list, min_resp_cnt) = read_state(|s| (s.rpc_list.clone(), s.min_response_count));
    let client = init_solana_client().await;
    let mut fut = Vec::with_capacity(rpc_list.len());
    for rpc_url in rpc_list {
        fut.push(async {
            client
                .query_transaction(signature.to_owned(), Some(rpc_url))
                .await
        });
    }

    let response_list = futures::future::join_all(fut).await;
    let mut transactions = vec![];
    for response in response_list {
        match response {
            Ok(resp) => match serde_json::from_str::<JsonRpcResponse<TransactionDetail>>(&resp) {
                Ok(t) => {
                    if let Some(e) = t.error {
                        return Err(format!("{}", e.message));
                    } else {
                        match t.result {
                            Some(tx_detail) => {
                                transactions.push(tx_detail.transaction);
                            }
                            None => return Err("result of query_transaction is None".into()),
                        }
                    }
                }
                Err(e) => {
                    log!(
                        ERROR,
                        "[query_transaction] serde_json::from_str error: {:?}",
                        e
                    );
                    continue;
                }
            },
            Err(e) => {
                log!(ERROR, "[query_transaction] response error: {:?}", e);
                continue;
            }
        }
    }
    if transactions.len() < min_resp_cnt as usize {
        return Err(format!(
            "not enough valid response, expected: {}, actual: {}",
            min_resp_cnt,
            transactions.len()
        ));
    }
    let first_tx = transactions.first().unwrap();
    if transactions.iter().any(|tx| tx != first_tx) {
        return Err("response is not all same".into());
    }
    Ok(first_tx.clone())
}

pub async fn get_signature_status(
    signatures: Vec<String>,
) -> Result<Vec<Option<TransactionStatus>>, CallError> {
    let (sol_canister, forward) = read_state(|s| (s.sol_canister, s.forward.to_owned()));

    let response: Result<(RpcResult<String>,), _> = ic_cdk::call(
        sol_canister,
        "sol_getSignatureStatuses",
        (signatures, forward),
    )
    .await;

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

    let status: Vec<Option<TransactionStatus>> = serde_json::from_str::<
        Vec<Option<TransactionStatus>>,
    >(&tx_status)
    .map_err(|err| CallError {
        method: "sol_getSignatureStatuses".to_string(),
        reason: Reason::CanisterError(err.to_string()),
    })?;
    Ok(status)
}

pub async fn init_solana_client() -> SolanaClient {
    if let Some(client) = read_state(|s| s.sol_client.clone()) {
        return client;
    }
    let (schnorr_key_name, sol_canister) =
        read_state(|s| (s.schnorr_key_name.to_owned(), s.sol_canister));

    let derived_path = fee_address_path();
    let forward: Option<String> = read_state(|s| s.forward.clone());
    let client = SolanaClient {
        sol_canister_id: sol_canister,
        payer: ecdsa_public_key(derived_path.clone()).await,
        payer_derive_path: derived_path,
        chainkey_name: schnorr_key_name,
        forward: forward,
        priority: None,
        key_type: KeyType::ChainKey,
    };
    mutate_state(|s| s.sol_client = Some(client.clone()));
    client
}

pub async fn init_port() -> Result<String, String> {
    let client = init_solana_client().await;
    let port_program_id = read_state(|s| s.port_program_id.clone());

    let (port, _) = port_address();
    let (_, vault_bump) = vault_address();

    let initialize = port_native::instruction::Initialize { vault_bump };
    let instruction = Instruction::new_with_bytes(
        port_program_id,
        &initialize.data(),
        vec![
            AccountMeta::new(port, false),
            AccountMeta::new(client.payer, true),
            AccountMeta::new_readonly(system_program_id(), false),
        ],
    );

    let signature = client
        .send_raw_transaction(
            &vec![instruction],
            vec![client.payer_derive_path.clone()],
            KeyType::ChainKey,
        )
        .await
        .map_err(|err| err.to_string())?;
    Ok(signature)
}

pub async fn redeem(ticket_id: String, receiver: Pubkey, amount: u64) -> Result<String, String> {
    let client = init_solana_client().await;
    let port_program_id = read_state(|s| s.port_program_id.clone());

    let (port, _) = port_address();
    let (vault, _) = vault_address();
    let (redeem_record, _) = Pubkey::find_program_address(
        &[&b"redeem"[..], port.as_ref(), ticket_id.as_bytes()],
        &port_program_id,
    );

    let initialize = port_native::instruction::Redeem { ticket_id, amount };
    let instruction = Instruction::new_with_bytes(
        port_program_id,
        &initialize.data(),
        vec![
            AccountMeta::new(port, false),
            AccountMeta::new(vault, false),
            AccountMeta::new(redeem_record, false),
            AccountMeta::new(client.payer, true),
            AccountMeta::new(receiver, false),
            AccountMeta::new_readonly(system_program_id(), false),
        ],
    );

    let signature = client
        .send_raw_transaction(
            &vec![instruction],
            vec![client.payer_derive_path.clone()],
            KeyType::ChainKey,
        )
        .await
        .map_err(|err| err.to_string())?;
    Ok(signature)
}

pub async fn ecdsa_public_key(derived_path: Vec<ByteBuf>) -> Pubkey {
    let schnorr_key_name = read_state(|s| s.schnorr_key_name.to_owned());

    let pk =
        ic_solana::eddsa::eddsa_public_key(KeyType::ChainKey, schnorr_key_name, derived_path).await;
    Pubkey::try_from(pk.as_slice()).unwrap()
}
