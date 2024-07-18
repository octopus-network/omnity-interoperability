use crate::guard::{TaskType, TimerGuard};

use super::{directive, ticket};
use std::time::Duration;
pub const QUERY_DERECTIVE_INTERVAL: Duration = Duration::from_secs(60);
pub const EXE_DERECTIVE_INTERVAL: Duration = Duration::from_secs(5);
pub const TICKET_INTERVAL: Duration = Duration::from_secs(5);
pub const SIGNATUE_STATUS_INTERVAL: Duration = Duration::from_secs(5);

pub fn start_schedule() {
    // query_directives task
    ic_cdk_timers::set_timer_interval(QUERY_DERECTIVE_INTERVAL, || {
        ic_cdk::spawn(async {
            let _guard = match TimerGuard::new(TaskType::GetDirectives) {
                Ok(guard) => guard,
                Err(_) => return,
            };

            let _ = directive::query_directives().await;
        });
    });

    // query_tickets task
    ic_cdk_timers::set_timer_interval(TICKET_INTERVAL, || {
        ic_cdk::spawn(async {
            let _guard = match TimerGuard::new(TaskType::GetTickets) {
                Ok(guard) => guard,
                Err(_) => return,
            };

            let _ = ticket::query_tickets().await;
        });
    });

    // query tx signature status
    ic_cdk_timers::set_timer_interval(SIGNATUE_STATUS_INTERVAL, || {
        ic_cdk::spawn(async {
            let _guard = match TimerGuard::new(TaskType::GetSignatureStatus) {
                Ok(guard) => guard,
                Err(_) => return,
            };

            let _ = ticket::get_signaute_status().await;
        });
    });
}

//TODO: stop running jobs
pub fn cannel_schedule() {}
