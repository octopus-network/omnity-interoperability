use crate::state::{mutate_state, CustomsState};
use std::marker::PhantomData;

const MAX_CONCURRENT: u64 = 100;

#[derive(Debug, PartialEq, Eq)]
pub enum GuardError {
    TooManyConcurrentRequests,
}

pub trait PendingRequests {
    fn pending_requests(state: &mut CustomsState) -> u64;
    fn incre_counter(state: &mut CustomsState);
    fn decre_counter(state: &mut CustomsState);
}

pub struct GenerateTicketUpdates;

impl PendingRequests for GenerateTicketUpdates {
    fn pending_requests(state: &mut CustomsState) -> u64 {
        state.generate_ticket_counter
    }

    fn incre_counter(state: &mut CustomsState) {
        state.generate_ticket_counter += 1;
    }

    fn decre_counter(state: &mut CustomsState) {
        state.generate_ticket_counter -= 1;
    }
}
pub struct ReleaseTokenUpdates;

impl PendingRequests for ReleaseTokenUpdates {
    fn pending_requests(state: &mut CustomsState) -> u64 {
        state.release_token_counter
    }

    fn incre_counter(state: &mut CustomsState) {
        state.release_token_counter += 1;
    }

    fn decre_counter(state: &mut CustomsState) {
        state.release_token_counter -= 1;
    }
}

/// Guards a block from being executed [MAX_CONCURRENT] or more times in parallel.
#[must_use]
pub struct Guard<PR: PendingRequests> {
    _marker: PhantomData<PR>,
}

impl<PR: PendingRequests> Guard<PR> {
    /// Attempts to create a new guard for the current block.
    /// Fails if there are at least [MAX_CONCURRENT] pending requests.
    pub fn new() -> Result<Self, GuardError> {
        mutate_state(|s| {
            let counter = PR::pending_requests(s);
            if counter >= MAX_CONCURRENT {
                return Err(GuardError::TooManyConcurrentRequests);
            }
            PR::incre_counter(s);
            Ok(Self {
                _marker: PhantomData,
            })
        })
    }
}

impl<PR: PendingRequests> Drop for Guard<PR> {
    fn drop(&mut self) {
        mutate_state(|s| PR::decre_counter(s));
    }
}

#[must_use]
pub struct TimerLogicGuard(());

impl TimerLogicGuard {
    pub fn new() -> Option<Self> {
        mutate_state(|s| {
            if s.is_timer_running {
                return None;
            }
            s.is_timer_running = true;
            Some(TimerLogicGuard(()))
        })
    }
}

impl Drop for TimerLogicGuard {
    fn drop(&mut self) {
        mutate_state(|s| {
            s.is_timer_running = false;
        });
    }
}

#[must_use]
pub struct ProcessHubMsgGuard(());

impl ProcessHubMsgGuard {
    pub fn new() -> Option<Self> {
        mutate_state(|s| {
            if s.is_process_hub_msg {
                return None;
            }
            s.is_process_hub_msg = true;
            Some(ProcessHubMsgGuard(()))
        })
    }
}

impl Drop for ProcessHubMsgGuard {
    fn drop(&mut self) {
        mutate_state(|s| {
            s.is_process_hub_msg = false;
        });
    }
}

pub fn generate_ticket_guard() -> Result<Guard<GenerateTicketUpdates>, GuardError> {
    Guard::new()
}

pub fn release_token_guard() -> Result<Guard<ReleaseTokenUpdates>, GuardError> {
    Guard::new()
}

#[cfg(test)]
mod tests {
    use super::TimerLogicGuard;
    use crate::{
        lifecycle::init::{init, BtcNetwork, InitArgs},
        state::read_state,
    };
    use ic_base_types::CanisterId;
    use omnity_types::ChainState;

    fn test_state_args() -> InitArgs {
        InitArgs {
            btc_network: BtcNetwork::Regtest,
            max_time_in_queue_nanos: 0,
            min_confirmations: None,
            chain_state: ChainState::Active,
            hub_principal: CanisterId::from(0).into(),
            runes_oracle_principal: CanisterId::from(0).into(),
            chain_id: "Bitcoin".into(),
        }
    }

    #[test]
    fn guard_timer_guard() {
        init(test_state_args());
        assert!(!read_state(|s| s.is_timer_running));

        let guard = TimerLogicGuard::new().expect("could not grab timer logic guard");
        assert!(TimerLogicGuard::new().is_none());
        assert!(read_state(|s| s.is_timer_running));

        drop(guard);
        assert!(!read_state(|s| s.is_timer_running));
    }
}
