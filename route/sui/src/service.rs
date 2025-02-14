#![allow(unused)]

use crate::auth::{is_admin, set_perms, Permission};
use crate::call_error::{CallError, Reason};
use crate::constants::SUI_COIN;
use crate::guard::TaskType;
use crate::ic_log::{DEBUG, ERROR};
use crate::ic_sui::ck_eddsa::KeyType;
use crate::ic_sui::fastcrypto::encoding::Base64;
use crate::ic_sui::rpc_client::{RpcClient, RpcError};
use crate::ic_sui::sui_json_rpc_types::sui_object::SuiObjectDataOptions;
use crate::ic_sui::sui_json_rpc_types::sui_transaction::SuiTransactionBlockResponseOptions;
use crate::ic_sui::sui_json_rpc_types::SuiEvent;
use crate::ic_sui::sui_providers::Provider;
use crate::ic_sui::sui_types::base_types::{ObjectID, SuiAddress};
use crate::ic_sui::sui_types::digests::TransactionDigest;
use crate::ic_sui::{self};
use crate::memory::init_config;

use candid::Principal;
use ic_cdk::{init, post_upgrade, pre_upgrade, query, update};

use serde_json::json;

use crate::handler::gen_ticket::{
    self, query_tx_from_multi_rpc, send_ticket, GenerateTicketError, GenerateTicketOk,
    GenerateTicketReq,
};
use crate::handler::mint_token::{self};

use crate::handler::scheduler;
use crate::lifecycle::{self, RouteArg, UpgradeArgs};

use crate::config::{
    mutate_config, read_config, MultiRpcConfig, Seqs, SnorKeyType, SuiPortAction, SuiRouteConfig,
    KEY_TYPE_NAME,
};
use crate::state::{replace_state, SuiRouteState, SuiToken, TokenResp, UpdateType};
use crate::types::{TicketId, Token, TokenId};

use crate::service::mint_token::MintTokenRequest;

use crate::state::{mutate_state, read_state, TxStatus};
use crate::types::ChainState;
use crate::types::{Chain, ChainId, Ticket};
use ic_canister_log::log;

use crate::types::Factor;
use ic_canisters_http_types::{HttpRequest, HttpResponse, HttpResponseBuilder};
use ic_cdk::api::management_canister::http_request::{
    HttpResponse as TransformedHttpResponse, TransformArgs,
};

use std::str::FromStr;

use std::time::Duration;

async fn get_random_seed() -> [u8; 64] {
    match ic_cdk::api::management_canister::main::raw_rand().await {
        Ok(rand) => {
            let mut rand = rand.0;
            rand.extend(rand.clone());
            let rand: [u8; 64] = rand.try_into().expect("Expected a Vec of length 64");
            rand
        }
        Err(err) => {
            ic_cdk::trap(format!("Error getting random seed: {:?}", err).as_str());
        }
    }
}

#[init]
fn init(args: RouteArg) {
    log!(DEBUG, "init args: {:?}", args);
    match args {
        RouteArg::Init(args) => {
            lifecycle::init(args);
        }
        RouteArg::Upgrade(_) => {
            panic!("expected InitArgs got UpgradeArgs");
        }
    }
    // init seeds
    ic_cdk_timers::set_timer(Duration::ZERO, || {
        ic_cdk::spawn(async move {
            let seed = get_random_seed().await;
            mutate_state(|s| s.seeds.insert(KEY_TYPE_NAME.to_string(), seed));
        });
    });
}

#[pre_upgrade]
fn pre_upgrade() {
    log!(DEBUG, "begin to execute pre_upgrade ...");
    scheduler::stop_schedule(None);
    lifecycle::pre_upgrade();
    log!(DEBUG, "pre_upgrade end!");
}

#[post_upgrade]
fn post_upgrade(args: Option<RouteArg>) {
    log!(DEBUG, "begin to execute post_upgrade with :{:?}", args);
    let mut upgrade_arg: Option<UpgradeArgs> = None;
    if let Some(route_arg) = args {
        upgrade_arg = match route_arg {
            RouteArg::Upgrade(upgrade_args) => upgrade_args,
            RouteArg::Init(_) => panic!("expected Option<UpgradeArgs> got InitArgs."),
        };
    }

    lifecycle::post_upgrade(upgrade_arg);
    scheduler::start_schedule(None);
    log!(DEBUG, "upgrade successfully!");
}

// devops method
#[query(guard = "is_admin")]
pub async fn get_route_config() -> SuiRouteConfig {
    read_config(|s| s.get().to_owned())
}
// devops method
#[update(guard = "is_admin", hidden = true)]
pub fn start_schedule(tasks: Option<Vec<TaskType>>) {
    log!(DEBUG, "start schedule task: {:?} ... ", tasks);
    scheduler::start_schedule(tasks);
}

// devops method
#[update(guard = "is_admin", hidden = true)]
pub fn stop_schedule(tasks: Option<Vec<TaskType>>) {
    log!(DEBUG, "stop schedule task: {:?} ...", tasks);
    scheduler::stop_schedule(tasks);
}

// devops method
#[query(guard = "is_admin", hidden = true)]
pub async fn active_tasks() -> Vec<TaskType> {
    read_config(|s| s.get().active_tasks.iter().map(|t| t.to_owned()).collect())
}

// devops method
#[update(guard = "is_admin", hidden = true)]
pub async fn update_schnorr_key(key_name: String) {
    mutate_config(|s| {
        let mut config = s.get().to_owned();
        config.schnorr_key_name = key_name;
        s.set(config);
    })
}

// devops method
#[query(guard = "is_admin")]
pub async fn forward() -> Option<String> {
    read_config(|s| s.get().forward.to_owned())
}
// devops method
#[update(guard = "is_admin", hidden = true)]
pub async fn update_forward(forward: Option<String>) {
    mutate_config(|s| {
        let mut config = s.get().to_owned();
        config.forward = forward;
        s.set(config);
    })
}

// devops method
#[update(guard = "is_admin", hidden = true)]
pub async fn query_key_type() -> SnorKeyType {
    read_config(|s| s.get().key_type.to_owned().into())
}

// devops method
#[update(guard = "is_admin", hidden = true)]
pub async fn update_key_type(key_type: SnorKeyType) {
    let key_type = match key_type {
        SnorKeyType::ChainKey => KeyType::ChainKey,
        SnorKeyType::Native => {
            let seed = get_random_seed().await;
            KeyType::Native(seed.to_vec())
        }
    };
    mutate_config(|s| {
        let mut config = s.get().to_owned();
        config.key_type = key_type;
        s.set(config);
    })
}

