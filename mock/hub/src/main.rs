use candid::candid_method;
use ic_cdk::query;
use ic_cdk_macros::{init, update};
use omnity_types::{self, ChainId, Seq, Ticket};
use std::ops::Bound::Included;
use std::{cell::RefCell, collections::BTreeMap};

fn main() {}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct State {
    tickets: BTreeMap<Seq, Ticket>,
    next_seq: Seq,
}

impl Default for State {
    fn default() -> Self {
        State {
            tickets: BTreeMap::default(),
            next_seq: 1,
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
            tickets: BTreeMap::default(),
            next_seq: 1,
        };
        *s.borrow_mut() = state;
    });
}

#[candid_method(update)]
#[update]
pub async fn send_ticket(_: Ticket) -> Result<(), omnity_types::Error> {
    Ok(())
}

#[query]
pub async fn query_tickets(
    _: ChainId,
    start: u64,
    end: u64,
) -> Result<Vec<(Seq, Ticket)>, omnity_types::Error> {
    read_state(|s| {
        let mut result = Vec::new();
        for (&seq, ticket) in s.tickets.range((Included(start), Included(end))) {
            result.push((seq, ticket.clone()));
        }
        Ok(result)
    })
}

#[candid_method(update)]
#[update]
pub async fn push_ticket(ticket: Ticket) -> Result<(), omnity_types::Error> {
    mutate_state(|s| {
        s.tickets.insert(s.next_seq, ticket);
        s.next_seq += 1;
        Ok(())
    })
}
