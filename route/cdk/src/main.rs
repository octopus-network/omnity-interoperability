use ic_cdk::{init, pre_upgrade, post_upgrade};

#[init]
fn init() {

}


#[pre_upgrade]
fn pre_upgrade() {

}

#[post_upgrade]
fn post_upgrade() {}

ic_cdk::export_candid!();

pub fn main() {}