use crate::state::mutate_state;
use candid::CandidType;
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

#[derive(CandidType, Serialize, Deserialize, Debug, Hash, Copy, Clone, PartialEq, Eq, EnumIter)]
pub enum TaskType {
    GetDirectives,
    GetTickets,
    HandleTickets,
}

#[derive(Debug, PartialEq, Eq)]
pub enum TimerGuardError {
    AlreadyProcessing,
}

#[must_use]
#[derive(Debug, PartialEq, Eq)]
pub struct TimerGuard {
    task: TaskType,
}

impl TimerGuard {
    pub fn new(task: TaskType) -> Result<Self, TimerGuardError> {
        mutate_state(|s| {
            if !s.active_tasks.insert(task) {
                return Err(TimerGuardError::AlreadyProcessing);
            }
            Ok(Self { task })
        })
    }
}

impl Drop for TimerGuard {
    fn drop(&mut self) {
        mutate_state(|s| {
            s.active_tasks.remove(&self.task);
        });
    }
}
