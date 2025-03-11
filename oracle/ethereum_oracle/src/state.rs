use crate::stable_memory;
use candid::{CandidType, Principal};
use ic_stable_structures::writer::Writer;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;

thread_local! {
    static STATE: RefCell<Option<OracleState>> = const { RefCell::new(None) };
}

#[derive(Deserialize, Serialize)]
pub struct InitArgs {
    pub execution_rpc: String,
    pub ethereum_route_principal: Principal,
}

#[derive(Deserialize, Serialize)]
pub struct OracleState {
    pub proxy_url: String,
    pub ethereum_route_principal: Principal,
    pub authorized_callers: Vec<String>,
}

impl OracleState {
    pub fn init(init_args: InitArgs) -> anyhow::Result<Self> {
        let ret = OracleState {
            proxy_url: init_args.execution_rpc,
            ethereum_route_principal: init_args.ethereum_route_principal,
            authorized_callers: vec![],
        };
        Ok(ret)
    }

    pub fn pre_upgrade(&self) {
        let mut state_bytes = vec![];
        let _ = ciborium::ser::into_writer(self, &mut state_bytes);
        let len = state_bytes.len() as u32;
        let mut memory = crate::stable_memory::get_upgrade_stash_memory();
        let mut writer = Writer::new(&mut memory, 0);
        writer
            .write(&len.to_le_bytes())
            .expect("failed to save hub state len");
        writer
            .write(&state_bytes)
            .expect("failed to save hub state");
    }

    pub fn post_upgrade() {
        use ic_stable_structures::Memory;
        let memory = stable_memory::get_upgrade_stash_memory();
        // Read the length of the state bytes.
        let mut state_len_bytes = [0; 4];
        memory.read(0, &mut state_len_bytes);
        let state_len = u32::from_le_bytes(state_len_bytes) as usize;
        let mut state_bytes = vec![0; state_len];
        memory.read(4, &mut state_bytes);
        let state: OracleState =
            ciborium::de::from_reader(&*state_bytes).expect("failed to decode state");
        replace_state(state);
    }
}


pub fn mutate_state<F, R>(f: F) -> R
    where
        F: FnOnce(&mut OracleState) -> R,
{
    STATE.with(|s| f(s.borrow_mut().as_mut().expect("State not initialized!")))
}

pub fn read_state<F, R>(f: F) -> R
    where
        F: FnOnce(&OracleState) -> R,
{
    STATE.with(|s| f(s.borrow().as_ref().expect("State not initialized!")))
}

/// Replaces the current state.
pub fn replace_state(state: OracleState) {
    STATE.with(|s| {
        *s.borrow_mut() = Some(state);
    });
}
