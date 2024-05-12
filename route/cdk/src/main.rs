use std::time::Duration;

use ic_cdk::{init, post_upgrade, pre_upgrade};
use ic_cdk_timers::set_timer_interval;

use cdk_route::cdk_scan::scan_cdk_task;
use cdk_route::hub_to_route::fetch_hub_periodic_task;
use cdk_route::route_to_cdk::to_cdk_task;
use cdk_route::state::{CdkRouteState, InitArgs, mutate_state, read_state};
/*
#[init]
fn init(args: InitArgs) {
    mutate_state(|s| *s = CdkRouteState::init(args).expect("params error"));
/*  set_timer_interval(Duration::from_secs(10), fetch_hub_periodic_task);
    set_timer_interval(Duration::from_secs(20), to_cdk_task);
    set_timer_interval(Duration::from_secs(30), scan_cdk_task);*/
}
*/

#[init]
fn init() {
    mutate_state(|s| *s = CdkRouteState::default());
}
#[pre_upgrade]
fn pre_upgrade() {
   read_state(|s|s.pre_upgrade());
}

#[post_upgrade]
fn post_upgrade() {
    CdkRouteState::post_upgrade();/*
    set_timer_interval(Duration::from_secs(10), fetch_hub_periodic_task);
    set_timer_interval(Duration::from_secs(20), to_cdk_task);
    set_timer_interval(Duration::from_secs(30), scan_cdk_task);*/
}

fn main() {}

ic_cdk::export_candid!();