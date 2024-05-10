use std::time::Duration;
use ic_cdk::{init, post_upgrade, pre_upgrade};
use ic_cdk_timers::set_timer_interval;
use cdk_route::state::{CdkRouteState, InitArgs, mutate_state};

#[init]
fn init(
    args: InitArgs
) {
    mutate_state(|s| *s = CdkRouteState::init(args));
    set_timer_interval(Duration::from_secs(10), periodic_task);
}


#[pre_upgrade]
fn pre_upgrade() {

}

#[post_upgrade]
fn post_upgrade() {}

fn periodic_task() {

    ic_cdk::spawn(async {



    });

}

ic_cdk::export_candid!();

pub fn main() {}

