use crate::constants::*;
use crate::hub_to_custom::{fetch_hub_directive_task, fetch_hub_ticket_task};
use ic_cdk_timers::set_timer_interval;
use std::time::Duration;

fn start_tasks() {
    set_timer_interval(
        Duration::from_secs(FETCH_HUB_TICKET_INTERVAL),
        fetch_hub_ticket_task,
    );
    set_timer_interval(
        Duration::from_secs(FETCH_HUB_DIRECTIVE_INTERVAL),
        fetch_hub_directive_task,
    );

    set_timer_interval()

}


