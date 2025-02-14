#![allow(unused)]
use crate::config::mutate_config;
use candid::CandidType;
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

#[derive(CandidType, Serialize, Deserialize, Debug, Hash, Copy, Clone, PartialEq, Eq, EnumIter)]
pub enum TaskType {
    GetDirectives,
    GetTickets,

    UpdateToken,
    MintToken,
    ClearTicket,
    BurnToken,
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
        mutate_config(|s| {
            let mut config = s.get().to_owned();
            if !config.active_tasks.insert(task) {
                return Err(TimerGuardError::AlreadyProcessing);
            }
            s.set(config);
            Ok(Self { task })
        })
    }
}

impl Drop for TimerGuard {
    fn drop(&mut self) {
        mutate_config(|s| {
            let mut config = s.get().to_owned();
            config.active_tasks.remove(&self.task);
            s.set(config);
        });
    }
}
