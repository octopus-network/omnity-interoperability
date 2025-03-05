use std::marker::PhantomData;

pub struct CommonGuard<PR: GuardBehavior> {
    pub request_key: PR::KeyType,
}

#[derive(Debug, PartialEq, Eq)]
pub enum GuardError {
    TooManyConcurrentRequests,
    KeyIsHandling,
}

impl<PR: GuardBehavior> CommonGuard<PR> {
    /// Attempts to create a new guard for the current block.
    /// Fails if there are at least [MAX_CONCURRENT] pending requests.
    pub fn new(request_key: PR::KeyType) -> Result<Self, GuardError> {
        let guard = Self {
            request_key,
        };
        PR::check_lock(&guard.request_key)?;
        PR::set_lock(&guard.request_key);
        Ok(guard)
    }
}

impl<PR: GuardBehavior> Drop for CommonGuard<PR> {
    fn drop(&mut self) {
        PR::release_lock(&self.request_key);
    }
}


pub trait GuardBehavior {
    type KeyType;
    fn check_lock(key: &Self::KeyType) -> Result<(), GuardError>;
    fn set_lock(key: &Self::KeyType);
    fn release_lock(key: &Self::KeyType);
}
