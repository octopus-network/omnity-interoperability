use crate::state::mutate_state;
use omnity_types::impl_guard_behavior;

#[must_use]
pub struct TimerLogicGuard(String);

impl TimerLogicGuard {
    pub fn new(task_name: String) -> Option<Self> {
        mutate_state(|s| {
            let running = s
                .is_timer_running
                .get(&task_name)
                .cloned()
                .unwrap_or_default();
            if running {
                return None;
            }
            s.is_timer_running.insert(task_name.clone(), true);
            Some(TimerLogicGuard(task_name))
        })
    }
}

impl Drop for TimerLogicGuard {
    fn drop(&mut self) {
        mutate_state(|s| s.is_timer_running.remove(&self.0));
    }
}

pub struct GenerateTicketGuardBehavior;
impl_guard_behavior!(GenerateTicketGuardBehavior, String);
