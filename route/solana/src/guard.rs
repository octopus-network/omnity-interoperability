use crate::state::mutate_state;

#[must_use]
pub struct TimerGuard(());

impl TimerGuard {
    pub fn new() -> Option<Self> {
        mutate_state(|s| {
            if s.is_timer_running {
                return None;
            }
            s.is_timer_running = true;
            Some(TimerGuard(()))
        })
    }
}

impl Drop for TimerGuard {
    fn drop(&mut self) {
        mutate_state(|s| {
            s.is_timer_running = false;
        });
    }
}
