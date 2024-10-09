mod audit;
mod bitcoin;
mod bitcoin_to_custom;
mod call_error;
mod custom_to_bitcoin;
mod generate_ticket;
mod guard;
mod hub;
mod hub_to_custom;
mod management;
mod ord;
pub mod service;
mod stable_memory;
pub(crate) mod state;
mod tasks;
mod types;

pub mod constants {
    use crate::ord::builder::fees::Fees;
    use bitcoin::Amount;

    pub const FETCH_HUB_TICKET_INTERVAL: u64 = 5;
    pub const FETCH_HUB_DIRECTIVE_INTERVAL: u64 = 60;
    pub const FETCH_HUB_TICKET_NAME: &str = "FETCH_HUB_TICKET";
    pub const FETCH_HUB_DIRECTIVE_NAME: &str = "FETCH_HUB_DIRECTIVE";
    pub const FINALIZE_LOCK_TICKET_NAME: &str = "FINALIZE_GENERATE_TICKET_NAME";
    pub const FINALIZE_LOCK_TICKET_INTERVAL: u64 = 120;
    pub const FINALIZE_UNLOCK_TICKET_NAME: &str = "FINALIZE_UNLOCK_TICKET_NAME";
    pub const FINALIZE_UNLOCK_TICKET_INTERVAL: u64 = 600;
    pub const SUBMIT_UNLOCK_TICKETS_NAME: &str = "SUBMIT_UNLOCK_TICKETS_NAME";
    pub const SUBMIT_UNLOCK_TICKETS_INTERVAL: u64 = 5;
    pub const BATCH_QUERY_LIMIT: u64 = 20;
    pub const PROD_KEY: &str = "key_1";
    pub const RPC_RETRY_TIMES: u8 = 5;
    pub const SEC_NANOS: u64 = 1_000_000_000;
    pub const MIN_NANOS: u64 = 60 * SEC_NANOS;
    pub const COMMIT_TX_VBYTES: u64 = 153;
    pub const REVEAL_TX_VBYTES: u64 = 161;
    pub const TRANSFER_TX_VBYTES: u64 = 208;
    pub const DEFAULT_FEE: Fees = Fees {
        commit_fee: Amount::from_sat(1000),
        reveal_fee: Amount::from_sat(1000),
        spend_fee: Amount::from_sat(1000),
    };
}

pub mod retry {
    use crate::constants::RPC_RETRY_TIMES;
    use ic_canister_log::log;
    use omnity_types::ic_log::{CRITICAL, ERROR, INFO};
    use std::fmt::Debug;
    use std::future::Future;

    pub async fn call_rpc_with_retry<
        P: Clone,
        T,
        E: Default + Clone + ToString + Debug,
        R: Future<Output = Result<T, E>>,
    >(
        params: P,
        call_rpc: fn(params: P) -> R,
    ) -> Result<T, E> {
        let mut rs = Err(E::default());
        for i in 0..RPC_RETRY_TIMES {
            log!(INFO, "[evm route]request rpc request times: {}", i + 1);
            let call_res = call_rpc(params.clone()).await;
            if call_res.is_ok() {
                rs = call_res;
                break;
            } else {
                let err = call_res.err().unwrap();
                log!(
                    ERROR,
                    "[evm route]call  rpc error: {}",
                    err.clone().to_string()
                );
                rs = Err(err);
            }
        }
        match rs {
            Ok(t) => Ok(t),
            Err(e) => {
                log!(CRITICAL, "rpc error after retry {:?}", &e);
                Err(e)
            }
        }
    }
}
