use std::time::Duration;

use ic_cdk_macros::{init, query, update};
use ic_cdk_timers::set_timer_interval;
use ic_log::writer::Logs;
use icp_route::lifecycle::{self, init::RouteArg};
use icp_route::log_util::init_log;
use icp_route::updates::generate_ticket::{
    GenerateTicketArgs, GenerateTicketError, GenerateTicketOk,
};
use icp_route::updates::{self};
use icp_route::{periodic_task, PERIODIC_TASK_INTERVAL};
use log::{self};

#[init]
fn init(args: RouteArg) {
    match args {
        RouteArg::Init(args) => {
            init_log();
            lifecycle::init::init(args);
            set_timer_interval(Duration::from_secs(PERIODIC_TASK_INTERVAL), periodic_task);
        }
        RouteArg::Upgrade() => {
            panic!("expected InitArgs got UpgradeArgs");
        }
    }
}

#[update]
async fn generate_ticket(
    args: GenerateTicketArgs,
) -> Result<GenerateTicketOk, GenerateTicketError> {
    updates::generate_ticket(args).await
}

#[query]
pub fn get_log_records(limit: usize, offset: usize) -> Logs {
    log::debug!("collecting {limit} log records");
    ic_log::take_memory_records(limit, offset)
}

fn main() {}

// Enable Candid export
ic_cdk::export_candid!();
