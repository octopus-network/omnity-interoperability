use crate::rpc::{self, RpcError};
use std::{cell::RefCell, rc::Rc, time::Duration};
use tm_verifier::{options::Options, types::*, ProdVerifier, Verdict, Verifier};

thread_local! {
    static CLIENT: RefCell<Option<Rc<LightClient<SysClock>>>> = RefCell::new(None);
}

pub trait Clock: Send + Sync {
    fn now(&self) -> Time;
}

#[derive(Clone)]
pub struct SysClock;

impl Clock for SysClock {
    fn now(&self) -> Time {
        Time::unix_epoch()
    }
}

pub struct LightClient<C: Clock> {
    pub chain_id: String,
    pub primary_rpcs: Vec<String>,
    pub init_trusted_state: LightBlock,
    pub options: Options,
    pub verifier: ProdVerifier,
    pub clock: C,
}

impl<C: Clock> LightClient<C> {
    fn setup(
        chain_id: String,
        primary_rpcs: Vec<String>,
        init_trusted_state: LightBlock,
        trust_threshold: TrustThreshold,
        trusting_period: Duration,
        clock_drift: Duration,
        clock: C,
    ) -> Self {
        let options = Options {
            trust_threshold,
            trusting_period,
            clock_drift,
        };
        LightClient {
            chain_id,
            primary_rpcs,
            init_trusted_state,
            options,
            clock,
            verifier: Default::default(),
        }
    }

    // TODO unit-tests friendly: use a seperate IO interface
    // TODO verify the mandatory blocks only
    async fn verify_to_target(&self, target: Height) -> Result<Height, RpcError> {
        // let height = self.trusted_state.height();
        // let recent_block = rpc::fetch_block(&self.primary_rpcs, None).await?;
        // if height >= recent_block.height() {
        //     return Ok(height);
        // }
        // // TODO verify the mandatory blocks using bisection
        // let unverified = recent_block;
        // let verdict = self.verifier.verify_update_header(
        //     unverified.as_untrusted_state(),
        //     self.trusted_state.as_trusted_state(),
        //     &self.options,
        //     self.clock.now(),
        // );
        // match verdict {
        //     Verdict::Success => {
        //         //self.trusted_state = unverified;
        //         // TODO
        //     }
        //     _ => {
        //         ic_cdk::println!(
        //             "Verification failed at {:?}: {:?}.",
        //             unverified.height(),
        //             verdict
        //         );
        //     }
        // }
        Ok(target)
    }

    async fn verify_to_highest(&self) -> Result<Height, RpcError> {
        let recent_block = rpc::fetch_block(&self.primary_rpcs, None).await?;
        self.verify_to_target(recent_block.height()).await
    }
}

// TODO rebind if something bad happened on initialization, e.g. couldn't fetch the `init_trust_state`
/// bind a COSMOS chain instance with config on initialization
pub(crate) fn bind(
    chain_id: String,
    primary_rpcs: Vec<String>,
    height: i64,
    trust_threshold: (u64, u64),
    trust_period: u64,
    clock_drift: u64,
) {
    ic_cdk::spawn(async move {
        let height = height.try_into().ok();
        let state = rpc::fetch_block(&primary_rpcs, height).await.unwrap();
        CLIENT.with_borrow_mut(|client| {
            *client = Some(Rc::new(LightClient::<SysClock>::setup(
                chain_id,
                primary_rpcs,
                state,
                TrustThreshold::new(trust_threshold.0, trust_threshold.1).unwrap(),
                Duration::from_secs(trust_period),
                Duration::from_secs(clock_drift),
                SysClock {},
            )));
        });
    });
}

pub(crate) fn acquire() -> Rc<LightClient<SysClock>> {
    CLIENT.with_borrow(|client| {
        client
            .as_ref()
            .expect("LightClient not initialized")
            .clone()
    })
}

/// start a timer to fetch blocks periodically
///
/// this is a little bit weird, the state could be only accessed through TLS while the `http_request` is an async function,
/// so we have to sperate the storage from lightclient itself
pub(crate) fn start(interval_secs: u64) {
    let interval = Duration::from_secs(interval_secs);
    ic_cdk::println!("Starting to fetch blocks with {interval:?}.");
    ic_cdk_timers::set_timer_interval(interval, || {
        ic_cdk::spawn(async move {
            let client = acquire();
            match client.verify_to_highest().await {
                Err(e) => ic_cdk::println!("{:?}", e),
                // TODO
                Ok(n) => {}
            }
        });
    });
}
