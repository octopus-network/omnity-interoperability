use crate::state::mutate_settings;

#[must_use]
pub struct TimerLogicGuard(String);

impl TimerLogicGuard {
    pub fn new(task_name: String) -> Option<Self> {
        mutate_settings( |s| {
            if s.is_timer_running.contains(&task_name) {
                return None::<TimerLogicGuard>;
            }

            s.is_timer_running.insert(task_name.clone());
            Some(TimerLogicGuard(task_name))
        })
    }
}

impl Drop for TimerLogicGuard {
    fn drop(&mut self) {
        mutate_settings(|s| {
            s.is_timer_running.remove(&self.0);
        });
    }
}
