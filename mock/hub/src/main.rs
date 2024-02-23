use candid::candid_method;
use ic_cdk_macros::{init, update};
use omnity_types::{self, Ticket};
use std::{cell::RefCell, collections::VecDeque};

fn main() {}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct State {
    tickets: VecDeque<Ticket>,
}

impl Default for State {
    fn default() -> Self {
        State {
            tickets: VecDeque::default(),
        }
    }
}

pub fn mutate_state<F, R>(f: F) -> R
where
    F: FnOnce(&mut State) -> R,
{
    STATE.with(|s| f(&mut s.borrow_mut()))
}

pub fn read_state<F, R>(f: F) -> R
where
    F: FnOnce(&State) -> R,
{
    STATE.with(|s| f(&s.borrow()))
}

thread_local! {
    static STATE: RefCell<State> = RefCell::default();
}

#[init]
fn init() {
    STATE.with(|s| {
        let state = State {
            tickets: VecDeque::default(),
        };
        *s.borrow_mut() = state;
    });
}

#[candid_method(update)]
#[update]
pub async fn send_tickets(_: omnity_types::Ticket) -> Result<(), omnity_types::Error> {
    Ok(())
}
