use crate::state::{mutate_state, CustomsState, read_state};
use std::marker::PhantomData;
use crate::guard::GuardError::KeyIsHandling;

const MAX_CONCURRENT: u64 = 100;

#[derive(Debug, PartialEq, Eq)]
pub enum GuardError {
    TooManyConcurrentRequests,
    KeyIsHandling,
}

pub trait PendingRequests {
    fn pending_requests(state: &mut CustomsState)  -> Result<(), GuardError>;
    fn incre_counter(state: &mut CustomsState);
    fn decre_counter(state: &mut CustomsState);
    fn key_is_handling(state: &mut CustomsState, requst_key: &String) -> Result<(), GuardError>;
    fn set_request_key(state: &mut CustomsState, request_key: String);
    fn remove_request_key(state: &mut CustomsState, request_key: &String);
}

pub struct GenerateTicketUpdates;

impl PendingRequests for GenerateTicketUpdates {
    fn pending_requests(state: &mut CustomsState) -> Result<(), GuardError> {
        if state.generate_ticket_counter >= MAX_CONCURRENT {
            return Err(GuardError::TooManyConcurrentRequests);
        }
        Ok(())
    }

    fn incre_counter(state: &mut CustomsState) {
        state.generate_ticket_counter += 1;
    }

    fn decre_counter(state: &mut CustomsState) {
        state.generate_ticket_counter -= 1;
    }

    fn key_is_handling(state: &mut CustomsState, request_key: &String) -> Result<(), GuardError> {
        if state.generating_txids.contains(request_key) {
            return Err(KeyIsHandling);
        }
        Ok(())
    }

    fn set_request_key(state: &mut CustomsState, request_key: String){
        state.generating_txids.insert(request_key);
    }

    fn remove_request_key(state: &mut CustomsState, request_key: &String) {
        state.generating_txids.remove(request_key);
    }
}
pub struct ReleaseTokenUpdates;

impl PendingRequests for ReleaseTokenUpdates {
    fn pending_requests(state: &mut CustomsState) -> Result<(), GuardError> {
        if state.release_token_counter >= MAX_CONCURRENT {
            return Err(GuardError::TooManyConcurrentRequests);
        }
        Ok(())
    }
    fn incre_counter(state: &mut CustomsState) {
        state.release_token_counter += 1;
    }
    fn decre_counter(state: &mut CustomsState) {
        state.release_token_counter -= 1;
    }

    fn key_is_handling(state: &mut CustomsState, _requst_key: &String) -> Result<(), GuardError> {
        Ok(())
    }

    fn set_request_key(state: &mut CustomsState, _request_key: String)  {
    }

    fn remove_request_key(state: &mut CustomsState, _request_key: &String) {
        //Nothing to do
    }
}

/// Guards a block from being executed [MAX_CONCURRENT] or more times in parallel.
#[must_use]
pub struct Guard<PR: PendingRequests> {
    request_key: String,
    _marker: PhantomData<PR>,
}

impl<PR: PendingRequests> Guard<PR> {
    /// Attempts to create a new guard for the current block.
    /// Fails if there are at least [MAX_CONCURRENT] pending requests.
    pub fn new(request_key: String) -> Result<Self, GuardError> {
        mutate_state(|s| {
            //check
            PR::pending_requests(s)?;
            PR::key_is_handling(s, &request_key)?;
            //lock
            PR::set_request_key(s, request_key.clone());
            PR::incre_counter(s);
            Ok(Self {
                request_key,
                _marker: PhantomData,
            })
        })
    }
}

impl<PR: PendingRequests> Drop for Guard<PR> {
    fn drop(&mut self) {
        mutate_state(|s| {
            PR::remove_request_key(s, &self.request_key);
            PR::decre_counter(s)
        });

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
pub struct ProcessDirectiveMsgGuard(());

impl ProcessDirectiveMsgGuard {
    pub fn new() -> Option<Self> {
        mutate_state(|s| {
            if s.is_process_directive_msg {
                return None;
            }
            s.is_process_directive_msg = true;
            Some(ProcessDirectiveMsgGuard(()))
        })
    }
}

impl Drop for ProcessDirectiveMsgGuard {
    fn drop(&mut self) {
        mutate_state(|s| {
            s.is_process_directive_msg = false;
        });
    }
}

#[must_use]
pub struct ProcessTicketMsgGuard(());

impl ProcessTicketMsgGuard {
    pub fn new() -> Option<Self> {
        mutate_state(|s| {
            if s.is_process_ticket_msg {
                return None;
            }
            s.is_process_ticket_msg = true;
            Some(ProcessTicketMsgGuard(()))
        })
    }
}

impl Drop for ProcessTicketMsgGuard {
    fn drop(&mut self) {
        mutate_state(|s| {
            s.is_process_ticket_msg = false;
        });
    }
}

#[must_use]
pub struct ProcessEtchingMsgGuard(());

impl crate::guard::ProcessEtchingMsgGuard {
    pub fn new() -> Option<Self> {
        mutate_state(|s| {
            if s.is_process_etching_msg {
                return None;
            }
            s.is_process_etching_msg = true;
            Some(crate::guard::ProcessEtchingMsgGuard(()))
        })
    }
}

impl Drop for crate::guard::ProcessEtchingMsgGuard {
    fn drop(&mut self) {
        mutate_state(|s| {
            s.is_process_etching_msg = false;
        });
    }
}

pub fn generate_ticket_guard(txid: String) -> Result<Guard<GenerateTicketUpdates>, GuardError> {
    Guard::new(txid)
}

pub fn release_token_guard() -> Result<Guard<ReleaseTokenUpdates>, GuardError> {
    Guard::new("".to_string())
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
            ecdsa_key_name: "some_key".to_string(),
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