// devops method
#[update]
pub async fn sui_route_address(key_type: SnorKeyType) -> Result<String, String> {
    let key_type = match key_type {
        SnorKeyType::ChainKey => KeyType::ChainKey,
        SnorKeyType::Native => {
            let seed = read_state(|s| {
                s.seeds
                    .get(&KEY_TYPE_NAME.to_string())
                    .unwrap_or_else(|| panic!("No key with name {:?}", &KEY_TYPE_NAME.to_string()))
            });
            KeyType::Native(seed.to_vec())
        }
    };
    let address = ic_sui::rpc_client::sui_route_address(key_type).await?;

    Ok(address.to_string())
}

// devops method
#[update(guard = "is_admin")]
pub async fn sui_sign(msg: Vec<u8>, key_type: SnorKeyType) -> Result<String, String> {
    let key_type = match key_type {
        SnorKeyType::ChainKey => KeyType::ChainKey,
        SnorKeyType::Native => {
            let seed = read_state(|s| {
                s.seeds
                    .get(&KEY_TYPE_NAME.to_string())
                    .unwrap_or_else(|| panic!("No key with name {:?}", &KEY_TYPE_NAME.to_string()))
            });
            KeyType::Native(seed.to_vec())
        }
    };
    let s = ic_sui::rpc_client::sui_sign(msg, key_type).await?;
    let sig = Base64::from_bytes(s.as_ref());
    Ok(sig.encoded())
}

// devops method
#[query(guard = "is_admin", hidden = true)]
pub async fn multi_rpc_config() -> MultiRpcConfig {
    read_config(|s| s.get().multi_rpc_config.to_owned())
}

// devops method
#[update(guard = "is_admin", hidden = true)]
pub async fn update_multi_rpc(multi_prc_cofig: MultiRpcConfig) {
    mutate_config(|s| {
        let mut config = s.get().to_owned();
        config.multi_rpc_config = multi_prc_cofig;
        s.set(config);
    })
}

// devops method
#[query(guard = "is_admin")]
pub async fn rpc_provider() -> Provider {
    read_config(|s| s.get().rpc_provider.to_owned())
}

// devops method
#[update(guard = "is_admin")]
pub async fn update_rpc_provider(provider: Provider) {
    mutate_config(|s| {
        let mut config = s.get().to_owned();
        config.rpc_provider = provider;
        s.set(config);
    })
}

// query supported chain list
#[query]
fn get_chain_list() -> Vec<Chain> {
    read_state(|s| {
        s.counterparties
            .iter()
            .filter(|(_, chain)| matches!(chain.chain_state, ChainState::Active))
            .map(|(_, chain)| chain.to_owned())
            .collect()
    })
}

// query supported chain list
#[query]
fn get_token_list() -> Vec<TokenResp> {
    //TODO: check sui token state
    read_state(|s| {
        s.tokens
            .iter()
            .map(|(_, token)| token.to_owned().into())
            .collect()
    })
}

// devops method
#[query(guard = "is_admin")]
fn get_token(token_id: TokenId) -> Option<Token> {
    read_state(|s| s.tokens.get(&token_id))
}

// devops method
#[update(guard = "is_admin")]
pub async fn get_gas_price() -> Result<u64, RpcError> {
    let (provider, nodes, forward) = read_config(|s| {
        (
            s.get().rpc_provider.to_owned(),
            s.get().nodes_in_subnet,
            s.get().forward.to_owned(),
        )
    });
    let client = RpcClient::new(provider, Some(nodes));

    let ret = client
        .get_gas_price(forward)
        .await
        .map_err(|e| RpcError::Text(e.to_string()))?;
    log!(DEBUG, "[service::get_gas_price] result: {} ", ret);
    Ok(ret)
}

// devops method
#[update(guard = "is_admin")]
pub async fn get_gas_budget() -> u64 {
    read_config(|s| s.get().gas_budget)
}
// devops method
#[update(guard = "is_admin")]
pub async fn update_gas_budget(gas_budget: u64) {
    mutate_config(|s| {
        let mut config = s.get().to_owned();
        config.gas_budget = gas_budget;
        s.set(config);
    })
}

#[update]
pub async fn get_balance(owner: String, coin_type: Option<String>) -> Result<u128, RpcError> {
    let owner = SuiAddress::from_str(&owner).map_err(|e| RpcError::Text(e.to_string()))?;
    let coin_type = coin_type.unwrap_or(SUI_COIN.to_string());
    log!(
        DEBUG,
        "[service::get_balance] owner: {} coin_type: {:?} ",
        owner.to_string(),
        coin_type,
    );
    let (provider, nodes, forward) = read_config(|s| {
        (
            s.get().rpc_provider.to_owned(),
            s.get().nodes_in_subnet,
            s.get().forward.to_owned(),
        )
    });

    let client = RpcClient::new(provider, Some(nodes));

    // query account info from solana
    let balance = client
        .get_balance(owner, Some(coin_type), forward)
        .await
        .map_err(|e| RpcError::Text(e.to_string()))?;
    log!(
        DEBUG,
        "[service::get_balance] account: {} current balance: {:?} ",
        owner.to_string(),
        balance,
    );
    Ok(balance.total_balance)
}

// devops method
// address_owner = "0xaf9306cac62396be300b175046140c392eed876bd8ac0efac6301cea286fa272";
// struct_type = "0x2::coin::Coin<0x2::sui::SUI>";
#[update(guard = "is_admin")]
pub async fn get_owner_objects(
    owner: String,
    struct_type: Option<String>,
) -> Result<String, RpcError> {
    let owner = SuiAddress::from_str(&owner).map_err(|e| RpcError::Text(e.to_string()))?;
    log!(
        DEBUG,
        "[service::get_owner_objects] owner: {:?} struct_type: {:?}",
        owner,
        struct_type
    );
    let obj_option = SuiObjectDataOptions {
        show_type: true,
        show_owner: true,
        show_previous_transaction: false,
        show_display: false,
        show_content: true,
        show_bcs: false,
        show_storage_rebate: false,
    };

    let query = match struct_type {
        None => {
            json!({
                "filter": {
                    "MatchAll": [
                        { "AddressOwner": owner }
                    ]
                },
                "options":obj_option
            })
        }
        Some(struct_type) => {
            json!({
                "filter": {
                    "MatchAll": [
                        { "StructType": struct_type },
                        { "AddressOwner": owner }
                    ]
                },
                "options":obj_option
            })
        }
    };

    let (provider, nodes, forward) = read_config(|s| {
        (
            s.get().rpc_provider.to_owned(),
            s.get().nodes_in_subnet,
            s.get().forward.to_owned(),
        )
    });
    let client = RpcClient::new(provider, Some(nodes));

    let ret = client
        .get_owned_objects(owner, Some(query), None, None, forward)
        .await
        .map_err(|e| RpcError::Text(e.to_string()))?;

    log!(
        DEBUG,
        "[service::get_owned_objects] get_owned_objects result: {:?} ",
        ret
    );

    let objects = serde_json::to_string(&ret).map_err(|e| RpcError::Text(e.to_string()))?;
    Ok(objects)
}

