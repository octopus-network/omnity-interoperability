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
    use bitcoin::Amount;
    use crate::ord::builder::fees::Fees;

    pub const FETCH_HUB_TICKET_INTERVAL: u64 = 5;
    pub const FETCH_HUB_DIRECTIVE_INTERVAL: u64 = 60;
    pub const FETCH_HUB_TICKET_NAME: &str = "FETCH_HUB_TICKET";
    pub const FETCH_HUB_DIRECTIVE_NAME: &str = "FETCH_HUB_DIRECTIVE";
    pub const FINALIZE_LOCK_TICKET_NAME: &str = "FINALIZE_GENERATE_TICKET_NAME";
    pub const FINALIZE_LOCK_TICKET_INTERVAL: u64 = 600;
    pub const FINALIZE_UNLOCK_TICKET_NAME: &str = "FINALIZE_UNLOCK_TICKET_NAME";
    pub const FINALIZE_UNLOCK_TICKET_INTERVAL: u64 = 600;
    pub const BATCH_QUERY_LIMIT: u64 = 20;
    pub const PROD_KEY: &str = "key_1";
    pub const SEC_NANOS: u64 = 1_000_000_000;
    pub const MIN_NANOS: u64 = 60 * SEC_NANOS;
    pub const COMMIT_TX_VBYTES: u64 = 153;
    pub const REVEAL_TX_VBYTES: u64 = 161;
    pub const TRANSFER_TX_VBYTES: u64 = 208;
    pub const DEFAULT_FEE: Fees =  Fees {
                                        commit_fee: Amount::from_sat(1000),
                                        reveal_fee: Amount::from_sat(1000),
                                        utxo_fee: Amount::from_sat(1000),
                                    };
}
