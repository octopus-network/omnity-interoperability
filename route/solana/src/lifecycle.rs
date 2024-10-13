// use crate::migration::{migrate, PreState};
use crate::types::ChainState;
use crate::{
    memory,
    state::{read_state, replace_state, SolanaRouteState},
};
use candid::{CandidType, Principal};
use ic_stable_structures::{writer::Writer, Memory};
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

    pub schnorr_key_name: Option<String>,
    pub sol_canister: Principal,
    pub fee_account: Option<String>,
    // pub multi_rpc_config: MultiRpcConfig,
    // pub forward: Option<String>,
}

pub fn init(args: InitArgs) {
    let state = SolanaRouteState::from(args);
    state.validate_config();
    replace_state(state);
}

pub fn pre_upgrade() {
    // Serialize the state.
    let mut state_bytes = vec![];

    let _ = read_state(|s| ciborium::ser::into_writer(&s, &mut state_bytes));
    // Write the length of the serialized bytes to memory, followed by the
    // by the bytes themselves.
    let len = state_bytes.len() as u32;
    let mut memory = memory::get_upgrades_memory();
    let mut writer = Writer::new(&mut memory, 0);
    writer
        .write(&len.to_le_bytes())
        .expect("failed to save hub state len");
    writer
        .write(&state_bytes)
        .expect("failed to save hub state");
}

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct UpgradeArgs {
    pub admin: Option<Principal>,
    pub chain_id: Option<String>,
    pub hub_principal: Option<Principal>,
    pub chain_state: Option<ChainState>,
    pub schnorr_key_name: Option<String>,
    pub sol_canister: Option<Principal>,
    pub fee_account: Option<String>,
}

pub fn post_upgrade(args: Option<UpgradeArgs>) {
    let memory = memory::get_upgrades_memory();
    // Read the length of the state bytes.
    let mut state_len_bytes = [0; 4];
    memory.read(0, &mut state_len_bytes);
    let state_len = u32::from_le_bytes(state_len_bytes) as usize;

    // Read the bytes
    let mut state_bytes = vec![0; state_len];
    memory.read(4, &mut state_bytes);

    let mut state: SolanaRouteState =
        ciborium::de::from_reader(&*state_bytes).expect("failed to decode state");

    // update state
    if let Some(args) = args {
        if let Some(admin) = args.admin {
            state.admin = admin;
        }
        if let Some(chain_id) = args.chain_id {
            state.chain_id = chain_id;
        }
        if let Some(hub_principal) = args.hub_principal {
            state.hub_principal = hub_principal;
        }
        if let Some(chain_state) = args.chain_state {
            state.chain_state = chain_state;
        }
        if let Some(schnorr_key_name) = args.schnorr_key_name {
            state.schnorr_key_name = schnorr_key_name;
        }
        if let Some(sol_canister) = args.sol_canister {
            state.sol_canister = sol_canister;
        }
        if let Some(fee_account) = args.fee_account {
            state.fee_account = fee_account;
        }
    }

    replace_state(state);
}
