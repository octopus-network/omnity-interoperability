use std::time::Duration;

use ic_cdk_timers::set_timer_interval;
use crate::bitcoin_to_custom::finalize_lock_ticket_task;

use crate::constants::*;
use crate::custom_to_bitcoin::finalize_unlock_tickets_task;
use crate::hub_to_custom::{fetch_hub_directive_task, fetch_hub_ticket_task};

pub fn start_tasks() {
    set_timer_interval(
        Duration::from_secs(FETCH_HUB_TICKET_INTERVAL),
        fetch_hub_ticket_task,
    );
    set_timer_interval(
        Duration::from_secs(FETCH_HUB_DIRECTIVE_INTERVAL),
        fetch_hub_directive_task,
    );
    set_timer_interval(
        Duration::from_secs(FINALIZE_LOCK_TICKET_INTERVAL),
        finalize_lock_ticket_task
    );

    set_timer_interval(
        Duration::from_secs(FINALIZE_UNLOCK_TICKET_INTERVAL),
        finalize_unlock_tickets_task
    );
}


