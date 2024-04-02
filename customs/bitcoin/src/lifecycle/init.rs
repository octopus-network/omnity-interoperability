use crate::lifecycle::upgrade::UpgradeArgs;
pub use crate::state::Mode;
use crate::state::{replace_state, CustomsState};
use candid::{CandidType, Deserialize, Principal};
use ic_btc_interface::Network;
use serde::Serialize;

pub const DEFAULT_MIN_CONFIRMATIONS: u32 = 6;

#[derive(CandidType, serde::Deserialize)]
pub enum CustomArg {
    Init(InitArgs),
    Upgrade(Option<UpgradeArgs>),
}

// TODO: Use `ic_btc_interface::Network` directly.
// The Bitcoin canister's network enum no longer has snake-case versions
// (refer to [PR171](https://github.com/dfinity/bitcoin-canister/pull/171)),
// instead it uses lower-case candid variants.
// A temporary fix for bitcoin customs is to create a new enum with capital letter variants.
#[derive(CandidType, Clone, Copy, Deserialize, Debug, Eq, PartialEq, Serialize, Hash)]
pub enum BtcNetwork {
    Mainnet,
    Testnet,
    Regtest,
}

impl From<BtcNetwork> for Network {
    fn from(network: BtcNetwork) -> Self {
        match network {
            BtcNetwork::Mainnet => Network::Mainnet,
            BtcNetwork::Testnet => Network::Testnet,
            BtcNetwork::Regtest => Network::Regtest,
        }
    }
}

impl From<Network> for BtcNetwork {
    fn from(network: Network) -> Self {
        match network {
            Network::Mainnet => BtcNetwork::Mainnet,
            Network::Testnet => BtcNetwork::Testnet,
            Network::Regtest => BtcNetwork::Regtest,
        }
    }
}

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct InitArgs {
    /// The bitcoin network that the customs will connect to
    pub btc_network: BtcNetwork,

    /// The name of the [EcdsaKeyId]. Use "dfx_test_key" for local replica and "test_key_1" for
    /// a testing key for testnet and mainnet
    pub ecdsa_key_name: String,

    /// Maximum time in nanoseconds that a transaction should spend in the queue
    /// before being sent.
    pub max_time_in_queue_nanos: u64,

    /// Specifies the minimum number of confirmations on the Bitcoin network
    /// required for the customs to accept a transaction.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_confirmations: Option<u32>,

    /// The mode controlling access to the customs.
    #[serde(default)]
    pub mode: Mode,

    pub hub_principal: Principal,

    pub runes_oracle_principal: Principal,

    pub chain_id: String,
}

pub fn init(args: InitArgs) {
    let state: CustomsState = CustomsState::from(args);
    state.validate_config();
    replace_state(state);
}
