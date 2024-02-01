use crate::state::{mutate_state, CustomState};
use std::marker::PhantomData;

const MAX_CONCURRENT: u64 = 100;

#[derive(Debug, PartialEq, Eq)]
pub enum GuardError {
    AlreadyProcessing,
    TooManyConcurrentRequests,
}

pub trait PendingRequests {
    fn pending_requests(state: &mut CustomState) -> u64;
    fn incre_counter(state: &mut CustomState);
    fn decre_counter(state: &mut CustomState);
}

pub struct GenerateTicketUpdates;

impl PendingRequests for GenerateTicketUpdates {
    fn pending_requests(state: &mut CustomState) -> u64 {
        state.generate_ticket_counter
    }
    fn incre_counter(state: &mut CustomState) {
        state.generate_ticket_counter += 1;
    }
    fn decre_counter(state: &mut CustomState) {
        state.generate_ticket_counter -= 1;
    }
}
pub struct ReleaseTokenUpdates;

impl PendingRequests for ReleaseTokenUpdates {
    fn pending_requests(state: &mut CustomState) -> u64 {
        state.release_token_counter
    }
    fn incre_counter(state: &mut CustomState) {
        state.release_token_counter += 1;
    }
    fn decre_counter(state: &mut CustomState) {
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
    use candid::Principal;
    use ic_base_types::CanisterId;

    fn test_principal(id: u64) -> Principal {
        Principal::try_from_slice(&id.to_le_bytes()).unwrap()
    }

    fn test_state_args() -> InitArgs {
        InitArgs {
            btc_network: BtcNetwork::Regtest,
            ecdsa_key_name: "some_key".to_string(),
            release_min_amount: 2000,
            max_time_in_queue_nanos: 0,
            min_confirmations: None,
            mode: crate::state::Mode::GeneralAvailability,
            hub_principal: Principal::from(CanisterId::from(0)),
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
