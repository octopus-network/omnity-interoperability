use std::collections::HashMap;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use candid::{Decode, Encode, Principal};
use cargo_metadata::MetadataCommand;
use escargot::CargoBuild;
use ic_base_types::{CanisterId, PrincipalId};
use ic_canisters_http_types::{HttpRequest, HttpResponse};
use ic_state_machine_tests::{StateMachine, WasmResult};
use omnity_hub::event::{Event, GetEventsArg};
use omnity_hub::lifecycle::init::{HubArg, InitArgs};
use omnity_hub::types::{ChainMeta, Proposal, Subscribers, TokenMeta, TokenResp};
use omnity_types::{Chain, ChainId, Directive, Seq, Ticket, TokenId, TokenOnChain, Topic};
use omnity_types::{ChainState, ChainType, Error, Factor};

const BINARY_NAME: &str = "omnity_hub";
const FEATURES: &[&str] = &[];
const DEFAULT_CARGO_TOML: &str = "./Cargo.toml";
// const DEFAULT_HUB_WASM_LOCATION: &str = "../.dfx/local/canisters/omnity_hub/omnity_hub.wasm.gz";
const DEFAULT_HUB_WASM_LOCATION: &str = "../target/wasm32-unknown-unknown/release/omnity_hub.wasm";

#[derive(Debug)]
pub struct OmnityHub {
    pub sm: StateMachine,
    pub hub_id: CanisterId,
    pub admin: PrincipalId,
}

impl OmnityHub {
    pub fn new() -> Self {
        let sm = StateMachine::new();
        let admin = PrincipalId::new_user_test_id(1);
        let hub_id = install_hub(&sm, admin.0);

        Self { sm, hub_id, admin }
    }

    pub fn validate_proposal(&self, proposals: &Vec<Proposal>) -> Result<Vec<String>, Error> {
        let ret = self
            .sm
            .query_as(
                self.admin,
                self.hub_id,
                "validate_proposal",
                Encode!(proposals).unwrap(),
            )
            .expect("failed to validate proposal");
        Decode!(&assert_reply(ret), Result<Vec<String>, Error>).unwrap()
    }

    pub fn sub_directives(
        &self,
        chain_id: &Option<ChainId>,
        topics: &Vec<Topic>,
    ) -> Result<(), Error> {
        let ret = self
            .sm
            .execute_ingress_as(
                self.admin,
                self.hub_id,
                "sub_directives",
                Encode!(chain_id, topics).unwrap(),
            )
            .expect("failed to sub_directives");
        Decode!(&assert_reply(ret), Result<(), Error>).unwrap()
    }

    pub fn unsub_directives(
        &self,
        chain_id: &Option<ChainId>,
        topics: &Vec<Topic>,
    ) -> Result<(), Error> {
        let ret = self
            .sm
            .execute_ingress_as(
                self.admin,
                self.hub_id,
                "unsub_directives",
                Encode!(chain_id, topics).unwrap(),
            )
            .expect("failed to unsub_directives");
        Decode!(&assert_reply(ret), Result<(), Error>).unwrap()
    }
    pub fn query_subscribers(
        &self,
        topic: &Option<Topic>,
    ) -> Result<Vec<(Topic, Subscribers)>, Error> {
        let ret = self
            .sm
            .query_as(
                self.admin,
                self.hub_id,
                "query_subscribers",
                Encode!(topic).unwrap(),
            )
            .expect("failed to query_subscribers");
        Decode!(&assert_reply(ret), Result<Vec<(Topic, Subscribers)>, Error>).unwrap()
    }

    pub fn execute_proposal(&self, proposals: &Vec<Proposal>) -> Result<(), Error> {
        let ret = self
            .sm
            .execute_ingress_as(
                self.admin,
                self.hub_id,
                "execute_proposal",
                Encode!(proposals).unwrap(),
            )
            .expect("failed to build directive");
        Decode!(&assert_reply(ret), Result<(), Error>).unwrap()
    }

    pub fn update_fee(&self, fees: &Vec<Factor>) -> Result<(), Error> {
        let ret = self
            .sm
            .execute_ingress_as(
                self.admin,
                self.hub_id,
                "update_fee",
                Encode!(fees).unwrap(),
            )
            .expect("failed to update fee");
        Decode!(&assert_reply(ret), Result<(), Error>).unwrap()
    }

    pub fn query_directives(
        &self,
        sender: &Option<PrincipalId>,
        chain_id: &Option<ChainId>,
        topic: &Option<Topic>,
        offset: &usize,
        limit: &usize,
    ) -> Result<Vec<(Seq, Directive)>, Error> {
        let sender = sender.unwrap_or(self.admin);
        let ret = self
            .sm
            .query_as(
                sender,
                self.hub_id,
                "query_directives",
                Encode!(chain_id, topic, offset, limit).unwrap(),
            )
            .expect("failed to query directives");
        Decode!(&assert_reply(ret), Result<Vec<(Seq, Directive)>, Error>).unwrap()
    }

