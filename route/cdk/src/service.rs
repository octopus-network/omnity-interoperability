use std::str::FromStr;

use anyhow::anyhow;
use candid::Principal;
use cketh_common::eth_rpc::LogEntry;
use cketh_common::eth_rpc_client::providers::RpcApi;
use ethers_core::abi::{ethereum_types, RawLog};
use ethers_core::types::U256;
use ethers_core::utils::keccak256;
use ic_cdk::api::management_canister::ecdsa::{ecdsa_public_key, EcdsaPublicKeyArgument};
use ic_cdk::{init, post_upgrade, pre_upgrade, query, update};
use itertools::Itertools;
use k256::PublicKey;

use crate::contract_types::{
    AbiSignature, DecodeLog, DirectiveExecuted, TokenBurned, TokenMinted, TokenTransportRequested,
};
use crate::contracts::{gen_eip1559_tx, gen_execute_directive_data, gen_mint_token_data};
use crate::eth_common::{
    broadcast, get_account_nonce, get_evm_finalized_height, get_gasprice, sign_transaction,
    EvmAddress,
};
use crate::evm_scan::{fetch_logs, handle_token_burn, handle_token_transport};
use crate::state::{
    key_derivation_path, key_id, minter_addr, mutate_state, read_state, replace_state,
    EvmRouteState, InitArgs, StateProfile,
};
use crate::types::{Directive, Seq, Ticket};
use crate::Error;

#[init]
fn init(args: InitArgs) {
    replace_state(EvmRouteState::init(args).expect("params error"));
    /*  set_timer_interval(Duration::from_secs(10), fetch_hub_periodic_task);
    set_timer_interval(Duration::from_secs(20), to_evm_task);
    set_timer_interval(Duration::from_secs(30), scan_evm_task);*/
}

#[pre_upgrade]
fn pre_upgrade() {
    read_state(|s| s.pre_upgrade());
}

#[post_upgrade]
fn post_upgrade() {
    EvmRouteState::post_upgrade();
    /*
    set_timer_interval(Duration::from_secs(10), fetch_hub_periodic_task);
    set_timer_interval(Duration::from_secs(20), to_evm_task);
    set_timer_interval(Duration::from_secs(30), scan_evm_task);*/
}

#[update]
async fn init_chain_pubkey(canister_id: Principal) -> String {
    let arg = EcdsaPublicKeyArgument {
        canister_id: Some(canister_id),
        derivation_path: key_derivation_path(),
        key_id: key_id(),
    };
    let res = ecdsa_public_key(arg)
        .await
        .map_err(|(_, e)| Error::ChainKeyError(e));
    match res {
        Ok((t,)) => {
            mutate_state(|s| s.pubkey = t.public_key.clone());
            hex::encode(t.public_key)
        }
        Err(e) => e.to_string(),
    }
}

#[query]
fn pubkey_and_evm_addr() -> (String, String) {
    let key = read_state(|s| s.pubkey.clone());
    let key_str = format!("0x{}", hex::encode(key.as_slice()));
    use k256::elliptic_curve::sec1::ToEncodedPoint;
    let key =
        PublicKey::from_sec1_bytes(key.as_slice()).expect("failed to parse the public key as SEC1");
    let point = key.to_encoded_point(false);
    // we re-encode the key to the decompressed representation.
    let point_bytes = point.as_bytes();
    assert_eq!(point_bytes[0], 0x04);
    let hash = keccak256(&point_bytes[1..]);
    let addr =
        ethers_core::utils::to_checksum(&ethereum_types::Address::from_slice(&hash[12..32]), None);
    (key_str, addr)
}

#[query]
fn route_state() -> StateProfile {
    read_state(|s| StateProfile::from(s))
}

#[update]
fn set_omnity_port_contract_addr(addr: String) {
    mutate_state(|s| s.omnity_port_contract = EvmAddress::from_str(addr.as_str()).unwrap());
}