// devops method
#[update(guard = "is_admin")]
pub async fn get_object(obj_id: String) -> Result<String, RpcError> {
    let obj = ObjectID::from_str(obj_id.as_ref()).map_err(|e| RpcError::Text(e.to_string()))?;
    log!(DEBUG, "[service::get_object] object: {:#?} ", obj);
    let (provider, nodes, forward) = read_config(|s| {
        (
            s.get().rpc_provider.to_owned(),
            s.get().nodes_in_subnet,
            s.get().forward.to_owned(),
        )
    });
    let client = RpcClient::new(provider, Some(nodes));

    // let obj_option = Some(SuiObjectDataOptions {
    //     show_type: true,
    //     show_owner: true,
    //     show_previous_transaction: false,
    //     show_display: false,
    //     show_content: true,
    //     show_bcs: false,
    //     show_storage_rebate: false,
    // });
    let obj_option = None;

    let ret = client
        .get_object(obj, obj_option, forward)
        .await
        .map_err(|e| RpcError::Text(e.to_string()))?;
    log!(DEBUG, "[service::get_object] result: {:#?} ", ret);

    let obj = serde_json::to_string(&ret).map_err(|e| RpcError::Text(e.to_string()))?;
    Ok(obj)
}

// devops method
#[update(guard = "is_admin")]
pub async fn check_object_exists(owner: String, obj_id: String) -> Result<bool, RpcError> {
    let owner = SuiAddress::from_str(&owner).map_err(|e| RpcError::Text(e.to_string()))?;
    log!(
        DEBUG,
        "[service::check_object_exists] owner: {:?} obj_id: {:?}",
        owner,
        obj_id
    );
    let obj = ObjectID::from_str(obj_id.as_ref()).map_err(|e| RpcError::Text(e.to_string()))?;
    log!(DEBUG, "[service::check_object_exists] object: {:#?} ", obj);
    let (provider, nodes, forward) = read_config(|s| {
        (
            s.get().rpc_provider.to_owned(),
            s.get().nodes_in_subnet,
            s.get().forward.to_owned(),
        )
    });
    let client = RpcClient::new(provider, Some(nodes));

    let ret = client
        .check_object_exists(owner, obj, forward)
        .await
        .map_err(|e| RpcError::Text(e.to_string()))?;
    log!(DEBUG, "[service::check_object_exists] result: {:#?} ", ret);
    Ok(ret)
}

// devops method
#[update(guard = "is_admin")]
pub async fn get_coins(owner: String, coin_type: Option<String>) -> Result<String, RpcError> {
    let owner = SuiAddress::from_str(&owner).map_err(|e| RpcError::Text(e.to_string()))?;
    log!(DEBUG, "[service::get_coins] owner: {:#?} ", owner);

    let (provider, nodes, forward) = read_config(|s| {
        (
            s.get().rpc_provider.to_owned(),
            s.get().nodes_in_subnet,
            s.get().forward.to_owned(),
        )
    });
    let client = RpcClient::new(provider, Some(nodes));
    let ret = client
        .get_coins(owner, coin_type, None, None, forward)
        .await
        .map_err(|e| RpcError::Text(e.to_string()))?;
    log!(DEBUG, "[service::get_coins] result: {:#?} ", ret);
    let coins = serde_json::to_string(&ret).map_err(|e| RpcError::Text(e.to_string()))?;
    Ok(coins)
}

// devops method
#[update(guard = "is_admin")]
pub async fn fetch_coin(
    owner: String,
    coin_type: Option<String>,
    threshold: u64,
) -> Result<String, RpcError> {
    let owner = SuiAddress::from_str(&owner).map_err(|e| RpcError::Text(e.to_string()))?;
    log!(DEBUG, "[service::fetch_coin] owner: {:#?} ", owner);

    let (provider, nodes, forward) = read_config(|s| {
        (
            s.get().rpc_provider.to_owned(),
            s.get().nodes_in_subnet,
            s.get().forward.to_owned(),
        )
    });
    let client = RpcClient::new(provider, Some(nodes));
    let ret = client
        .fetch_coin(owner, coin_type, threshold, forward)
        .await
        .map_err(|e| RpcError::Text(e.to_string()))?;
    log!(DEBUG, "[service::fetch_coin] result: {:#?} ", ret);
    let coins = serde_json::to_string(&ret).map_err(|e| RpcError::Text(e.to_string()))?;
    Ok(coins)
}

// devops method
#[update(guard = "is_admin")]
pub async fn get_transaction_block(degist: String) -> Result<String, RpcError> {
    let tx_digest = TransactionDigest::from_str(degist.as_ref()).unwrap();

    log!(
        DEBUG,
        "[service::get_transaction_block] TransactionDigest: {:#?} ",
        tx_digest
    );

    let options = SuiTransactionBlockResponseOptions {
        show_input: true,
        show_raw_input: false,
        show_effects: true,
        show_events: true,
        show_object_changes: true,
        show_balance_changes: true,
        show_raw_effects: false,
    };

    let (provider, nodes, forward) = read_config(|s| {
        (
            s.get().rpc_provider.to_owned(),
            s.get().nodes_in_subnet,
            s.get().forward.to_owned(),
        )
    });
    let client = RpcClient::new(provider, Some(nodes));

    let ret = client
        .get_transaction_block(tx_digest, Some(options), forward)
        .await?;
    log!(
        DEBUG,
        "[service::get_transaction_block] get_transaction_block result: {:?} ",
        ret
    );
    let json_response = serde_json::to_string(&ret).map_err(|e| RpcError::Text(e.to_string()))?;
    Ok(json_response)
}

