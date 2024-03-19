use bitcoin::util::psbt::serialize::Deserialize;
use bitcoin::{Address as BtcAddress, Network as BtcNetwork};
use bitcoin_customs::destination::Destination;
use bitcoin_customs::lifecycle::init::{CustomArg, InitArgs};
use bitcoin_customs::queries::{GenTicketStatusArgs, ReleaseTokenStatusArgs};
use bitcoin_customs::state::{GenTicketStatus, RuneId, RunesBalance};
use bitcoin_customs::state::{Mode, ReleaseTokenStatus};
use bitcoin_customs::updates::generate_ticket::{GenerateTicketArgs, GenerateTicketError};
use bitcoin_customs::updates::get_btc_address::GetBtcAddressArgs;
use bitcoin_customs::updates::update_btc_utxos::UpdateBtcUtxosErr;
use bitcoin_customs::updates::update_runes_balance::{
    UpdateRunesBalanceArgs, UpdateRunesBalanceError,
};
use bitcoin_customs::{Log, MIN_RELAY_FEE_PER_VBYTE, MIN_RESUBMISSION_DELAY};
use candid::{Decode, Encode};
use ic_base_types::{CanisterId, PrincipalId};
use ic_bitcoin_canister_mock::{OutPoint, PushUtxosToAddress, Utxo};
use ic_btc_interface::{Network, Txid};
use ic_canisters_http_types::{HttpRequest, HttpResponse};
use ic_state_machine_tests::{Cycles, StateMachine, StateMachineBuilder, WasmResult};
use ic_test_utilities_load_wasm::load_wasm;
use omnity_types::{
    Chain, ChainState, ChainType, Directive, Ticket, ToggleAction, ToggleState, Token, TxAction,
};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;

const MIN_CONFIRMATIONS: u32 = 12;
const MAX_TIME_IN_QUEUE: Duration = Duration::from_secs(10);
const COSMOS_HUB: &str = "cosmoshub";
const TOKEN_1: &str = "150:1";
const TOKEN_2: &str = "151:1";

fn customs_wasm() -> Vec<u8> {
    load_wasm(
        std::env::var("CARGO_MANIFEST_DIR").unwrap(),
        "bitcoin_customs",
        &[],
    )
}

