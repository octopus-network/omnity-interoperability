use std::fs;

use candid::{Decode, Encode};
use cargo_metadata::MetadataCommand;
use escargot::CargoBuild;
use ic_base_types::{CanisterId, PrincipalId};
use ic_state_machine_tests::{StateMachine, WasmResult};
use omnity_hub::types::Proposal;
use omnity_types::{Chain, ChainId, Directive, Seq, Ticket, Token, TokenId, TokenOnChain, Topic};
use omnity_types::{ChainState, ChainType, Error, Fee};

const BINARY_NAME: &str = "omnity_hub";
const FEATURES: &[&str] = &[];
const DEFAULT_CARGO_TOML: &str = "./Cargo.toml";
// const DEFAULT_HUB_WASM_LOCATION: &str = "../.dfx/local/canisters/omnity_hub/omnity_hub.wasm";
const DEFAULT_HUB_WASM_LOCATION: &str = "../target/wasm32-unknown-unknown/release/omnity_hub.wasm";

// build hub wasm
fn build_hub() -> Vec<u8> {
    let target_dir = MetadataCommand::new()
        .manifest_path(&DEFAULT_CARGO_TOML)
        .no_deps()
        .exec()
        .unwrap_or_else(|e| {
            panic!(
                "Failed to run cargo metadata on {}: {}",
                DEFAULT_CARGO_TOML, e
            )
        })
        .target_directory;
    println!("build target dir:{}", target_dir);

    let mut cargo_build = CargoBuild::new()
        .target("wasm32-unknown-unknown")
        .release()
        .bin(BINARY_NAME)
        .manifest_path(&DEFAULT_CARGO_TOML)
        .target_dir(target_dir);

    if !FEATURES.is_empty() {
        cargo_build = cargo_build.features(FEATURES.join(" "));
    }

    let binary = cargo_build
        .run()
        .expect("Cargo failed to compile a Wasm binary");
    println!("wasm file path:{:?}", binary.path());
    fs::read(binary.path()).unwrap_or_else(|e| {
        panic!(
            "failed to load Wasm from {}: {}",
            binary.path().display(),
            e
        )
    })
}

fn hub_wasm() -> Vec<u8> {
    std::fs::read(DEFAULT_HUB_WASM_LOCATION.to_string()).unwrap_or_else(|e| {
        println!(
            "not found wasm file from {}: {}, need to build hub",
            DEFAULT_HUB_WASM_LOCATION, e
        );
        // build hub
        build_hub()
    })
}
fn install_hub(sm: &StateMachine) -> CanisterId {
    sm.install_canister(hub_wasm(), vec![], None)
        .expect("install hub error !")
}

fn assert_reply(result: WasmResult) -> Vec<u8> {
    match result {
        WasmResult::Reply(bytes) => bytes,
        WasmResult::Reject(reject) => {
            panic!("Expected a successful reply, got a reject: {}", reject)
        }
    }
}

#[derive(Debug)]
pub struct OmnityHub {
    pub sm: StateMachine,
    pub hub_id: CanisterId,
    pub controller: PrincipalId,
}

impl OmnityHub {
    pub fn new() -> Self {
        let sm = StateMachine::new();
        let hub_id = install_hub(&sm);
        let controller = sm.canister_status(hub_id).unwrap().unwrap().controller();
        println!(
            "hub canister id: {}, controller:{}",
            hub_id.to_string(),
            controller.to_string()
        );
        Self {
            sm,
            hub_id,
            controller,
        }
    }

    pub fn validate_proposal(&self, proposals: &Vec<Proposal>) -> Result<Vec<String>, Error> {
        let ret = self
            .sm
            .query_as(
                self.controller,
                self.hub_id,
                "validate_proposal",
                Encode!(proposals).unwrap(),
            )
            .expect("failed to validate proposal");
        Decode!(&assert_reply(ret), Result<Vec<String>, Error>).unwrap()
    }

    pub fn build_directive(&self, proposals: &Vec<Proposal>) -> Result<(), Error> {
        let ret = self
            .sm
            .execute_ingress_as(
                self.controller,
                self.hub_id,
                "build_directive",
                Encode!(proposals).unwrap(),
            )
            .expect("failed to build directive");
        Decode!(&assert_reply(ret), Result<(), Error>).unwrap()
    }

