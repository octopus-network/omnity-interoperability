use crate::destination::Destination;
use crate::state::{ReleaseTokenRequest, RuneId, RunesBalance, RunesUtxo, SubmittedBtcTransaction};
use crate::{
    address::BitcoinAddress, build_unsigned_transaction, estimate_fee, greedy,
    signature::EncodedSignature, tx, BuildTxError,
};
use crate::{
    lifecycle::init::InitArgs,
    state::{CustomsState, Mode, ReleaseTokenStatus},
};
use bitcoin::network::constants::Network as BtcNetwork;
use bitcoin::util::psbt::serialize::{Deserialize, Serialize};
use ic_base_types::CanisterId;
use ic_btc_interface::{Network, OutPoint, Satoshi, Txid, Utxo};
use proptest::proptest;
use proptest::{
    array::uniform20,
    array::uniform32,
    collection::{btree_set, vec as pvec, SizeRange},
    option,
    prelude::{any, Strategy},
};
use proptest::{prop_assert, prop_assert_eq, prop_assume, prop_oneof};
use serde_bytes::ByteBuf;
use std::collections::{BTreeMap, BTreeSet};
use std::str::FromStr;

fn dummy_utxo_from_value(v: u64) -> Utxo {
    let mut bytes = [0u8; 32];
    bytes[0..8].copy_from_slice(&v.to_be_bytes());
    Utxo {
        outpoint: OutPoint {
            txid: bytes.into(),
            vout: 0,
        },
        value: v,
        height: 0,
    }
}

fn address_to_script_pubkey(address: &BitcoinAddress) -> bitcoin::Script {
    let address_string = address.display(Network::Mainnet);
    let btc_address = bitcoin::Address::from_str(&address_string).unwrap();
    btc_address.script_pubkey()
}

fn network_to_btc_network(network: Network) -> BtcNetwork {
    match network {
        Network::Mainnet => BtcNetwork::Bitcoin,
        Network::Testnet => BtcNetwork::Testnet,
        Network::Regtest => BtcNetwork::Regtest,
    }
}

fn address_to_btc_address(address: &BitcoinAddress, network: Network) -> bitcoin::Address {
    use bitcoin::util::address::{Payload, WitnessVersion};
    match address {
        BitcoinAddress::P2wpkhV0(pkhash) => bitcoin::Address {
            payload: Payload::WitnessProgram {
                version: WitnessVersion::V0,
                program: pkhash.to_vec(),
            },
            network: network_to_btc_network(network),
        },
        BitcoinAddress::P2wshV0(script_hash) => bitcoin::Address {
            payload: Payload::WitnessProgram {
                version: WitnessVersion::V0,
                program: script_hash.to_vec(),
            },
            network: network_to_btc_network(network),
        },
        BitcoinAddress::P2pkh(pkhash) => bitcoin::Address {
            payload: Payload::PubkeyHash(bitcoin::PubkeyHash::from_hash(
                bitcoin::hashes::Hash::from_slice(pkhash).unwrap(),
            )),
            network: network_to_btc_network(network),
        },
        BitcoinAddress::P2sh(script_hash) => bitcoin::Address {
            payload: Payload::ScriptHash(bitcoin::ScriptHash::from_hash(
                bitcoin::hashes::Hash::from_slice(script_hash).unwrap(),
            )),
            network: network_to_btc_network(network),
        },
        BitcoinAddress::P2trV1(pkhash) => bitcoin::Address {
            payload: Payload::WitnessProgram {
                version: WitnessVersion::V1,
                program: pkhash.to_vec(),
            },
            network: network_to_btc_network(network),
        },
        BitcoinAddress::OpReturn(script) => bitcoin::Address {
            payload: Payload::WitnessProgram {
                version: WitnessVersion::V1,
                program: script.clone(),
            },
            network: network_to_btc_network(network),
        },
    }
}

fn as_txid(hash: &[u8; 32]) -> bitcoin::Txid {
    bitcoin::Txid::from_hash(bitcoin::hashes::Hash::from_slice(hash).unwrap())
}

fn p2wpkh_script_code(pkhash: &[u8; 20]) -> bitcoin::Script {
    use bitcoin::blockdata::{opcodes, script::Builder};

    Builder::new()
        .push_opcode(opcodes::all::OP_DUP)
        .push_opcode(opcodes::all::OP_HASH160)
        .push_slice(&pkhash[..])
        .push_opcode(opcodes::all::OP_EQUALVERIFY)
        .push_opcode(opcodes::all::OP_CHECKSIG)
        .into_script()
}

