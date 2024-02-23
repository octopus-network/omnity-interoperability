use assert_matches::assert_matches;
use bitcoin::util::psbt::serialize::Deserialize;
use bitcoin::{Address as BtcAddress, Network as BtcNetwork};
use bitcoin_customs::destination::Destination;
use bitcoin_customs::lifecycle::{
    init::{CustomArg, InitArgs},
    upgrade::UpgradeArgs,
};
use bitcoin_customs::queries::{
    EstimateFeeArg, GenTicketStatusArgs, RedeemFee, ReleaseTokenStatusArgs,
};
use bitcoin_customs::state::{CustomsState, GenTicketStatus, RunesBalance, RunesId};
use bitcoin_customs::state::{Mode, ReleaseTokenStatus};
use bitcoin_customs::updates::generate_ticket::{GenerateTicketArgs, GenerateTicketError};
use bitcoin_customs::updates::get_btc_address::GetBtcAddressArgs;
use bitcoin_customs::updates::update_btc_utxos::UpdateBtcUtxosErr;
use bitcoin_customs::updates::update_runes_balance::{
    UpdateRunesBalanceError, UpdateRunesBlanceArgs,
};
use bitcoin_customs::{CustomsInfo, Log, MIN_RELAY_FEE_PER_VBYTE, MIN_RESUBMISSION_DELAY};
use candid::{Decode, Encode, Nat, Principal};
use ic_base_types::{CanisterId, PrincipalId};
use ic_bitcoin_canister_mock::{OutPoint, PushUtxoToAddress, Utxo};
use ic_btc_interface::{Network, Txid};
use ic_canisters_http_types::{HttpRequest, HttpResponse};
use ic_state_machine_tests::{Cycles, StateMachine, StateMachineBuilder, WasmResult};
use ic_test_utilities_load_wasm::load_wasm;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;

const MIN_CONFIRMATIONS: u32 = 12;
const MAX_TIME_IN_QUEUE: Duration = Duration::from_secs(10);
const WITHDRAWAL_ADDRESS: &str = "bc1q34aq5drpuwy3wgl9lhup9892qp6svr8ldzyy7c";

