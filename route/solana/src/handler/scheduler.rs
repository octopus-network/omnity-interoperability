use ic_cdk_timers::TimerId;
use std::cell::RefCell;
use std::collections::HashMap;

use crate::{
    constants::{HANDLE_TICKET_INTERVAL, QUERY_DERECTIVE_INTERVAL, QUERY_TICKET_INTERVAL},
    guard::{TaskType, TimerGuard},
};

use super::{directive, ticket};

thread_local! {
    static TIMER_GUARD: RefCell<HashMap<TaskType,TimerId>> = RefCell::new(HashMap::default());
}

pub fn start_schedule() {
    // query_directives task
    let directive_timer_id = ic_cdk_timers::set_timer_interval(QUERY_DERECTIVE_INTERVAL, || {
        ic_cdk::spawn(async {
            let _guard = match TimerGuard::new(TaskType::GetDirectives) {
                Ok(guard) => guard,
                Err(_) => return,
            };

            directive::query_directives().await;
        });
    });
    ic_cdk::println!(" started query_directives task : {:?}", directive_timer_id);
    TIMER_GUARD.with_borrow_mut(|guard| {
        guard.insert(TaskType::GetDirectives, directive_timer_id);
    });

    // query_tickets task
    let ticket_timer_id = ic_cdk_timers::set_timer_interval(QUERY_TICKET_INTERVAL, || {
        ic_cdk::spawn(async {
            let _guard = match TimerGuard::new(TaskType::GetTickets) {
                Ok(guard) => guard,
                Err(_) => return,
            };

            ticket::query_tickets().await;
        });
    });
    ic_cdk::println!(" started query_tickets task : {:?}", ticket_timer_id);
    TIMER_GUARD.with_borrow_mut(|guard| {
        guard.insert(TaskType::GetTickets, ticket_timer_id);
    });

    // handle to mint token based on ticket
    let handle_ticket_timer_id = ic_cdk_timers::set_timer_interval(HANDLE_TICKET_INTERVAL, || {
        ic_cdk::spawn(async {
            let _guard = match TimerGuard::new(TaskType::HandleTickets) {
                Ok(guard) => guard,
                Err(_) => return,
            };

            ticket::handle_tickets().await;
        });
    });
    ic_cdk::println!(
        " started handle_tickets task : {:?}",
        handle_ticket_timer_id
    );
    TIMER_GUARD.with_borrow_mut(|guard| {
        guard.insert(TaskType::HandleTickets, handle_ticket_timer_id);
    });
}

// clear the running tasks
pub fn cannel_schedule() {
    TIMER_GUARD.with_borrow_mut(|guard| {
        guard
            .iter()
            .for_each(|(_task_type, task_id)| ic_cdk_timers::clear_timer(*task_id));
        guard.clear()
    });
}
