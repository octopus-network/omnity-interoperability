use std::collections::HashMap;

use candid::Principal;
use cosmoswasm_route::{
    business::add_new_token::add_new_token,
    cw::client::CosmosWasmClient,
    lifecycle::{self, init::InitArgs},
    schnorr::cw_schnorr_public_key,
    state::{self},
};
use cosmrs::tendermint;
use ic_cdk::{
    api::{
        call::{CallResult, RejectionCode},
        management_canister::http_request::HttpResponse,
    },
    init, update,
};
use omnity_types::{log::init_log, Token};

#[init]
pub fn init(args: InitArgs) {
    lifecycle::init::init(args);

    // init_log(Some(init_stable_log()));
}

fn check_anonymous_caller() {
    if ic_cdk::caller() == Principal::anonymous() {
        panic!("anonymous caller not allowed")
    }
}

#[update]
async fn generate_ticket(tx_hash: String) -> Result {}

// #[update]
// pub async fn test_cosmos_tx() {
//     let url = state::read_state(|state| state.cw_url);
//     let chain_id = state::read_state(|state| state.chain_id);
//     let client = CosmosWasmClient::new(url, chain_id);
//     let contract_id = state::read_state(|state| state.cw_port_contract_address);

//     client.execute_msg(
//         contract_id,
//         msg,
//         sender_public_key,
//         sender_account_id,
//         key_id
//     );

// }

#[update]
pub async fn cosmos_address() -> Result<String, String> {
    let schnorr_public_key = cw_schnorr_public_key()
        .await
        .map_err(|e| serde_json::to_string(&e).unwrap())?;
    let tendermint_public_key = tendermint::public_key::PublicKey::from_raw_secp256k1(
        schnorr_public_key.public_key.as_slice(),
    )
    .unwrap();
    let sender_public_key = cosmrs::crypto::PublicKey::from(tendermint_public_key);
    let sender_account_id = sender_public_key.account_id("osmo").unwrap();
    Ok(sender_account_id.to_string())
}

#[update]
pub async fn test_add_token() -> Result<HttpResponse, String> {
    add_new_token(Token {
        token_id: "token_id".to_string(),
        name: "name".to_string(),
        symbol: "symbol".to_string(),
        decimals: 2u8,
        icon: None,
        metadata: HashMap::new(),
    })
    .await
    .map_err(|e| serde_json::to_string(&e).unwrap())
}

fn main() {}

ic_cdk::export_candid!();
