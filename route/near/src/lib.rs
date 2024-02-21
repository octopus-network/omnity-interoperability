mod lightclient;
mod registry;
mod rpc;

use lightclient::*;

#[ic_cdk::init]
fn init() {
    // Dummy
}

/// query the verified block height
#[ic_cdk::query]
fn height() -> u64 {
    DummyLightClient.latest_height()
}
