use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct UpgradeArgs {
    pub chain_id: Option<String>,
    pub hub_principal: Option<Principal>,
}

pub fn post_upgrade(upgrade_args: Option<UpgradeArgs>) {
    if let Some(upgrade_args) = upgrade_args {}
}
