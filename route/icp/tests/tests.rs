use candid::{Decode, Encode};
use ic_base_types::{CanisterId, PrincipalId};
use ic_state_machine_tests::{StateMachine, WasmResult};
use ic_test_utilities_load_wasm::load_wasm;
use icp_route::{
    lifecycle::init::{InitArgs, RouteArg},
    updates::generate_ticket::{GenerateTicketArgs, GenerateTicketError},
};
use omnity_types::{Chain, ChainState, ChainType, Directive, Ticket, Token};
use std::{path::PathBuf, time::Duration};

const SYMBOL: &str = "FIRST•RUNE•TOKEN";
const TOKEN_ID: &str = "Bitcoin-RUNES-FIRST•RUNE•TOKEN";
const RUNE_ID: &str = "150:1";

fn route_wasm() -> Vec<u8> {
    load_wasm(
        std::env::var("CARGO_MANIFEST_DIR").unwrap(),
        "icp_route",
        &[],
    )
}

fn hub_mock_wasm() -> Vec<u8> {
    load_wasm(
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("mock")
            .join("hub"),
        "hub_mock",
        &[],
    )
}

fn install_hub(env: &StateMachine) -> CanisterId {
    env.install_canister(hub_mock_wasm(), vec![], None)
        .expect("install hub error !")
}

fn install_router(env: &StateMachine, hub_id: CanisterId) -> CanisterId {
    env.install_canister(
        route_wasm(),
        Encode!(&RouteArg::Init(InitArgs {
            chain_id: "eICP".into(),
            hub_principal: hub_id.into(),
        }))
        .unwrap(),
        None,
    )
    .expect("install route error !")
}

fn assert_reply(result: WasmResult) -> Vec<u8> {
    match result {
        WasmResult::Reply(bytes) => bytes,
        WasmResult::Reject(reject) => {
            panic!("Expected a successful reply, got a reject: {}", reject)
        }
    }
}

struct RouteSetup {
    pub env: StateMachine,
    pub caller: PrincipalId,
    pub hub_id: CanisterId,
    pub route_id: CanisterId,
}

impl RouteSetup {
    pub fn new() -> Self {
        let env = StateMachine::new();
        let hub_id = install_hub(&env);
        let route_id = install_router(&env, hub_id.clone());
        let caller = PrincipalId::new_user_test_id(1);

        Self {
            env,
            caller,
            hub_id,
            route_id,
        }
    }

    pub fn generate_ticket(&self, args: &GenerateTicketArgs) -> Result<(), GenerateTicketError> {
        Decode!(
            &assert_reply(
                self.env
                    .execute_ingress_as(
                        self.caller,
                        self.route_id,
                        "generate_ticket",
                        Encode!(args)
                        .unwrap()
                    )
                    .expect("failed to generate ticket")
            ),
            Result<(), GenerateTicketError>
        )
        .unwrap()
    }

    pub fn get_chain_list(&self) -> Vec<Chain> {
        Decode!(
            &assert_reply(
                self.env
                    .execute_ingress_as(
                        self.caller,
                        self.route_id,
                        "get_chain_list",
                        Encode!().unwrap(),
                    )
                    .expect("failed to get chain list")
            ),
            Vec<Chain>
        )
        .unwrap()
    }

    pub fn get_token_list(&self) -> Vec<Token> {
        Decode!(
            &assert_reply(
                self.env
                    .execute_ingress_as(
                        self.caller,
                        self.route_id,
                        "get_token_list",
                        Encode!().unwrap(),
                    )
                    .expect("failed to get token list")
            ),
            Vec<Token>
        )
        .unwrap()
    }

    pub fn await_chain(&self, chain_id: String, max_ticks: usize) {
        for _ in 0..max_ticks {
            let chains = self.get_chain_list();
            if chains.iter().any(|c| c.chain_id == chain_id) {
                return;
            }
        }
        panic!("the routes did not add the chain in {}", max_ticks)
    }

    pub fn await_token(&self, token_id: String, max_ticks: usize) {
        for _ in 0..max_ticks {
            let tokens = self.get_token_list();
            if tokens.iter().any(|t| t.token_id == token_id) {
                return;
            }
        }
        panic!("the routes did not add the token in {}", max_ticks)
    }

    pub fn push_ticket(&self, ticket: Ticket) {
        assert_reply(
            self.env
                .execute_ingress(self.hub_id, "push_ticket", Encode!(&ticket).unwrap())
                .expect("failed to push a ticket"),
        );
    }

    pub fn push_directives(&self, directives: Vec<Directive>) {
        assert_reply(
            self.env
                .execute_ingress(
                    self.hub_id,
                    "push_directives",
                    Encode!(&directives).unwrap(),
                )
                .expect("failed to push a directive"),
        );
    }
}

#[test]
fn test_add_chain() {
    let route = RouteSetup::new();
    let chain_id = "Bitcoin".to_string();
    route.push_directives(vec![Directive::AddChain(Chain {
        chain_id: chain_id.clone(),
        chain_type: ChainType::SettlementChain,
        chain_state: ChainState::Active,
        contract_address: None,
    })]);
    route.env.advance_time(Duration::from_secs(10));
    route.await_chain(chain_id, 10);
}

#[test]
fn test_add_token() {
    let route = RouteSetup::new();
    route.push_directives(vec![Directive::AddToken(Token {
        token_id: TOKEN_ID.into(),
        symbol: SYMBOL.into(),
        issue_chain: "Bitcoin".into(),
        decimals: 0,
        icon: None,
        metadata: None,
    })]);
    route.env.advance_time(Duration::from_secs(10));
    route.await_token(TOKEN_ID.into(), 10);
}