    pub fn get_chains(
        &self,
        chain_type: &Option<ChainType>,
        chain_state: &Option<ChainState>,
        offset: &usize,
        limit: &usize,
    ) -> Result<Vec<Chain>, Error> {
        let ret = self
            .sm
            .query(
                self.hub_id,
                "get_chains",
                Encode!(chain_type, chain_state, offset, limit).unwrap(),
            )
            .expect("failed to get chains");
        Decode!(&assert_reply(ret), Result<Vec<Chain>, Error>).unwrap()
    }
    pub fn get_tokens(
        &self,
        chain_id: &Option<ChainId>,
        token_id: &Option<TokenId>,
        offset: &usize,
        limit: &usize,
    ) -> Result<Vec<TokenResp>, Error> {
        let ret = self
            .sm
            .query(
                self.hub_id,
                "get_tokens",
                Encode!(chain_id, token_id, offset, limit).unwrap(),
            )
            .expect("failed to get tokens");
        Decode!(&assert_reply(ret), Result<Vec<TokenResp>, Error>).unwrap()
    }
    pub fn get_fees(
        &self,
        chain_id: &Option<ChainId>,
        token_id: &Option<TokenId>,
        offset: &usize,
        limit: &usize,
    ) -> Result<Vec<(ChainId, TokenId, u128)>, Error> {
        let ret = self
            .sm
            .query(
                self.hub_id,
                "get_fees",
                Encode!(chain_id, token_id, offset, limit).unwrap(),
            )
            .expect("failed to get fees");
        Decode!(
            &assert_reply(ret),
            Result<Vec<(ChainId, TokenId, u128)>, Error>
        )
        .unwrap()
    }
    pub fn send_ticket(&self, sender: &Option<PrincipalId>, ticket: &Ticket) -> Result<(), Error> {
        let sender = sender.unwrap_or(self.admin);
        let ret = self
            .sm
            .execute_ingress_as(sender, self.hub_id, "send_ticket", Encode!(ticket).unwrap())
            .expect("failed to send ticket");
        Decode!(&assert_reply(ret), Result<(), Error>).unwrap()
    }
    pub fn query_tickets(
        &self,
        sender: &Option<PrincipalId>,
        chain_id: &Option<ChainId>,
        offset: &usize,
        limit: &usize,
    ) -> Result<Vec<(Seq, Ticket)>, Error> {
        let sender = sender.unwrap_or(self.admin);
        let ret = self
            .sm
            .query_as(
                sender,
                self.hub_id,
                "query_tickets",
                Encode!(chain_id, offset, limit).unwrap(),
            )
            .expect("failed to query tickets");
        Decode!(&assert_reply(ret), Result<Vec<(Seq, Ticket)>, Error>).unwrap()
    }

    pub fn get_chain_tokens(
        &self,
        chain_id: &Option<ChainId>,
        token_id: &Option<TokenId>,
        offset: &usize,
        limit: &usize,
    ) -> Result<Vec<TokenOnChain>, Error> {
        let ret = self
            .sm
            .query(
                self.hub_id,
                "get_chain_tokens",
                Encode!(chain_id, token_id, offset, limit).unwrap(),
            )
            .expect("failed to get chain tokens");
        Decode!(&assert_reply(ret), Result<Vec<TokenOnChain>, Error>).unwrap()
    }
    pub fn get_txs_with_chain(
        &self,
        src_chain: &Option<ChainId>,
        dst_chain: &Option<ChainId>,
        token_id: &Option<TokenId>,
        // time range: from .. end
        time_range: &Option<(u64, u64)>,
        offset: &usize,
        limit: &usize,
    ) -> Result<Vec<Ticket>, Error> {
        let ret = self
            .sm
            .query(
                self.hub_id,
                "get_txs_with_chain",
                Encode!(src_chain, dst_chain, token_id, time_range, offset, limit).unwrap(),
            )
            .expect("failed to get tx");
        Decode!(&assert_reply(ret), Result<Vec<Ticket>, Error>).unwrap()
    }

