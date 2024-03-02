use ic_agent::export::Principal;
use runes_oracle::{customs::Customs, executor::Executor, indexer::Indexer};

const NODE_URL: &str = "http://localhost:4943";
const INDEXER_URL: &str = "http://localhost:23456";
const CUSTOMS_CANISTER_ID: &str = "be2us-64aaa-aaaaa-qaabq-cai";

#[tokio::main]
async fn main() {
    let customs_canister = Principal::from_text(String::from(CUSTOMS_CANISTER_ID))
        .expect("failed to parse customs canister id");
    let customs = Customs::new(NODE_URL.into(), customs_canister).await;
    let indexer = Indexer::new(INDEXER_URL.into());

    Executor::new(customs, indexer).start().await;
}
