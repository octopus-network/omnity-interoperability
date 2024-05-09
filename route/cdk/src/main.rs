use ic_cdk::{init, post_upgrade, pre_upgrade};
use cdk_route::state::{CdkRouteState, InitArgs, mutate_state};

#[init]
fn init(
    args: InitArgs
) {
    mutate_state(|s| *s = CdkRouteState::init(args));
}


#[pre_upgrade]
fn pre_upgrade() {

}

#[post_upgrade]
fn post_upgrade() {}

ic_cdk::export_candid!();

pub fn main() {}

