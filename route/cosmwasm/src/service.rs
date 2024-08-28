use crate::*;
use business::{
    process_directive::process_directive_msg_task, ticket_task::process_ticket_msg_task,
};
use cosmrs::tendermint;
use cosmwasm::{
    client::{query_cw_public_key, OSMO_ACCOUNT_PREFIX},
    port::PortContractExecutor,
    TxHash,
};
use ic_cdk::{init, post_upgrade, query, update};
use ic_cdk_timers::set_timer_interval;
use lifecycle::init::InitArgs;
use memory::{init_stable_log, insert_redeem_ticket, mutate_state, read_state};
use omnity_types::log::init_log;
use std::time::Duration;

pub const INTERVAL_QUERY_DIRECTIVE: u64 = 60;
pub const INTERVAL_QUERY_TICKET: u64 = 5;

#[init]
pub async fn init(args: InitArgs) {
    lifecycle::init::init(args);

    init_log(Some(init_stable_log()));
}

#[update]
pub async fn cache_public_key_and_start_timer() {
    let public_key_response = query_cw_public_key()
        .await
        .expect("failed to query cw public key");

    mutate_state(|state| {
        state.cw_public_key_vec = Some(public_key_response.public_key.clone());
    });

    set_timer_interval(
        Duration::from_secs(INTERVAL_QUERY_DIRECTIVE),
        process_directive_msg_task,
    );
    set_timer_interval(
        Duration::from_secs(INTERVAL_QUERY_TICKET),
        process_ticket_msg_task,
    );
}

#[update]
pub async fn redeem(tx_hash: TxHash) -> std::result::Result<TicketId, String> {
    let port_contract_executor = PortContractExecutor::from_state();
    let event = port_contract_executor
        .query_redeem_token_event(tx_hash.clone())
        .await
        .map_err(|e| e.to_string())?;

    let (hub_principal, chain_id) = read_state(|s| (s.hub_principal, s.chain_id.clone()));
    let ticket = Ticket {
        ticket_id: tx_hash.clone(),
        ticket_type: omnity_types::TicketType::Normal,
        ticket_time: ic_cdk::api::time(),
        src_chain: chain_id,
        dst_chain: event.target_chain.clone(),
        action: omnity_types::TxAction::Redeem,
        token: event.token_id.clone(),
        amount: event.amount.to_string(),
        sender: Some(event.sender),
        receiver: event.receiver,
        memo: None,
    };

    hub::send_ticket(hub_principal, ticket.clone())
        .await
        .map_err(|e| e.to_string())?;

    insert_redeem_ticket(tx_hash, ticket.ticket_id.clone());

    Ok(ticket.ticket_id)
}

// fn check_anonymous_caller() {
//     if ic_cdk::caller() == Principal::anonymous() {
//         panic!("anonymous caller not allowed")
//     }
// }

#[update]
pub async fn tendermint_address() -> std::result::Result<String, String> {
    let ecdsa_public_key_response = query_cw_public_key().await.map_err(|e| e.to_string())?;

    let tendermint_public_key = tendermint::public_key::PublicKey::from_raw_secp256k1(
        &ecdsa_public_key_response.public_key.as_slice(),
    )
    .unwrap();

    let sender_public_key = cosmrs::crypto::PublicKey::from(tendermint_public_key);
    let sender_account_id = sender_public_key.account_id(OSMO_ACCOUNT_PREFIX).unwrap();
    Ok(sender_account_id.to_string())
}

#[query]
pub fn route_status()-> RouteState {
    read_state(|s| s.clone())
}

#[post_upgrade]
fn post_upgrade() {
    lifecycle::upgrade::post_upgrade();

    init_log(Some(init_stable_log()));
}

ic_cdk::export_candid!();