    pub fn update_fee(&self, fees: &Vec<Fee>) -> Result<(), Error> {
        let ret = self
            .sm
            .execute_ingress_as(
                self.controller,
                self.hub_id,
                "update_fee",
                Encode!(fees).unwrap(),
            )
            .expect("failed to update fee");
        Decode!(&assert_reply(ret), Result<(), Error>).unwrap()
    }

    pub fn query_directives(
        &self,
        chain_id: &Option<ChainId>,
        topic: &Option<Topic>,
        from: &usize,
        offset: &usize,
    ) -> Result<Vec<(Seq, Directive)>, Error> {
        let ret = self
            .sm
            .query_as(
                self.controller,
                self.hub_id,
                "query_directives",
                Encode!(chain_id, topic, from, offset).unwrap(),
            )
            .expect("failed to query_directives");
        Decode!(&assert_reply(ret), Result<Vec<(Seq, Directive)>, Error>).unwrap()
    }

    pub fn get_chains(
        &self,
        chain_type: &Option<ChainType>,
        chain_state: &Option<ChainState>,
        from: &usize,
        offset: &usize,
    ) -> Result<Vec<Chain>, Error> {
        let ret = self
            .sm
            .query(
                self.hub_id,
                "get_chains",
                Encode!(chain_type, chain_state, from, offset).unwrap(),
            )
            .expect("failed to get chains");
        Decode!(&assert_reply(ret), Result<Vec<Chain>, Error>).unwrap()
    }
    pub fn get_tokens(
        &self,
        chain_id: &Option<ChainId>,
        token_id: &Option<TokenId>,
        from: &usize,
        offset: &usize,
    ) -> Result<Vec<Token>, Error> {
        let ret = self
            .sm
            .query(
                self.hub_id,
                "get_tokens",
                Encode!(chain_id, token_id, from, offset).unwrap(),
            )
            .expect("failed to get tokens");
        Decode!(&assert_reply(ret), Result<Vec<Token>, Error>).unwrap()
    }
    pub fn get_fees(
        &self,
        chain_id: &Option<ChainId>,
        token_id: &Option<TokenId>,
        from: &usize,
        offset: &usize,
    ) -> Result<Vec<Fee>, Error> {
        let ret = self
            .sm
            .query(
                self.hub_id,
                "get_fees",
                Encode!(chain_id, token_id, from, offset).unwrap(),
            )
            .expect("failed to get fees");
        Decode!(&assert_reply(ret), Result<Vec<Fee>, Error>).unwrap()
    }
    pub fn send_ticket(&self, ticket: &Ticket) -> Result<(), Error> {
        let ret = self
            .sm
            .execute_ingress_as(
                self.controller,
                self.hub_id,
                "send_ticket",
                Encode!(ticket).unwrap(),
            )
            .expect("failed to send ticket");
        Decode!(&assert_reply(ret), Result<(), Error>).unwrap()
    }
    pub fn query_tickets(
        &self,
        chain_id: &Option<ChainId>,
        from: &usize,
        offset: &usize,
    ) -> Result<Vec<(Seq, Ticket)>, Error> {
        let ret = self
            .sm
            .query(
                self.hub_id,
                "query_tickets",
                Encode!(chain_id, from, offset).unwrap(),
            )
            .expect("failed to query tickets");
        Decode!(&assert_reply(ret), Result<Vec<(Seq, Ticket)>, Error>).unwrap()
    }

    pub fn get_chain_tokens(
        &self,
        chain_id: &Option<ChainId>,
        token_id: &Option<TokenId>,
        from: &usize,
        offset: &usize,
    ) -> Result<Vec<TokenOnChain>, Error> {
        let ret = self
            .sm
            .query(
                self.hub_id,
                "get_chain_tokens",
                Encode!(chain_id, token_id, from, offset).unwrap(),
            )
            .expect("failed to get chain tokens");
        Decode!(&assert_reply(ret), Result<Vec<TokenOnChain>, Error>).unwrap()
    }
    pub fn get_txs(
        &self,
        src_chain: &Option<ChainId>,
        dst_chain: &Option<ChainId>,
        token_id: &Option<TokenId>,
        // time range: from .. end
        time_range: &Option<(u64, u64)>,
        from: &usize,
        offset: &usize,
    ) -> Result<Vec<Ticket>, Error> {
        let ret = self
            .sm
            .query(
                self.hub_id,
                "get_txs",
                Encode!(src_chain, dst_chain, token_id, time_range, from, offset).unwrap(),
            )
            .expect("failed to get tx");
        Decode!(&assert_reply(ret), Result<Vec<Ticket>, Error>).unwrap()
    }
}
