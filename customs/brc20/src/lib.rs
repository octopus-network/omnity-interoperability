pub(crate) mod state;
mod hub;
mod stable_memory;
mod types;
mod custom_to_bitcoin;
mod bitcoin_to_custom;
mod generate_ticket;
mod call_error;
mod ord;
mod tasks;
mod guard;
mod hub_to_custom;
mod audit;
mod management;
mod bitcoin;
pub mod service;

pub mod constants {
    pub const FETCH_HUB_TICKET_INTERVAL: u64 = 5;
    pub const FETCH_HUB_DIRECTIVE_INTERVAL: u64 = 60;
    pub const FETCH_HUB_TICKET_NAME: &str = "FETCH_HUB_TICKET";
    pub const FETCH_HUB_DIRECTIVE_NAME: &str = "FETCH_HUB_DIRECTIVE";
    pub const BATCH_QUERY_LIMIT: u64 = 20;
    pub const PROD_KEY: &str = "key_1";
    pub const SEC_NANOS: u64 = 1_000_000_000;
    pub const MIN_NANOS: u64 = 60 * SEC_NANOS;
}