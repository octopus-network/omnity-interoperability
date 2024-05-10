use cdk_route::cdk_scan::scan_cdk_task;
use cdk_route::hub_to_route::fetch_hub_periodic_task;
use cdk_route::route_to_cdk::to_cdk_task;
use cdk_route::state::{mutate_state, CdkRouteState, InitArgs};
use ic_cdk::{init, post_upgrade, pre_upgrade};
use ic_cdk_timers::set_timer_interval;
use std::time::Duration;

#[init]
fn init(args: InitArgs) {
    mutate_state(|s| *s = CdkRouteState::init(args).expect("params error"));
    set_timer_interval(Duration::from_secs(10), fetch_hub_periodic_task);
    set_timer_interval(Duration::from_secs(20), to_cdk_task);
    set_timer_interval(Duration::from_secs(30), scan_cdk_task);
}

#[pre_upgrade]
fn pre_upgrade() {}

#[post_upgrade]
fn post_upgrade() {}

ic_cdk::export_candid!();

pub fn main() {}