// devops method
#[update(guard = "is_admin")]
pub async fn get_events(digest: String) -> Result<String, RpcError> {
    let tx_digest =
        TransactionDigest::from_str(digest.as_ref()).map_err(|e| RpcError::Text(e.to_string()))?;
    log!(DEBUG, "[service::get_events] get_events: {:#?} ", tx_digest);

    let (provider, nodes, forward) = read_config(|s| {
        (
            s.get().rpc_provider.to_owned(),
            s.get().nodes_in_subnet,
            s.get().forward.to_owned(),
        )
    });
    let client = RpcClient::new(provider, Some(nodes));

    let ret = client
        .get_events(tx_digest, forward)
        .await
        .map_err(|e| RpcError::Text(e.to_string()))?;
    log!(DEBUG, "[service::get_events] result: {:#?} ", ret);
    let events = serde_json::to_string(&ret).map_err(|e| RpcError::Text(e.to_string()))?;
    Ok(events)
}

// devops method
#[update(guard = "is_admin")]
pub async fn parse_redeem_events(digest: String) -> Result<(), RpcError> {
    use crate::service::gen_ticket::BurnEvent;
    use crate::service::gen_ticket::CollectFeeEvent;
    use crate::service::gen_ticket::RedeemEvent;

    let tx_digest =
        TransactionDigest::from_str(digest.as_ref()).map_err(|e| RpcError::Text(e.to_string()))?;
    log!(
        DEBUG,
        "[service::parse_redeem_events] tx_digest: {:#?} ",
        tx_digest
    );

    let (provider, nodes, forward) = read_config(|s| {
        (
            s.get().rpc_provider.to_owned(),
            s.get().nodes_in_subnet,
            s.get().forward.to_owned(),
        )
    });
    let client = RpcClient::new(provider, Some(nodes));

    let events = client
        .get_events(tx_digest, forward)
        .await
        .map_err(|e| RpcError::Text(e.to_string()))?;

    log!(
        DEBUG,
        "[service::parse_redeem_events] get_events result: {:#?} ",
        events
    );

    for event in &events {
        let parsed_json = serde_json::to_string(&event.parsed_json).unwrap();
        log!(
            DEBUG,
            "[service::parse_redeem_events] parsed_json: {:#?}",
            parsed_json
        );

        if let Ok(collect_fee_event) =
            serde_json::from_value::<CollectFeeEvent>(event.parsed_json.to_owned())
        {
            log!(
                DEBUG,
                "[service::parse_redeem_events] collect_fee_event: {:#?}",
                collect_fee_event
            );
        } else if let Ok(burn_event) =
            serde_json::from_value::<BurnEvent>(event.parsed_json.to_owned())
        {
            log!(
                DEBUG,
                "[service::parse_redeem_events] burn_event: {:#?}",
                burn_event
            );
        } else if let Ok(redeem_event) =
            serde_json::from_value::<RedeemEvent>(event.parsed_json.to_owned())
        {
            log!(
                DEBUG,
                "[service::parse_redeem_events] redeem_event: {:#?}",
                redeem_event
            );
        } else {
            log!(
                DEBUG,
                "[service::parse_redeem_events] Unknown Parsed Value: {:?}",
                event.parsed_json
            );
        }
    }
    Ok(())
}

#[update(guard = "is_admin")]
pub async fn transfer_sui(recipient: String, amount: u64) -> Result<String, RpcError> {
    let (provider, nodes, forward) = read_config(|s| {
        (
            s.get().rpc_provider.to_owned(),
            s.get().nodes_in_subnet,
            s.get().forward.to_owned(),
        )
    });
    let client = RpcClient::new(provider, Some(nodes));
    let recipient = SuiAddress::from_str(&recipient).map_err(|e| RpcError::Text(e.to_string()))?;

    // transfer the new coin to a different address
    let ret = client
        .transfer_sui(amount, recipient, None, forward)
        .await
        .map_err(|e| RpcError::Text(e.to_string()))?;
    log!(DEBUG, "[service::transfer_sui] result: {:#?} ", ret);

    Ok(ret.digest.to_string())
}

#[query]
pub async fn sui_port_action() -> SuiPortAction {
    read_config(|s| s.get().sui_port_action.to_owned())
}

// devops method
// after deploy or upgrade the sui port contract, call this interface to update sui token info
#[update(guard = "is_admin")]
pub async fn update_sui_port_action(action: SuiPortAction) {
    mutate_config(|s| {
        let mut config = s.get().to_owned();
        config.sui_port_action = action;
        s.set(config);
    })
}

#[query]
pub async fn sui_token(token_id: TokenId) -> Option<SuiToken> {
    read_state(|s| s.sui_tokens.get(&token_id))
}

// devops method
// after deploy or upgrade the sui port contract, call this interface to update sui token info
#[update(guard = "is_admin")]
pub async fn update_sui_token(token_id: TokenId, sui_token: SuiToken) -> Result<(), String> {
    mutate_state(|s| {
        s.sui_tokens.insert(token_id.to_string(), sui_token);
    });

    Ok(())
}

#[update(guard = "is_admin")]
pub async fn transfer_objects(recipient: String, obj_ids: Vec<String>) -> Result<String, RpcError> {
    let recipient = SuiAddress::from_str(&recipient).map_err(|e| RpcError::Text(e.to_string()))?;
    let obj_ids = obj_ids
        .iter()
        .map(|obj_id| {
            ObjectID::from_str(obj_id.as_ref()).map_err(|e| RpcError::Text(e.to_string()))
        })
        .collect::<Result<Vec<_>, _>>()?;

    let (provider, nodes, forward) = read_config(|s| {
        (
            s.get().rpc_provider.to_owned(),
            s.get().nodes_in_subnet,
            s.get().forward.to_owned(),
        )
    });
    let client = RpcClient::new(provider, Some(nodes));

    // transfer the new coin to a different address
    let ret = client
        .transfer_objects(recipient, obj_ids, None, forward)
        .await
        .map_err(|e| RpcError::Text(e.to_string()))?;
    log!(DEBUG, "[service::transfer_object] result: {:#?} ", ret);

    Ok(ret.digest.to_string())
}

