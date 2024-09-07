use crate::memory::mutate_guard_running_task;

#[must_use]
pub struct LogicGuard(String);

impl LogicGuard {
    pub fn new(task_name: String) -> Option<Self> {
        mutate_guard_running_task(|s| {
            if s.contains(&task_name) {
                return None::<LogicGuard>;
            }

            s.insert(task_name.clone());
            Some(LogicGuard(task_name))
        })
    }
}

impl Drop for LogicGuard {
    fn drop(&mut self) {
        mutate_guard_running_task(|s| {
            s.remove(&self.0);
        });
    }
}
