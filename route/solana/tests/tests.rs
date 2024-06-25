use candid::{Decode, Encode, Principal};
use ic_base_types::{CanisterId, PrincipalId};
use ic_ic00_types::CanisterSettingsArgsBuilder;

use ic_state_machine_tests::{Cycles, StateMachine, WasmResult};
use ic_test_utilities_load_wasm::load_wasm;
use icp_ledger::{AccountIdentifier, InitArgs as LedgerInitArgs, LedgerCanisterPayload, Tokens};

use omnity_types::{
    Chain, ChainState, ChainType, Directive, Factor, FeeTokenFactor, TargetChainFactor, Ticket,
};
use solana_route::{
    handler::directive::TokenResp,
    lifecycle::init::{InitArgs, RouteArg},
    state::MintTokenStatus,
};
use std::{collections::HashMap, path::PathBuf, str::FromStr, time::Duration};

const SETTLEMENT_CHAIN: &str = "Bitcoin";
const EXECUTION_CHAIN: &str = "eICP";

const SYMBOL1: &str = "FIRST•RUNE•TOKEN";
const TOKEN_ID1: &str = "Bitcoin-RUNES-FIRST•RUNE•TOKEN";
const SYMBOL2: &str = "SECOND•RUNE•TOKEN";
const TOKEN_ID2: &str = "Bitcoin-RUNES-SECOND•RUNE•TOKEN";
const LEDGER_WASM: &[u8] = include_bytes!("../../../ledger-canister.wasm");

fn minting_account() -> PrincipalId {
    PrincipalId::new_user_test_id(1)
}

fn caller_account() -> PrincipalId {
    PrincipalId::new_user_test_id(2)
}

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

fn install_ledger(env: &StateMachine) {
    let mut initial_values = HashMap::<AccountIdentifier, Tokens>::new();
    initial_values.insert(
        AccountIdentifier::new(caller_account(), None),
        Tokens::from_e8s(1000_000_000),
    );

    let payload = LedgerCanisterPayload::Init(LedgerInitArgs {
        minting_account: AccountIdentifier::new(minting_account(), None),
        icrc1_minting_account: None,
        initial_values: initial_values,
        max_message_size_bytes: None,
        transaction_window: None,
        archive_options: None,
        send_whitelist: Default::default(),
        transfer_fee: None,
        token_symbol: Some("ICP".to_string()),
        token_name: Some("Internet Computer".to_string()),
        feature_flags: None,
        maximum_number_of_accounts: None,
        accounts_overflow_trim_quantity: None,
    });

    // MAINNET_LEDGER_CANISTER_ID canister_id_to_u64 = 2, so the ledger canister must deploy thirdly
    let ledger_id = env.create_canister_with_cycles(None, Cycles::new(100_000_000_000_0000), None);

    env.install_existing_canister(ledger_id, LEDGER_WASM.to_vec(), Encode!(&payload).unwrap())
        .expect("install ledger error !");
}

fn install_router(env: &StateMachine, hub_id: CanisterId) -> CanisterId {
    let setting = CanisterSettingsArgsBuilder::new()
        .with_controllers(vec![hub_id.into(), caller_account()])
        .build();

    let route_id =
        env.create_canister_with_cycles(None, Cycles::new(100_000_000_000_0000), Some(setting));
    env.install_existing_canister(
        route_id,
        route_wasm(),
        Encode!(&RouteArg::Init(InitArgs {
            chain_id: EXECUTION_CHAIN.into(),
            hub_principal: hub_id.into(),
            chain_state: ChainState::Active,
            admin: Principal::from_str("2").unwrap(),
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
    // pub fn new() -> Self {}

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

    pub fn get_token_list(&self) -> Vec<TokenResp> {
        todo!()
    }

    pub fn get_redeem_fee(&self) -> Option<u64> {
        Decode!(
            &assert_reply(
                self.env
                    .execute_ingress_as(
                        self.caller,
                        self.route_id,
                        "get_redeem_fee",
                        Encode!(&SETTLEMENT_CHAIN).unwrap(),
                    )
                    .expect("failed to get redeem fee")
            ),
            Option<u64>
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

    pub fn await_fee(&self, max_ticks: usize) {
        for _ in 0..max_ticks {
            let fee = self.get_redeem_fee();
            if fee.is_some() {
                return;
            }
        }
        panic!("the routes did not add the redeem fee in {}", max_ticks)
    }

    pub fn await_finalization(&self, ticket_id: String, max_ticks: usize) {
        for _ in 0..max_ticks {
            if let MintTokenStatus::Finalized { .. } = self.mint_token_status(ticket_id.clone()) {
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
        canister_id: route.route_id.to_string(),
        chain_type: ChainType::SettlementChain,
        chain_state: ChainState::Active,
        contract_address: None,
        counterparties: None,
        fee_token: None,
    })]);
    route.env.advance_time(Duration::from_secs(10));
    route.await_chain(SETTLEMENT_CHAIN.into(), 10);
}

fn add_token(route: &RouteSetup, symbol: String, token_id: String) {}

fn set_fee(route: &RouteSetup) {
    route.push_directives(vec![
        Directive::UpdateFee(Factor::UpdateTargetChainFactor(TargetChainFactor {
            target_chain_id: "Bitcoin".into(),
            target_chain_factor: 10_000,
        })),
        Directive::UpdateFee(Factor::UpdateFeeTokenFactor(FeeTokenFactor {
            fee_token: "ICP".into(),
            fee_token_factor: 1000,
        })),
    ]);

    route.env.advance_time(Duration::from_secs(20));
    route.await_fee(10);
}

#[test]
fn test_add_chain() {}

#[test]
fn test_add_token() {}

fn mint_token(
    ticket_id: String,
    route: &RouteSetup,
    token_id: String,
    receiver: String,
    amount: String,
) {
}

#[test]
fn test_mint_token() {}

#[test]
fn test_mint_token_to_account() {}

#[test]
fn test_generate_ticket() {}

#[test]
fn test_mint_multi_tokens() {}

#[test]
pub fn test_transfer_fee() {}