#[update(guard = "is_admin")]
pub async fn split_coin(
    coin_id: String,
    amount: u64,
    recipient: String,
) -> Result<String, RpcError> {
    let coin_id =
        ObjectID::from_str(coin_id.as_ref()).map_err(|e| RpcError::Text(e.to_string()))?;
    let recipient = SuiAddress::from_str(&recipient).map_err(|e| RpcError::Text(e.to_string()))?;

    let (provider, nodes, forward) = read_config(|s| {
        (
            s.get().rpc_provider.to_owned(),
            s.get().nodes_in_subnet,
            s.get().forward.to_owned(),
        )
    });
    let client = RpcClient::new(provider, Some(nodes));

    // transfer the new coin to a different address
    let ret = client
        .split_coin(coin_id, amount, recipient, None, forward)
        .await
        .map_err(|e| RpcError::Text(e.to_string()))?;
    log!(DEBUG, "[service::split_coin] result: {:#?} ", ret);

    Ok(ret.digest.to_string())
}

#[update(guard = "is_admin")]
pub async fn merge_coin(base_coin: String, merge_coins: Vec<String>) -> Result<String, RpcError> {
    let base_coin =
        ObjectID::from_str(base_coin.as_ref()).map_err(|e| RpcError::Text(e.to_string()))?;
    let merge_coins = merge_coins
        .iter()
        .map(|coin| ObjectID::from_str(coin).map_err(|e| RpcError::Text(e.to_string())))
        .collect::<Result<Vec<_>, _>>()?;
    let (provider, nodes, forward) = read_config(|s| {
        (
            s.get().rpc_provider.to_owned(),
            s.get().nodes_in_subnet,
            s.get().forward.to_owned(),
        )
    });
    let client = RpcClient::new(provider, Some(nodes));

    // transfer the new coin to a different address
    let ret = client
        .merge_coin(base_coin, merge_coins, None, forward)
        .await
        .map_err(|e| RpcError::Text(e.to_string()))?;
    log!(DEBUG, "[service::merge_coin] result: {:#?} ", ret);

    Ok(ret.digest.to_string())
}

#[update(guard = "is_admin")]
pub async fn create_ticket_table(recipient: String) -> Result<String, RpcError> {
    let recipient = SuiAddress::from_str(&recipient).map_err(|e| RpcError::Text(e.to_string()))?;

    let (action, provider, nodes, forward) = read_config(|s| {
        (
            s.get().sui_port_action.to_owned(),
            s.get().rpc_provider.to_owned(),
            s.get().nodes_in_subnet,
            s.get().forward.to_owned(),
        )
    });
    let client = RpcClient::new(provider, Some(nodes));

    // transfer the new coin to a different address
    let ret = client
        .create_ticket_table(action, recipient, None, forward)
        .await
        .map_err(|e| RpcError::Text(e.to_string()))?;
    log!(DEBUG, "[service::create_ticket_table] result: {:#?} ", ret);

    Ok(ret.digest.to_string())
}

#[update(guard = "is_admin")]
pub async fn remove_ticket_from_port(ticket_id: String) -> Result<String, RpcError> {
    let (action, provider, nodes, forward) = read_config(|s| {
        (
            s.get().sui_port_action.to_owned(),
            s.get().rpc_provider.to_owned(),
            s.get().nodes_in_subnet,
            s.get().forward.to_owned(),
        )
    });
    let client = RpcClient::new(provider, Some(nodes));

    let ret = client
        .remove_ticket(action, ticket_id.to_string(), None, forward)
        .await
        .map_err(|e| RpcError::Text(e.to_string()))?;
    log!(
        DEBUG,
        "[service::remove_ticket_from_port] result: {:#?} ",
        ret
    );
    // remove ticket from stable storage
    mutate_state(|s| s.clr_ticket_queue.remove(&ticket_id));
    Ok(ret.digest.to_string())
}

#[update(guard = "is_admin")]
pub async fn drop_ticket_table() -> Result<String, RpcError> {
    let (action, provider, nodes, forward) = read_config(|s| {
        (
            s.get().sui_port_action.to_owned(),
            s.get().rpc_provider.to_owned(),
            s.get().nodes_in_subnet,
            s.get().forward.to_owned(),
        )
    });
    let client = RpcClient::new(provider, Some(nodes));

    // transfer the new coin to a different address
    let ret = client
        .drop_ticket_table(action, None, forward)
        .await
        .map_err(|e| RpcError::Text(e.to_string()))?;
    log!(DEBUG, "[service::drop_ticket_table] result: {:#?} ", ret);

    Ok(ret.digest.to_string())
}

// devops method
#[update(guard = "is_admin")]
pub async fn mint_to_with_ticket(
    ticket_id: String,
    token_id: String,
    recipient: String,
    amount: u64,
) -> Result<String, RpcError> {
    let sui_token = match read_state(|s| s.sui_tokens.get(&token_id)) {
        None => {
            return Err(RpcError::Text(format!(
                "[service::mint_to_with_ticket] Not found sui token for {}",
                token_id
            )))
        }
        Some(sui_token) => sui_token,
    };
    log!(
        DEBUG,
        "[service::mint_to_with_ticket] sui_token: {:#?} ",
        sui_token
    );
    let (action, provider, nodes, gas_budget, forward) = read_config(|s| {
        (
            s.get().sui_port_action.to_owned(),
            s.get().rpc_provider.to_owned(),
            s.get().nodes_in_subnet,
            s.get().gas_budget,
            s.get().forward.to_owned(),
        )
    });
    let client = RpcClient::new(provider, Some(nodes));
    // transfer the new coin to a different address
    let recipient = SuiAddress::from_str(&recipient).map_err(|e| RpcError::Text(e.to_string()))?;
    let ret = client
        .mint_with_ticket(
            action,
            ticket_id,
            sui_token,
            recipient,
            amount,
            Some(gas_budget),
            forward,
        )
        .await
        .map_err(|e| RpcError::Text(e.to_string()))?;
    log!(DEBUG, "[service::mint_to_with_ticket] result: {:#?} ", ret);
    let digest = ret.digest;
    Ok(digest.to_string())
}