fn customs_wasm() -> Vec<u8> {
    load_wasm(
        std::env::var("CARGO_MANIFEST_DIR").unwrap(),
        "bitcoin-customs",
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
        "hub-mock",
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
        hub_principal: CanisterId::from(0).into(),
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

fn range_to_txid(range: std::ops::RangeInclusive<u8>) -> Txid {
    vec_to_txid(range.collect::<Vec<u8>>())
}

#[test]
fn test_install_bitcoin_customs_canister() {
    let env = StateMachine::new();
    install_customs(&env);
}

// #[test]
// fn test_wrong_upgrade_parameter() {
//     let env = StateMachine::new();

//     // wrong init args

//     let args = CustomArg::Init(CkbtcMinterInitArgs {
//         btc_network: Network::Regtest.into(),
//         ecdsa_key_name: "".into(),
//         release_min_amount: 100_000,
//         ledger_id: CanisterId::from_u64(0),
//         max_time_in_queue_nanos: MAX_TIME_IN_QUEUE.as_nanos() as u64,
//         min_confirmations: Some(6_u32),
//         mode: Mode::GeneralAvailability,
//         kyt_fee: Some(1001),
//         kyt_principal: None,
//     });
//     let args = Encode!(&args).unwrap();
//     if env.install_canister(minter_wasm(), args, None).is_ok() {
//         panic!("init expected to fail")
//     }
//     let args = CustomArg::Init(CkbtcMinterInitArgs {
//         btc_network: Network::Regtest.into(),
//         ecdsa_key_name: "some_key".into(),
//         release_min_amount: 100_000,
//         ledger_id: CanisterId::from_u64(0),
//         max_time_in_queue_nanos: MAX_TIME_IN_QUEUE.as_nanos() as u64,
//         min_confirmations: Some(6_u32),
//         mode: Mode::GeneralAvailability,
//         kyt_fee: Some(1001),
//         kyt_principal: None,
//     });
//     let args = Encode!(&args).unwrap();
//     if env.install_canister(minter_wasm(), args, None).is_ok() {
//         panic!("init expected to fail")
//     }

//     // install the minter

//     let minter_id = install_minter(&env, CanisterId::from(0));

//     // upgrade only with wrong parameters

//     let upgrade_args = UpgradeArgs {
//         release_min_amount: Some(100),
//         min_confirmations: None,
//         max_time_in_queue_nanos: Some(100),
//         mode: Some(Mode::ReadOnly),
//         kyt_principal: None,
//         kyt_fee: None,
//     };
//     let minter_arg = CustomArg::Upgrade(Some(upgrade_args));
//     if env
//         .upgrade_canister(minter_id, minter_wasm(), Encode!(&minter_arg).unwrap())
//         .is_ok()
//     {
//         panic!("upgrade expected to fail")
//     }
// }

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
        hub_principal: CanisterId::from(0).into(),
    });
    let args = Encode!(&args).unwrap();
    let customs_id = env.install_canister(customs_wasm(), args, None).unwrap();

    let btc_address_1 = get_btc_address(
        &env,
        customs_id,
        &GetBtcAddressArgs {
            target_chain_id: String::from("cosmoshub"),
            receiver: String::from("cosmos1kwf682z5rxj38jsemljvdh67ykswns77j3euur"),
        },
    );
    let address_1 = Address::from_str(&btc_address_1).expect("invalid bitcoin address");
    let btc_address_2 = get_btc_address(
        &env,
        customs_id,
        &GetBtcAddressArgs {
            target_chain_id: String::from("cosmoshub"),
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

        env.install_existing_canister(
            customs_id,
            customs_wasm(),
            Encode!(&CustomArg::Init(InitArgs {
                btc_network: Network::Mainnet.into(),
                ecdsa_key_name: "master_ecdsa_public_key".to_string(),
                max_time_in_queue_nanos: 100,
                min_confirmations: Some(MIN_CONFIRMATIONS),
                mode: Mode::GeneralAvailability,
                hub_principal: hub_id.into(),
            }))
            .unwrap(),
        )
        .expect("failed to install the customs");

        let caller = PrincipalId::new_user_test_id(1);
        let runes_oracle = PrincipalId::new_node_test_id(2);

        env.install_existing_canister(hub_id, hub_mock_wasm(), vec![])
            .expect("failed to install the hub canister");

        env.execute_ingress(
            bitcoin_id,
            "set_fee_percentiles",
            Encode!(&(1..=100).map(|i| i * 100).collect::<Vec<u64>>()).unwrap(),
        )
        .expect("failed to set fee percentiles");

        Self {
            env,
            caller,
            runes_oracle,
            bitcoin_id,
            customs_id,
        }
    }

    pub fn set_fee_percentiles(&self, fees: &Vec<u64>) {
        self.env
            .execute_ingress(
                self.bitcoin_id,
                "set_fee_percentiles",
                Encode!(fees).unwrap(),
            )
            .expect("failed to set fee percentiles");
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

    pub fn push_utxo(&self, address: String, utxo: Utxo) {
        assert_reply(
            self.env
                .execute_ingress(
                    self.bitcoin_id,
                    "push_utxo_to_address",
                    Encode!(&PushUtxoToAddress { address, utxo }).unwrap(),
                )
                .expect("failed to push a UTXO"),
        );
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

    pub fn get_customs_info(&self) -> CustomsInfo {
        Decode!(
            &assert_reply(
                self.env
                    .execute_ingress(self.customs_id, "get_customs_info", Encode!().unwrap(),)
                    .expect("failed to get customs info")
            ),
            CustomsInfo
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

    pub fn refresh_fee_percentiles(&self) {
        Decode!(
            &assert_reply(
                self.env
                    .execute_ingress_as(
                        self.caller,
                        self.customs_id,
                        "refresh_fee_percentiles",
                        Encode!().unwrap()
                    )
                    .expect("failed to refresh fee percentiles")
            ),
            Option<Nat>
        )
        .unwrap();
    }

    pub fn estimate_redeem_fee(&self, runes_id: RunesId, amount: Option<u128>) -> RedeemFee {
        self.refresh_fee_percentiles();
        Decode!(
            &assert_reply(
                self.env
                    .query(
                        self.customs_id,
                        "estimate_redeem_fee",
                        Encode!(&EstimateFeeArg { runes_id, amount }).unwrap()
                    )
                    .expect("failed to query minter fee estimate")
            ),
            RedeemFee
        )
        .unwrap()
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
        args: &UpdateRunesBlanceArgs,
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
                        self.runes_oracle,
                        self.customs_id,
                        "update_btc_utxos",
                        vec![]
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

    pub fn finalize_transaction(&self, tx: &bitcoin::Transaction) {
        let btc_change_utxo = tx.output.last().unwrap();
        let btc_change_address =
            BtcAddress::from_script(&btc_change_utxo.script_pubkey, BtcNetwork::Bitcoin).unwrap();

        let main_address = self.get_main_btc_address(String::from("BTC"));
        assert_eq!(btc_change_address.to_string(), main_address);

        self.env
            .advance_time(MIN_CONFIRMATIONS * Duration::from_secs(600) + Duration::from_secs(1));
        let txid_bytes: [u8; 32] = tx.txid().to_vec().try_into().unwrap();
        // TODO push runes utxos
        self.push_utxo(
            btc_change_address.to_string(),
            Utxo {
                value: btc_change_utxo.value,
                height: 0,
                outpoint: OutPoint {
                    txid: txid_bytes.into(),
                    vout: 1,
                },
            },
        );
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
        target_chain_id: String::from("cosmoshub"),
        receiver: String::from("cosmos1fwaeqe84kaymymmqv0wyj75hzsdq4gfqm5xvvv"),
        runes_id: 1,
        amount: 1000,
        txid: range_to_txid(1..=32),
    });
    assert_eq!(result, Err(GenerateTicketError::NoNewUtxos));
}

#[test]
fn test_gen_ticket_with_insufficient_confirmations() {
    let customs = CustomsSetup::new();

    customs.set_tip_height(100);

    let txid = range_to_txid(1..=32);
    let utxo = Utxo {
        height: 99,
        outpoint: OutPoint { txid, vout: 1 },
        value: 546,
    };

    let target_chain_id = "cosmoshub".to_string();
    let receiver = "cosmos1fwaeqe84kaymymmqv0wyj75hzsdq4gfqm5xvvv".to_string();
    let deposit_address = customs.get_btc_address(Destination {
        target_chain_id: target_chain_id.clone(),
        receiver: receiver.clone(),
        token: None,
    });

    customs.push_utxo(deposit_address, utxo);
    let result = customs.generate_ticket(&GenerateTicketArgs {
        target_chain_id,
        receiver,
        runes_id: 1,
        amount: 100_000_000,
        txid,
    });
    assert_eq!(result, Err(GenerateTicketError::NoNewUtxos));
}

#[test]
fn test_gen_ticket_success() {
    let customs = CustomsSetup::new();

    customs.set_tip_height(100);

    let txid = range_to_txid(1..=32);
    let utxo = Utxo {
        height: 80,
        outpoint: OutPoint { txid, vout: 1 },
        value: 546,
    };

    let target_chain_id = "cosmoshub".to_string();
    let receiver = "cosmos1fwaeqe84kaymymmqv0wyj75hzsdq4gfqm5xvvv".to_string();
    let deposit_address = customs.get_btc_address(Destination {
        target_chain_id: target_chain_id.clone(),
        receiver: receiver.clone(),
        token: None,
    });

    customs.push_utxo(deposit_address, utxo);
    let result = customs.generate_ticket(&GenerateTicketArgs {
        target_chain_id,
        receiver,
        runes_id: 1,
        amount: 100_000_000,
        txid,
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

    let txid = range_to_txid(1..=32);
    let utxo = Utxo {
        height: 80,
        outpoint: OutPoint { txid, vout: 1 },
        value: 546,
    };

    let target_chain_id = "cosmoshub".to_string();
    let receiver = "cosmos1fwaeqe84kaymymmqv0wyj75hzsdq4gfqm5xvvv".to_string();
    let deposit_address = customs.get_btc_address(Destination {
        target_chain_id: target_chain_id.clone(),
        receiver: receiver.clone(),
        token: None,
    });

    let args = GenerateTicketArgs {
        target_chain_id,
        receiver,
        runes_id: 1,
        amount: 100_000_000,
        txid,
    };

    customs.push_utxo(deposit_address, utxo);
    let _ = customs.generate_ticket(&args);
    let result = customs.generate_ticket(&args);
    assert_eq!(result, Err(GenerateTicketError::AlreadySubmitted));
}

#[test]
fn test_update_runes_balance_no_utxo() {
    let customs = CustomsSetup::new();
    let result = customs.update_runes_balance(&UpdateRunesBlanceArgs {
        txid: range_to_txid(1..=32),
        vout: 1,
        balance: RunesBalance {
            runes_id: 1,
            value: 100_000_000,
        },
    });
    assert_eq!(result, Err(UpdateRunesBalanceError::UtxoNotFound));
}

#[test]
fn test_update_runes_balance_invalid() {
    let customs = CustomsSetup::new();

    customs.set_tip_height(100);

    let txid = range_to_txid(1..=32);
    let vout = 1;
    let utxo = Utxo {
        height: 80,
        outpoint: OutPoint { txid, vout },
        value: 546,
    };

    let target_chain_id = "cosmoshub".to_string();
    let receiver = "cosmos1fwaeqe84kaymymmqv0wyj75hzsdq4gfqm5xvvv".to_string();
    let deposit_address = customs.get_btc_address(Destination {
        target_chain_id: target_chain_id.clone(),
        receiver: receiver.clone(),
        token: None,
    });

    let args = GenerateTicketArgs {
        target_chain_id,
        receiver,
        runes_id: 1,
        amount: 100_000_000,
        txid,
    };

    customs.push_utxo(deposit_address, utxo);
    let result = customs.generate_ticket(&args);
    assert_eq!(result, Ok(()));

    let result = customs.update_runes_balance(&UpdateRunesBlanceArgs {
        txid,
        vout,
        balance: RunesBalance {
            runes_id: 1,
            // inconsistent with the value of generate ticket
            value: 100_000,
        },
    });
    assert_eq!(
        result,
        Err(UpdateRunesBalanceError::MismatchWithGenTicketReq)
    );

    let status = customs.generate_ticket_status(txid);
    assert_eq!(status, GenTicketStatus::Invalid);
}

#[test]
fn test_update_runes_balance_success() {
    let customs = CustomsSetup::new();

    let args = update_runes_balance(&customs);

    let status = customs.generate_ticket_status(args.txid);
    assert_eq!(status, GenTicketStatus::Finalized);
}

#[test]
fn test_duplicate_update_runes_balance() {
    let customs = CustomsSetup::new();

    let args = update_runes_balance(&customs);

    let status = customs.generate_ticket_status(args.txid);
    assert_eq!(status, GenTicketStatus::Finalized);

    let result = customs.update_runes_balance(&args);
    assert_eq!(result, Err(UpdateRunesBalanceError::AleardyProcessed));
}

fn update_runes_balance(customs: &CustomsSetup) -> UpdateRunesBlanceArgs {
    customs.set_tip_height(100);

    let txid = range_to_txid(1..=32);
    let vout = 1;
    let utxo = Utxo {
        height: 80,
        outpoint: OutPoint { txid, vout },
        value: 546,
    };

    let target_chain_id = "cosmoshub".to_string();
    let receiver = "cosmos1fwaeqe84kaymymmqv0wyj75hzsdq4gfqm5xvvv".to_string();
    let deposit_address = customs.get_btc_address(Destination {
        target_chain_id: target_chain_id.clone(),
        receiver: receiver.clone(),
        token: None,
    });

    customs.push_utxo(deposit_address, utxo);
    let result = customs.generate_ticket(&GenerateTicketArgs {
        target_chain_id,
        receiver,
        runes_id: 1,
        amount: 100_000_000,
        txid,
    });
    assert_eq!(result, Ok(()));

    let args = UpdateRunesBlanceArgs {
        txid,
        vout,
        balance: RunesBalance {
            runes_id: 1,
            value: 100_000_000,
        },
    };
    let result = customs.update_runes_balance(&args);
    assert_eq!(result, Ok(()));
    args
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
            "get_btc_address",git s
            Encode!(arg).unwrap()
        )
        .expect("failed to transfer funds")
        .bytes(),
        String
    )
    .expect("failed to decode String response")
}

// #[test]
// fn test_min_retrieval_amount() {
//     let ckbtc = CkBtcSetup::new();

//     ckbtc.refresh_fee_percentiles();
//     let retrieve_btc_min_amount = ckbtc.get_minter_info().release_min_amount;
//     assert_eq!(retrieve_btc_min_amount, 100_000);

//     // The numbers used in this test have been re-computed using a python script using integers.
//     ckbtc.set_fee_percentiles(&vec![0; 100]);
//     ckbtc.refresh_fee_percentiles();
//     let retrieve_btc_min_amount = ckbtc.get_minter_info().release_min_amount;
//     assert_eq!(retrieve_btc_min_amount, 100_000);

//     ckbtc.set_fee_percentiles(&vec![116_000; 100]);
//     ckbtc.refresh_fee_percentiles();
//     let retrieve_btc_min_amount = ckbtc.get_minter_info().release_min_amount;
//     assert_eq!(retrieve_btc_min_amount, 150_000);

//     ckbtc.set_fee_percentiles(&vec![342_000; 100]);
//     ckbtc.refresh_fee_percentiles();
//     let retrieve_btc_min_amount = ckbtc.get_minter_info().release_min_amount;
//     assert_eq!(retrieve_btc_min_amount, 150_000);

//     ckbtc.set_fee_percentiles(&vec![343_000; 100]);
//     ckbtc.refresh_fee_percentiles();
//     let retrieve_btc_min_amount = ckbtc.get_minter_info().release_min_amount;
//     assert_eq!(retrieve_btc_min_amount, 200_000);
// }

// #[test]
// fn test_get_logs() {
//     let ckbtc = CkBtcSetup::new();

//     // Test that the endpoint does not trap.
//     let _log = ckbtc.get_logs();
// }

// #[test]
// fn test_filter_logs() {
//     let ckbtc = CkBtcSetup::new();

//     // Trigger an even to add some logs.

//     let deposit_value = 100_000_000;
//     let utxo = Utxo {
//         height: 0,
//         outpoint: OutPoint {
//             txid: range_to_txid(1..=32),
//             vout: 1,
//         },
//         value: deposit_value,
//     };

//     let user = Principal::from(ckbtc.caller);

//     ckbtc.deposit_utxo(user, utxo);

//     let system_time = ckbtc.env.time();

//     let nanos = system_time
//         .duration_since(std::time::SystemTime::UNIX_EPOCH)
//         .expect("Time went backwards")
//         .as_nanos();

//     let request = HttpRequest {
//         method: "".to_string(),
//         url: format!("/logs?time={}", nanos),
//         headers: vec![],
//         body: serde_bytes::ByteBuf::new(),
//     };
//     let response = Decode!(
//         &assert_reply(
//             ckbtc
//                 .env
//                 .query(ckbtc.minter_id, "http_request", Encode!(&request).unwrap(),)
//                 .expect("failed to get minter info")
//         ),
//         HttpResponse
//     )
//     .unwrap();
//     let logs: Log =
//         serde_json::from_slice(&response.body).expect("failed to parse ckbtc minter log");

//     let request = HttpRequest {
//         method: "".to_string(),
//         url: format!("/logs?time={}", nanos + 30 * 1_000_000_000),
//         headers: vec![],
//         body: serde_bytes::ByteBuf::new(),
//     };
//     let response = Decode!(
//         &assert_reply(
//             ckbtc
//                 .env
//                 .query(ckbtc.minter_id, "http_request", Encode!(&request).unwrap(),)
//                 .expect("failed to get minter info")
//         ),
//         HttpResponse
//     )
//     .unwrap();
//     let logs_filtered: Log =
//         serde_json::from_slice(&response.body).expect("failed to parse ckbtc minter log");

//     assert_ne!(logs.entries.len(), logs_filtered.entries.len());
// }

// #[test]
// fn test_retrieve_btc_with_approval() {
//     let ckbtc = CkBtcSetup::new();

//     // Step 1: deposit ckBTC

//     let deposit_value = 100_000_000;
//     let utxo = Utxo {
//         height: 0,
//         outpoint: OutPoint {
//             txid: range_to_txid(1..=32),
//             vout: 1,
//         },
//         value: deposit_value,
//     };

//     let user = Principal::from(ckbtc.caller);

//     ckbtc.deposit_utxo(user, utxo);
//     assert_eq!(ckbtc.balance_of(user), Nat::from(deposit_value - KYT_FEE));

//     // Step 2: request a withdrawal

//     let withdrawal_amount = 50_000_000;
//     ckbtc.approve_minter(user, withdrawal_amount, None);
//     let fee_estimate = ckbtc.estimate_withdrawal_fee(Some(withdrawal_amount));

//     let RetrieveBtcOk { block_index } = ckbtc
//         .retrieve_btc_with_approval(WITHDRAWAL_ADDRESS.to_string(), withdrawal_amount, None)
//         .expect("retrieve_btc failed");

//     let get_transaction_request = GetTransactionsRequest {
//         start: block_index.into(),
//         length: 1_u8.into(),
//     };
//     let res = ckbtc.get_transactions(get_transaction_request);
//     let memo = res.transactions[0].burn.clone().unwrap().memo.unwrap();
//     use bitcoin_custom::memo::BurnMemo;

//     let decoded_data = minicbor::decode::<BurnMemo>(&memo.0).expect("failed to decode memo");
//     assert_eq!(
//         decoded_data,
//         BurnMemo::Convert {
//             address: Some(WITHDRAWAL_ADDRESS),
//             kyt_fee: Some(KYT_FEE),
//             status: None,
//         },
//         "memo not found in burn"
//     );

//     ckbtc.env.advance_time(MAX_TIME_IN_QUEUE);

//     // Step 3: wait for the transaction to be submitted

//     let txid = ckbtc.await_btc_transaction(block_index, 10);
//     let mempool = ckbtc.mempool();
//     assert_eq!(
//         mempool.len(),
//         1,
//         "ckbtc transaction did not appear in the mempool"
//     );
//     let tx = mempool
//         .get(&txid)
//         .expect("the mempool does not contain the withdrawal transaction");

//     assert_eq!(2, tx.output.len());
//     assert_eq!(
//         tx.output[0].value,
//         withdrawal_amount - fee_estimate.minter_fee - fee_estimate.bitcoin_fee
//     );

//     // Step 4: confirm the transaction

//     ckbtc.finalize_transaction(tx);
//     assert_eq!(ckbtc.await_finalization(block_index, 10), txid);
// }

// #[test]
// fn test_retrieve_btc_with_approval_from_subaccount() {
//     let ckbtc = CkBtcSetup::new();

//     // Step 1: deposit ckBTC

//     let deposit_value = 100_000_000;
//     let utxo = Utxo {
//         height: 0,
//         outpoint: OutPoint {
//             txid: range_to_txid(1..=32),
//             vout: 1,
//         },
//         value: deposit_value,
//     };

//     let user = Principal::from(ckbtc.caller);
//     let subaccount: Option<[u8; 32]> = Some([1; 32]);
//     let user_account = Account {
//         owner: user,
//         subaccount,
//     };

//     ckbtc.deposit_utxo(user_account, utxo);
//     assert_eq!(
//         ckbtc.balance_of(user_account),
//         Nat::from(deposit_value - KYT_FEE)
//     );

//     // Step 2: request a withdrawal

//     let withdrawal_amount = 50_000_000;
//     ckbtc.approve_minter(user, withdrawal_amount, subaccount);
//     let fee_estimate = ckbtc.estimate_withdrawal_fee(Some(withdrawal_amount));

//     let RetrieveBtcOk { block_index } = ckbtc
//         .retrieve_btc_with_approval(
//             WITHDRAWAL_ADDRESS.to_string(),
//             withdrawal_amount,
//             subaccount,
//         )
//         .expect("retrieve_btc failed");

//     let get_transaction_request = GetTransactionsRequest {
//         start: block_index.into(),
//         length: 1_u8.into(),
//     };
//     let res = ckbtc.get_transactions(get_transaction_request);
//     let memo = res.transactions[0].burn.clone().unwrap().memo.unwrap();
//     use bitcoin_custom::memo::BurnMemo;

//     let decoded_data = minicbor::decode::<BurnMemo>(&memo.0).expect("failed to decode memo");
//     assert_eq!(
//         decoded_data,
//         BurnMemo::Convert {
//             address: Some(WITHDRAWAL_ADDRESS),
//             kyt_fee: Some(KYT_FEE),
//             status: None,
//         },
//         "memo not found in burn"
//     );

//     assert_eq!(
//         ckbtc.retrieve_btc_status_v2_by_account(Some(user_account)),
//         vec![BtcRetrievalStatusV2 {
//             block_index,
//             status_v2: Some(ckbtc.retrieve_btc_status_v2(block_index))
//         }]
//     );

//     ckbtc.env.advance_time(MAX_TIME_IN_QUEUE);

//     // Step 3: wait for the transaction to be submitted

//     let txid = ckbtc.await_btc_transaction(block_index, 10);
//     let mempool = ckbtc.mempool();
//     assert_eq!(
//         mempool.len(),
//         1,
//         "ckbtc transaction did not appear in the mempool"
//     );
//     let tx = mempool
//         .get(&txid)
//         .expect("the mempool does not contain the withdrawal transaction");

//     assert_eq!(2, tx.output.len());
//     assert_eq!(
//         tx.output[0].value,
//         withdrawal_amount - fee_estimate.minter_fee - fee_estimate.bitcoin_fee
//     );

//     // Step 4: confirm the transaction

//     ckbtc.finalize_transaction(tx);
//     assert_eq!(ckbtc.await_finalization(block_index, 10), txid);

//     assert_eq!(
//         ckbtc.retrieve_btc_status_v2_by_account(Some(user_account)),
//         vec![BtcRetrievalStatusV2 {
//             block_index,
//             status_v2: Some(ckbtc.retrieve_btc_status_v2(block_index))
//         }]
//     );
// }

// #[test]
// fn test_retrieve_btc_with_approval_fail() {
//     let ckbtc = CkBtcSetup::new();

//     // Step 1: deposit ckBTC

//     let deposit_value = 100_000_000;
//     let utxo = Utxo {
//         height: 0,
//         outpoint: OutPoint {
//             txid: range_to_txid(1..=32),
//             vout: 1,
//         },
//         value: deposit_value,
//     };

//     let user = Principal::from(ckbtc.caller);
//     let user_account = Account {
//         owner: user,
//         subaccount: Some([1; 32]),
//     };

//     ckbtc.deposit_utxo(user_account, utxo);
//     assert_eq!(
//         ckbtc.balance_of(user_account),
//         Nat::from(deposit_value - KYT_FEE)
//     );

//     // Step 2: request a withdrawal with ledger stopped

//     let withdrawal_amount = 50_000_000;
//     ckbtc.approve_minter(user, u64::MAX, Some([1; 32]));

//     let stop_canister_result = ckbtc.env.stop_canister(ckbtc.ledger_id);
//     assert_matches!(stop_canister_result, Ok(_));

//     let retrieve_btc_result = ckbtc.retrieve_btc_with_approval(
//         WITHDRAWAL_ADDRESS.to_string(),
//         withdrawal_amount,
//         Some([1; 32]),
//     );
//     assert_matches!(
//         retrieve_btc_result,
//         Err(RetrieveBtcWithApprovalError::TemporarilyUnavailable(_))
//     );
//     let start_canister_result = ckbtc.env.start_canister(ckbtc.ledger_id);
//     assert_matches!(start_canister_result, Ok(_));

//     assert_eq!(
//         ckbtc.balance_of(user_account),
//         Nat::from(deposit_value - KYT_FEE - TRANSFER_FEE)
//     );

//     // Check that we reimburse ckBTC if the KYT check of the address fails

//     ckbtc
//         .env
//         .upgrade_canister(
//             ckbtc.kyt_id,
//             kyt_wasm(),
//             Encode!(&LifecycleArg::UpgradeArg(ic_ckbtc_kyt::UpgradeArg {
//                 minter_id: None,
//                 maintainers: None,
//                 mode: Some(KytMode::RejectAll),
//             }))
//             .unwrap(),
//         )
//         .expect("failed to upgrade the KYT canister");

//     let retrieve_btc_result = ckbtc.retrieve_btc_with_approval(
//         WITHDRAWAL_ADDRESS.to_string(),
//         withdrawal_amount,
//         Some([1; 32]),
//     );
//     assert_matches!(
//         retrieve_btc_result,
//         Err(RetrieveBtcWithApprovalError::GenericError { .. })
//     );
//     ckbtc.env.tick();
//     assert_eq!(
//         ckbtc.balance_of(user_account),
//         Nat::from(deposit_value - 2 * KYT_FEE - TRANSFER_FEE)
//     );

//     ckbtc
//         .env
//         .execute_ingress(ckbtc.minter_id, "distribute_kyt_fee", Encode!().unwrap())
//         .expect("failed to transfer funds");

//     assert_eq!(
//         ckbtc.balance_of(Principal::from(ckbtc.kyt_provider)),
//         Nat::from(2 * KYT_FEE)
//     );

//     // Check that we reimburse ckBTC if the call to the KYT canister fails

//     let stop_canister_result = ckbtc.env.stop_canister(ckbtc.kyt_id);
//     assert_matches!(stop_canister_result, Ok(_));

//     let retrieve_btc_result = ckbtc.retrieve_btc_with_approval(
//         WITHDRAWAL_ADDRESS.to_string(),
//         withdrawal_amount,
//         Some([1; 32]),
//     );
//     assert_matches!(
//         retrieve_btc_result,
//         Err(RetrieveBtcWithApprovalError::GenericError { .. })
//     );

//     let reimbursed_tx_block_index_2 = BtcRetrievalStatusV2 {
//         block_index: 2,
//         status_v2: Some(RetrieveBtcStatusV2::Reimbursed(ReimbursedDeposit {
//             account: user_account,
//             amount: withdrawal_amount,
//             reason: TaintedDestination {
//                 kyt_provider: ckbtc.kyt_provider.into(),
//                 kyt_fee: KYT_FEE,
//             },
//             mint_block_index: 3,
//         })),
//     };

//     assert_eq!(
//         ckbtc.retrieve_btc_status_v2_by_account(Some(user_account)),
//         vec![
//             reimbursed_tx_block_index_2.clone(),
//             BtcRetrievalStatusV2 {
//                 block_index: 5,
//                 status_v2: Some(RetrieveBtcStatusV2::WillReimburse(ReimburseDepositTask {
//                     account: user_account,
//                     amount: withdrawal_amount,
//                     reason: CallFailed
//                 }))
//             }
//         ]
//     );

//     ckbtc.env.tick();
//     assert_eq!(
//         ckbtc.balance_of(user_account),
//         Nat::from(deposit_value - 2 * KYT_FEE - TRANSFER_FEE)
//     );

//     assert_eq!(
//         ckbtc.retrieve_btc_status_v2_by_account(Some(user_account)),
//         vec![
//             reimbursed_tx_block_index_2,
//             BtcRetrievalStatusV2 {
//                 block_index: 5,
//                 status_v2: Some(RetrieveBtcStatusV2::Reimbursed(ReimbursedDeposit {
//                     account: user_account,
//                     amount: withdrawal_amount,
//                     reason: CallFailed,
//                     mint_block_index: 6
//                 }))
//             }
//         ]
//     );
// }