fn unsigned_tx_to_bitcoin_tx(tx: &tx::UnsignedTransaction) -> bitcoin::Transaction {
    bitcoin::Transaction {
        version: tx::TX_VERSION as i32,
        lock_time: tx.lock_time,
        input: tx
            .inputs
            .iter()
            .map(|txin| bitcoin::TxIn {
                previous_output: bitcoin::OutPoint {
                    txid: as_txid(&txin.previous_output.txid.into()),
                    vout: txin.previous_output.vout,
                },
                sequence: txin.sequence,
                script_sig: bitcoin::Script::default(),
                witness: bitcoin::Witness::default(),
            })
            .collect(),
        output: tx
            .outputs
            .iter()
            .map(|txout| bitcoin::TxOut {
                value: txout.value,
                script_pubkey: address_to_script_pubkey(&txout.address),
            })
            .collect(),
    }
}

fn signed_tx_to_bitcoin_tx(tx: &tx::SignedTransaction) -> bitcoin::Transaction {
    bitcoin::Transaction {
        version: tx::TX_VERSION as i32,
        lock_time: tx.lock_time,
        input: tx
            .inputs
            .iter()
            .map(|txin| bitcoin::TxIn {
                previous_output: bitcoin::OutPoint {
                    txid: as_txid(&txin.previous_output.txid.into()),
                    vout: txin.previous_output.vout,
                },
                sequence: txin.sequence,
                script_sig: bitcoin::Script::default(),
                witness: bitcoin::Witness::from_vec(vec![
                    txin.signature.as_slice().to_vec(),
                    txin.pubkey.to_vec(),
                ]),
            })
            .collect(),
        output: tx
            .outputs
            .iter()
            .map(|txout| bitcoin::TxOut {
                value: txout.value,
                script_pubkey: address_to_script_pubkey(&txout.address),
            })
            .collect(),
    }
}

#[test]
fn greedy_smoke_test() {
    let mut utxos: BTreeSet<Utxo> = (1..10u64).map(dummy_utxo_from_value).collect();
    assert_eq!(utxos.len(), 9_usize);

    let res = greedy(15, &mut utxos, |u| u.value as u128);

    assert_eq!(res[0].value, 9_u64);
    assert_eq!(res[1].value, 6_u64);
}

#[test]
fn should_have_same_input_and_output_count() {
    let rune_id: RuneId = 1;
    let mut available_runes_utxos = BTreeSet::new();
    let mut available_btc_utxos = BTreeSet::new();
    for i in 0..crate::UTXOS_COUNT_THRESHOLD {
        available_runes_utxos.insert(RunesUtxo {
            raw: Utxo {
                outpoint: OutPoint {
                    txid: [9; 32].into(),
                    vout: i as u32,
                },
                value: 0,
                height: 10,
            },
            runes: RunesBalance {
                rune_id,
                vout: i as u32,
                amount: 0,
            },
        });
    }
    available_runes_utxos.insert(RunesUtxo {
        raw: Utxo {
            outpoint: OutPoint {
                txid: [0; 32].into(),
                vout: 0,
            },
            value: 546,
            height: 10,
        },
        runes: RunesBalance {
            rune_id,
            vout: 0,
            amount: 100_000,
        },
    });

    available_runes_utxos.insert(RunesUtxo {
        raw: Utxo {
            outpoint: OutPoint {
                txid: [1; 32].into(),
                vout: 0,
            },
            value: 546,
            height: 10,
        },
        runes: RunesBalance {
            rune_id,
            vout: 0,
            amount: 100_000,
        },
    });

    available_runes_utxos.insert(RunesUtxo {
        raw: Utxo {
            outpoint: OutPoint {
                txid: [2; 32].into(),
                vout: 0,
            },
            value: 546,
            height: 10,
        },
        runes: RunesBalance {
            rune_id,
            vout: 0,
            amount: 100,
        },
    });

    available_runes_utxos.insert(RunesUtxo {
        raw: Utxo {
            outpoint: OutPoint {
                txid: [3; 32].into(),
                vout: 0,
            },
            value: 546,
            height: 11,
        },
        runes: RunesBalance {
            rune_id,
            vout: 0,
            amount: 100,
        },
    });

    available_btc_utxos.insert(Utxo {
        outpoint: OutPoint {
            txid: [4; 32].into(),
            vout: 0,
        },
        value: 100_000,
        height: 11,
    });

    let runes_main_addr = BitcoinAddress::P2wpkhV0([0; 20]);
    let btc_main_addr = BitcoinAddress::P2wpkhV0([1; 20]);
    let out1_addr = BitcoinAddress::P2wpkhV0([2; 20]);
    let out2_addr = BitcoinAddress::P2wpkhV0([3; 20]);
    let fee_per_vbyte = 10000;

    let (tx, runes_change_output, btc_change_output, _, _) = build_unsigned_transaction(
        &rune_id,
        &mut available_runes_utxos,
        &mut available_btc_utxos,
        runes_main_addr,
        btc_main_addr,
        vec![(out1_addr.clone(), 100_000), (out2_addr.clone(), 99_999)],
        fee_per_vbyte,
        false,
    )
    .expect("failed to build a transaction");

    assert_eq!(tx.outputs.len(), tx.inputs.len());
    assert_eq!(runes_change_output.vout, 1);
    assert_eq!(btc_change_output.vout, (tx.outputs.len() - 1) as u32);
}