// devops method
#[update(guard = "is_admin")]
pub async fn mint_token(
    token_id: String,
    recipient: String,
    amount: u64,
) -> Result<String, RpcError> {
    let sui_token = match read_state(|s| s.sui_tokens.get(&token_id)) {
        None => {
            return Err(RpcError::Text(format!(
                "[service::mint_token] Not found sui token for {}",
                token_id
            )))
        }
        Some(sui_token) => sui_token,
    };
    log!(DEBUG, "[service::mint_token] sui_token: {:#?} ", sui_token);
    let (action, provider, nodes, gas_budget, forward) = read_config(|s| {
        (
            s.get().sui_port_action.to_owned(),
            s.get().rpc_provider.to_owned(),
            s.get().nodes_in_subnet,
            s.get().gas_budget,
            s.get().forward.to_owned(),
        )
    });
    let client = RpcClient::new(provider, Some(nodes));
    // transfer the new coin to a different address
    let recipient = SuiAddress::from_str(&recipient).map_err(|e| RpcError::Text(e.to_string()))?;
    let ret = client
        .mint_token(
            action,
            sui_token,
            recipient,
            amount,
            Some(gas_budget),
            forward,
        )
        .await
        .map_err(|e| RpcError::Text(e.to_string()))?;
    log!(DEBUG, "[service::mint_token] result: {:#?} ", ret);
    let digest = ret.digest;
    Ok(digest.to_string())
}

// devops method
#[update(guard = "is_admin")]
pub async fn burn_token(token_id: String, burn_coin_id: String) -> Result<String, RpcError> {
    let burn_coin_id =
        ObjectID::from_str(burn_coin_id.as_ref()).map_err(|e| RpcError::Text(e.to_string()))?;

    let sui_token = match read_state(|s| s.sui_tokens.get(&token_id)) {
        None => {
            return Err(RpcError::Text(format!(
                "Not found sui token for {}",
                token_id
            )))
        }
        Some(sui_token) => sui_token,
    };

    log!(DEBUG, "[service::burn_token] sui_token: {:#?} ", sui_token);

    let (provider, nodes, forward) = read_config(|s| {
        (
            s.get().rpc_provider.to_owned(),
            s.get().nodes_in_subnet,
            s.get().forward.to_owned(),
        )
    });
    let client = RpcClient::new(provider, Some(nodes));

    // transfer the new coin to a different address
    let ret = client
        .burn_token(sui_token, burn_coin_id, None, forward)
        .await
        .map_err(|e| RpcError::Text(e.to_string()))?;
    log!(DEBUG, "[service::burn_token] result: {:#?} ", ret);
    let digest = ret.digest;
    Ok(digest.to_string())
}

// devops method, add token manually
#[update(guard = "is_admin")]
pub async fn add_token(token: Token) -> Option<Token> {
    mutate_state(|s| {
        s.tokens
            .insert(token.token_id.to_string(), token.to_owned())
    })
}

// devops method
#[update(guard = "is_admin", hidden = true)]
fn update_token(token: Token) -> Result<Option<Token>, CallError> {
    mutate_state(|s| match s.tokens.get(&token.token_id) {
        None => Err(CallError {
            method: "[service::update_token] update_token".to_string(),
            reason: Reason::CanisterError(format!(
                "Not found token id {} ",
                token.token_id.to_string()
            )),
        }),
        Some(_) => Ok(s
            .tokens
            .insert(token.token_id.to_string(), token.to_owned())),
    })
    // Ok(())
}

// devops method
#[query(hidden = true)]
fn get_ticket_from_queue(ticket_id: String) -> Option<(u64, Ticket)> {
    read_state(|s| {
        s.tickets_queue
            .iter()
            .find(|(_seq, ticket)| ticket.ticket_id.eq(&ticket_id))
    })
}

// devops method
#[query(hidden = true)]
fn get_tickets_from_queue() -> Vec<(u64, Ticket)> {
    read_state(|s| {
        s.tickets_queue
            .iter()
            .map(|(seq, ticket)| (seq, ticket))
            .collect()
    })
}

// devops method
#[update(guard = "is_admin", hidden = true)]
pub async fn remove_ticket_from_quene(ticket_id: String) -> Option<Ticket> {
    mutate_state(|s| {
        let ticket = s
            .tickets_queue
            .iter()
            .find(|(_seq, ticket)| ticket.ticket_id.eq(&ticket_id));

        match ticket {
            None => None,
            Some((seq, _ticket)) => s.tickets_queue.remove(&seq),
        }
    })
}

// query mint_token_statue for the given ticket id
#[query]
pub async fn mint_token_status(ticket_id: String) -> Result<TxStatus, CallError> {
    let req = read_state(|s| s.mint_token_requests.get(&ticket_id));
    match req {
        None => Err(CallError {
            method: "[service::mint_token_status] mint_token_status".to_string(),
            reason: Reason::CanisterError(format!(
                "Not found ticket({}) MintTokenStatus",
                ticket_id.to_string()
            )),
        }),

        Some(req) => Ok(req.status),
    }
}

// query mint token tx hash or signature for the given ticket id
#[query]
pub async fn mint_token_tx_hash(ticket_id: String) -> Result<Option<String>, CallError> {
    let req = read_state(|s| s.mint_token_requests.get(&ticket_id).to_owned());
    match req {
        None => Err(CallError {
            method: "[service::mint_token_tx_hash] mint_token_tx_hash".to_string(),
            reason: Reason::CanisterError(format!(
                "Not found ticket({}) mint token tx hash",
                ticket_id.to_string()
            )),
        }),

        Some(req) => Ok(req.digest),
    }
}

// devops method
#[query]
pub async fn mint_token_req(ticket_id: String) -> Result<MintTokenRequest, CallError> {
    let req = read_state(|s| s.mint_token_requests.get(&ticket_id));
    match req {
        None => Err(CallError {
            method: "[service::mint_token_req] mint_token_req".to_string(),
            reason: Reason::CanisterError(format!(
                "Not found ticket({}) mint token request",
                ticket_id.to_string()
            )),
        }),

        Some(req) => Ok(req),
    }
}

// devops method
#[query(guard = "is_admin")]
pub async fn mint_token_reqs(offset: usize, limit: usize) -> Vec<MintTokenRequest> {
    read_state(|s| {
        s.mint_token_requests
            .iter()
            .skip(offset)
            .take(limit)
            .map(|(_, v)| v)
            .collect::<Vec<_>>()
    })
}

// devops method
#[update(guard = "is_admin")]
pub async fn update_mint_token_req(req: MintTokenRequest) -> Result<MintTokenRequest, CallError> {
    mutate_state(|s| {
        s.mint_token_requests
            .insert(req.ticket_id.to_string(), req.to_owned())
    });

    match read_state(|s| s.mint_token_requests.get(&req.ticket_id)) {
        None => Err(CallError {
            method: "[service::update_mint_token_req] update_mint_token_req".to_string(),
            reason: Reason::CanisterError(format!(
                "Not found ticket({}) mint token request",
                req.ticket_id.to_string()
            )),
        }),
        Some(req) => Ok(req),
    }
}

