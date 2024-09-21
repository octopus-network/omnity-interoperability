use candid::CandidType;
use ic_cdk_macros::{export_candid, init, post_upgrade, pre_upgrade, update};
use serde::{Deserialize, Serialize};
use crate::state::{BitcoinNetwork, IndexerState};
use crate::state::replace_state;
use crate::state::read_state;

#[derive(CandidType, Serialize, Deserialize, Debug, Clone)]
pub struct InitArgs {
    pub api_key: String,
    pub network: BitcoinNetwork,

}

#[init]
fn init(init_args: InitArgs) {
    replace_state(IndexerState::init(init_args).expect("params error"));
}

#[pre_upgrade]
fn pre_upgrade() {
    read_state(|s| s.pre_upgrade());
}

#[post_upgrade]
fn post_upgrade() {
    IndexerState::post_upgrade();
}

#[update]
pub async fn get_indexed_transfer(args: QueryTransferArgs) -> Option<Brc20TransferEvent>{

    Some(Default::default())
}


#[derive(CandidType, Serialize, Deserialize, Default, Debug)]
pub struct Brc20TransferEvent {
    pub amout: u128,
    pub from: String,
    pub to: String,
    pub valid: bool,

}

#[derive(CandidType, Serialize, Deserialize, Default, Debug)]
pub struct QueryTransferArgs {
    pub tx_id: String,
    pub from_addr: String,
    pub ticker: String,
    pub to_addr: String,
    pub amt: u128,
}

ic_cdk::export_candid!();

