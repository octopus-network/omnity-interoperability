use crate::logs::P0;
use crate::state::eventlog::{replay, Event};
use crate::state::replace_state;
use crate::storage::{count_events, events, record_event};
use candid::{CandidType, Deserialize, Principal};
use ic_canister_log::log;
use omnity_types::ChainState;
use serde::Serialize;

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct UpgradeArgs {
    /// Specifies the minimum number of confirmations on the Bitcoin network
    /// required for the customs to accept a transaction.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_confirmations: Option<u32>,

    /// Maximum time in nanoseconds that a transaction should spend in the queue
    /// before being sent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_time_in_queue_nanos: Option<u64>,

    /// The mode in which the customs is running.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain_state: Option<ChainState>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub hub_principal: Option<Principal>,
}

pub fn post_upgrade(upgrade_args: Option<UpgradeArgs>) {
    if let Some(upgrade_args) = upgrade_args {
        log!(
            P0,
            "[upgrade]: updating configuration with {:?}",
            upgrade_args
        );
        record_event(&Event::Upgrade(upgrade_args));
    };

    let start = ic_cdk::api::instruction_counter();

    log!(P0, "[upgrade]: replaying {} events", count_events());

    let state = replay(events()).unwrap_or_else(|e| {
        ic_cdk::trap(&format!(
            "[upgrade]: failed to replay the event log: {:?}",
            e
        ))
    });

    state.validate_config();

    replace_state(state);

    let end = ic_cdk::api::instruction_counter();

    log!(
        P0,
        "[upgrade]: replaying events consumed {} instructions",
        end - start
    );
}
