#![allow(unused)]
use crate::config::{mutate_config, replace_config, take_config, SuiRouteConfig};
use crate::ic_sui::sui_providers::Provider;
use crate::memory::init_config;
use crate::migration::{migrate_config, PreConfig};
use crate::state::{replace_state, SuiRouteState};
use crate::types::ChainState;

use candid::{CandidType, Principal};

use crate::ic_log::DEBUG;
use ic_canister_log::log;
use serde::{Deserialize, Serialize};
#[derive(CandidType, serde::Deserialize, Clone, Debug)]
pub enum RouteArg {
    Init(InitArgs),
    Upgrade(Option<UpgradeArgs>),
}

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct InitArgs {
    pub admin: Principal,
    pub chain_id: String,
    pub hub_principal: Principal,
    pub chain_state: ChainState,
    pub fee_account: String,
    pub schnorr_key_name: Option<String>,
    pub rpc_provider: Option<Provider>,
    pub nodes_in_subnet: Option<u32>,
    pub gas_budget: Option<u64>,
}

pub fn init(args: InitArgs) {
    let config = SuiRouteConfig::from(args);
    config.validate_config();
    log!(DEBUG, "lifecycle::init config:{:?}", config);
    let mut stable_config = init_config();
    stable_config.set(config);
    replace_config(stable_config);
    let state = SuiRouteState::init();
    replace_state(state);
}

pub fn pre_upgrade() {}

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct UpgradeArgs {
    pub admin: Option<Principal>,
    pub chain_id: Option<String>,
    pub hub_principal: Option<Principal>,
    pub chain_state: Option<ChainState>,
    pub schnorr_key_name: Option<String>,
    pub rpc_provider: Option<Provider>,
    pub nodes_in_subnet: Option<u32>,
    pub fee_account: Option<String>,
    pub gas_budget: Option<u64>,
}

pub fn post_upgrade(args: Option<UpgradeArgs>) {
    // load state
    let state = SuiRouteState::init();
    replace_state(state);

    // load config
    let stable_config = init_config();
    replace_config(stable_config);

    // update config args based on UpgradeArgs
    mutate_config(|s| {
        let mut config = s.get().to_owned();
        log!(DEBUG, "lifecycle::post_upgrade config:{:?}", config);
        if let Some(args) = args {
            if let Some(admin) = args.admin {
                config.admin = admin;
            }
            if let Some(chain_id) = args.chain_id {
                config.chain_id = chain_id;
            }
            if let Some(hub_principal) = args.hub_principal {
                config.hub_principal = hub_principal;
            }
            if let Some(chain_state) = args.chain_state {
                config.chain_state = chain_state;
            }
            if let Some(schnorr_key_name) = args.schnorr_key_name {
                config.schnorr_key_name = schnorr_key_name;
            }
            if let Some(provider) = args.rpc_provider {
                config.rpc_provider = provider;
            }
            if let Some(nodes_in_subnet) = args.nodes_in_subnet {
                config.nodes_in_subnet = nodes_in_subnet;
            }
            if let Some(fee_account) = args.fee_account {
                config.fee_account = fee_account;
            }
            if let Some(gas_budget) = args.gas_budget {
                config.gas_budget = gas_budget;
            }
        }
        config.validate_config();
        s.set(config);
    });
}