// devops method
#[update(guard = "is_admin")]
pub async fn update_token_meta(
    token_id: String,
    update_type: UpdateType,
) -> Result<String, RpcError> {
    let sui_token = match read_state(|s| s.sui_tokens.get(&token_id)) {
        None => {
            return Err(RpcError::Text(format!(
                "[service::update_token_meta] Not found sui token for {}",
                token_id
            )))
        }
        Some(sui_token) => sui_token,
    };

    log!(
        DEBUG,
        "[service::update_token_meta] sui_token: {:#?} ",
        sui_token
    );
    let (provider, nodes, forward) = read_config(|s| {
        (
            s.get().rpc_provider.to_owned(),
            s.get().nodes_in_subnet,
            s.get().forward.to_owned(),
        )
    });
    let client = RpcClient::new(provider, Some(nodes));

    let ret = client
        .update_token_meta(sui_token, update_type, None, forward)
        .await;
    log!(DEBUG, "[service::update_token_meta] result: {:#?} ", ret);
    let digest = ret.unwrap().digest;
    Ok(digest.to_string())
}

// devops method
#[query(hidden = true)]
pub async fn failed_mint_reqs() -> Vec<(TicketId, MintTokenRequest)> {
    read_state(|s| {
        s.mint_token_requests
            .iter()
            .filter(|(_, v)| matches!(v.status, TxStatus::TxFailed { .. }))
            .map(|(k, v)| (k, v))
            .take(3)
            .collect()
    })
}

// devops method
#[update(guard = "is_admin", hidden = true)]
pub async fn mint_token_with_req(n_req: MintTokenRequest) -> Result<TxStatus, String> {
    let req = match read_state(|s| s.mint_token_requests.get(&n_req.ticket_id)) {
        None => {
            log!(
                DEBUG,
                "[service::mint_token_with_req] not found mint token req for ticket: {:?} ",
                n_req.ticket_id
            );
            n_req
        }
        Some(o_req) => {
            if o_req.eq(&n_req) {
                o_req
            } else {
                n_req
            }
        }
    };

    log!(
        DEBUG,
        "[service::mint_token_with_req] mint token request: {:?} ",
        req
    );
    mint_token::handle_mint_token(req.to_owned()).await;

    let q = read_state(|s| s.mint_token_requests.get(&req.ticket_id)).expect(
        format!(
            "Not found mint token request for ticket id: {} ",
            req.ticket_id.to_string()
        )
        .as_str(),
    );
    Ok(q.status)
}

#[update(guard = "is_admin", hidden = true)]
pub async fn update_mint_token_status(
    ticket_id: String,
    status: TxStatus,
) -> Result<Option<MintTokenRequest>, String> {
    mutate_state(|s| {
        if let Some(req) = s.mint_token_requests.get(&ticket_id).as_mut() {
            req.status = status;
            s.mint_token_requests
                .insert(req.ticket_id.to_string(), req.to_owned());
        }
    });

    let latest_req = read_state(|s| s.mint_token_requests.get(&ticket_id));

    Ok(latest_req)
}
// devops method
#[update(guard = "is_admin", hidden = true)]
pub async fn update_tx_to_hub(sig: String, ticket_id: String) -> Result<(), CallError> {
    let hub_principal = read_config(|s| s.get().hub_principal);

    match mint_token::update_tx_to_hub(hub_principal, ticket_id.to_string(), sig.to_owned()).await {
        Ok(()) => {
            log!(
                DEBUG,
                "[service::update_tx_to_hub] successfully update tx hash ({})) to hub! ",
                sig
            );
            //only finalized mint_req, remove the handled ticket from queue
            // remove_ticket_from_quene(ticket_id.to_string()).await;
        }
        Err(err) => {
            log!(
                ERROR,
                "[service::update_tx_to_hub] failed to update tx hash ({})) to hub : {}",
                sig,
                err
            );
        }
    }

    Ok(())
}

// query collect fee account
#[query]
pub async fn get_fee_account() -> String {
    read_config(|s| s.get().fee_account.to_string())
}

// update collect fee account
#[update(guard = "is_admin", hidden = true)]
pub async fn update_fee_account(fee_account: String) {
    mutate_config(|s| {
        let mut config = s.get().to_owned();
        config.fee_account = fee_account;
        s.set(config);
    })
}

// query fee account for the dst chain
#[query]
pub fn get_redeem_fee(chain_id: ChainId) -> Option<u128> {
    read_config(|s| s.get().get_fee(chain_id))
}

#[update(guard = "is_admin", hidden = true)]
pub async fn update_redeem_fee(fee: Factor) {
    mutate_config(|s| {
        let mut config = s.get().to_owned();
        config.update_fee(fee);
        s.set(config);
    })
}

// generate ticket ,called by front end or other sys
#[update]
async fn generate_ticket(args: GenerateTicketReq) -> Result<GenerateTicketOk, GenerateTicketError> {
    gen_ticket::generate_ticket(args).await
}

// devops method
#[update(guard = "is_admin", hidden = true)]
async fn valid_tx_from_multi_rpc(digest: String) -> Result<String, String> {
    let (provider, nodes) =
        read_config(|s| (s.get().rpc_provider.to_owned(), s.get().nodes_in_subnet));
    let client = RpcClient::new(provider, Some(nodes));
    let multi_rpc_config = read_config(|s| s.get().multi_rpc_config.to_owned());
    // let tx_digest = TransactionDigest::from_str(digest.as_ref()).unwrap();
    let tx_response =
        query_tx_from_multi_rpc(&client, digest, multi_rpc_config.rpc_list.to_owned()).await;
    let json_response = multi_rpc_config.valid_and_get_result(&tx_response)?;
    let ret = serde_json::to_string(&json_response).map_err(|err| err.to_string())?;
    Ok(ret)
}

// devops method
#[query(guard = "is_admin", hidden = true)]
pub fn gen_tickets_req(ticket_id: String) -> Option<GenerateTicketReq> {
    read_state(|s| s.gen_ticket_reqs.get(&ticket_id))
}