#[test]
fn test_not_enough_gas() {
    let rune_id: RuneId = 1;
    let mut available_runes_utxos = BTreeSet::new();
    let mut available_btc_utxos = BTreeSet::new();
    available_runes_utxos.insert(RunesUtxo {
        raw: Utxo {
            outpoint: OutPoint {
                txid: [0; 32].into(),
                vout: 0,
            },
            value: 546,
            height: 10,
        },
        runes: RunesBalance {
            rune_id,
            vout: 0,
            amount: 100_000,
        },
    });

    let runes_main_addr = BitcoinAddress::P2wpkhV0([0; 20]);
    let btc_main_addr = BitcoinAddress::P2wpkhV0([1; 20]);
    let out1_addr = BitcoinAddress::P2wpkhV0([2; 20]);
    let out2_addr = BitcoinAddress::P2wpkhV0([3; 20]);
    let fee_per_vbyte = 10000;

    assert_eq!(
        build_unsigned_transaction(
            &rune_id,
            &mut available_runes_utxos,
            &mut available_btc_utxos,
            runes_main_addr,
            btc_main_addr,
            vec![(out1_addr.clone(), 99_900), (out2_addr.clone(), 100)],
            fee_per_vbyte,
            false,
        ),
        Err(BuildTxError::NotEnoughGas)
    );
}

fn arb_amount() -> impl Strategy<Value = Satoshi> {
    1..10_000_000_000u64
}

fn vec_to_txid(vec: Vec<u8>) -> Txid {
    let bytes: [u8; 32] = vec.try_into().expect("Can't convert to [u8; 32]");
    bytes.into()
}

fn arb_out_point() -> impl Strategy<Value = tx::OutPoint> {
    (pvec(any::<u8>(), 32), any::<u32>()).prop_map(|(txid, vout)| tx::OutPoint {
        txid: vec_to_txid(txid),
        vout,
    })
}

fn arb_unsigned_input(
    value: impl Strategy<Value = Satoshi>,
) -> impl Strategy<Value = tx::UnsignedInput> {
    (arb_out_point(), value, any::<u32>()).prop_map(|(previous_output, value, sequence)| {
        tx::UnsignedInput {
            previous_output,
            value,
            sequence,
        }
    })
}

fn arb_signed_input() -> impl Strategy<Value = tx::SignedInput> {
    (
        arb_out_point(),
        any::<u32>(),
        pvec(1u8..0xff, 64),
        pvec(any::<u8>(), 32),
    )
        .prop_map(
            |(previous_output, sequence, sec1, pubkey)| tx::SignedInput {
                previous_output,
                sequence,
                signature: EncodedSignature::from_sec1(&sec1),
                pubkey: ByteBuf::from(pubkey),
            },
        )
}

fn arb_address() -> impl Strategy<Value = BitcoinAddress> {
    prop_oneof![
        uniform20(any::<u8>()).prop_map(BitcoinAddress::P2wpkhV0),
        uniform32(any::<u8>()).prop_map(BitcoinAddress::P2wshV0),
        uniform32(any::<u8>()).prop_map(BitcoinAddress::P2trV1),
        uniform20(any::<u8>()).prop_map(BitcoinAddress::P2pkh),
        uniform20(any::<u8>()).prop_map(BitcoinAddress::P2sh),
    ]
}

fn arb_tx_out() -> impl Strategy<Value = tx::TxOut> {
    (arb_amount(), arb_address()).prop_map(|(value, address)| tx::TxOut { value, address })
}

fn arb_runes_utxo(amount: impl Strategy<Value = u128>) -> impl Strategy<Value = RunesUtxo> {
    (amount, pvec(any::<u8>(), 32), 0..5u32).prop_map(|(value, txid, vout)| RunesUtxo {
        raw: Utxo {
            outpoint: OutPoint {
                txid: vec_to_txid(txid),
                vout,
            },
            value: 546,
            height: 0,
        },
        runes: RunesBalance {
            rune_id: 1,
            vout,
            amount: value,
        },
    })
}