    pub fn get_logs(
        &self,
        max_skip_timestamp: &Option<u64>,
        offset: &usize,
        limit: &usize,
    ) -> Vec<String> {
        let url = if let Some(max_skip_timestamp) = max_skip_timestamp {
            format!(
                "/logs?time={}&offset={}&limit={}",
                max_skip_timestamp, offset, limit
            )
        } else {
            format!("/logs?offset={}&limit={}", offset, limit)
        };

        let request = HttpRequest {
            method: "".to_string(),
            url: url,
            headers: vec![],
            body: serde_bytes::ByteBuf::new(),
        };
        let response = Decode!(
            &assert_reply(
                self.sm
                    .query(self.hub_id, "http_request", Encode!(&request).unwrap(),)
                    .expect("failed to get logs")
            ),
            HttpResponse
        )
        .unwrap();
        serde_json::from_slice(&response.body).expect("failed to parse hub log")
    }

    pub fn get_events(&self, start: &u64, length: &u64) -> Vec<Event> {
        let artgs = GetEventsArg {
            start: *start,
            length: *length,
        };
        let ret = self
            .sm
            .query(self.hub_id, "get_events", Encode!(&artgs).unwrap())
            .expect("failed to get chain tokens");
        Decode!(&assert_reply(ret), Vec<Event>).unwrap()
    }

