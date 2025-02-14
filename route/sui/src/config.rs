#![allow(unused)]
use crate::constants::{DEFAULT_GAS_BUDGET, FEE_TOKEN, NODES_IN_FIDUCIARY_SUBNET};

use crate::ic_log::{DEBUG, ERROR};
use crate::ic_sui::ck_eddsa::KeyType;
use crate::ic_sui::constants::NODES_IN_SUBNET;
use crate::ic_sui::rpc_client::RpcResult;
use crate::ic_sui::sui_json_rpc_types::SuiEvent;
use crate::ic_sui::sui_providers::Provider;

use crate::memory::Memory;
use crate::types::{ChainId, ChainState, Factor};
use crate::{auth::Permission, constants::SCHNORR_KEY_NAME, guard::TaskType, lifecycle::InitArgs};
use candid::{CandidType, Principal};

use ic_canister_log::log;

use ic_stable_structures::storable::Bound;
use ic_stable_structures::StableCell;
use ic_stable_structures::Storable;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::{
    cell::RefCell,
    collections::{BTreeMap, HashMap, HashSet},
};
pub type CanisterId = Principal;
pub type Owner = String;
pub type MintAccount = String;
pub type AssociatedAccount = String;
pub type StableRouteConfig = StableCell<SuiRouteConfig, Memory>;

thread_local! {
    static CONFIG: RefCell<Option<StableRouteConfig>> = RefCell::default();
}

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, Default, PartialEq, Eq)]
pub struct Seqs {
    pub next_ticket_seq: u64,
    pub next_directive_seq: u64,
}

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, Default, PartialEq, Eq)]
pub struct MultiRpcConfig {
    pub rpc_list: Vec<String>,
    pub minimum_response_count: u32,
}

impl MultiRpcConfig {
    pub fn new(rpc_list: Vec<String>, minimum_response_count: u32) -> Result<Self, String> {
        let s = Self {
            rpc_list,
            minimum_response_count,
        };
        s.check_config_valid()?;

        Ok(s)
    }

    pub fn check_config_valid(&self) -> Result<(), String> {
        if self.minimum_response_count == 0 {
            return Err("minimum_response_count should be greater than 0".to_string());
        }
        if self.rpc_list.len() < self.minimum_response_count as usize {
            return Err(
                "rpc_list length should be greater than minimum_response_count".to_string(),
            );
        }
        Ok(())
    }

    pub fn valid_and_get_result(
        &self,
        responses: &Vec<RpcResult<Vec<SuiEvent>>>,
    ) -> Result<Vec<SuiEvent>, String> {
        self.check_config_valid()?;
        let mut events_list = vec![];
        // let mut success_response_body_list = vec![];

        for response in responses {
            log!(
                DEBUG,
                "[state::valid_and_get_result] input response: {:?}",
                response
            );
            match response {
                Ok(events) => events_list.push(events),
                Err(e) => {
                    log!(
                        ERROR,
                        "[state::valid_and_get_result] response error: {:?}",
                        e.to_string()
                    );
                    continue;
                }
            }
        }

        if events_list.len() < self.minimum_response_count as usize {
            return Err(format!(
                "Not enough valid response, expected: {}, actual: {}",
                self.minimum_response_count,
                events_list.len()
            ));
        }

        // The minimum_response_count should greater than 0
        let mut i = 1;
        while i < events_list.len() {
            if events_list[i - 1] != events_list[i] {
                return Err("Response mismatch".to_string());
            }
            i += 1;
        }

        Ok(events_list[0].to_owned())
    }
}

pub const KEY_TYPE_NAME: &str = "Native";
#[derive(CandidType, Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub enum SnorKeyType {
    ChainKey,
    Native,
}

impl From<KeyType> for SnorKeyType {
    fn from(key_type: KeyType) -> Self {
        match key_type {
            KeyType::ChainKey => SnorKeyType::ChainKey,
            KeyType::Native(_) => SnorKeyType::Native,
        }
    }
}

#[derive(CandidType, Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct SuiPortAction {
    pub package: String,
    pub module: String,
    pub functions: HashSet<String>,
    pub port_owner_cap: String,
    pub ticket_table: String,
    pub upgrade_cap: String,
}