fn arb_btc_utxo(amount: impl Strategy<Value = Satoshi>) -> impl Strategy<Value = Utxo> {
    (amount, pvec(any::<u8>(), 32), 0..5u32).prop_map(|(value, txid, vout)| Utxo {
        outpoint: OutPoint {
            txid: vec_to_txid(txid),
            vout,
        },
        value,
        height: 0,
    })
}

fn arb_destination() -> impl Strategy<Value = Destination> {
    (
        any::<String>(),
        any::<String>(),
        option::of(any::<String>()),
    )
        .prop_map(|(target_chain_id, receiver, token)| Destination {
            target_chain_id,
            receiver,
            token,
        })
}

fn arb_release_token_requests(
    amount: impl Strategy<Value = u128>,
    num: impl Into<SizeRange>,
) -> impl Strategy<Value = Vec<ReleaseTokenRequest>> {
    let request_strategy = (
        amount,
        arb_address(),
        any::<String>(),
        1569975147000..2069975147000u64,
    )
        .prop_map(
            |(amount, address, ticket_id, received_at)| ReleaseTokenRequest {
                ticket_id,
                rune_id: 1_u128,
                amount,
                address,
                received_at,
            },
        );
    pvec(request_strategy, num).prop_map(|mut reqs| {
        reqs.sort_by_key(|req| req.received_at);

        for (i, req) in reqs.iter_mut().enumerate() {
            req.ticket_id = format!("{}", i);
        }

        reqs
    })
}

