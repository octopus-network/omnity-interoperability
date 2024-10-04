use std::cell::RefCell;
use std::collections::BTreeMap;
use candid::CandidType;
use ic_stable_structures::writer::Writer;
use serde::{Deserialize, Serialize};
use crate::service::InitArgs;
use crate::stable_memory;
use crate::stable_memory::Memory;
use crate::state::BitcoinNetwork::Mainnet;

thread_local! {
    static STATE: RefCell<Option<IndexerState>> = RefCell::new(None);
}

#[derive(Deserialize, Serialize)]
pub struct IndexerState {
    pub api_keys: BTreeMap<String, String>,
    pub network: BitcoinNetwork,
    pub proxy_url: String,
}

impl IndexerState {
    pub fn init(init_args: InitArgs) -> anyhow::Result<Self> {
        let ret = IndexerState {
            api_keys: Default::default(),
            network: init_args.network,
            proxy_url: init_args.proxy_url,
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
        let state: IndexerState =
            ciborium::de::from_reader(&*state_bytes).expect("failed to decode state");
        replace_state(state);
    }
}

#[derive(Serialize, Deserialize,Copy, Clone, Debug, CandidType)]
pub enum  BitcoinNetwork {
    Mainnet, Testnet
}
pub fn api_key(rpc_name: &str) -> String {
    read_state(|s|s.api_keys.get(rpc_name).unwrap_or(&"na".to_string()).clone())
}

pub fn proxy_url() -> String {
    read_state(|s|s.proxy_url.clone())
}
pub fn mutate_state<F, R>(f: F) -> R
    where
        F: FnOnce(&mut IndexerState) -> R,
{
    STATE.with(|s| f(s.borrow_mut().as_mut().expect("State not initialized!")))
}

pub fn read_state<F, R>(f: F) -> R
    where
        F: FnOnce(&IndexerState) -> R,
{
    STATE.with(|s| f(s.borrow().as_ref().expect("State not initialized!")))
}

/// Replaces the current state.
pub fn replace_state(state: IndexerState) {
    STATE.with(|s| {
        *s.borrow_mut() = Some(state);
    });
}

pub fn take_state<F, R>(f: F) -> R
    where
        F: FnOnce(IndexerState) -> R,
{
    STATE.with(|s| f(s.take().expect("State not initialized!")))
}
