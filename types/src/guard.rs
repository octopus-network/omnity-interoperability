use std::fmt::Display;

pub struct CommonGuard<GB: GuardBehavior> {
    pub request_key: GB::KeyType,
}

#[derive(Debug, PartialEq, Eq)]
pub enum GuardError {
    TooManyConcurrentRequests,
    KeyIsHandling,
}

impl Display for GuardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            GuardError::TooManyConcurrentRequests => { "too many concurrent requests".to_string() }
            GuardError::KeyIsHandling => { "request is duplicate".to_string() }
        };
        write!(f, "{}", str)
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


#[macro_export]
macro_rules! impl_guard_behavior {
    ($name:ty,  $ty: ty) => {
        use omnity_types::guard::{GuardBehavior, GuardError};
        impl GuardBehavior for $name {
            type KeyType = $ty;
            fn check_lock(key: &Self::KeyType) -> Result<(), GuardError> {
                if mutate_state(|s|s.generating_ticketid.contains(key)) {
                    Err(GuardError::KeyIsHandling)
                }else {
                    Ok(())
                }
            }

            fn set_lock(key: &Self::KeyType) {
                mutate_state(|s|s.generating_ticketid.insert(key.clone()));
            }

            fn release_lock(key: &Self::KeyType) {
                mutate_state(|s|s.generating_ticketid.remove(key));
            }
        }
    };
}
