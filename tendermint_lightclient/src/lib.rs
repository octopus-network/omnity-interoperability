mod lightclient;
mod rpc;

#[ic_cdk::init]
fn init(chain_id: String, height: u64, trust_period: u64, clock_drift: u64, primary_rpc: String) {
    lightclient::bind(chain_id, height, trust_period, clock_drift, primary_rpc);
}

/// query the verified block height
#[ic_cdk::query]
fn height() -> u64 {
    lightclient::try_get_state(|s| s.map(|s|s.height().value()).unwrap_or_default())
}