proptest! {
    #[test]
    fn queue_holds_one_copy_of_each_task(
        timestamps in pvec(1_000_000_u64..1_000_000_000, 2..100),
    ) {
        use crate::tasks::{Task, TaskQueue, TaskType};

        let mut task_queue: TaskQueue = Default::default();
        for (i, ts) in timestamps.iter().enumerate() {
            task_queue.schedule_at(*ts, TaskType::ProcessLogic);
            prop_assert_eq!(task_queue.len(), 1, "queue: {:?}", task_queue);

            let task = task_queue.pop_if_ready(u64::MAX).unwrap();

            prop_assert_eq!(task_queue.len(), 0);

            prop_assert_eq!(&task, &Task{
                execute_at: timestamps[0..=i].iter().cloned().min().unwrap(),
                task_type: TaskType::ProcessLogic
            });
            task_queue.schedule_at(task.execute_at, task.task_type);

            prop_assert_eq!(task_queue.len(), 1);
        }
    }

    #[test]
    fn greedy_solution_properties(
        values in pvec(1u64..1_000_000_000, 1..10),
        target in 1u64..1_000_000_000,
    ) {
        let mut utxos: BTreeSet<Utxo> = values
            .into_iter()
            .map(dummy_utxo_from_value)
            .collect();

        let total = utxos.iter().map(|u| u.value).sum::<u64>();

        if total < target {
            utxos.insert(dummy_utxo_from_value(target - total));
        }

        let original_utxos = utxos.clone();

        let solution = greedy(target as u128, &mut utxos, |u| u.value as u128);

        prop_assert!(
            !solution.is_empty(),
            "greedy() must always find a solution given enough available UTXOs"
        );

        prop_assert!(
            solution.iter().map(|u| u.value).sum::<u64>() >= target,
            "greedy() must reach the specified target amount"
        );

        prop_assert!(
            solution.iter().all(|u| original_utxos.contains(u)),
            "greedy() must select utxos from the available set"
        );

        prop_assert!(
            solution.iter().all(|u| !utxos.contains(u)),
            "greedy() must remove found UTXOs from the available set"
        );
    }

    #[test]
    fn greedy_does_not_modify_input_when_fails(
        values in pvec(1u64..1_000_000_000, 1..10),
    ) {
        let mut utxos: BTreeSet<Utxo> = values
            .into_iter()
            .map(dummy_utxo_from_value)
            .collect();

        let total = utxos.iter().map(|u| u.value).sum::<u64>();

        let original_utxos = utxos.clone();
        let solution = greedy((total + 1) as u128, &mut utxos, |u| u.value as u128);

        prop_assert!(solution.is_empty());
        prop_assert_eq!(utxos, original_utxos);
    }

    #[test]
    fn unsigned_tx_encoding_model(
        inputs in pvec(arb_unsigned_input(5_000u64..1_000_000_000), 1..20),
        outputs in pvec(arb_tx_out(), 1..20),
        lock_time in any::<u32>(),
    ) {
        let arb_tx = tx::UnsignedTransaction { inputs, outputs, lock_time };
        println!("{:?}", arb_tx);
        let btc_tx = unsigned_tx_to_bitcoin_tx(&arb_tx);
        println!("{:?}", btc_tx.serialize());

        let tx_bytes = tx::encode_into(&arb_tx, Vec::<u8>::new());
        println!("{:?}", tx_bytes);
        let decoded_btc_tx = bitcoin::Transaction::deserialize(&tx_bytes).expect("failed to deserialize an unsigned transaction");

        prop_assert_eq!(btc_tx.serialize(), tx_bytes);
        prop_assert_eq!(&decoded_btc_tx, &btc_tx);
        prop_assert_eq!(&arb_tx.txid().as_ref().to_vec(), &*btc_tx.txid());
    }

    #[test]
    fn unsigned_tx_sighash_model(
        inputs_data in pvec(
            (
                arb_btc_utxo(5_000u64..1_000_000_000),
                any::<u32>(),
                pvec(any::<u8>(), tx::PUBKEY_LEN)
            ),
            1..20
        ),
        outputs in pvec(arb_tx_out(), 1..20),
        lock_time in any::<u32>(),
    ) {
        let inputs: Vec<tx::UnsignedInput> = inputs_data
            .iter()
            .map(|(utxo, seq, _)| tx::UnsignedInput {
                previous_output: utxo.outpoint.clone(),
                value: utxo.value,
                sequence: *seq,
            })
            .collect();
        let arb_tx = tx::UnsignedTransaction { inputs, outputs, lock_time };
        let btc_tx = unsigned_tx_to_bitcoin_tx(&arb_tx);

        let sighasher = tx::TxSigHasher::new(&arb_tx);
        let mut btc_sighasher = bitcoin::util::sighash::SighashCache::new(&btc_tx);

        for (i, (utxo, _, pubkey)) in inputs_data.iter().enumerate() {
            let mut buf = Vec::<u8>::new();
            let pkhash = tx::hash160(pubkey);

            sighasher.encode_sighash_data(&arb_tx.inputs[i], &pkhash, &mut buf);

            let mut btc_buf = Vec::<u8>::new();
            let script_code = p2wpkh_script_code(&pkhash);
            btc_sighasher.segwit_encode_signing_data_to(&mut btc_buf, i, &script_code, utxo.value, bitcoin::EcdsaSighashType::All)
                .expect("failed to encode sighash data");
            prop_assert_eq!(hex::encode(&buf), hex::encode(&btc_buf));

            let sighash = sighasher.sighash(&arb_tx.inputs[i], &pkhash);
            let btc_sighash = btc_sighasher.segwit_signature_hash(i, &script_code, utxo.value, bitcoin::EcdsaSighashType::All).unwrap();
            prop_assert_eq!(hex::encode(sighash), hex::encode(btc_sighash));
        }
    }

    #[test]
    fn signed_tx_encoding_model(
        inputs in pvec(arb_signed_input(), 1..20),
        outputs in pvec(arb_tx_out(), 1..20),
        lock_time in any::<u32>(),
    ) {
        let arb_tx = tx::SignedTransaction { inputs, outputs, lock_time };
        println!("{:?}", arb_tx);
        let btc_tx = signed_tx_to_bitcoin_tx(&arb_tx);
        println!("{:?}", btc_tx.serialize());

        let tx_bytes = tx::encode_into(&arb_tx, Vec::<u8>::new());
        println!("{:?}", tx_bytes);
        let decoded_btc_tx = bitcoin::Transaction::deserialize(&tx_bytes).expect("failed to deserialize a signed transaction");

        prop_assert_eq!(btc_tx.serialize(), tx_bytes);
        prop_assert_eq!(&decoded_btc_tx, &btc_tx);
        prop_assert_eq!(&arb_tx.wtxid(), &*btc_tx.wtxid());
        prop_assert_eq!(arb_tx.vsize(), btc_tx.vsize());
    }

    #[test]
    fn check_output_order(
        mut runes_utxos in btree_set(arb_runes_utxo(1_000_000u128..1_000_000_000), 1..20),
        mut btc_utxos in btree_set(arb_btc_utxo(5_000u64..1_000_000_000), 1..20),
        dst_pkhash in uniform20(any::<u8>()),
        runes_pkhash in uniform20(any::<u8>()),
        btc_pkhash in uniform20(any::<u8>()),
        target in 50000..100000u128,
        fee_per_vbyte in 1000..2000u64,
    ) {
        prop_assume!(dst_pkhash != runes_pkhash);

        let (unsigned_tx, _, _, _, _) = build_unsigned_transaction(
            &1_u128,
            &mut runes_utxos,
            &mut btc_utxos,
            BitcoinAddress::P2wpkhV0(runes_pkhash),
            BitcoinAddress::P2wpkhV0(btc_pkhash),
            vec![(BitcoinAddress::P2wpkhV0(dst_pkhash), target)],
            fee_per_vbyte,
            false
        )
        .expect("failed to build transaction");

        prop_assert_eq!(&unsigned_tx.outputs.get(1).unwrap().address, &BitcoinAddress::P2wpkhV0(runes_pkhash));
        prop_assert_eq!(&unsigned_tx.outputs.get(2).unwrap().address, &BitcoinAddress::P2wpkhV0(dst_pkhash));
    }

    #[test]
    fn build_tx_not_enough_funds(
        mut runes_utxos in btree_set(arb_runes_utxo(5_000u128..1_000_000_000), 1..20),
        mut btc_utxos in btree_set(arb_btc_utxo(5_000u64..1_000_000_000), 1..20),
        dst_pkhash in uniform20(any::<u8>()),
        runes_pkhash in uniform20(any::<u8>()),
        btc_pkhash in uniform20(any::<u8>()),
        fee_per_vbyte in 1000..2000u64,
    ) {
        let runes_utxos_copy = runes_utxos.clone();
        let btc_utxos_copy = btc_utxos.clone();

        let total_value = runes_utxos.iter().map(|u| u.runes.amount).sum::<u128>();

        prop_assert_eq!(
            build_unsigned_transaction(
                &1_u128,
                &mut runes_utxos,
                &mut btc_utxos,
                BitcoinAddress::P2wpkhV0(runes_pkhash),
                BitcoinAddress::P2wpkhV0(btc_pkhash),
                vec![(BitcoinAddress::P2wpkhV0(dst_pkhash), total_value * 2)],
                fee_per_vbyte,
                false,
            ).expect_err("build transaction should fail because the amount is too high"),
            BuildTxError::NotEnoughFunds
        );
        prop_assert_eq!(&runes_utxos_copy, &runes_utxos);
        prop_assert_eq!(&btc_utxos_copy, &btc_utxos);
    }

    #[test]
    fn add_utxos_maintains_invariants(
        utxos_dest_idx in pvec((arb_btc_utxo(5_000u64..1_000_000_000), 0..5usize), 10..20),
        destinations in pvec(arb_destination(), 5),
    ) {
        let mut state = CustomsState::from(InitArgs {
            btc_network: Network::Regtest.into(),
            ecdsa_key_name: "".to_string(),
            max_time_in_queue_nanos: 0,
            min_confirmations: None,
            mode: Mode::GeneralAvailability,
            hub_principal: CanisterId::from_u64(1).into(),
            runes_oracle_principal: CanisterId::from_u64(2).into(),
        });
        for (utxo, dest_idx) in utxos_dest_idx {
            state.add_utxos(destinations[dest_idx].clone(), vec![utxo], false);
            state.check_invariants().expect("invariant check failed");
        }
    }

    #[test]
    fn batching_preserves_invariants(
        utxos_dest_idx in pvec((arb_runes_utxo(5_000u128..1_000_000_000), 0..5usize), 10..20),
        destinations in pvec(arb_destination(), 5),
        requests in arb_release_token_requests(5_000u128..1_000_000_000, 1..25),
        limit in 1..25usize,
    ) {
        let mut state = CustomsState::from(InitArgs {
            btc_network: Network::Regtest.into(),
            ecdsa_key_name: "".to_string(),
            max_time_in_queue_nanos: 0,
            min_confirmations: None,
            mode: Mode::GeneralAvailability,
            hub_principal: CanisterId::from_u64(1).into(),
            runes_oracle_principal: CanisterId::from_u64(2).into(),
        });

        let mut available_amount = 0;
        for (utxo, dest_idx) in utxos_dest_idx {
            available_amount += utxo.runes.amount;
            state.add_utxos(destinations[dest_idx].clone(), vec![utxo.raw.clone()], true);
            state.update_runes_balance(utxo.raw.outpoint.txid, utxo.runes);
        }
        for req in requests {
            state.push_back_pending_request(req.clone());
            prop_assert_eq!(state.release_token_status(&req.ticket_id), ReleaseTokenStatus::Pending);
        }

        let batch = state.build_batch(&1_u128, limit);

        for req in batch.iter() {
            prop_assert_eq!(state.release_token_status(&req.ticket_id), ReleaseTokenStatus::Unknown);
        }

        prop_assert!(batch.iter().map(|req| req.amount).sum::<u128>() <= available_amount);
        prop_assert!(batch.len() <= limit);

        state.check_invariants().expect("invariant check failed");
    }

    #[test]
    fn tx_replacement_preserves_invariants(
        destinations in pvec(arb_destination(), 5),
        btc_main_dest in arb_destination(),
        runes_utxos_dest_idx in pvec((arb_runes_utxo(5_000_000u128..1_000_000_000), 0..5usize), 10..=10),
        btc_utxos in pvec(arb_btc_utxo(5_000_000u64..1_000_000_000), 10..=10),
        requests in arb_release_token_requests(5_000_000u128..10_000_000, 1..5),
        runes_pkhash in uniform20(any::<u8>()),
        btc_pkhash in uniform20(any::<u8>()),
        resubmission_chain_length in 1..=5,
    ) {
        let rune_id: RuneId = 1;
        let mut state = CustomsState::from(InitArgs {
            btc_network: Network::Regtest.into(),
            ecdsa_key_name: "".to_string(),
            max_time_in_queue_nanos: 0,
            min_confirmations: None,
            mode: Mode::GeneralAvailability,
            hub_principal: CanisterId::from_u64(1).into(),
            runes_oracle_principal: CanisterId::from_u64(2).into(),
        });

        for (utxo, dest_idx) in runes_utxos_dest_idx {
            state.add_utxos(destinations[dest_idx].clone(), vec![utxo.raw.clone()], true);
            state.update_runes_balance(utxo.raw.outpoint.txid, utxo.runes);
        }
        state.add_utxos(btc_main_dest, btc_utxos, false);

        let fee_per_vbyte = 100_000u64;

        let (tx, runes_change_output, btc_change_output, runes_utxos, btc_utxos) = build_unsigned_transaction(
            &rune_id,
            &mut state.available_runes_utxos,
            &mut state.available_fee_utxos,
            BitcoinAddress::P2wpkhV0(runes_pkhash),
            BitcoinAddress::P2wpkhV0(btc_pkhash),
            requests.iter().map(|r| (r.address.clone(), r.amount)).collect(),
            fee_per_vbyte,
            false
        )
        .expect("failed to build transaction");
        let mut txids = vec![tx.txid()];
        let submitted_at = 1_234_567_890;

        state.push_submitted_transaction(SubmittedBtcTransaction {
            rune_id,
            requests: requests.clone(),
            txid: txids[0],
            runes_utxos: runes_utxos.clone(),
            btc_utxos: btc_utxos.clone(),
            submitted_at,
            runes_change_output,
            btc_change_output,
            fee_per_vbyte: Some(fee_per_vbyte),
            raw_tx: "".into(),
        });

        state.check_invariants().expect("violated invariants");

        for i in 1..=resubmission_chain_length {
            let prev_txid = txids.last().unwrap();
            // Build a replacement transaction
            let (tx, runes_change_output, btc_change_output, _, _) = build_unsigned_transaction(
                &rune_id,
                &mut runes_utxos.clone().into_iter().collect(),
                &mut btc_utxos.clone().into_iter().collect(),
                BitcoinAddress::P2wpkhV0(runes_pkhash),
                BitcoinAddress::P2wpkhV0(btc_pkhash),
                requests.iter().map(|r| (r.address.clone(), r.amount)).collect(),
                fee_per_vbyte + 1000 * i as u64,
                false
            )
            .expect("failed to build transaction");

            let new_txid = tx.txid();

            state.replace_transaction(prev_txid, SubmittedBtcTransaction {
                rune_id,
                requests: requests.clone(),
                txid: new_txid,
                runes_utxos: runes_utxos.clone(),
                btc_utxos: btc_utxos.clone(),
                submitted_at,
                runes_change_output,
                btc_change_output,
                fee_per_vbyte: Some(fee_per_vbyte),
                raw_tx: "".into(),
            });

            for txid in &txids {
                prop_assert_eq!(state.find_last_replacement_tx(txid), Some(&new_txid));
            }

            txids.push(new_txid);

            assert_eq!(i as usize, state.longest_resubmission_chain_size());
            state.check_invariants().expect("violated invariants after transaction resubmission");
        }

        for txid in &txids {
            // Ensure that finalizing any transaction in the chain removes the entire chain.
            let mut state = state.clone();
            state.finalize_transaction(txid);
            prop_assert_eq!(&state.submitted_transactions, &vec![]);
            prop_assert_eq!(&state.stuck_transactions, &vec![]);
            prop_assert_eq!(&state.replacement_txid, &BTreeMap::new());
            prop_assert_eq!(&state.rev_replacement_txid, &BTreeMap::new());
            state.check_invariants().expect("violated invariants after transaction finalization");
        }
    }

    #[test]
    fn btc_v0_p2wpkh_address_parsing(mut pkbytes in pvec(any::<u8>(), 32)) {
        use crate::address::network_and_public_key_to_p2wpkh;
        pkbytes.insert(0, 0x02);

        for network in [Network::Mainnet, Network::Testnet, Network::Regtest].iter() {
            let addr = network_and_public_key_to_p2wpkh(*network, &pkbytes);
            prop_assert_eq!(
                Ok(BitcoinAddress::P2wpkhV0(tx::hash160(&pkbytes))),
                BitcoinAddress::parse(&addr, *network)
            );
        }
    }

    #[test]
    fn btc_address_parsing_model(mut pkbytes in pvec(any::<u8>(), 32)) {
        pkbytes.insert(0, 0x02);

        let pk_result = bitcoin::PublicKey::from_slice(&pkbytes);

        prop_assume!(pk_result.is_ok());

        let pk = pk_result.unwrap();
        let pkhash = tx::hash160(&pkbytes);

        for network in [Network::Mainnet, Network::Testnet, Network::Regtest].iter() {
            let btc_net = network_to_btc_network(*network);
            let btc_addr = bitcoin::Address::p2pkh(&pk, btc_net);
            prop_assert_eq!(
                Ok(BitcoinAddress::P2pkh(tx::hash160(&pkbytes))),
                BitcoinAddress::parse(&btc_addr.to_string(), *network)
            );

            let btc_addr = bitcoin::Address::p2wpkh(&pk, btc_net).unwrap();
            prop_assert_eq!(
                Ok(BitcoinAddress::P2wpkhV0(pkhash)),
                BitcoinAddress::parse(&btc_addr.to_string(), *network)
            );
        }
    }

    #[test]
    fn btc_address_display_model(address in arb_address()) {
        for network in [Network::Mainnet, Network::Testnet].iter() {
            let addr_str = address.display(*network);
            let btc_addr = address_to_btc_address(&address, *network);
            prop_assert_eq!(btc_addr, bitcoin::Address::from_str(&addr_str).unwrap());
        }
    }

    #[test]
    fn address_roundtrip(address in arb_address()) {
        for network in [Network::Mainnet, Network::Testnet, Network::Regtest].iter() {
            let addr_str = address.display(*network);
            prop_assert_eq!(BitcoinAddress::parse(&addr_str, *network), Ok(address.clone()));
        }
    }

    #[test]
    fn sec1_to_der_positive_parses(sig in pvec(1u8..0x0f, 64)) {
        use simple_asn1::{from_der, ASN1Block::{Sequence, Integer}};

        let der = crate::signature::sec1_to_der(&sig);
        let decoded = from_der(&der).expect("failed to decode DER");
        if let[Sequence(_, items)] = &decoded[..] {
            if let [Integer(_, r), Integer(_, s)] = &items[..] {
                let (_, r_be) = r.to_bytes_be();
                let (_, s_be) = s.to_bytes_be();
                prop_assert_eq!(&r_be[..], &sig[..32]);
                prop_assert_eq!(&s_be[..], &sig[32..]);
                return Ok(());
            }
        }
        prop_assert!(false, "expected a DER sequence with two items, got: {:?}", decoded);
    }

    #[test]
    fn sec1_to_der_non_zero_parses(sig in pvec(any::<u8>(), 64)) {
        use simple_asn1::{from_der, ASN1Block::{Sequence, Integer}};

        prop_assume!(sig[..32].iter().any(|x| *x > 0));
        prop_assume!(sig[32..].iter().any(|x| *x > 0));

        let der = crate::signature::sec1_to_der(&sig);
        let decoded = from_der(&der).expect("failed to decode DER");

        if let[Sequence(_, items)] = &decoded[..] {
            if let [Integer(_, _r), Integer(_, _s)] = &items[..] {
                return Ok(());
            }
        }
        prop_assert!(false, "expected a DER sequence with two items, got: {:?}", decoded);
    }

    #[test]
    fn encode_valid_signatures(sig in pvec(any::<u8>(), 64)) {
        prop_assume!(sig[..32].iter().any(|x| *x > 0));
        prop_assume!(sig[32..].iter().any(|x| *x > 0));

        let encoded = crate::signature::EncodedSignature::from_sec1(&sig);
        crate::signature::validate_encoded_signature(encoded.as_slice()).expect("invalid signature");
    }

    #[test]
    fn test_fee_range(
        utxos in btree_set(arb_runes_utxo(5_000u128..1_000_000_000), 0..20),
        amount in option::of(any::<u128>()),
        fee_per_vbyte in 2000..10000u64,
    ) {
        const SMALLEST_TX_SIZE_VBYTES: u64 = 140; // one input, two outputs

        let estimate = estimate_fee(1, &utxos, amount, fee_per_vbyte);
        let lower_bound =  SMALLEST_TX_SIZE_VBYTES * fee_per_vbyte / 1000;
        let estimate_amount = estimate.bitcoin_fee;
        prop_assert!(
            estimate_amount >= lower_bound,
            "The fee estimate {} is below the lower bound {}",
            estimate_amount,
            lower_bound
        );
    }
}
