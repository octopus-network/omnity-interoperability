use candid::candid_method;
use ic_cdk::query;
use ic_cdk_macros::{init, update};
use omnity_types::{self, ChainId, Directive, Seq, Ticket, Topic};
use std::{cell::RefCell, collections::BTreeMap};

fn main() {}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct State {
    tickets: BTreeMap<Seq, Ticket>,
    directives: BTreeMap<Seq, Directive>,
    next_ticket_seq: Seq,
    next_directive_seq: Seq,
}

impl Default for State {
    fn default() -> Self {
        State {
            tickets: BTreeMap::default(),
            directives: BTreeMap::default(),
            next_ticket_seq: 0,
            next_directive_seq: 0,
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
            directives: BTreeMap::default(),
            next_ticket_seq: 0,
            next_directive_seq: 0,
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
    _: Option<ChainId>,
    offset: usize,
    limit: usize,
) -> Result<Vec<(Seq, Ticket)>, omnity_types::Error> {
    read_state(|s| {
        Ok(s.tickets
            .iter()
            .skip(offset)
            .take(limit)
            .map(|(seq, ticket)| (*seq, ticket.clone()))
            .collect())
    })
}

#[query]
pub async fn query_directives(
    _: Option<ChainId>,
    _: Option<Topic>,
    offset: usize,
    limit: usize,
) -> Result<Vec<(Seq, Directive)>, omnity_types::Error> {
    read_state(|s| {
        Ok(s.directives
            .iter()
            .skip(offset)
            .take(limit)
            .map(|(seq, dire)| (*seq, dire.clone()))
            .collect())
    })
}

#[candid_method(update)]
#[update]
pub async fn push_ticket(ticket: Ticket) -> Result<(), omnity_types::Error> {
    mutate_state(|s| {
        s.tickets.insert(s.next_ticket_seq, ticket);
        s.next_ticket_seq += 1;
        Ok(())
    })
}

#[candid_method(update)]
#[update]
pub async fn push_directives(directives: Vec<Directive>) -> Result<(), omnity_types::Error> {
    mutate_state(|s| {
        for dire in directives {
            s.directives.insert(s.next_directive_seq, dire);
            s.next_directive_seq += 1;
        }
        Ok(())
    })
}

// Enable Candid export
ic_cdk::export_candid!();
