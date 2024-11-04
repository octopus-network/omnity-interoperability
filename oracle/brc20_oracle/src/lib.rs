mod bestinslot;
mod height;
mod okx;
mod service;
mod stable_memory;
mod state;
mod unisat;

pub mod constant_args {
    pub const IDEMPOTENCY_KEY: &str = "X-Idempotency";
    pub const FORWARD_SOLANA_RPC: &str = "X-Forward-Solana";
}