impl Storable for SuiPortAction {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        let bytes = bincode::serialize(&self).expect("failed to serialize SuiPortAction");
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        bincode::deserialize(bytes.as_ref()).expect("failed to deserialize SuiPortAction")
    }

    const BOUND: Bound = Bound::Unbounded;
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct SuiRouteConfig {
    pub chain_id: String,
    pub hub_principal: Principal,
    pub seqs: Seqs,
    pub fee_token_factor: Option<u128>,
    pub target_chain_factor: BTreeMap<ChainId, u128>,
    pub chain_state: ChainState,
    pub schnorr_key_name: String,
    pub rpc_provider: Provider,
    pub nodes_in_subnet: u32,
    pub fee_account: String,
    pub gas_budget: u64,
    // Locks preventing concurrent execution timer tasks
    pub active_tasks: HashSet<TaskType>,
    pub admin: Principal,
    pub caller_perms: HashMap<String, Permission>,
    pub multi_rpc_config: MultiRpcConfig,
    pub forward: Option<String>,
    pub enable_debug: bool,
    pub key_type: KeyType,
    pub sui_port_action: SuiPortAction,
    // pub sui_route_address: HashMap<KeyType, Vec<u8>>,
}
impl Default for SuiRouteConfig {
    fn default() -> Self {
        Self {
            chain_id: String::default(),
            hub_principal: Principal::anonymous(),
            seqs: Seqs::default(),
            fee_token_factor: None,
            target_chain_factor: BTreeMap::default(),
            chain_state: ChainState::Active,
            schnorr_key_name: SCHNORR_KEY_NAME.to_string(),
            rpc_provider: Provider::default(),
            nodes_in_subnet: NODES_IN_SUBNET,
            fee_account: String::default(),
            gas_budget: DEFAULT_GAS_BUDGET,
            active_tasks: HashSet::default(),
            admin: Principal::anonymous(),
            caller_perms: HashMap::default(),
            multi_rpc_config: MultiRpcConfig::default(),
            forward: None,
            enable_debug: false,
            key_type: KeyType::ChainKey,
            sui_port_action: SuiPortAction::default(),
            // sui_route_address: HashMap::default(),
        }
    }
}

impl From<InitArgs> for SuiRouteConfig {
    fn from(args: InitArgs) -> Self {
        Self {
            chain_id: args.chain_id,
            hub_principal: args.hub_principal,
            seqs: Seqs::default(),
            fee_token_factor: None,
            target_chain_factor: Default::default(),
            chain_state: args.chain_state,
            schnorr_key_name: args
                .schnorr_key_name
                .unwrap_or(SCHNORR_KEY_NAME.to_string()),
            rpc_provider: args.rpc_provider.unwrap_or(Provider::default()),
            nodes_in_subnet: args.nodes_in_subnet.unwrap_or(NODES_IN_FIDUCIARY_SUBNET),
            active_tasks: Default::default(),
            admin: args.admin,
            caller_perms: HashMap::from([(args.admin.to_string(), Permission::Update)]),
            fee_account: args.fee_account,
            gas_budget: args.gas_budget.unwrap_or(DEFAULT_GAS_BUDGET),
            multi_rpc_config: MultiRpcConfig::default(),
            forward: None,
            enable_debug: false,
            key_type: KeyType::ChainKey,
            sui_port_action: SuiPortAction::default(),
            // sui_route_address: HashMap::default(),
        }
    }
}

impl Storable for SuiRouteConfig {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        let bytes = bincode::serialize(&self).expect("failed to serialize SuiRouteConfig");
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        bincode::deserialize(bytes.as_ref()).expect("failed to deserialize SuiRouteConfig")
    }

    const BOUND: Bound = Bound::Unbounded;
}

impl SuiRouteConfig {
    pub fn validate_config(&self) {}
    pub fn get_fee(&self, chain_id: ChainId) -> Option<u128> {
        read_config(|s| {
            s.get()
                .target_chain_factor
                .get(&chain_id)
                .map_or(None, |target_chain_factor| {
                    s.get()
                        .fee_token_factor
                        .map(|fee_token_factor| target_chain_factor * fee_token_factor)
                })
        })
    }
    pub fn update_fee(&mut self, fee: Factor) {
        match fee {
            Factor::UpdateTargetChainFactor(factor) => {
                self.target_chain_factor.insert(
                    factor.target_chain_id.to_owned(),
                    factor.target_chain_factor,
                );
            }

            Factor::UpdateFeeTokenFactor(token_factor) => {
                if token_factor.fee_token == FEE_TOKEN {
                    self.fee_token_factor = Some(token_factor.fee_token_factor);
                }
            }
        }
    }
}

pub fn take_config<F, R>(f: F) -> R
where
    F: FnOnce(SuiRouteConfig) -> R,
{
    CONFIG.with(|c| {
        let config = c.take().expect("Config not initialized!").get().to_owned();
        f(config)
    })
}

pub fn mutate_config<F, R>(f: F) -> R
where
    F: FnOnce(&mut StableRouteConfig) -> R,
{
    CONFIG.with(|c| f(c.borrow_mut().as_mut().expect("Config not initialized!")))
}

pub fn read_config<F, R>(f: F) -> R
where
    F: FnOnce(&StableRouteConfig) -> R,
{
    CONFIG.with(|c| f(c.borrow().as_ref().expect("State not initialized!")))
}

pub fn replace_config(config: StableRouteConfig) {
    CONFIG.with(|c| {
        // *c.borrow_mut() = config;
        c.replace(Some(config));
        // c.set(config).expect("failed to replace config");
    });
}
