mod service;
mod unisat;
mod state;
mod stable_memory;
mod okx;
mod rpc;

pub mod  constant_args {
    pub const IDEMPOTENCY_KEY: &str = "X-Idempotency";
    pub const FORWARD_SOLANA_RPC: &str = "X-Forward-Solana";
}