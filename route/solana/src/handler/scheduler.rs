use super::{directive, port_msg, ticket};
use std::time::Duration;
pub const QUERY_DERECTIVE_INTERVAL: Duration = Duration::from_secs(60);
pub const EXE_DERECTIVE_INTERVAL: Duration = Duration::from_secs(5);
pub const TICKET_INTERVAL: Duration = Duration::from_secs(5);
pub const PORT_MSG_INTERVAL: Duration = Duration::from_secs(5);

pub fn schedule_jobs() {
    // query_directives task
    ic_cdk_timers::set_timer_interval(QUERY_DERECTIVE_INTERVAL, || {
        ic_cdk::spawn(async {
            let _guard = match crate::guard::TimerGuard::new() {
                Some(guard) => guard,
                None => return,
            };

            let _ = directive::query_directives().await;
        });
    });

    // execute directives
    ic_cdk_timers::set_timer_interval(EXE_DERECTIVE_INTERVAL, || {
        ic_cdk::spawn(async {
            let _guard = match crate::guard::TimerGuard::new() {
                Some(guard) => guard,
                None => return,
            };

            let _ = directive::execute_directives().await;
        });
    });

    // query_tickets task
    ic_cdk_timers::set_timer_interval(TICKET_INTERVAL, || {
        ic_cdk::spawn(async {
            let _guard = match crate::guard::TimerGuard::new() {
                Some(guard) => guard,
                None => return,
            };

            let _ = ticket::query_tickets().await;
        });
    });

    // handle port msg
    ic_cdk_timers::set_timer_interval(PORT_MSG_INTERVAL, || {
        ic_cdk::spawn(async {
            let _guard = match crate::guard::TimerGuard::new() {
                Some(guard) => guard,
                None => return,
            };

            let _ = port_msg::handle_port_msg().await;
        });
    });
}
