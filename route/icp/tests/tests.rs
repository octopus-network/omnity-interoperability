use candid::{Decode, Encode, Nat, Principal};
use ic_base_types::{CanisterId, PrincipalId};
use ic_state_machine_tests::{Cycles, StateMachine, WasmResult};
use ic_test_utilities_load_wasm::load_wasm;
use icp_route::{
    lifecycle::init::{InitArgs, RouteArg},
    state::MintTokenStatus,
    updates::generate_ticket::{GenerateTicketArgs, GenerateTicketError, GenerateTicketOk},
};
use icrc_ledger_types::{
    icrc1::account::Account,
    icrc2::approve::{ApproveArgs, ApproveError},
};
use omnity_types::{Chain, ChainState, ChainType, Directive, Ticket, TicketId, Token, TxAction};
use std::{path::PathBuf, str::FromStr, time::Duration};

const SETTLEMENT_CHAIN: &str = "Bitcoin";
const EXECUTION_CHAIN: &str = "eICP";
const SYMBOL: &str = "FIRST•RUNE•TOKEN";
const TOKEN_ID: &str = "Bitcoin-RUNES-FIRST•RUNE•TOKEN";

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
    let route_id = env.create_canister_with_cycles(None, Cycles::new(100_000_000_000_0000), None);
    env.install_existing_canister(
        route_id,
        route_wasm(),
        Encode!(&RouteArg::Init(InitArgs {
            chain_id: EXECUTION_CHAIN.into(),
            hub_principal: hub_id.into(),
        }))
        .unwrap(),
    )
    .expect("install route error !");
    route_id
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

    pub fn generate_ticket(
        &self,
        args: &GenerateTicketArgs,
    ) -> Result<GenerateTicketOk, GenerateTicketError> {
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
            Result<GenerateTicketOk, GenerateTicketError>
        )
        .unwrap()
    }

    pub fn mint_token_status(&self, ticket_id: String) -> MintTokenStatus {
        Decode!(
            &assert_reply(
                self.env
                    .execute_ingress_as(
                        self.caller,
                        self.route_id,
                        "mint_token_status",
                        Encode!(&ticket_id).unwrap(),
                    )
                    .expect("failed to get mint token status")
            ),
            MintTokenStatus
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

    pub fn icrc2_approve(&self, ledger_id: CanisterId, amount: Nat) {
        let _ = Decode!(
            &assert_reply(
                self.env
                    .execute_ingress_as(
                        self.caller,
                        ledger_id,
                        "icrc2_approve",
                        Encode!(&ApproveArgs {
                            from_subaccount: None,
                            spender: Principal::from(self.route_id).into(),
                            amount,
                            expected_allowance: None,
                            expires_at: None,
                            fee: None,
                            memo: None,
                            created_at_time: None,
                        })
                        .unwrap()
                    )
                    .expect("failed to execute icrc2 approve")
            ),
            Result<Nat, ApproveError>
        )
        .unwrap();
    }

    pub fn get_token_ledger(&self, token_id: String) -> Option<Principal> {
        Decode!(
            &assert_reply(
                self.env
                    .execute_ingress_as(
                        self.caller,
                        self.route_id,
                        "get_token_ledger",
                        Encode!(&token_id).unwrap(),
                    )
                    .expect("failed to get token ledger")
            ),
            Option<Principal>
        )
        .unwrap()
    }

    pub fn icrc1_balance_of(&self, ledger_id: CanisterId, owner: Principal) -> Nat {
        Decode!(
            &assert_reply(
                self.env
                    .execute_ingress_as(
                        self.caller,
                        ledger_id,
                        "icrc1_balance_of",
                        Encode!(&Account {
                            owner: owner,
                            subaccount: None,
                        })
                        .unwrap(),
                    )
                    .expect("failed to get token ledger")
            ),
            Nat
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

    pub fn await_await_finalization(&self, ticket_id: String, max_ticks: usize) {
        for _ in 0..max_ticks {
            let status = self.mint_token_status(ticket_id.clone());
            if status == MintTokenStatus::Finalized {
                return;
            }
        }
        panic!("the routes did not mint token in {}", max_ticks)
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

fn add_chain(route: &RouteSetup) {
    route.push_directives(vec![Directive::AddChain(Chain {
        chain_id: SETTLEMENT_CHAIN.into(),
        chain_type: ChainType::SettlementChain,
        chain_state: ChainState::Active,
        contract_address: None,
    })]);
    route.env.advance_time(Duration::from_secs(10));
    route.await_chain(SETTLEMENT_CHAIN.into(), 10);
}

fn add_token(route: &RouteSetup) {
    route.push_directives(vec![Directive::AddToken(Token {
        token_id: TOKEN_ID.into(),
        symbol: SYMBOL.into(),
        issue_chain: SETTLEMENT_CHAIN.into(),
        decimals: 0,
        icon: None,
        metadata: None,
    })]);
    route.env.advance_time(Duration::from_secs(10));
    route.await_token(TOKEN_ID.into(), 10);
}

#[test]
fn test_add_chain() {
    let route = RouteSetup::new();
    add_chain(&route);
}

#[test]
fn test_add_token() {
    let route = RouteSetup::new();
    add_token(&route);
    assert!(route.get_token_ledger(TOKEN_ID.into()).is_some());
}

fn mint_token(route: &RouteSetup, receiver: String, amount: String) {
    let ticket_id: TicketId = "test_ticket".into();
    route.push_ticket(Ticket {
        ticket_id: ticket_id.clone(),
        ticket_time: 1708911143,
        src_chain: SETTLEMENT_CHAIN.into(),
        dst_chain: EXECUTION_CHAIN.into(),
        action: TxAction::Transfer,
        token: TOKEN_ID.into(),
        amount: amount.into(),
        sender: None,
        receiver: receiver.to_string(),
        memo: None,
    });
    route.env.advance_time(Duration::from_secs(5));
    route.await_await_finalization(ticket_id.clone(), 10);
}

#[test]
fn test_mint_token() {
    let route = RouteSetup::new();
    add_chain(&route);
    add_token(&route);

    let amount = "1000000";
    let receiver =
        Principal::from_str("hsefg-sb4rm-qb5o2-vzqqa-ugrfq-tpdli-tazi3-3lmja-ur77u-tfncz-jqe")
            .unwrap();

    mint_token(&route, receiver.to_string(), amount.into());

    let ledger_id = route
        .get_token_ledger(TOKEN_ID.into())
        .expect("token ledger should exist");

    let balance = route.icrc1_balance_of(
        CanisterId::unchecked_from_principal(PrincipalId(ledger_id)),
        receiver,
    );
    assert_eq!(balance, Nat::from_str(amount).unwrap());
}

#[test]
fn test_generate_ticket() {
    let route = RouteSetup::new();
    add_chain(&route);
    add_token(&route);

    let amount = "1000000";
    mint_token(&route, route.caller.to_string(), amount.into());

    let ledger_id = route
        .get_token_ledger(TOKEN_ID.into())
        .expect("token ledger should exist");

    let ledger_id = CanisterId::unchecked_from_principal(PrincipalId(ledger_id));
    let redeem_amount = 400000_u128;
    route.icrc2_approve(ledger_id, Nat::from(redeem_amount));

    route
        .generate_ticket(&GenerateTicketArgs {
            target_chain_id: SETTLEMENT_CHAIN.into(),
            receiver: "bc1qyhm0eg6ffqw7zrytcc7hw5c85l25l9nnzzx9vr".into(),
            token_id: TOKEN_ID.into(),
            amount: redeem_amount,
            from_subaccount: None,
        })
        .expect("should generate ticket success");

    let balance = route.icrc1_balance_of(ledger_id, route.caller.into());
    // minus the transfer fee of 10,000
    assert_eq!(balance, Nat::from_str("590000").unwrap());
}
