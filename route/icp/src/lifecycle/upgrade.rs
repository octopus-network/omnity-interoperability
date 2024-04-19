use crate::{
    state::{eventlog::Event, replace_state, RouteState},
    storage::record_event,
};
use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct UpgradeArgs {
    pub chain_id: Option<String>,
    pub hub_principal: Option<Principal>,
}

pub fn post_upgrade(upgrade_args: Option<UpgradeArgs>) {
    let (mut stable_state,): (RouteState,) =
        ic_cdk::storage::stable_restore().expect("failed to restore state");

    if let Some(upgrade_args) = upgrade_args {
        log::info!("[upgrade]: updating configuration with {:?}", upgrade_args);

        stable_state.upgrade(upgrade_args.clone());
        record_event(&Event::Upgrade(upgrade_args));
    };
    replace_state(stable_state);
}