fn bitcoin_mock_wasm() -> Vec<u8> {
    load_wasm(
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("mock")
            .join("bitcoin"),
        "ic-bitcoin-canister-mock",
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

fn install_customs(env: &StateMachine) -> CanisterId {
    let args = InitArgs {
        btc_network: Network::Regtest.into(),
        // The name of the [EcdsaKeyId]. Use "dfx_test_key" for local replica and "test_key_1" for
        // a testing key for testnet and mainnet
        ecdsa_key_name: "dfx_test_key".parse().unwrap(),
        max_time_in_queue_nanos: 0,
        min_confirmations: Some(1),
        mode: Mode::GeneralAvailability,
        hub_principal: CanisterId::from_u64(1).into(),
        runes_oracle_principal: CanisterId::from_u64(2).into(),
        chain_id: "Bitcoin".into(),
    };
    let customs_arg = CustomArg::Init(args);
    env.install_canister(customs_wasm(), Encode!(&customs_arg).unwrap(), None)
        .unwrap()
}

fn assert_reply(result: WasmResult) -> Vec<u8> {
    match result {
        WasmResult::Reply(bytes) => bytes,
        WasmResult::Reject(reject) => {
            panic!("Expected a successful reply, got a reject: {}", reject)
        }
    }
}

fn input_utxos(tx: &bitcoin::Transaction) -> Vec<bitcoin::OutPoint> {
    tx.input.iter().map(|txin| txin.previous_output).collect()
}

fn assert_replacement_transaction(old: &bitcoin::Transaction, new: &bitcoin::Transaction) {
    assert_ne!(old.txid(), new.txid());
    assert_eq!(input_utxos(old), input_utxos(new));

    let new_out_value = new.output.iter().map(|out| out.value).sum::<u64>();
    let prev_out_value = old.output.iter().map(|out| out.value).sum::<u64>();
    let relay_cost = new.vsize() as u64 * MIN_RELAY_FEE_PER_VBYTE / 1000;

    assert!(
        new_out_value + relay_cost <= prev_out_value,
        "the transaction fees should have increased by at least {relay_cost}. prev out value: {prev_out_value}, new out value: {new_out_value}"
    );
}

fn vec_to_txid(vec: Vec<u8>) -> Txid {
    let bytes: [u8; 32] = vec.try_into().expect("Vector length must be exactly 32");
    bytes.into()
}

fn random_txid() -> Txid {
    let txid: [u8; 32] = rand::random();
    txid.into()
}

#[test]
fn test_install_bitcoin_customs_canister() {
    let env = StateMachine::new();
    install_customs(&env);
}

#[test]
fn test_customs() {
    use bitcoin::Address;

    let env = StateMachine::new();
    let args = CustomArg::Init(InitArgs {
        btc_network: Network::Regtest.into(),
        ecdsa_key_name: "master_ecdsa_public_key".into(),
        max_time_in_queue_nanos: MAX_TIME_IN_QUEUE.as_nanos() as u64,
        min_confirmations: Some(6_u32),
        mode: Mode::GeneralAvailability,
        hub_principal: CanisterId::from_u64(1).into(),
        runes_oracle_principal: CanisterId::from_u64(2).into(),
        chain_id: "Bitcoin".into(),
    });
    let args = Encode!(&args).unwrap();
    let customs_id = env.install_canister(customs_wasm(), args, None).unwrap();

    let btc_address_1 = get_btc_address(
        &env,
        customs_id,
        &GetBtcAddressArgs {
            target_chain_id: String::from(COSMOS_HUB),
            receiver: String::from("cosmos1kwf682z5rxj38jsemljvdh67ykswns77j3euur"),
        },
    );
    let address_1 = Address::from_str(&btc_address_1).expect("invalid bitcoin address");
    let btc_address_2 = get_btc_address(
        &env,
        customs_id,
        &GetBtcAddressArgs {
            target_chain_id: String::from(COSMOS_HUB),
            receiver: String::from("cosmos12thfgc5swxymm549p7u0qtzvqdepq2m3j4srn6"),
        },
    );
    let address_2 = Address::from_str(&btc_address_2).expect("invalid bitcoin address");
    assert_ne!(address_1, address_2);
}

fn mainnet_bitcoin_canister_id() -> CanisterId {
    CanisterId::try_from(
        PrincipalId::from_str(ic_config::execution_environment::BITCOIN_MAINNET_CANISTER_ID)
            .unwrap(),
    )
    .unwrap()
}

fn install_bitcoin_mock_canister(env: &StateMachine) {
    let args = Network::Mainnet;
    let cid = mainnet_bitcoin_canister_id();
    env.create_canister_with_cycles(Some(cid.into()), Cycles::new(0), None);

    env.install_existing_canister(cid, bitcoin_mock_wasm(), Encode!(&args).unwrap())
        .unwrap();
}

struct CustomsSetup {
    pub env: StateMachine,
    pub caller: PrincipalId,
    pub runes_oracle: PrincipalId,
    pub bitcoin_id: CanisterId,
    pub customs_id: CanisterId,
    pub hub_id: CanisterId,
}

impl CustomsSetup {
    pub fn new() -> Self {
        let bitcoin_id = mainnet_bitcoin_canister_id();
        let env = StateMachineBuilder::new()
            .with_default_canister_range()
            .with_extra_canister_range(bitcoin_id..=bitcoin_id)
            .build();

        install_bitcoin_mock_canister(&env);
        let customs_id =
            env.create_canister_with_cycles(None, Cycles::new(100_000_000_000_000), None);
        let hub_id = env.create_canister(None);

        let caller = PrincipalId::new_user_test_id(1);
        let runes_oracle = PrincipalId::new_node_test_id(2);

        env.install_existing_canister(
            customs_id,
            customs_wasm(),
            Encode!(&CustomArg::Init(InitArgs {
                btc_network: Network::Mainnet.into(),
                ecdsa_key_name: "master_ecdsa_public_key".to_string(),
                max_time_in_queue_nanos: 0,
                min_confirmations: Some(MIN_CONFIRMATIONS),
                mode: Mode::GeneralAvailability,
                hub_principal: hub_id.into(),
                runes_oracle_principal: runes_oracle.into(),
                chain_id: "Bitcoin".into(),
            }))
            .unwrap(),
        )
        .expect("failed to install the customs");

        env.install_existing_canister(hub_id, hub_mock_wasm(), vec![])
            .expect("failed to install the hub canister");

        env.execute_ingress(
            bitcoin_id,
            "set_fee_percentiles",
            Encode!(&(1..=100).map(|i| i * 100).collect::<Vec<u64>>()).unwrap(),
        )
        .expect("failed to set fee percentiles");

        let customs = Self {
            env,
            caller,
            runes_oracle,
            bitcoin_id,
            customs_id,
            hub_id,
        };
        let directives = vec![
            Directive::AddChain(Chain {
                chain_id: COSMOS_HUB.into(),
                chain_type: ChainType::ExecutionChain,
                chain_state: ChainState::Active,
                contract_address: None,
            }),
            Directive::AddToken(Token {
                token_id: TOKEN_1.into(),
                symbol: "FIRST_RUNE".into(),
                issue_chain: COSMOS_HUB.into(),
                decimals: 0,
                icon: None,
            }),
            Directive::AddToken(Token {
                token_id: TOKEN_2.into(),
                symbol: "SECOND_RUNE".into(),
                issue_chain: COSMOS_HUB.into(),
                decimals: 0,
                icon: None,
            }),
        ];
        customs.push_directives(directives);
        customs.env.advance_time(Duration::from_secs(5));
        for _ in 0..10 {
            let chains = customs.get_chain_list();
            let tokens = customs.get_token_list();
            if !chains.is_empty() && tokens.len() == 2 {
                break;
            }
            customs.env.tick();
        }
        customs
    }

    pub fn set_tip_height(&self, tip_height: u32) {
        self.env
            .execute_ingress(
                self.bitcoin_id,
                "set_tip_height",
                Encode!(&tip_height).unwrap(),
            )
            .expect("failed to set fee tip height");
    }

    pub fn push_utxos(&self, req: Vec<(String, Utxo)>) {
        let mut utxos: BTreeMap<String, Vec<Utxo>> = BTreeMap::new();
        req.iter().for_each(|(address, utxo)| {
            utxos.entry(address.clone()).or_default().push(utxo.clone())
        });

        assert_reply(
            self.env
                .execute_ingress(
                    self.bitcoin_id,
                    "push_utxos_to_address",
                    Encode!(&PushUtxosToAddress { utxos }).unwrap(),
                )
                .expect("failed to push a UTXO"),
        );
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

    pub fn get_chain_list(&self) -> Vec<Chain> {
        Decode!(
            &assert_reply(
                self.env
                    .execute_ingress_as(
                        self.caller,
                        self.customs_id,
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
                        self.customs_id,
                        "get_token_list",
                        Encode!().unwrap(),
                    )
                    .expect("failed to get token list")
            ),
            Vec<Token>
        )
        .unwrap()
    }

    pub fn get_btc_address(&self, destination: impl Into<Destination>) -> String {
        let dest: Destination = destination.into();
        Decode!(
            &assert_reply(
                self.env
                    .execute_ingress_as(
                        self.caller,
                        self.customs_id,
                        "get_btc_address",
                        Encode!(&GetBtcAddressArgs {
                            target_chain_id: dest.target_chain_id,
                            receiver: dest.receiver,
                        })
                        .unwrap(),
                    )
                    .expect("failed to get btc address")
            ),
            String
        )
        .unwrap()
    }

    pub fn get_main_btc_address(&self, token: String) -> String {
        Decode!(
            &assert_reply(
                self.env
                    .execute_ingress_as(
                        self.caller,
                        self.customs_id,
                        "get_main_btc_address",
                        Encode!(&token).unwrap(),
                    )
                    .expect("failed to get main btc address")
            ),
            String
        )
        .unwrap()
    }

    pub fn get_logs(&self) -> Log {
        let request = HttpRequest {
            method: "".to_string(),
            url: "/logs".to_string(),
            headers: vec![],
            body: serde_bytes::ByteBuf::new(),
        };
        let response = Decode!(
            &assert_reply(
                self.env
                    .query(self.customs_id, "http_request", Encode!(&request).unwrap(),)
                    .expect("failed to get customs info")
            ),
            HttpResponse
        )
        .unwrap();
        serde_json::from_slice(&response.body).expect("failed to parse customs log")
    }

    pub fn generate_ticket(&self, args: &GenerateTicketArgs) -> Result<(), GenerateTicketError> {
        Decode!(
            &assert_reply(
                self.env
                    .execute_ingress_as(
                        self.caller,
                        self.customs_id,
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

    pub fn generate_ticket_status(&self, txid: Txid) -> GenTicketStatus {
        Decode!(
            &assert_reply(
                self.env
                    .query(
                        self.customs_id,
                        "generate_ticket_status",
                        Encode!(&GenTicketStatusArgs { txid }).unwrap()
                    )
                    .expect("failed to get generate ticket status")
            ),
            GenTicketStatus
        )
        .unwrap()
    }

    pub fn update_runes_balance(
        &self,
        args: &UpdateRunesBalanceArgs,
    ) -> Result<(), UpdateRunesBalanceError> {
        Decode!(
            &assert_reply(
                self.env
                    .execute_ingress_as(
                        self.runes_oracle,
                        self.customs_id,
                        "update_runes_balance",
                        Encode!(args)
                        .unwrap()
                    )
                    .expect("failed to update runes balance")
            ),
            Result<(), UpdateRunesBalanceError>
        )
        .unwrap()
    }

    pub fn update_btc_utxos(&self) -> Result<Vec<Utxo>, UpdateBtcUtxosErr> {
        Decode!(
            &assert_reply(
                self.env
                    .execute_ingress_as(
                        self.caller,
                        self.customs_id,
                        "update_btc_utxos",
                        Encode!().unwrap()
                    )
                    .expect("failed to update btc utxos")
            ),
            Result<Vec<Utxo>, UpdateBtcUtxosErr>
        )
        .unwrap()
    }

    pub fn release_token_status(&self, ticket_id: String) -> ReleaseTokenStatus {
        Decode!(
            &assert_reply(
                self.env
                    .query(
                        self.customs_id,
                        "release_token_status",
                        Encode!(&ReleaseTokenStatusArgs { ticket_id }).unwrap()
                    )
                    .expect("failed to get release token status")
            ),
            ReleaseTokenStatus
        )
        .unwrap()
    }

    pub fn tick_until<R>(
        &self,
        description: &str,
        max_ticks: u64,
        mut condition: impl FnMut(&CustomsSetup) -> Option<R>,
    ) -> R {
        if let Some(result) = condition(self) {
            return result;
        }
        for _ in 0..max_ticks {
            self.env.tick();
            if let Some(result) = condition(self) {
                return result;
            }
        }
        self.print_customs_logs();
        self.print_customs_events();
        panic!(
            "did not reach condition '{}' in {} ticks",
            description, max_ticks
        )
    }

    /// Check that the given condition holds for the specified number of state machine ticks.
    pub fn assert_for_n_ticks(
        &self,
        description: &str,
        num_ticks: u64,
        mut condition: impl FnMut(&CustomsSetup) -> bool,
    ) {
        for n in 0..num_ticks {
            self.env.tick();
            if !condition(self) {
                panic!(
                    "Condition '{}' does not hold after {} ticks",
                    description, n
                );
            }
        }
    }

    pub fn await_btc_transaction(&self, ticket_id: String, max_ticks: usize) -> Txid {
        let mut last_status = None;
        for _ in 0..max_ticks {
            dbg!(self.get_logs());
            let status = self.release_token_status(ticket_id.clone());
            match status {
                ReleaseTokenStatus::Submitted(txid) => {
                    return txid;
                }
                status => {
                    last_status = Some(status);
                    self.env.tick();
                }
            }
        }
        panic!(
            "the customs did not submit a transaction in {} ticks; last status {:?}",
            max_ticks, last_status
        )
    }

    pub fn print_customs_events(&self) {
        use bitcoin_customs::state::eventlog::{Event, GetEventsArg};
        let events = Decode!(
            &assert_reply(
                self.env
                    .query(
                        self.customs_id,
                        "get_events",
                        Encode!(&GetEventsArg {
                            start: 0,
                            length: 2000,
                        })
                        .unwrap()
                    )
                    .expect("failed to query customs events")
            ),
            Vec<Event>
        )
        .unwrap();
        println!("{:#?}", events);
    }

    pub fn print_customs_logs(&self) {
        let log = self.get_logs();
        for entry in log.entries {
            println!(
                "{} {}:{} {}",
                entry.timestamp, entry.file, entry.line, entry.message
            );
        }
    }

    pub fn await_pending(&self, ticket_id: String, max_ticks: usize) {
        let mut last_status = None;
        for _ in 0..max_ticks {
            let status = self.release_token_status(ticket_id.clone());
            match status {
                ReleaseTokenStatus::Pending => {
                    return;
                }
                status => {
                    last_status = Some(status);
                    self.env.tick();
                }
            }
        }
        panic!(
            "the customs did not pending the transaction in {} ticks; last status: {:?}",
            max_ticks, last_status
        )
    }

    pub fn await_finalization(&self, ticket_id: String, max_ticks: usize) -> Txid {
        let mut last_status = None;
        for _ in 0..max_ticks {
            let status = self.release_token_status(ticket_id.clone());
            match status {
                ReleaseTokenStatus::Confirmed(txid) => {
                    return txid;
                }
                status => {
                    last_status = Some(status);
                    self.env.tick();
                }
            }
        }
        panic!(
            "the customs did not finalize the transaction in {} ticks; last status: {:?}",
            max_ticks, last_status
        )
    }

    pub fn finalize_transaction(&self, tx: &bitcoin::Transaction, rune_id: String) {
        let runes_change_utxo = &tx.output[1];
        let btc_change_utxo = tx.output.last().unwrap();

        let runes_change_address =
            BtcAddress::from_script(&runes_change_utxo.script_pubkey, BtcNetwork::Bitcoin).unwrap();

        assert_eq!(
            runes_change_address.to_string(),
            self.get_main_btc_address(rune_id.to_string())
        );

        let btc_change_address =
            BtcAddress::from_script(&btc_change_utxo.script_pubkey, BtcNetwork::Bitcoin).unwrap();

        assert_eq!(
            btc_change_address.to_string(),
            self.get_main_btc_address("BTC".into())
        );

        self.env
            .advance_time(MIN_CONFIRMATIONS * Duration::from_secs(600) + Duration::from_secs(1));
        let txid_bytes: [u8; 32] = tx.txid().to_vec().try_into().unwrap();

        self.push_utxos(vec![
            (
                runes_change_address.to_string(),
                Utxo {
                    value: runes_change_utxo.value,
                    height: 0,
                    outpoint: OutPoint {
                        txid: txid_bytes.into(),
                        vout: 1,
                    },
                },
            ),
            (
                btc_change_address.to_string(),
                Utxo {
                    value: btc_change_utxo.value,
                    height: 0,
                    outpoint: OutPoint {
                        txid: txid_bytes.into(),
                        vout: (tx.output.len() - 1) as u32,
                    },
                },
            ),
        ]);
    }

    pub fn mempool(&self) -> BTreeMap<Txid, bitcoin::Transaction> {
        Decode!(
            &assert_reply(
                self.env
                    .execute_ingress(self.bitcoin_id, "get_mempool", Encode!().unwrap())
                    .expect("failed to call get_mempool on the bitcoin mock")
            ),
            Vec<Vec<u8>>
        )
        .unwrap()
        .iter()
        .map(|tx_bytes| {
            let tx = bitcoin::Transaction::deserialize(tx_bytes)
                .expect("failed to parse a bitcoin transaction");

            (vec_to_txid(tx.txid().to_vec()), tx)
        })
        .collect()
    }

    pub fn customs_self_check(&self) {
        Decode!(
            &assert_reply(
                self.env
                    .query(self.customs_id, "self_check", Encode!().unwrap())
                    .expect("failed to query self_check")
            ),
            Result<(), String>
        )
        .unwrap()
        .expect("customs self-check failed")
    }
}

#[test]
fn test_gen_ticket_no_new_utxos() {
    let customs = CustomsSetup::new();
    let result = customs.generate_ticket(&GenerateTicketArgs {
        target_chain_id: String::from(COSMOS_HUB),
        receiver: String::from("cosmos1fwaeqe84kaymymmqv0wyj75hzsdq4gfqm5xvvv"),
        rune_id: TOKEN_1.into(),
        amount: 1000,
        txid: random_txid().to_string(),
    });
    assert_eq!(result, Err(GenerateTicketError::NoNewUtxos));
}

#[test]
fn test_gen_ticket_with_insufficient_confirmations() {
    let customs = CustomsSetup::new();

    customs.set_tip_height(100);

    let txid = random_txid();
    let utxo = Utxo {
        height: 99,
        outpoint: OutPoint { txid, vout: 1 },
        value: 546,
    };

    let target_chain_id = COSMOS_HUB.to_string();
    let receiver = "cosmos1fwaeqe84kaymymmqv0wyj75hzsdq4gfqm5xvvv".to_string();
    let deposit_address = customs.get_btc_address(Destination {
        target_chain_id: target_chain_id.clone(),
        receiver: receiver.clone(),
        token: None,
    });

    customs.push_utxos(vec![(deposit_address, utxo)]);
    let result = customs.generate_ticket(&GenerateTicketArgs {
        target_chain_id,
        receiver,
        rune_id: TOKEN_1.into(),
        amount: 100_000_000,
        txid: txid.to_string(),
    });
    assert_eq!(result, Err(GenerateTicketError::NoNewUtxos));
}

#[test]
fn test_deactive_chain() {
    let customs = CustomsSetup::new();

    customs.set_tip_height(100);

    customs.push_directives(vec![Directive::ToggleChainState(ToggleState {
        chain_id: COSMOS_HUB.into(),
        action: ToggleAction::Deactivate,
    })]);

    customs.env.advance_time(Duration::from_secs(5));
    for _ in 0..10 {
        if customs.get_chain_list().last().map_or(false, |c| c.chain_state == ChainState::Deactive) {
            return;
        }
        customs.env.tick();
    }
    panic!("failed to toggle chain state to deactive!");
}

#[test]
fn test_gen_ticket_success() {
    let customs = CustomsSetup::new();

    customs.set_tip_height(100);

    let txid = random_txid();
    let utxo = Utxo {
        height: 80,
        outpoint: OutPoint { txid, vout: 1 },
        value: 546,
    };

    let target_chain_id = COSMOS_HUB.to_string();
    let receiver = "cosmos1fwaeqe84kaymymmqv0wyj75hzsdq4gfqm5xvvv".to_string();
    let deposit_address = customs.get_btc_address(Destination {
        target_chain_id: target_chain_id.clone(),
        receiver: receiver.clone(),
        token: None,
    });

    customs.push_utxos(vec![(deposit_address, utxo)]);
    let result = customs.generate_ticket(&GenerateTicketArgs {
        target_chain_id,
        receiver,
        rune_id: TOKEN_1.into(),
        amount: 100_000_000,
        txid: txid.to_string(),
    });
    assert_eq!(result, Ok(()));

    match customs.generate_ticket_status(txid) {
        GenTicketStatus::Pending(_) => {}
        _ => panic!("expect generate ticket pending status"),
    };
}

#[test]
fn test_duplicate_submit_gen_ticket() {
    let customs = CustomsSetup::new();

    customs.set_tip_height(100);

    let txid = random_txid();
    let utxo = Utxo {
        height: 80,
        outpoint: OutPoint { txid, vout: 1 },
        value: 546,
    };

    let target_chain_id = COSMOS_HUB.to_string();
    let receiver = "cosmos1fwaeqe84kaymymmqv0wyj75hzsdq4gfqm5xvvv".to_string();
    let deposit_address = customs.get_btc_address(Destination {
        target_chain_id: target_chain_id.clone(),
        receiver: receiver.clone(),
        token: None,
    });

    let args = GenerateTicketArgs {
        target_chain_id,
        receiver,
        rune_id: TOKEN_1.into(),
        amount: 100_000_000,
        txid: txid.to_string(),
    };

    customs.push_utxos(vec![(deposit_address, utxo)]);
    let _ = customs.generate_ticket(&args);
    let result = customs.generate_ticket(&args);
    assert_eq!(result, Err(GenerateTicketError::AlreadySubmitted));
}

#[test]
fn test_update_runes_balance_no_utxo() {
    let customs = CustomsSetup::new();
    let result = customs.update_runes_balance(&UpdateRunesBalanceArgs {
        txid: random_txid(),
        balances: vec![RunesBalance {
            rune_id: RuneId {
                height: 150,
                index: 1,
            },
            vout: 1,
            amount: 100_000_000,
        }],
    });
    assert_eq!(result, Err(UpdateRunesBalanceError::UtxoNotFound));
}

#[test]
fn test_update_runes_balance_invalid() {
    let customs = CustomsSetup::new();

    customs.set_tip_height(100);

    let txid = random_txid();
    let vout = 1;
    let utxo = Utxo {
        height: 80,
        outpoint: OutPoint { txid, vout },
        value: 546,
    };

    let target_chain_id = COSMOS_HUB.to_string();
    let receiver = "cosmos1fwaeqe84kaymymmqv0wyj75hzsdq4gfqm5xvvv".to_string();
    let deposit_address = customs.get_btc_address(Destination {
        target_chain_id: target_chain_id.clone(),
        receiver: receiver.clone(),
        token: None,
    });

    let args = GenerateTicketArgs {
        target_chain_id,
        receiver,
        rune_id: TOKEN_1.into(),
        amount: 100_000_000,
        txid: txid.to_string(),
    };

    customs.push_utxos(vec![(deposit_address, utxo)]);
    let result = customs.generate_ticket(&args);
    assert_eq!(result, Ok(()));

    let result = customs.update_runes_balance(&UpdateRunesBalanceArgs {
        txid,
        balances: vec![RunesBalance {
            rune_id: RuneId {
                height: 150,
                index: 1,
            },
            vout,
            // inconsistent with the value of generate ticket
            amount: 100_000,
        }],
    });
    assert_eq!(
        result,
        Err(UpdateRunesBalanceError::MismatchWithGenTicketReq)
    );

    let status = customs.generate_ticket_status(txid);
    assert_eq!(status, GenTicketStatus::Invalid);
}

#[test]
fn test_update_runes_balance_multi_utxos() {
    let customs = CustomsSetup::new();

    customs.set_tip_height(100);

    let txid = random_txid();
    let utxo1 = Utxo {
        height: 80,
        outpoint: OutPoint { txid, vout: 1 },
        value: 546,
    };
    let utxo2 = Utxo {
        height: 80,
        outpoint: OutPoint { txid, vout: 2 },
        value: 546,
    };

    let target_chain_id = COSMOS_HUB.to_string();
    let receiver = "cosmos1fwaeqe84kaymymmqv0wyj75hzsdq4gfqm5xvvv".to_string();
    let deposit_address = customs.get_btc_address(Destination {
        target_chain_id: target_chain_id.clone(),
        receiver: receiver.clone(),
        token: None,
    });

    let args = GenerateTicketArgs {
        target_chain_id,
        receiver,
        rune_id: TOKEN_1.into(),
        amount: 300_000_000,
        txid: txid.to_string(),
    };

    customs.push_utxos(vec![
        (deposit_address.clone(), utxo1),
        (deposit_address, utxo2),
    ]);
    let result = customs.generate_ticket(&args);
    assert_eq!(result, Ok(()));

    let result = customs.update_runes_balance(&UpdateRunesBalanceArgs {
        txid,
        balances: vec![
            RunesBalance {
                rune_id: RuneId {
                    height: 150,
                    index: 1,
                },
                vout: 1,
                amount: 100_000_000,
            },
            RunesBalance {
                rune_id: RuneId {
                    height: 150,
                    index: 1,
                },
                vout: 2,
                amount: 200_000_000,
            },
        ],
    });
    assert_eq!(result, Ok(()));

    let status = customs.generate_ticket_status(txid);
    assert_eq!(status, GenTicketStatus::Finalized);
}

#[test]
fn test_update_runes_balance_success() {
    let customs = CustomsSetup::new();

    let args = deposit_runes_to_main_address(&customs, TOKEN_1.into());

    let status = customs.generate_ticket_status(args.txid);
    assert_eq!(status, GenTicketStatus::Finalized);
}

#[test]
fn test_duplicate_update_runes_balance() {
    let customs = CustomsSetup::new();

    let args = deposit_runes_to_main_address(&customs, TOKEN_1.into());

    let status = customs.generate_ticket_status(args.txid);
    assert_eq!(status, GenTicketStatus::Finalized);

    let result = customs.update_runes_balance(&args);
    assert_eq!(result, Err(UpdateRunesBalanceError::AleardyProcessed));
}

fn deposit_runes_to_main_address(
    customs: &CustomsSetup,
    rune_id: String,
) -> UpdateRunesBalanceArgs {
    customs.set_tip_height(100);

    let txid = random_txid();
    let vout = 1;
    let utxo = Utxo {
        height: 80,
        outpoint: OutPoint { txid, vout },
        value: 546,
    };

    let target_chain_id = COSMOS_HUB.to_string();
    let receiver = "cosmos1fwaeqe84kaymymmqv0wyj75hzsdq4gfqm5xvvv".to_string();
    let deposit_address = customs.get_btc_address(Destination {
        target_chain_id: target_chain_id.clone(),
        receiver: receiver.clone(),
        token: None,
    });

    customs.push_utxos(vec![(deposit_address, utxo)]);
    let result = customs.generate_ticket(&GenerateTicketArgs {
        target_chain_id,
        receiver,
        rune_id: rune_id.clone(),
        amount: 100_000_000,
        txid: txid.to_string(),
    });
    assert_eq!(result, Ok(()));

    let args = UpdateRunesBalanceArgs {
        txid,
        balances: vec![RunesBalance {
            rune_id: RuneId::from_str(&rune_id).unwrap(),
            vout,
            amount: 100_000_000,
        }],
    };
    let result = customs.update_runes_balance(&args);
    assert_eq!(result, Ok(()));
    args
}

fn deposit_btc_to_main_address(customs: &CustomsSetup) {
    customs.set_tip_height(100);

    let main_address = customs.get_main_btc_address("BTC".into());

    let txid = random_txid();
    let vout = 1;
    let utxo = Utxo {
        height: 80,
        outpoint: OutPoint { txid, vout },
        value: 100_000_000,
    };

    customs.push_utxos(vec![(main_address, utxo.clone())]);

    match customs.update_btc_utxos() {
        Ok(utxos) => assert_eq!(utxos[0], utxo),
        Err(_) => panic!("fail to update btc utxos"),
    }
}

pub fn get_btc_address(
    env: &StateMachine,
    customs_id: CanisterId,
    arg: &GetBtcAddressArgs,
) -> String {
    Decode!(
        &env.execute_ingress_as(
            CanisterId::from_u64(100).into(),
            customs_id,
            "get_btc_address",
            Encode!(arg).unwrap()
        )
        .expect("failed to transfer funds")
        .bytes(),
        String
    )
    .expect("failed to decode String response")
}

#[test]
fn test_finalize_release_token_tx() {
    let customs = CustomsSetup::new();

    // deposit sufficient btc and runes
    deposit_runes_to_main_address(&customs, TOKEN_1.into());
    deposit_btc_to_main_address(&customs);

    let ticket_id: String = "ticket_id1".into();
    let ticket = Ticket {
        ticket_id: ticket_id.clone(),
        ticket_time: 1708911143,
        src_chain: COSMOS_HUB.into(),
        dst_chain: "BTC".into(),
        action: TxAction::Redeem,
        token: TOKEN_1.into(),
        amount: "1000000".into(),
        sender: Some("cosmos1fwaeqe84kaymymmqv0wyj75hzsdq4gfqm5xvvv".into()),
        receiver: "bc1qyhm0eg6ffqw7zrytcc7hw5c85l25l9nnzzx9vr".into(),
        memo: None,
    };
    customs.push_ticket(ticket);

    customs.env.advance_time(Duration::from_secs(5));
    customs.await_pending(ticket_id.clone(), 10);

    customs.env.advance_time(Duration::from_secs(5));
    let txid = customs.await_btc_transaction(ticket_id.clone(), 10);

    let mempool = customs.mempool();
    let tx = mempool
        .get(&txid)
        .expect("the mempool does not contain the release transaction");

    customs.finalize_transaction(tx, TOKEN_1.into());
    assert_eq!(customs.await_finalization(ticket_id, 10), txid);
    // customs.customs_self_check();
}

#[test]
fn test_finalize_batch_release_token_tx() {
    let customs = CustomsSetup::new();

    // deposit sufficient btc and runes
    deposit_runes_to_main_address(&customs, TOKEN_1.into());
    deposit_btc_to_main_address(&customs);

    let recivers = vec![
        "bc1qyhm0eg6ffqw7zrytcc7hw5c85l25l9nnzzx9vr",
        "bc1qyc692qvdgyy9culeuhhl7lv50uu5ss5f8preem",
        "bc1q74zh6anfe6ynnc86980y4haeqgqx3x424t4pay",
        "bc1qlnjgjs50tdjlca34aj3tm4fxsy7jd8vzkvy5g5",
        "bc1qsk3rh4glx4lzrwlk8v3p7wkj9felfkqc33z7yq",
    ];

    for i in 0..5 {
        let ticket_id: String = format!("ticket_id{}", i).into();
        let ticket = Ticket {
            ticket_id: ticket_id.clone(),
            ticket_time: 1708911143,
            src_chain: COSMOS_HUB.into(),
            dst_chain: "BTC".into(),
            action: TxAction::Redeem,
            token: TOKEN_1.into(),
            amount: "1000000".into(),
            sender: Some("cosmos1fwaeqe84kaymymmqv0wyj75hzsdq4gfqm5xvvv".into()),
            receiver: recivers[i].into(),
            memo: None,
        };
        customs.push_ticket(ticket);
    }

    customs.env.advance_time(Duration::from_secs(5));
    customs.await_pending("ticket_id1".into(), 10);

    customs.env.advance_time(Duration::from_secs(5));
    let txid = customs.await_btc_transaction("ticket_id1".into(), 10);

    let mempool = customs.mempool();
    let tx = mempool
        .get(&txid)
        .expect("the mempool does not contain the release transaction");

    customs.finalize_transaction(tx, TOKEN_1.into());
    assert_eq!(customs.await_finalization("ticket_id1".into(), 10), txid);

    for i in 1..5 {
        assert_eq!(
            customs.release_token_status(format!("ticket_id{}", i)),
            ReleaseTokenStatus::Confirmed(txid)
        );
    }
}

#[test]
fn test_exist_two_submitted_tx() {
    let customs = CustomsSetup::new();

    // Step 1: deposit sufficient btc and runes

    deposit_runes_to_main_address(&customs, TOKEN_1.into());
    deposit_runes_to_main_address(&customs, TOKEN_1.into());
    deposit_btc_to_main_address(&customs);
    deposit_btc_to_main_address(&customs);

    // Step 2: push the first ticket

    let first_ticket_id: String = "ticket_id1".into();
    let first_ticket = Ticket {
        ticket_id: first_ticket_id.clone(),
        ticket_time: 1708911143,
        src_chain: COSMOS_HUB.into(),
        dst_chain: "BTC".into(),
        action: TxAction::Redeem,
        token: TOKEN_1.into(),
        amount: "1000000".into(),
        sender: Some("cosmos1fwaeqe84kaymymmqv0wyj75hzsdq4gfqm5xvvv".into()),
        receiver: "bc1qyhm0eg6ffqw7zrytcc7hw5c85l25l9nnzzx9vr".into(),
        memo: None,
    };
    customs.push_ticket(first_ticket);
    customs.env.advance_time(Duration::from_secs(5));
    customs.await_pending(first_ticket_id.clone(), 10);

    // Step 3: wait for the first transaction to be submitted

    customs.env.advance_time(Duration::from_secs(5));
    let first_txid = customs.await_btc_transaction(first_ticket_id.clone(), 10);
    let mempool = customs.mempool();
    let first_tx: &bitcoin::Transaction = mempool
        .get(&first_txid)
        .expect("the mempool does not contain the release transaction");

    // Step 4: push the second ticket

    let second_ticket_id: String = "ticket_id2".into();
    let second_ticket = Ticket {
        ticket_id: second_ticket_id.clone(),
        ticket_time: 1708911146,
        src_chain: COSMOS_HUB.into(),
        dst_chain: "BTC".into(),
        action: TxAction::Redeem,
        token: TOKEN_1.into(),
        amount: "1000000".into(),
        sender: Some("cosmos1fwaeqe84kaymymmqv0wyj75hzsdq4gfqm5xvvv".into()),
        receiver: "bc1qlnjgjs50tdjlca34aj3tm4fxsy7jd8vzkvy5g5".into(),
        memo: None,
    };
    customs.push_ticket(second_ticket);
    customs.env.advance_time(Duration::from_secs(5));
    customs.await_pending(second_ticket_id.clone(), 10);

    // Step 5: wait for the second transaction to be submitted

    customs.env.advance_time(Duration::from_secs(5));
    let second_txid = customs.await_btc_transaction(second_ticket_id.clone(), 10);
    let mempool = customs.mempool();
    let second_tx: &bitcoin::Transaction = mempool
        .get(&second_txid)
        .expect("the mempool does not contain the release transaction");

    // Step 6: finalize these two transactions

    customs.finalize_transaction(first_tx, TOKEN_1.into());
    customs.finalize_transaction(second_tx, TOKEN_1.into());

    assert_eq!(customs.await_finalization(first_ticket_id, 10), first_txid);
    assert_eq!(
        customs.await_finalization(second_ticket_id, 10),
        second_txid
    );
}

#[test]
fn test_transaction_use_prev_change_output() {
    let customs = CustomsSetup::new();

    // Step 1: deposit sufficient btc and runes

    deposit_runes_to_main_address(&customs, TOKEN_1.into());
    deposit_btc_to_main_address(&customs);

    // Step 2: push the first ticket

    let first_ticket_id: String = "ticket_id1".into();
    let first_ticket = Ticket {
        ticket_id: first_ticket_id.clone(),
        ticket_time: 1708911143,
        src_chain: COSMOS_HUB.into(),
        dst_chain: "BTC".into(),
        action: TxAction::Redeem,
        token: TOKEN_1.into(),
        amount: "1000000".into(),
        sender: Some("cosmos1fwaeqe84kaymymmqv0wyj75hzsdq4gfqm5xvvv".into()),
        receiver: "bc1qyhm0eg6ffqw7zrytcc7hw5c85l25l9nnzzx9vr".into(),
        memo: None,
    };
    customs.push_ticket(first_ticket);
    customs.env.advance_time(Duration::from_secs(5));
    customs.await_pending(first_ticket_id.clone(), 10);

    // Step 3: wait for the first transaction to be submitted

    customs.env.advance_time(Duration::from_secs(5));
    let first_txid = customs.await_btc_transaction(first_ticket_id.clone(), 10);
    let mempool = customs.mempool();
    let first_tx: &bitcoin::Transaction = mempool
        .get(&first_txid)
        .expect("the mempool does not contain the release transaction");

    // Step 4: finalize the first transaction

    customs.finalize_transaction(first_tx, TOKEN_1.into());
    assert_eq!(customs.await_finalization(first_ticket_id, 10), first_txid);

    // Step 5: push the second ticket

    let second_ticket_id: String = "ticket_id2".into();
    let second_ticket = Ticket {
        ticket_id: second_ticket_id.clone(),
        ticket_time: 1708911146,
        src_chain: COSMOS_HUB.into(),
        dst_chain: "BTC".into(),
        action: TxAction::Redeem,
        token: TOKEN_1.into(),
        amount: "1000000".into(),
        sender: Some("cosmos1fwaeqe84kaymymmqv0wyj75hzsdq4gfqm5xvvv".into()),
        receiver: "bc1qlnjgjs50tdjlca34aj3tm4fxsy7jd8vzkvy5g5".into(),
        memo: None,
    };
    customs.push_ticket(second_ticket);
    customs.env.advance_time(Duration::from_secs(5));
    customs.await_pending(second_ticket_id.clone(), 10);

    // Step 6: wait for the second transaction to be submitted

    customs.env.advance_time(Duration::from_secs(5));
    let second_txid = customs.await_btc_transaction(second_ticket_id.clone(), 10);
    let mempool = customs.mempool();
    let second_tx: &bitcoin::Transaction = mempool
        .get(&second_txid)
        .expect("the mempool does not contain the release transaction");

    assert_eq!(second_tx.input[0].previous_output.txid, first_tx.txid());

    // Step 7: finalize the second transaction

    customs.finalize_transaction(second_tx, TOKEN_1.into());
    assert_eq!(
        customs.await_finalization(second_ticket_id, 10),
        second_txid
    );
}

#[test]
fn test_transaction_multi_runes_id() {
    let customs = CustomsSetup::new();

    // Step 1: deposit sufficient btc and runes

    deposit_runes_to_main_address(&customs, TOKEN_1.into());
    deposit_runes_to_main_address(&customs, TOKEN_2.into());
    deposit_btc_to_main_address(&customs);
    deposit_btc_to_main_address(&customs);

    // Step 2: push tickets

    let first_ticket_id: String = "ticket_id1".into();
    let first_ticket = Ticket {
        ticket_id: first_ticket_id.clone(),
        ticket_time: 1708911143,
        src_chain: COSMOS_HUB.into(),
        dst_chain: "BTC".into(),
        action: TxAction::Redeem,
        token: TOKEN_1.into(),
        amount: "1000000".into(),
        sender: Some("cosmos1fwaeqe84kaymymmqv0wyj75hzsdq4gfqm5xvvv".into()),
        receiver: "bc1qyhm0eg6ffqw7zrytcc7hw5c85l25l9nnzzx9vr".into(),
        memo: None,
    };

    let second_ticket_id: String = "ticket_id2".into();
    let second_ticket = Ticket {
        ticket_id: second_ticket_id.clone(),
        ticket_time: 1708911146,
        src_chain: COSMOS_HUB.into(),
        dst_chain: "BTC".into(),
        action: TxAction::Redeem,
        token: TOKEN_2.into(),
        amount: "1000000".into(),
        sender: Some("cosmos1fwaeqe84kaymymmqv0wyj75hzsdq4gfqm5xvvv".into()),
        receiver: "bc1qlnjgjs50tdjlca34aj3tm4fxsy7jd8vzkvy5g5".into(),
        memo: None,
    };
    customs.push_ticket(first_ticket);
    customs.push_ticket(second_ticket);
    customs.env.advance_time(Duration::from_secs(5));
    customs.await_pending(first_ticket_id.clone(), 10);
    customs.await_pending(second_ticket_id.clone(), 10);

    // Step 3: wait for the first transaction to be submitted

    customs.env.advance_time(Duration::from_secs(5));
    let first_txid = customs.await_btc_transaction(first_ticket_id.clone(), 10);
    let second_txid = customs.await_btc_transaction(second_ticket_id.clone(), 10);
    assert_ne!(first_txid, second_txid);

    let mempool = customs.mempool();
    let first_tx: &bitcoin::Transaction = mempool
        .get(&first_txid)
        .expect("the mempool does not contain the release transaction");
    let second_tx: &bitcoin::Transaction = mempool
        .get(&second_txid)
        .expect("the mempool does not contain the release transaction");

    // Step 4: finalize transactions

    customs.finalize_transaction(first_tx, TOKEN_1.into());
    assert_eq!(customs.await_finalization(first_ticket_id, 10), first_txid);
    customs.finalize_transaction(second_tx, TOKEN_2.into());
    assert_eq!(
        customs.await_finalization(second_ticket_id, 10),
        second_txid
    );
}

#[test]
fn test_transaction_resubmission_finalize_new() {
    let customs = CustomsSetup::new();

    // Step 1: deposit sufficient btc and runes

    deposit_runes_to_main_address(&customs, TOKEN_1.into());
    deposit_btc_to_main_address(&customs);

    // Step 2: push a ticket

    let ticket_id: String = "ticket_id1".into();
    let ticket = Ticket {
        ticket_id: ticket_id.clone(),
        ticket_time: 1708911143,
        src_chain: COSMOS_HUB.into(),
        dst_chain: "BTC".into(),
        action: TxAction::Redeem,
        token: TOKEN_1.into(),
        amount: "1000000".into(),
        sender: Some("cosmos1fwaeqe84kaymymmqv0wyj75hzsdq4gfqm5xvvv".into()),
        receiver: "bc1qyhm0eg6ffqw7zrytcc7hw5c85l25l9nnzzx9vr".into(),
        memo: None,
    };
    customs.push_ticket(ticket);
    customs.env.advance_time(Duration::from_secs(5));
    customs.await_pending(ticket_id.clone(), 10);

    // Step 3: wait for the transaction to be submitted

    customs.env.advance_time(Duration::from_secs(5));
    let txid = customs.await_btc_transaction(ticket_id.clone(), 10);
    let mempool = customs.mempool();
    let tx = mempool
        .get(&txid)
        .expect("the mempool does not contain the release transaction");

    // Step 4: wait for the transaction resubmission

    customs
        .env
        .advance_time(MIN_RESUBMISSION_DELAY - Duration::from_secs(1));

    customs.assert_for_n_ticks("no resubmission before the delay", 5, |ckbtc| {
        ckbtc.mempool().len() == 1
    });

    // We need to wait at least 5 seconds before the next resubmission because it's the internal
    // timer interval.
    customs.env.advance_time(Duration::from_secs(5));

    let mempool = customs.tick_until("mempool has a replacement transaction", 10, |ckbtc| {
        let mempool = ckbtc.mempool();
        (mempool.len() > 1).then_some(mempool)
    });

    let new_txid = customs.await_btc_transaction(ticket_id.clone(), 10);
    let new_tx = mempool
        .get(&new_txid)
        .expect("the pool does not contain the new transaction");

    assert_replacement_transaction(tx, new_tx);

    // Step 5: finalize the new transaction

    customs.finalize_transaction(new_tx, TOKEN_1.into());
    assert_eq!(customs.await_finalization(ticket_id, 10), new_txid);
    // customs.customs_self_check();
}

#[test]
fn test_transaction_resubmission_finalize_old() {
    let customs = CustomsSetup::new();

    // Step 1: deposit sufficient btc and runes

    deposit_runes_to_main_address(&customs, TOKEN_1.into());
    deposit_btc_to_main_address(&customs);

    // Step 2: push a ticket

    let ticket_id: String = "ticket_id1".into();
    let ticket = Ticket {
        ticket_id: ticket_id.clone(),
        ticket_time: 1708911143,
        src_chain: COSMOS_HUB.into(),
        dst_chain: "BTC".into(),
        action: TxAction::Redeem,
        token: TOKEN_1.into(),
        amount: "1000000".into(),
        sender: Some("cosmos1fwaeqe84kaymymmqv0wyj75hzsdq4gfqm5xvvv".into()),
        receiver: "bc1qyhm0eg6ffqw7zrytcc7hw5c85l25l9nnzzx9vr".into(),
        memo: None,
    };
    customs.push_ticket(ticket);
    customs.env.advance_time(Duration::from_secs(5));
    customs.await_pending(ticket_id.clone(), 10);

    // Step 3: wait for the transaction to be submitted

    customs.env.advance_time(Duration::from_secs(5));
    let txid = customs.await_btc_transaction(ticket_id.clone(), 10);
    let mempool = customs.mempool();
    let tx = mempool
        .get(&txid)
        .expect("the mempool does not contain the release transaction");

    // Step 4: wait for the transaction resubmission

    customs
        .env
        .advance_time(MIN_RESUBMISSION_DELAY + Duration::from_secs(1));

    let mempool = customs.tick_until("mempool has a replacement transaction", 10, |ckbtc| {
        let mempool = ckbtc.mempool();
        (mempool.len() > 1).then_some(mempool)
    });

    let new_txid = customs.await_btc_transaction(ticket_id.clone(), 10);

    let new_tx = mempool
        .get(&new_txid)
        .expect("the pool does not contain the new transaction");

    assert_replacement_transaction(tx, new_tx);

    // Step 5: finalize the old transaction

    customs.finalize_transaction(tx, TOKEN_1.into());
    assert_eq!(customs.await_finalization(ticket_id, 10), txid);
    // customs.minter_self_check();
}

#[test]
fn test_transaction_resubmission_finalize_middle() {
    let customs = CustomsSetup::new();

    // Step 1: deposit sufficient btc and runes

    deposit_runes_to_main_address(&customs, TOKEN_1.into());
    deposit_btc_to_main_address(&customs);

    // Step 2: push a ticket

    let ticket_id: String = "ticket_id1".into();
    let ticket = Ticket {
        ticket_id: ticket_id.clone(),
        ticket_time: 1708911143,
        src_chain: COSMOS_HUB.into(),
        dst_chain: "BTC".into(),
        action: TxAction::Redeem,
        token: TOKEN_1.into(),
        amount: "1000000".into(),
        sender: Some("cosmos1fwaeqe84kaymymmqv0wyj75hzsdq4gfqm5xvvv".into()),
        receiver: "bc1qyhm0eg6ffqw7zrytcc7hw5c85l25l9nnzzx9vr".into(),
        memo: None,
    };
    customs.push_ticket(ticket);
    customs.env.advance_time(Duration::from_secs(5));
    customs.await_pending(ticket_id.clone(), 10);

    // Step 3: wait for the transaction to be submitted

    customs.env.advance_time(Duration::from_secs(5));
    let original_txid = customs.await_btc_transaction(ticket_id.clone(), 10);
    let mempool = customs.mempool();
    let original_tx = mempool
        .get(&original_txid)
        .expect("the mempool does not contain the release transaction");

    // Step 4: wait for the first transaction resubmission

    customs
        .env
        .advance_time(MIN_RESUBMISSION_DELAY + Duration::from_secs(1));

    let mempool_2 = customs.tick_until("mempool contains a replacement transaction", 10, |ckbtc| {
        let mempool = ckbtc.mempool();
        (mempool.len() > 1).then_some(mempool)
    });

    let second_txid = customs.await_btc_transaction(ticket_id.clone(), 10);

    let second_tx = mempool_2
        .get(&second_txid)
        .expect("the pool does not contain the second transaction");

    assert_replacement_transaction(original_tx, second_tx);

    // Step 5: wait for the second transaction resubmission
    customs
        .env
        .advance_time(MIN_RESUBMISSION_DELAY + Duration::from_secs(1));

    let mempool_3 = customs.tick_until("mempool contains the third transaction", 10, |ckbtc| {
        let mempool = ckbtc.mempool();
        (mempool.len() > 2).then_some(mempool)
    });

    let third_txid = customs.await_btc_transaction(ticket_id.clone(), 10);
    assert_ne!(third_txid, second_txid);
    assert_ne!(third_txid, original_txid);

    let third_tx = mempool_3
        .get(&third_txid)
        .expect("the pool does not contain the third transaction");

    assert_replacement_transaction(second_tx, third_tx);

    // Step 6: finalize the middle transaction

    customs.finalize_transaction(second_tx, TOKEN_1.into());
    assert_eq!(customs.await_finalization(ticket_id, 10), second_txid);
    // customs.minter_self_check();
}

#[test]
fn test_get_logs() {
    let customs = CustomsSetup::new();

    // Test that the endpoint does not trap.
    let _log = customs.get_logs();
}

#[test]
fn test_filter_logs() {
    let customs = CustomsSetup::new();

    // Trigger an even to add some logs.

    deposit_runes_to_main_address(&customs, TOKEN_1.into());

    let system_time = customs.env.time();

    let nanos = system_time
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_nanos();

    let request = HttpRequest {
        method: "".to_string(),
        url: format!("/logs?time={}", nanos),
        headers: vec![],
        body: serde_bytes::ByteBuf::new(),
    };
    let response = Decode!(
        &assert_reply(
            customs
                .env
                .query(
                    customs.customs_id,
                    "http_request",
                    Encode!(&request).unwrap(),
                )
                .expect("failed to get minter info")
        ),
        HttpResponse
    )
    .unwrap();
    let logs: Log =
        serde_json::from_slice(&response.body).expect("failed to parse ckbtc minter log");

    let request = HttpRequest {
        method: "".to_string(),
        url: format!("/logs?time={}", nanos + 30 * 1_000_000_000),
        headers: vec![],
        body: serde_bytes::ByteBuf::new(),
    };
    let response = Decode!(
        &assert_reply(
            customs
                .env
                .query(
                    customs.customs_id,
                    "http_request",
                    Encode!(&request).unwrap(),
                )
                .expect("failed to get minter info")
        ),
        HttpResponse
    )
    .unwrap();
    let logs_filtered: Log =
        serde_json::from_slice(&response.body).expect("failed to parse ckbtc minter log");

    assert_ne!(logs.entries.len(), logs_filtered.entries.len());
}