    pub fn upgrade(&self) {
        //Encode!(&HubArg::Init(InitArgs { admin })).unwrap(),
        let ret = self.sm.upgrade_canister(
            self.hub_id,
            hub_wasm(),
            Encode!(&HubArg::Upgrade(None)).unwrap(),
        );
        println!("upgrade result:{:?}", ret)
    }
}

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
fn install_hub(sm: &StateMachine, admin: Principal) -> CanisterId {
    sm.install_canister(
        hub_wasm(),
        Encode!(&HubArg::Init(InitArgs { admin })).unwrap(),
        None,
    )
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

pub fn default_topic() -> Vec<Topic> {
    vec![
        Topic::AddChain(None),
        Topic::AddToken(None),
        Topic::UpdateTargetChainFactor(None),
        Topic::UpdateFeeTokenFactor(None),
        Topic::ActivateChain,
        Topic::DeactivateChain,
    ]
}

pub fn canister_ids() -> Vec<PrincipalId> {
    vec![
        PrincipalId::new_user_test_id(0),
        PrincipalId::new_user_test_id(1),
        PrincipalId::new_user_test_id(2),
        PrincipalId::new_user_test_id(3),
        PrincipalId::new_user_test_id(4),
        PrincipalId::new_user_test_id(5),
    ]
}
pub fn chain_ids() -> Vec<String> {
    vec![
        "Bitcoin".to_string(),
        "Ethereum".to_string(),
        "ICP".to_string(),
        "EVM-Arbitrum".to_string(),
        "EVM-Optimistic".to_string(),
        "EVM-Starknet".to_string(),
    ]
}

pub fn chains() -> Vec<Proposal> {
    let chains = vec![
        Proposal::AddChain(ChainMeta {
            chain_id: "Bitcoin".to_string(),
            chain_type: ChainType::SettlementChain,
            chain_state: ChainState::Active,
            canister_id: PrincipalId::new_user_test_id(0).to_string(),
            contract_address: None,
            counterparties: None,
            fee_token: None,
        }),
        Proposal::AddChain(ChainMeta {
            chain_id: "Ethereum".to_string(),
            chain_type: ChainType::SettlementChain,
            chain_state: ChainState::Active,
            canister_id: PrincipalId::new_user_test_id(1).to_string(),
            contract_address: Some("Ethereum constract address".to_string()),
            counterparties: Some(vec!["Bitcoin".to_string()]),
            fee_token: None,
        }),
        Proposal::AddChain(ChainMeta {
            chain_id: "ICP".to_string(),
            chain_type: ChainType::SettlementChain,
            chain_state: ChainState::Active,
            canister_id: PrincipalId::new_user_test_id(2).to_string(),
            contract_address: Some("bkyz2-fmaaa-aaafa-qadaab-cai".to_string()),
            counterparties: Some(vec!["Bitcoin".to_string(), "Ethereum".to_string()]),
            fee_token: None,
        }),
        Proposal::AddChain(ChainMeta {
            chain_id: "EVM-Arbitrum".to_string(),
            chain_type: ChainType::ExecutionChain,
            chain_state: ChainState::Active,
            canister_id: PrincipalId::new_user_test_id(3).to_string(),
            contract_address: Some("Arbitrum constract address".to_string()),
            counterparties: Some(vec![
                "Bitcoin".to_string(),
                "Ethereum".to_string(),
                "ICP".to_string(),
            ]),
            fee_token: Some("Ethereum-ERC20-ARB".to_owned()),
        }),
        Proposal::AddChain(ChainMeta {
            chain_id: "EVM-Optimistic".to_string(),
            chain_type: ChainType::ExecutionChain,
            chain_state: ChainState::Active,
            canister_id: PrincipalId::new_user_test_id(4).to_string(),
            contract_address: Some("Optimistic constract address".to_string()),
            counterparties: Some(vec![
                "Bitcoin".to_string(),
                "Ethereum".to_string(),
                "ICP".to_string(),
                "EVM-Arbitrum".to_string(),
            ]),
            fee_token: Some("Ethereum-ERC20-OP".to_owned()),
        }),
        Proposal::AddChain(ChainMeta {
            chain_id: "EVM-Starknet".to_string(),
            chain_type: ChainType::ExecutionChain,
            chain_state: ChainState::Active,
            canister_id: PrincipalId::new_user_test_id(5).to_string(),
            contract_address: Some("Starknet constract address".to_string()),
            counterparties: Some(vec![
                "Bitcoin".to_string(),
                "Ethereum".to_string(),
                "ICP".to_string(),
                "EVM-Arbitrum".to_string(),
                "EVM-Optimistic".to_string(),
            ]),
            fee_token: Some("Ethereum-ERC20-StarkNet".to_owned()),
        }),
    ];

    chains
}

pub fn tokens() -> Vec<Proposal> {
    let tokens = vec![
        Proposal::AddToken(TokenMeta {
            token_id: "Bitcoin-RUNES-150:1".to_string(),
            name: "BTC".to_owned(),
            symbol: "BTC".to_owned(),
            issue_chain: "Bitcoin".to_string(),
            decimals: 18,
            icon: None,
            metadata: HashMap::from([("rune_id".to_string(), "150:1".to_string())]),
            dst_chains: vec![
                "Ethereum".to_string(),
                "ICP".to_string(),
                "EVM-Arbitrum".to_string(),
                "EVM-Optimistic".to_string(),
                "EVM-Starknet".to_string(),
            ],
        }),
        Proposal::AddToken(TokenMeta {
            token_id: "ETH".to_string(),
            name: "ETH".to_owned(),
            symbol: "ETH".to_owned(),
            issue_chain: "Ethereum".to_string(),
            decimals: 18,
            icon: None,
            metadata: HashMap::default(),
            dst_chains: vec![
                "Bitcoin".to_string(),
                "ICP".to_string(),
                "EVM-Arbitrum".to_string(),
                "EVM-Optimistic".to_string(),
                "EVM-Starknet".to_string(),
            ],
        }),
        Proposal::AddToken(TokenMeta {
            token_id: "ICP".to_string(),
            name: "ICP".to_owned(),
            symbol: "ICP".to_owned(),
            issue_chain: "ICP".to_string(),
            decimals: 18,
            icon: None,
            metadata: HashMap::default(),
            dst_chains: vec![
                "Bitcoin".to_string(),
                "Ethereum".to_string(),
                "EVM-Arbitrum".to_string(),
                "EVM-Optimistic".to_string(),
                "EVM-Starknet".to_string(),
            ],
        }),
        Proposal::AddToken(TokenMeta {
            token_id: "Ethereum-ERC20-ARB".to_string(),
            name: "ARB".to_owned(),
            symbol: "ARB".to_owned(),
            issue_chain: "Ethereum".to_string(),
            decimals: 18,
            icon: None,
            metadata: HashMap::default(),
            dst_chains: vec![
                "Bitcoin".to_string(),
                "Ethereum".to_string(),
                "ICP".to_string(),
                "EVM-Optimistic".to_string(),
                "EVM-Starknet".to_string(),
            ],
        }),
        Proposal::AddToken(TokenMeta {
            token_id: "Ethereum-ERC20-OP".to_string(),
            name: "OP".to_owned(),
            symbol: "OP".to_owned(),
            issue_chain: "Ethereum".to_string(),
            decimals: 18,
            icon: None,
            metadata: HashMap::default(),
            dst_chains: vec![
                "Bitcoin".to_string(),
                "Ethereum".to_string(),
                "ICP".to_string(),
                "EVM-Arbitrum".to_string(),
                "EVM-Starknet".to_string(),
            ],
        }),
        Proposal::AddToken(TokenMeta {
            token_id: "Ethereum-ERC20-StarkNet".to_string(),
            name: "StarkNet".to_owned(),
            symbol: "StarkNet".to_owned(),
            issue_chain: "Ethereum".to_string(),
            decimals: 18,
            icon: None,
            metadata: HashMap::default(),
            dst_chains: vec![
                "Bitcoin".to_string(),
                "Ethereum".to_string(),
                "ICP".to_string(),
                "EVM-Arbitrum".to_string(),
                "EVM-Optimistic".to_string(),
            ],
        }),
    ];
    tokens
}

pub fn get_timestamp() -> u64 {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    since_the_epoch.as_millis() as u64
}
