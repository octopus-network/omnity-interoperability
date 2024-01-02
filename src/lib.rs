use std::{cell::RefCell, time::Duration};
use thiserror::Error;
use tm_verifier::{options::Options, types::*, ProdVerifier, Verdict, Verifier};

thread_local! {
    static COSMOS_STATES: RefCell<Option<Port<SubnetClock>>> = RefCell::new(None);
}

#[derive(Clone)]
pub struct Port<C: Clock> {
    pub chain_id: String,
    pub primary_rpcs: Vec<String>,
    pub trusted_state: LightBlock,
    pub options: Options,
    pub verifier: ProdVerifier,
    pub clock: C,
}

pub trait Clock: Send + Sync {
    fn now(&self) -> Time;
}

// TODO
#[derive(Clone)]
pub struct SubnetClock {}

impl Clock for SubnetClock {
    fn now(&self) -> Time {
        Time::unix_epoch()
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("IO request {0} couldn't be executed.")]
    Io(String),
}

impl<C: Clock> Port<C> {
    pub fn setup(
        chain_id: String,
        primary_rpcs: Vec<String>,
        init_trusted_height: Height,
        trust_threshold: TrustThreshold,
        trusting_period: Duration,
        clock_drift: Duration,
        clock: C,
    ) -> Self {
        // TODO handle the unwrap
        let trusted_state = Self::fetch_blocks(init_trusted_height, 1)
            .unwrap()
            .pop()
            .expect("empty blocks already filtered;qed");
        let options = Options {
            trust_threshold,
            trusting_period,
            clock_drift,
        };
        Self {
            chain_id,
            primary_rpcs,
            trusted_state,
            options,
            verifier: Default::default(),
            clock,
        }
    }

    // TODO fetch the block
    pub fn fetch_blocks(height: Height, nums: usize) -> Result<Vec<LightBlock>, Error> {
        ic_cdk::println!("Fetching {} block(s) from {:?}.", nums, height);
        Ok(vec![])
    }

    // TODO fetch the remote height
    pub fn fetch_height() -> Result<Height, Error> {
        ic_cdk::println!("Fetching remote height.");
        Ok(Height::from(1u32))
    }

    // TODO verify the mandatory blocks only
    pub fn verify_forward(&mut self) -> Result<Height, Error> {
        let next_height = self.trusted_state.height().increment();
        let higest_height = Self::fetch_height()?;
        if next_height >= higest_height {
            return Ok(self.trusted_state.height());
        }
        let nums = usize::min(
            10, // TODO move to config
            (u64::from(higest_height) - u64::from(next_height)) as usize,
        );
        let blocks = Self::fetch_blocks(next_height, nums)?;
        // TODO verify the mandatory blocks only
        for unverified in blocks {
            let verdict = self.verifier.verify_update_header(
                unverified.as_untrusted_state(),
                self.trusted_state.as_trusted_state(),
                &self.options,
                self.clock.now(),
            );
            match verdict {
                Verdict::Success => {
                    self.trusted_state = unverified;
                }
                _ => {
                    ic_cdk::println!(
                        "Verification failed at {:?}: {:?}.",
                        unverified.height(),
                        verdict
                    );
                    return Ok(self.trusted_state.height());
                }
            }
        }
        Ok(self.trusted_state.height())
    }
}

// TODO rebind if something bad happened on initialization, e.g. couldn't fetch the `init_trust_state`
/// bind a COSMOS chain instance with config on initialization
fn bind(
    chain_id: String,
    primary_rpcs: Vec<String>,
    height: i64,
    trust_threshold: (u64, u64),
    trust_period: u64,
    clock_drift: u64,
) {
    COSMOS_STATES.with(|states| {
        let mut port = states.borrow_mut();
        *port = Some(Port::setup(
            chain_id,
            primary_rpcs,
            height.try_into().unwrap(),
            TrustThreshold::new(trust_threshold.0, trust_threshold.1).unwrap(),
            Duration::from_secs(trust_period),
            Duration::from_secs(clock_drift),
            SubnetClock {},
        ));
    });
}

/// start a timer to fetch blocks periodically
fn start(interval_secs: u64) {
    let interval = Duration::from_secs(interval_secs);
    ic_cdk::println!("Starting to fetch blocks with {interval:?}.");
    ic_cdk_timers::set_timer_interval(interval, || {
        COSMOS_STATES.with(|states| {
            let mut port = states.borrow_mut();
            let _ = port
                .as_mut()
                .expect("already initialized;qed")
                .verify_forward();
        });
    });
}

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
    bind(
        chain_id,
        primary_rpcs,
        height,
        trust_threshold,
        trust_period,
        clock_drift,
    );
    start(interval_secs);
}

