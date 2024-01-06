use crate::rpc::{self, RpcError};
use std::{cell::RefCell, rc::Rc, time::Duration};
use tm_verifier::{options::Options, types::*, ProdVerifier, Verdict, Verifier};

thread_local! {
    static CLIENT: RefCell<Option<Rc<LightClient<SysClock>>>> = RefCell::new(None);
    static STATE: RefCell<Option<LightBlock>> = RefCell::new(None);
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
    pub primary_rpc: String,
    pub init_trusted_height: Height,
    pub options: Options,
    pub verifier: ProdVerifier,
    pub clock: C,
}

impl<C: Clock> LightClient<C> {
    fn setup(
        chain_id: String,
        primary_rpc: String,
        init_trusted_height: Height,
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
            primary_rpc,
            init_trusted_height,
            options,
            clock,
            verifier: Default::default(),
        }
    }

    async fn verify_to_target(
        &self,
        target: Height,
        state: &mut Option<LightBlock>,
        mandatory_blocks: Vec<LightBlock>,
    ) -> Result<(), RpcError> {
        let mut current_state = state.take().expect("LightClient not initialized");
        if current_state.height() >= target {
            state.replace(current_state);
            return Ok(());
        }
        for untrusted in mandatory_blocks {
            let verdict = self.verifier.verify_update_header(
                untrusted.as_untrusted_state(),
                current_state.as_trusted_state(),
                &self.options,
                self.clock.now(),
            );
            match verdict {
                Verdict::Success => {
                    current_state = untrusted;
                }
                _ => {
                    ic_cdk::println!(
                        "Verification failed at {:?}: {:?}.",
                        untrusted.height(),
                        verdict
                    );
                    break;
                }
            }
        }
        state.replace(current_state);
        Ok(())
    }

    // TODO if we would like to pull blocks from RPC from canister
    async fn collect_mandatory_blocks(&self, _target: Height, _current: Height) -> Vec<LightBlock> {
        vec![]
    }

    async fn verify_to_highest(
        &self,
        target: Height,
        state: &mut Option<LightBlock>,
    ) -> Result<(), RpcError> {
        let mandatory_blocks = self
            .collect_mandatory_blocks(
                target,
                state.as_ref().expect("already initialized").height(),
            )
            .await;
        self.verify_to_target(target, state, mandatory_blocks).await
    }
}

// TODO rebind if something bad happened on initialization
/// bind a COSMOS chain instance with config on initialization
pub(crate) fn bind(
    chain_id: String,
    height: u64,
    trust_period: u64,
    clock_drift: u64,
    primary_rpc: String,
) {
    // FIXME now we know that we can't use await in init function
    ic_cdk::spawn(async move {
        let height = height.try_into().ok();
        let block = rpc::fetch_block(&primary_rpc, height)
            .await
            .inspect_err(|e| ic_cdk::println!("{:?}", e))
            .expect("initialization failed");
        STATE.with_borrow_mut(|state| {
            *state = Some(block);
        });
        CLIENT.with_borrow_mut(|client| {
            *client = Some(Rc::new(LightClient::<SysClock>::setup(
                chain_id,
                primary_rpc,
                height.unwrap(),
                TrustThreshold::ONE_THIRD,
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

pub(crate) fn try_get_state<F, R>(f: F) -> R
where
    F: Fn(Option<&LightBlock>) -> R,
{
    STATE.with_borrow(|state| f(state.as_ref()))
}

/// start a timer to fetch blocks periodically
pub(crate) fn start(interval_secs: u64) {
    let interval = Duration::from_secs(interval_secs);
    ic_cdk::println!("Starting to fetch blocks with {interval:?}.");
    ic_cdk_timers::set_timer_interval(interval, || {
        ic_cdk::spawn(async move {
            let client = acquire();
            let mut state = STATE.take();
            if let Ok(block) = rpc::fetch_block(&client.primary_rpc, None).await {
                if let Err(e) = client.verify_to_highest(block.height(), &mut state).await {
                    ic_cdk::println!("{:?}", e);
                }
            }
            STATE.replace(state);
        });
    });
}