// devops method
#[query(guard = "is_admin", hidden = true)]
pub fn gen_tickets_reqs(offset: usize, limit: usize) -> Vec<GenerateTicketReq> {
    read_state(|s| {
        s.gen_ticket_reqs
            .iter()
            .skip(offset)
            .take(limit)
            .map(|(_, v)| v.to_owned())
            .collect()
    })
}

// devops method
#[update(guard = "is_admin", hidden = true)]
pub async fn remove_gen_tickets_req(ticket_id: String) -> Option<GenerateTicketReq> {
    mutate_state(|state| state.gen_ticket_reqs.remove(&ticket_id))
}

// devops method
#[query(guard = "is_admin", hidden = true)]
pub fn get_failed_tickets_to_hub() -> Vec<Ticket> {
    read_state(|s| {
        s.tickets_failed_to_hub
            .iter()
            .map(|(_, ticket)| ticket)
            .collect()
    })
}

// devops method
#[query(guard = "is_admin", hidden = true)]
pub fn get_failed_ticket_to_hub(ticket_id: String) -> Option<Ticket> {
    read_state(|s| s.tickets_failed_to_hub.get(&ticket_id))
}

// devops method
// when gen ticket and send it to hub failed ,call this method
#[update(guard = "is_admin", hidden = true)]
pub async fn send_failed_tickets_to_hub() -> Result<(), GenerateTicketError> {
    let tickets_size = read_state(|s| s.tickets_failed_to_hub.len());
    while !read_state(|s| s.tickets_failed_to_hub.is_empty()) {
        let (ticket_id, ticket) = mutate_state(|rs| rs.tickets_failed_to_hub.pop_first()).unwrap();

        let hub_principal = read_config(|s| (s.get().hub_principal));
        if let Err(err) = send_ticket(hub_principal, ticket.to_owned())
            .await
            .map_err(|err| GenerateTicketError::SendTicketErr(format!("{}", err)))
        {
            mutate_state(|state| {
                state
                    .tickets_failed_to_hub
                    .insert(ticket_id, ticket.to_owned());
            });
            log!(
                ERROR,
                "[service::send_failed_tickets_to_hub] failed to resend ticket: {}",
                ticket.ticket_id
            );
            return Err(err);
        }
    }
    log!(
        DEBUG,
        "[service::send_failed_tickets_to_hub] successfully resend {} tickets",
        tickets_size
    );
    Ok(())
}

// devops method
// when gen ticket and send it to hub failed ,call this method
#[update(guard = "is_admin", hidden = true)]
pub async fn send_failed_ticket_to_hub(ticket_id: String) -> Result<(), GenerateTicketError> {
    if let Some(ticket) = read_state(|rs| rs.tickets_failed_to_hub.get(&ticket_id)) {
        let hub_principal = read_config(|s| (s.get().hub_principal));
        match send_ticket(hub_principal, ticket.to_owned()).await {
            Ok(()) => {
                mutate_state(|state| state.tickets_failed_to_hub.remove(&ticket_id));
                log!(
                    DEBUG,
                    "[service::send_failed_ticket_to_hub] successfully resend ticket : {} ",
                    ticket_id
                );
                return Ok(());
            }
            Err(err) => {
                log!(
                    ERROR,
                    "[service::send_failed_ticket_to_hub] failed to resend ticket: {}, error: {:?}",
                    ticket_id,
                    err
                );
                return Err(GenerateTicketError::SendTicketErr(format!("{}", err)));
            }
        }
    }

    Ok(())
}

// devops method
#[update(guard = "is_admin", hidden = true)]
pub async fn remove_failed_tickets_to_hub(ticket_id: String) -> Option<Ticket> {
    mutate_state(|state| state.tickets_failed_to_hub.remove(&ticket_id))
}

// devops method
#[query(guard = "is_admin", hidden = true)]
pub async fn seqs() -> Seqs {
    read_config(|s| s.get().seqs.to_owned())
}

// devops method
#[update(guard = "is_admin", hidden = true)]
pub async fn update_seqs(seqs: Seqs) {
    mutate_config(|s| {
        let mut config = s.get().to_owned();
        config.seqs = seqs;
        s.set(config);
    })
}

// devops method
#[update(guard = "is_admin", hidden = true)]
pub async fn set_permissions(caller: Principal, perm: Permission) {
    set_perms(caller.to_string(), perm)
}

// devops method
#[update(guard = "is_admin", hidden = true)]
pub fn debug(enable: bool) {
    mutate_config(|s| {
        let mut config = s.get().to_owned();
        config.enable_debug = enable;
        s.set(config);
    });
}

/// Cleans up the HTTP response headers to make them deterministic.
///
/// # Arguments
///
/// * `args` - Transformation arguments containing the HTTP response.
///
#[query(hidden = true)]
fn cleanup_response(mut args: TransformArgs) -> TransformedHttpResponse {
    // The response header contains non-deterministic fields that make it impossible to reach consensus!
    // Errors seem deterministic and do not contain data that can break consensus.
    // Clear non-deterministic fields from the response headers.

    // log!(
    //     DEBUG,
    //     "[service::cleanup_response] cleanup_response TransformArgs: {:?}",
    //     args
    // );
    args.response.headers.clear();
    // log!(
    //     DEBUG,
    //     "[service::cleanup_response] response.headers: {:?}",
    //     args.response.headers
    // );
    args.response
}

#[query(hidden = true)]
fn http_request(req: HttpRequest) -> HttpResponse {
    match req.path() {
        "/logs" => {
            let endable_debug = read_config(|s| s.get().enable_debug);
            crate::ic_log::http_log(req, endable_debug)
        }

        _ => HttpResponseBuilder::not_found().build(),
    }
}

// Enable Candid export
ic_cdk::export_candid!();

mod test {
    use crate::ic_sui::sui_types::{base_types::SuiAddress, digests::TransactionDigest};
    use std::str::FromStr;

    #[test]
    fn test_sui_digest() {
        let digest = "5MN731gZhoTNiS339do6sJ62amDfQsVsTVni4enJT2Qv";
        let tx_digest = TransactionDigest::from_str(digest).unwrap();
        println!("TransactionDigest: {:?} ", tx_digest);
    }
    #[test]
    fn test_sui_address() {
        let owner_str = "0x365eb9f54539cf07332773f756a392d5af507b3b8990f84e52ee6f6b6b57534b";
        println!("owner_str: {:?} ", owner_str);
        let owner_address = SuiAddress::from_str(&owner_str).unwrap();
        println!("owner_address: {:?} ", owner_address);
    }
}
