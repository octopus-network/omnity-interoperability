use crate::state::mutate_state;

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

pub struct CommonGuard<GB: GuardBehavior> {
    pub request_key: GB::KeyType,
}

#[derive(Debug, PartialEq, Eq)]
pub enum GuardError {
    TooManyConcurrentRequests,
    KeyIsHandling,
}

impl ToString for GuardError {
    fn to_string(&self) -> String {
        match self {
            GuardError::TooManyConcurrentRequests => { "too many concurrent requests".to_string() }
            GuardError::KeyIsHandling => { "request is duplicate".to_string() }
        }
    }
}

impl<GB: GuardBehavior> CommonGuard<GB> {
    /// Attempts to create a new guard for the current block.
    /// Fails if there are at least [MAX_CONCURRENT] pending requests.
    pub fn new(request_key: GB::KeyType) -> Result<Self, GuardError> {
        let guard = Self {
            request_key,
        };
        GB::check_lock(&guard.request_key)?;
        GB::set_lock(&guard.request_key);
        Ok(guard)
    }
}

impl<GB: GuardBehavior> Drop for CommonGuard<GB> {
    fn drop(&mut self) {
        GB::release_lock(&self.request_key);
    }
}


pub trait GuardBehavior {
    type KeyType;
    fn check_lock(key: &Self::KeyType) -> Result<(), GuardError>;
    fn set_lock(key: &Self::KeyType);
    fn release_lock(key: &Self::KeyType);
}

pub struct GenerateTicketGuardBehavior;

impl GuardBehavior for GenerateTicketGuardBehavior {
    type KeyType = String;
    fn check_lock(key: &Self::KeyType) -> Result<(), GuardError> {
        if mutate_state(|s| s.generating_ticketid.contains(key)) {
            Err(GuardError::KeyIsHandling)
        } else {
            Ok(())
        }
    }

    fn set_lock(key: &Self::KeyType) {
        mutate_state(|s| s.generating_ticketid.insert(key.clone()));
    }

    fn release_lock(key: &Self::KeyType) {
        mutate_state(|s| s.generating_ticketid.remove(key));
    }
}