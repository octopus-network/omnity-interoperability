use std::cell::RefCell;

use candid::Principal;

use crate::lifecycle::init::InitArgs;

thread_local! {
    static __STATE: RefCell<Option<RouteState>> = RefCell::default();
}

pub struct RouteState {
    pub schnorr_canister_principal: Principal,
}

impl From<InitArgs> for RouteState {
    fn from(args: InitArgs) -> Self {
        Self {
            schnorr_canister_principal: args.schnorr_canister_principal,
        }
    }
}

pub fn take_state<F, R>(f: F) -> R
where
    F: FnOnce(RouteState) -> R,
{
    __STATE.with(|s| f(s.take().expect("State not initialized!")))
}

pub fn read_state<F, R>(f: F) -> R
where
    F: FnOnce(&RouteState) -> R,
{
    __STATE.with(|s| f(s.borrow().as_ref().expect("State not initialized!")))
}

pub fn replace_state(state: RouteState) {
    __STATE.with(|s| {
        *s.borrow_mut() = Some(state);
    });
}