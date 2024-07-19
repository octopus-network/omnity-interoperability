use crate::{
    constants::{QUERY_DERECTIVE_INTERVAL, TICKET_INTERVAL},
    guard::{TaskType, TimerGuard},
};

use super::{directive, sol_call, ticket};

pub fn start_schedule() {
    // query_directives task
    ic_cdk_timers::set_timer_interval(QUERY_DERECTIVE_INTERVAL, || {
        ic_cdk::spawn(async {
            let _guard = match TimerGuard::new(TaskType::GetDirectives) {
                Ok(guard) => guard,
                Err(_) => return,
            };

            directive::query_directives().await;
        });
    });

    // query_tickets task
    ic_cdk_timers::set_timer_interval(TICKET_INTERVAL, || {
        ic_cdk::spawn(async {
            let _guard = match TimerGuard::new(TaskType::GetTickets) {
                Ok(guard) => guard,
                Err(_) => return,
            };

            ticket::query_tickets().await;
        });
    });

    // query tx signature status
    // ic_cdk_timers::set_timer_interval(SIGNATUE_STATUS_INTERVAL, || {
    //     ic_cdk::spawn(async {
    //         let _guard = match TimerGuard::new(TaskType::GetSignatureStatus) {
    //             Ok(guard) => guard,
    //             Err(_) => return,
    //         };

    //         sol_tx::get_signaute_status().await;
    //     });
    // });
}

//TODO: stop running jobs
pub fn cannel_schedule_jobs() {}
