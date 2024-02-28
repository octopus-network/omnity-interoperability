use futures::executor::block_on;
use ic_agent::export::Principal;
use runes_oracle::{customs::Customs, executor::Executor, indexer::Indexer};

const NODE_URL: &str = "http://localhost:8000";
const INDEXER_URL: &str = "https://bitcoin.indexer.testnet.octopus.network/api";
const CUSTOMS_CANISTER_ID: &str = "bd3st-beaaa-aaaaa-qaaba-cai";

fn main() {
    let customs_canister = Principal::from_text(String::from(CUSTOMS_CANISTER_ID))
        .expect("failed to parse customs canister id");
    let customs = Customs::new(NODE_URL.into(), customs_canister);
    let indexer = Indexer::new(INDEXER_URL.into());

    block_on(Executor::new(customs, indexer).start());
}
