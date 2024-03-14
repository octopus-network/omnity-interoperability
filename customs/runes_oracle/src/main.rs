use ic_agent::{export::Principal, identity::Secp256k1Identity};
use runes_oracle::{customs::Customs, executor::Executor, indexer::Indexer};
use std::fs;

const NODE_URL: &str = "http://localhost:4943";
const INDEXER_URL: &str = "http://localhost:23456";
const CUSTOMS_CANISTER_ID: &str = "be2us-64aaa-aaaaa-qaabq-cai";

#[tokio::main]
async fn main() {
    env_logger::init();

    let customs_canister = Principal::from_text(String::from(CUSTOMS_CANISTER_ID))
        .expect("failed to parse customs canister id");

    let pem = fs::File::open("identity.pem").expect("failed to open pem file");
    let identity = Secp256k1Identity::from_pem(pem).expect("failed to parse pem");

    let customs = Customs::new(NODE_URL.into(), customs_canister, identity).await;
    let indexer = Indexer::new(INDEXER_URL.into());

    Executor::new(customs, indexer).start().await;
}
