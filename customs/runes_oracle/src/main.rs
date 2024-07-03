use clap::Parser;
use ic_agent::{export::Principal, identity::Secp256k1Identity};
use runes_oracle::{client::Client, executor::Executor, indexer::Indexer};
use std::fs;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(env, long, default_value = "http://localhost:23456")]
    indexer_url: String,

    #[arg(env, short, long, default_value = "identity.pem")]
    pem_path: String,

    #[arg(env, short, long, default_value = "http://localhost:4943")]
    ic_gateway: String,

    #[arg(env, short, long, default_value = "be2us-64aaa-aaaaa-qaabq-cai")]
    customs_canister_id: String,

    #[arg(env, short, long, default_value = "7wupf-wiaaa-aaaar-qaeya-cai")]
    hub_canister_id: String,
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let args = Args::parse();
    println!("args {:?}", args);

    let customs_canister = Principal::from_text(args.customs_canister_id)
        .expect("failed to parse customs canister id");

    let hub_canister =
        Principal::from_text(args.hub_canister_id).expect("failed to parse customs canister id");

    let pem = fs::File::open(args.pem_path).expect("failed to open pem file");
    let identity = Secp256k1Identity::from_pem(pem).expect("failed to parse pem");

    let client = Client::new(args.ic_gateway, identity).await;
    let indexer = Indexer::new(args.indexer_url);

    Executor::new(client, indexer, customs_canister, hub_canister)
        .start()
        .await;
}
