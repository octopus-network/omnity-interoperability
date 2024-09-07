use std::time::Duration;

use ic_cdk_timers::set_timer_interval;

use crate::{business::{process_directive::process_directive_task, ticket_task::process_ticket_task}, const_args, memory::{insert_periodic_job_manager, PeriodicJobManager}};

pub fn start_all_periodic_jobs() {
    start_process_directive_job();
    start_process_ticket_job();
}

pub fn start_process_directive_job() {
    let interval = Duration::from_secs(const_args::INTERVAL_QUERY_DIRECTIVE);
    let timer_id = set_timer_interval(
        interval,
        process_directive_task,
    );

    let job_name = const_args::PROCESS_DIRECTIVE_JOB_NAME.to_string();
    insert_periodic_job_manager(
        job_name, 
        PeriodicJobManager::new(
            const_args::PROCESS_DIRECTIVE_JOB_NAME.to_string(), 
            timer_id, 
            const_args::INTERVAL_QUERY_DIRECTIVE
        )
    );
    
}

pub fn start_process_ticket_job() {
    let interval = Duration::from_secs(const_args::INTERVAL_QUERY_TICKET);
    let timer_id = set_timer_interval(
        interval,
        process_ticket_task,
    );

    let job_name = const_args::PROCESS_TICKET_JOB_NAME.to_string();

    insert_periodic_job_manager(
        job_name, 
        PeriodicJobManager::new(
            const_args::PROCESS_TICKET_JOB_NAME.to_string(), 
            timer_id, 
            const_args::INTERVAL_QUERY_DIRECTIVE
        )
    );

}