mod lightclient;
mod rpc;

#[ic_cdk::init]
fn init(
    interval_secs: u64,
    chain_id: String,
    primary_rpcs: Vec<String>,
    height: i64,
    trust_threshold: (u64, u64),
    trust_period: u64,
    clock_drift: u64,
) {
    lightclient::bind(
        chain_id,
        primary_rpcs,
        height,
        trust_threshold,
        trust_period,
        clock_drift,
    );
    lightclient::start(interval_secs);
}

#[ic_cdk::post_upgrade]
fn post_upgrade(interval_secs: u64) {
    lightclient::start(interval_secs);
}

/// query the verified block height
#[ic_cdk::query]
fn height() -> u64 {
    0
}
