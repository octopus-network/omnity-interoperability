use ic_cdk_macros::{init, update};
use icp_route::lifecycle::{self, init::RouteArg};
use icp_route::tasks::{schedule_now, TaskType};
use icp_route::updates::generate_ticket::{
    GenerateTicketArgs, GenerateTicketError, GenerateTicketOk,
};
use icp_route::updates::{self};

#[init]
fn init(args: RouteArg) {
    match args {
        RouteArg::Init(args) => {
            // storage::record_event(&Event::Init(args.clone()));
            lifecycle::init::init(args);
            schedule_now(TaskType::ProcessHubMessages);
        }
        RouteArg::Upgrade() => {
            panic!("expected InitArgs got UpgradeArgs");
        }
    }
}

#[export_name = "canister_global_timer"]
fn timer() {
    icp_route::timer();
}

#[update]
async fn generate_ticket(
    args: GenerateTicketArgs,
) -> Result<GenerateTicketOk, GenerateTicketError> {
    updates::generate_ticket(args).await
}

fn main() {}

// Enable Candid export
ic_cdk::export_candid!();