#[ic_cdk::post_upgrade]
fn post_upgrade(interval_secs: u64) {
    start(interval_secs);
}

/// query the verified block height
#[ic_cdk::query]
fn height() -> u64 {
    0
}

mod rpc {
    use ic_cdk::api::management_canister::http_request::{
        http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod, HttpResponse,
        TransformArgs, TransformContext,
    };
    use serde::Serialize;

    #[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
    pub enum RpcMethod {
        /// Get ABCI info
        AbciInfo,
        /// Get ABCI query
        AbciQuery,
        /// Get block info
        Block,
        /// Get block info by hash
        BlockByHash,
        /// Get ABCI results for a particular block
        BlockResults,
        /// Search for blocks by their BeginBlock and EndBlock events
        BlockSearch,
        /// Get blockchain info
        Blockchain,
        /// Broadcast transaction asynchronously
        BroadcastTxAsync,
        /// Broadcast transaction synchronously
        BroadcastTxSync,
        /// Broadcast transaction commit
        BroadcastTxCommit,
        /// Get commit info for a block
        Commit,
        /// Get consensus parameters
        ConsensusParams,
        /// Get consensus state
        ConsensusState,
        /// Get genesis file
        Genesis,
        /// Get block header
        Header,
        /// Get block header by hash
        HeaderByHash,
        /// Get health info
        Health,
        /// Get network info
        NetInfo,
        /// Get node status
        Status,
        /// Find transaction by hash
        Tx,
        /// Search for transactions with their results
        TxSearch,
        /// Get validator info for a block
        Validators,
        /// Subscribe to events
        Subscribe,
        /// Unsubscribe from events
        Unsubscribe,
        /// Broadcast evidence
        BroadcastEvidence,
    }

    impl RpcMethod {
        /// Get a static string which represents this method name
        pub fn as_str(self) -> &'static str {
            match self {
                RpcMethod::AbciInfo => "abci_info",
                RpcMethod::AbciQuery => "abci_query",
                RpcMethod::Block => "block",
                RpcMethod::BlockByHash => "block_by_hash",
                RpcMethod::BlockResults => "block_results",
                RpcMethod::BlockSearch => "block_search",
                RpcMethod::Blockchain => "blockchain",
                RpcMethod::BroadcastEvidence => "broadcast_evidence",
                RpcMethod::BroadcastTxAsync => "broadcast_tx_async",
                RpcMethod::BroadcastTxSync => "broadcast_tx_sync",
                RpcMethod::BroadcastTxCommit => "broadcast_tx_commit",
                RpcMethod::Commit => "commit",
                RpcMethod::ConsensusParams => "consensus_params",
                RpcMethod::ConsensusState => "consensus_state",
                RpcMethod::Genesis => "genesis",
                RpcMethod::Header => "header",
                RpcMethod::HeaderByHash => "header_by_hash",
                RpcMethod::Health => "health",
                RpcMethod::NetInfo => "net_info",
                RpcMethod::Status => "status",
                RpcMethod::Subscribe => "subscribe",
                RpcMethod::Tx => "tx",
                RpcMethod::TxSearch => "tx_search",
                RpcMethod::Unsubscribe => "unsubscribe",
                RpcMethod::Validators => "validators",
            }
        }
    }

    #[derive(Serialize, Debug)]
    struct Payload {
        pub jsonrpc: &'static str,
        pub id: i64,
        pub method: &'static str,
        pub params: serde_json::Value,
    }

    async fn make_rpc<P>(
        url: impl ToString,
        method: RpcMethod,
        params: P,
    ) -> Result<(), super::Error>
    where
        P: Into<serde_json::Value>,
    {
        let payload = Payload {
            jsonrpc: "2.0",
            id: 1,
            method: method.as_str(),
            params: params.into(),
        };
        let body = serde_json::to_vec(&payload).unwrap();
        let args = CanisterHttpRequestArgument {
            url: url.to_string(),
            method: HttpMethod::POST,
            body: Some(body),
            max_response_bytes: None,
            transform: None,
            headers: vec![
                HttpHeader {
                    name: "Content-Type".to_string(),
                    value: "application/json".to_string(),
                },
                HttpHeader {
                    name: "User-Agent".to_string(),
                    value: format!("ic_tendermint_lightclient/{}", env!("CARGO_PKG_VERSION")),
                },
            ],
        };
        let response = http_request(args)
            .await
            .map_err(|(_, e)| super::Error::Io(e))?;
        Ok(())
    }

    pub async fn fetch_signed_header() {}

    pub async fn fetch_validator_set() {}
}
