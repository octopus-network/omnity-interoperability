use candid::candid_method;
use ic_btc_interface::{
    Address, GetCurrentFeePercentilesRequest, GetUtxosRequest, GetUtxosResponse,
    MillisatoshiPerByte, Network, Utxo, UtxosFilterInRequest,
};
use ic_cdk::api::management_canister::bitcoin::{BitcoinNetwork, SendTransactionRequest};
use ic_cdk_macros::{init, update};
use serde_bytes::ByteBuf;
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};

// We use 12 as the default tip height to mint all
// the utxos with height 1 in the minter.
const DEFAULT_TIP_HEIGHT: u32 = 12;

fn main() {}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct State {
    pub fee_percentiles: Vec<u64>,
    // The network used in the bitcoin canister.
    pub network: Network,
    // Is the bitcoin canister available.
    pub is_available: bool,
    pub address_to_utxos: BTreeMap<Address, BTreeSet<Utxo>>,
    pub utxo_to_address: BTreeMap<Utxo, Address>,
    // Pending transactions.
    pub mempool: BTreeSet<ByteBuf>,
    pub tip_height: u32,
}

impl Default for State {
    fn default() -> Self {
        State {
            fee_percentiles: [0; 100].into(),
            network: Network::Mainnet,
            is_available: true,
            address_to_utxos: BTreeMap::new(),
            utxo_to_address: BTreeMap::new(),
            mempool: BTreeSet::new(),
            tip_height: DEFAULT_TIP_HEIGHT,
        }
    }
}

pub fn mutate_state<F, R>(f: F) -> R
where
    F: FnOnce(&mut State) -> R,
{
    STATE.with(|s| f(&mut s.borrow_mut()))
}

pub fn read_state<F, R>(f: F) -> R
where
    F: FnOnce(&State) -> R,
{
    STATE.with(|s| f(&s.borrow()))
}

thread_local! {
    static STATE: RefCell<State> = RefCell::default();
}

#[init]
fn init(network: Network) {
    STATE.with(|s| {
        let state = State {
            network,
            fee_percentiles: [0; 100].into(),
            is_available: true,
            utxo_to_address: BTreeMap::new(),
            address_to_utxos: BTreeMap::new(),
            mempool: BTreeSet::new(),
            tip_height: DEFAULT_TIP_HEIGHT,
        };
        *s.borrow_mut() = state;
    });
}

#[candid_method(update)]
#[update]
fn set_tip_height(tip_height: u32) {
    mutate_state(|s| s.tip_height = tip_height);
}

#[candid_method(update)]
#[update]
fn bitcoin_get_utxos(utxos_request: GetUtxosRequest) -> GetUtxosResponse {
    read_state(|s| {
        assert_eq!(utxos_request.network, s.network.into());

        let mut utxos = s
            .address_to_utxos
            .get(&utxos_request.address)
            .cloned()
            .unwrap_or_default()
            .iter()
            .cloned()
            .collect::<Vec<Utxo>>();

        if let Some(UtxosFilterInRequest::MinConfirmations(min_confirmations)) =
            utxos_request.filter
        {
            utxos.retain(|u| s.tip_height + 1 >= u.height + min_confirmations);
        }

        GetUtxosResponse {
            utxos,
            tip_block_hash: vec![],
            tip_height: s.tip_height,
            // TODO Handle pagination.
            next_page: None,
        }
    })
}

#[candid_method(update)]
#[update]
fn push_utxos_to_address(req: ic_bitcoin_canister_mock::PushUtxosToAddress) {
    mutate_state(|s| {
        for (address, utxos) in &req.utxos {
            for utxo in utxos {
                s.utxo_to_address.insert(utxo.clone(), address.clone());
                s.address_to_utxos
                    .entry(address.clone())
                    .or_default()
                    .insert(utxo.clone());
            }
        }
    });
}

#[candid_method(update)]
#[update]
fn remove_utxo(utxo: Utxo) {
    let address = read_state(|s| s.utxo_to_address.get(&utxo).cloned().unwrap());
    mutate_state(|s| {
        s.utxo_to_address.remove(&utxo);
        s.address_to_utxos
            .get_mut(&address)
            .expect("utxo not found at address")
            .remove(&utxo);
    });
}

#[candid_method(update)]
#[update]
fn bitcoin_get_current_fee_percentiles(
    _: GetCurrentFeePercentilesRequest,
) -> Vec<MillisatoshiPerByte> {
    read_state(|s| s.fee_percentiles.clone())
}

#[candid_method(update)]
#[update]
fn set_fee_percentiles(fee_percentiles: Vec<MillisatoshiPerByte>) {
    mutate_state(|s| s.fee_percentiles = fee_percentiles);
}

#[candid_method(update)]
#[update]
fn bitcoin_send_transaction(transaction: SendTransactionRequest) {
    mutate_state(|s| {
        let cdk_network = match transaction.network {
            BitcoinNetwork::Mainnet => Network::Mainnet,
            BitcoinNetwork::Testnet => Network::Testnet,
            BitcoinNetwork::Regtest => Network::Regtest,
        };
        assert_eq!(cdk_network, s.network);
        if s.is_available {
            s.mempool.insert(ByteBuf::from(transaction.transaction));
        }
    })
}

#[candid_method(update)]
#[update]
fn change_availability(is_available: bool) {
    mutate_state(|s| s.is_available = is_available);
}

#[candid_method(update)]
#[update]
fn get_mempool() -> Vec<ByteBuf> {
    read_state(|s| s.mempool.iter().cloned().collect::<Vec<ByteBuf>>())
}

#[candid_method(update)]
#[update]
fn reset_mempool() {
    mutate_state(|s| s.mempool = BTreeSet::new());
}