#[update]
fn set_evm_chain_id(chain_id: u64) {
    mutate_state(|s| s.evm_chain_id = chain_id);
}

fn is_admin() -> Result<(), String> {
    let c = ic_cdk::caller();
    match read_state(|s| s.admin == c) {
        true => Ok(()),
        false => Err("permission deny".to_string()),
    }
}

#[update]
pub async fn test_send_directive_to_evm(d: Directive, seq: Seq) -> String {
    let data = gen_execute_directive_data(&d, U256::from(seq));
    let nonce = get_account_nonce(minter_addr()).await.unwrap();
    let fee = match d {
        Directive::AddToken(_) => Some(2000000u32),
        _ => None,
    };
    let tx = gen_eip1559_tx(data, get_gasprice().await.ok(), nonce, fee);
    let raw = sign_transaction(tx).await.unwrap();
    broadcast(raw).await.unwrap()
}

#[update]
pub async fn test_send_ticket_to_evm(t: Ticket) -> String {
    let data = gen_mint_token_data(&t);
    let nonce = get_account_nonce(minter_addr()).await.unwrap();
    let tx = gen_eip1559_tx(data, get_gasprice().await.ok(), nonce, None);
    let raw = sign_transaction(tx).await.unwrap();
    broadcast(raw).await.unwrap()
}
#[update]
pub async fn test_get_finalized_height() -> u64 {
    get_evm_finalized_height().await.unwrap()
}

#[update]
pub fn test_set_rpc_url(url: String) {
    mutate_state(|s| s.rpc_privders = vec![RpcApi {
        url,
        headers: None,
    }])
}
pub async fn test_scan(from: u64, to: u64) -> anyhow::Result<Vec<LogEntry>> {
    let contract_addr = read_state(|s| s.omnity_port_contract.to_hex());
    let logs = fetch_logs(from, to, contract_addr).await?;

    for l in logs.clone() {
        if l.removed {
            return Err(anyhow!("log is removed"));
        }
        let block = l.block_number.ok_or(anyhow!("block is pending"))?;
        let log_index = l.log_index.ok_or(anyhow!("log is pending"))?;
        let log_key = std::format!("{}-{}", block, log_index);
        let topic1 = l.topics.first().ok_or(anyhow!("topic is none"))?.0;
        let raw_log: RawLog = RawLog {
            topics: l.topics.iter().map(|topic| topic.0.into()).collect_vec(),
            data: l.data.0.clone(),
        };
        if topic1 == TokenBurned::signature_hash() {
            if read_state(|s| s.handled_evm_event.contains(&log_key)) {
                continue;
            }
            let token_burned = TokenBurned::decode_log(&raw_log)
                .map_err(|e| super::Error::ParseEventError(e.to_string()))?;
            handle_token_burn(&l, token_burned).await?;
        } else if topic1 == TokenMinted::signature_hash() {
            let token_mint = TokenMinted::decode_log(&raw_log)
                .map_err(|e| super::Error::ParseEventError(e.to_string()))?;
            mutate_state(|s| s.pending_tickets_map.remove(&token_mint.ticket_id));
        } else if topic1 == TokenTransportRequested::signature_hash() {
            if read_state(|s| s.handled_evm_event.contains(&log_key)) {
                continue;
            }
            let token_transport = TokenTransportRequested::decode_log(&raw_log)
                .map_err(|e| super::Error::ParseEventError(e.to_string()))?;
            handle_token_transport(&l, token_transport).await?;
        } else if topic1 == DirectiveExecuted::signature_hash() {
        }

        mutate_state(|s| s.handled_evm_event.insert(log_key));
    }
    mutate_state(|s| s.scan_start_height = to);
    Ok(logs)
}

#[update]
async fn test_scan_blocks(from: u64, to: u64) -> Vec<LogEntry> {
    test_scan(from, to).await.unwrap()
}

ic_cdk::export_candid!();
