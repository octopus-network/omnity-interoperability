use crate::address::{main_bitcoin_address, main_destination, BitcoinAddress};
use crate::logs::{P0, P1};
use crate::queries::RedeemFee;
use crate::runestone::{Edict, Runestone};
use crate::state::{audit, mutate_state, BtcChangeOutput};
use candid::{CandidType, Deserialize};
use destination::Destination;
use ic_btc_interface::{MillisatoshiPerByte, Network, OutPoint, Txid, Utxo};
use ic_canister_log::log;
use ic_ic00_types::DerivationPath;
use num_traits::SaturatingSub;
use omnity_types::rune_id::RuneId;
use omnity_types::{ChainState, Directive, TokenId, TxAction};
use scopeguard::{guard, ScopeGuard};
use serde::Serialize;
use serde_bytes::ByteBuf;
use state::{
    read_state, GenTicketRequestV2, RuneTxRequest, RunesBalance, RunesChangeOutput, RunesUtxo,
    SubmittedBtcTransactionV2, BTC_TOKEN,
};
use std::collections::{BTreeMap, BTreeSet};
use std::iter::Sum;
use std::str::FromStr;
use std::time::Duration;
use updates::rune_tx::{generate_rune_tx_request, GenRuneTxReqError, RuneTxArgs};

pub mod address;
pub mod call_error;
pub mod destination;
pub mod guard;
pub mod hub;
pub mod lifecycle;
pub mod logs;
pub mod management;
pub mod metrics;
pub mod queries;
pub mod runestone;
pub mod signature;
pub mod state;
pub mod storage;
pub mod tx;
pub mod updates;

#[cfg(test)]
mod tests;

/// Time constants
const SEC_NANOS: u64 = 1_000_000_000;
const MIN_NANOS: u64 = 60 * SEC_NANOS;
/// The minimum number of pending request in the queue before we try to make
/// a batch transaction.
pub const MIN_PENDING_REQUESTS: usize = 5;
pub const MAX_REQUESTS_PER_BATCH: usize = 10;
pub const BATCH_QUERY_LIMIT: u64 = 20;

pub const INTERVAL_PROCESSING: Duration = Duration::from_secs(5);
pub const INTERVAL_QUERY_DIRECTIVES: Duration = Duration::from_secs(60);
pub const FEE_ESTIMATE_DELAY: Duration = Duration::from_secs(60 * 60);

/// The minimum fee increment for transaction resubmission.
/// See https://en.bitcoin.it/wiki/Miner_fees#Relaying for more detail.
pub const MIN_RELAY_FEE_PER_VBYTE: MillisatoshiPerByte = 1_000;

/// The minimum time the customs should wait before replacing a stuck transaction.
pub const MIN_RESUBMISSION_DELAY: Duration = Duration::from_secs(24 * 60 * 60);

/// The threshold for the number of UTXOs under management before
/// trying to match the number of outputs with the number of inputs
/// when building transactions.
pub const UTXOS_COUNT_THRESHOLD: usize = 1_000;

#[derive(Clone, serde::Serialize, Deserialize, Debug)]
pub enum Priority {
    P0,
    P1,
}

#[derive(Clone, serde::Serialize, Deserialize, Debug)]
pub struct LogEntry {
    pub timestamp: u64,
    pub priority: Priority,
    pub file: String,
    pub line: u32,
    pub message: String,
    pub counter: u64,
}

#[derive(Clone, Default, serde::Serialize, Deserialize, Debug)]
pub struct Log {
    pub entries: Vec<LogEntry>,
}

#[derive(CandidType, Debug, Deserialize, Serialize)]
pub struct CustomsInfo {
    pub min_confirmations: u32,
    pub chain_state: ChainState,
}

#[derive(CandidType, Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct ECDSAPublicKey {
    pub public_key: Vec<u8>,
    pub chain_code: Vec<u8>,
}

#[derive(CandidType, Clone, Debug, Deserialize, Serialize)]
pub struct TokenResp {
    pub token_id: TokenId,
    pub symbol: String,
    pub decimals: u8,
    pub icon: Option<String>,
    pub rune_id: String,
}

struct SignTxRequest {
    network: Network,
    unsigned_tx: tx::UnsignedTransaction,
    runes_change_output: RunesChangeOutput,
    btc_change_output: BtcChangeOutput,
    outpoint_destination: BTreeMap<OutPoint, Destination>,
    /// The original requests that we keep around to place back to the queue
    /// if the signature fails.
    requests: Vec<state::RuneTxRequest>,
    /// The list of Runes UTXOs we use as transaction inputs.
    runes_utxos: Vec<RunesUtxo>,
    /// The list of BTC UTXOs we use as transaction inputs.
    btc_utxos: Vec<Utxo>,
}

/// Undoes changes we make to the customs state when we construct a pending transaction.
/// We call this function if we fail to sign or send a Bitcoin transaction.
fn undo_sign_request(
    requests: Vec<state::RuneTxRequest>,
    runes_utxos: Vec<RunesUtxo>,
    btc_utxos: Vec<Utxo>,
) {
    state::mutate_state(|s| {
        for utxo in runes_utxos {
            assert!(s.available_runes_utxos.insert(utxo));
        }
        for utxo in btc_utxos {
            assert!(s.available_fee_utxos.insert(utxo));
        }
        // Insert requests in reverse order so that they are still sorted.
        s.push_from_in_flight_to_pending_requests(requests);
    })
}

/// Updates the UTXOs for the main account of the custom to pick up change from
/// previous redeem token requests.
async fn fetch_main_utxos(
    addresses: Vec<(Destination, BitcoinAddress)>,
    btc_network: Network,
    min_confirmations: u32,
) -> BTreeMap<Destination, Vec<Utxo>> {
    let mut result = BTreeMap::default();
    for (main_dest, main_address) in addresses {
        let utxos = match management::get_utxos(
            btc_network,
            &main_address.display(btc_network),
            min_confirmations,
            management::CallSource::Custom,
        )
        .await
        {
            Ok(response) => response.utxos,
            Err(e) => {
                log!(
                    P0,
                    "[fetch_main_utxos]: failed to fetch UTXOs for the main address {}: {}",
                    main_address.display(btc_network),
                    e
                );
                continue;
            }
        };

        result.insert(
            main_dest.clone(),
            state::read_state(|s| match s.utxos_state_destinations.get(&main_dest) {
                Some(known_utxos) => utxos
                    .into_iter()
                    .filter(|u| !known_utxos.contains(u))
                    .collect(),
                None => utxos,
            }),
        );
    }
    result
}

/// Returns an estimate for transaction fees in millisatoshi per vbyte. Returns
/// None if the bitcoin canister is unavailable or does not have enough data for
/// an estimate yet.
pub async fn estimate_fee_per_vbyte() -> Option<MillisatoshiPerByte> {
    /// The default fee we use on regtest networks if there are not enough data
    /// to compute the median fee.
    const DEFAULT_FEE: MillisatoshiPerByte = 5_000;

    let btc_network = state::read_state(|s| s.btc_network);
    match management::get_current_fees(btc_network).await {
        Ok(fees) => {
            if btc_network == Network::Regtest {
                return Some(DEFAULT_FEE);
            }
            if fees.len() >= 100 {
                state::mutate_state(|s| {
                    s.last_fee_per_vbyte = fees.clone();
                });
                Some(fees[50])
            } else {
                log!(
                    P0,
                    "[estimate_fee_per_vbyte]: not enough data points ({}) to compute the fee",
                    fees.len()
                );
                None
            }
        }
        Err(err) => {
            log!(
                P0,
                "[estimate_fee_per_vbyte]: failed to get median fee per vbyte: {}",
                err
            );
            None
        }
    }
}

async fn process_tickets() {
    if read_state(|s| s.chain_state == ChainState::Deactive) {
        return;
    }

    let (hub_principal, offset) = read_state(|s| (s.hub_principal, s.next_ticket_seq));
    match hub::query_tickets(hub_principal, offset, BATCH_QUERY_LIMIT).await {
        Err(err) => {
            log!(
                P0,
                "[submit_release_token_requests] temporarily unavailable: {}",
                err
            );
        }
        Ok(tickets) => {
            let mut next_seq = offset;
            for (seq, ticket) in tickets {
                let amount = if let Ok(amount) = u128::from_str_radix(ticket.amount.as_str(), 10) {
                    amount
                } else {
                    // Shouldn't happen, the hub must ensure the correctness of the data.
                    log!(
                        P0,
                        "[submit_release_token_requests]: failed to parse amount of ticket"
                    );
                    next_seq = seq + 1;
                    continue;
                };

                let args = RuneTxArgs {
                    ticket_id: ticket.ticket_id,
                    token_id: ticket.token,
                    src_chain: ticket.src_chain,
                    action: ticket.action,
                    amount,
                    receiver: ticket.receiver,
                };
                match generate_rune_tx_request(args).await {
                    Err(GenRuneTxReqError::AlreadyProcessing)
                    | Err(GenRuneTxReqError::AlreadyProcessed)
                    | Ok(_) => {}
                    Err(GenRuneTxReqError::TemporarilyUnavailable(_)) => {
                        log!(
                            P0,
                            "[submit_release_token_requests] temporarily unavailable"
                        );
                        break;
                    }
                    Err(err) => {
                        log!(P0, "[submit_release_token_requests] err: {:?}", err);
                    }
                }
                next_seq = seq + 1;
            }
            mutate_state(|s| audit::update_next_ticket_seq(s, next_seq));
        }
    }
}

async fn process_directive() {
    let (hub_principal, offset) = read_state(|s| (s.hub_principal, s.next_directive_seq));
    match hub::query_directives(hub_principal, offset, BATCH_QUERY_LIMIT).await {
        Err(err) => {
            log!(P0, "[process_directive] temporarily unavailable: {}", err);
        }
        Ok(directives) => mutate_state(|s| {
            for (_, directive) in &directives {
                match directive {
                    Directive::AddChain(chain) | Directive::UpdateChain(chain) => {
                        audit::add_chain(s, chain.clone())
                    }
                    Directive::AddToken(token) | Directive::UpdateToken(token) => {
                        if let Some(rune_id) = token.metadata.clone().get("rune_id") {
                            match RuneId::from_str(rune_id) {
                                Err(err) => {
                                    log!(
                                        P0,
                                        "[process_directive] failed to parse rune id: {}",
                                        err
                                    );
                                }
                                Ok(rune_id) => audit::add_token(s, rune_id, token.clone()),
                            }
                        } else {
                            log!(
                                P0,
                                "[process_directive] token {} not found rune_id in metadata",
                                token.token_id
                            );
                        }
                    }
                    Directive::ToggleChainState(toggle) => {
                        audit::toggle_chain_state(s, toggle.clone())
                    }
                    Directive::UpdateFee(_) => {}
                }
            }
            let next_seq = directives.last().map_or(offset, |(seq, _)| seq + 1);
            audit::update_next_directive_seq(s, next_seq);
        }),
    }
}

/// Constructs and sends out signed bitcoin transactions for pending retrieve
/// requests.
async fn submit_rune_txs() {
    let fee_millisatoshi_per_vbyte = match estimate_fee_per_vbyte().await {
        Some(fee) => fee,
        None => return,
    };

    let runes_list = read_state(|s| {
        s.pending_rune_tx_requests
            .iter()
            .map(|(rune_id, _)| rune_id.clone())
            .collect::<Vec<RuneId>>()
    });
    for rune_id in runes_list {
        // We make requests if we have old requests in the queue or if have enough
        // requests to fill a batch.
        if !state::read_state(|s| {
            s.can_form_a_batch(rune_id, MIN_PENDING_REQUESTS, ic_cdk::api::time())
        }) {
            continue;
        }

        let main_chain_id = read_state(|s| s.chain_id.clone());
        let ecdsa_public_key = updates::get_btc_address::init_ecdsa_public_key().await;
        let btc_main_address = address::main_bitcoin_address(
            &ecdsa_public_key,
            main_chain_id.clone(),
            String::from(BTC_TOKEN),
        );

        // Each runes tokens use isolated main addresses
        let runes_main_address =
            address::main_bitcoin_address(&ecdsa_public_key, main_chain_id, rune_id.to_string());

        let maybe_sign_request = state::mutate_state(|s| {
            let batch = s.build_batch(rune_id, MAX_REQUESTS_PER_BATCH);

            if batch.is_empty() {
                return None;
            }

            match build_unsigned_transaction(
                rune_id,
                &mut s.available_runes_utxos,
                &mut s.available_fee_utxos,
                runes_main_address,
                btc_main_address,
                new_build_tx_req(batch.clone()),
                fee_millisatoshi_per_vbyte,
                false,
            ) {
                Ok((
                    unsigned_tx,
                    runes_change_output,
                    btc_change_output,
                    runes_utxos,
                    btc_utxos,
                )) => {
                    for req in batch.iter() {
                        s.push_in_flight_request(
                            req.ticket_id.clone(),
                            state::InFlightStatus::Signing,
                        );
                    }

                    Some(SignTxRequest {
                        runes_change_output,
                        btc_change_output,
                        outpoint_destination: filter_output_destinations(s, &unsigned_tx),
                        network: s.btc_network,
                        unsigned_tx,
                        requests: batch,
                        runes_utxos,
                        btc_utxos,
                    })
                }
                Err(err) => {
                    log!(P0,
                        "[submit_pending_requests]: {:?} to unsigned transaction for requests at ticket ids [{}]",
                        err,
                        batch.iter().map(|req| req.ticket_id.clone()).collect::<Vec<_>>().join(",")
                    );

                    s.push_from_in_flight_to_pending_requests(batch);
                    None
                }
            }
        });

        if let Some(req) = maybe_sign_request {
            log!(
                P1,
                "[submit_pending_requests]: signing a new transaction: {}",
                hex::encode(tx::encode_into(&req.unsigned_tx, Vec::new()))
            );

            // This guard ensures that we return pending requests and UTXOs back to
            // the state if the signing or sending a transaction fails or panics.
            let requests_guard = guard(
                (req.requests, req.runes_utxos, req.btc_utxos),
                |(reqs, runes_utxos, btc_utxos)| {
                    undo_sign_request(reqs, runes_utxos, btc_utxos);
                },
            );

            let txid = req.unsigned_tx.txid();

            match sign_transaction(&req.outpoint_destination, req.unsigned_tx).await {
                Ok(signed_tx) => {
                    state::mutate_state(|s| {
                        for release_req in requests_guard.0.iter() {
                            s.push_in_flight_request(
                                release_req.ticket_id.clone(),
                                state::InFlightStatus::Sending { txid },
                            );
                        }
                    });

                    log!(
                        P0,
                        "[submit_pending_requests]: sending a signed transaction {}",
                        hex::encode(tx::encode_into(&signed_tx, Vec::new()))
                    );
                    match management::send_transaction(&signed_tx, req.network).await {
                        Ok(()) => {
                            log!(
                                P1,
                                "[submit_pending_requests]: successfully sent transaction {}",
                                &txid,
                            );

                            // Defuse the guard because we sent the transaction
                            // successfully.
                            let (requests, runes_utxos, btc_utxos) =
                                ScopeGuard::into_inner(requests_guard);

                            state::mutate_state(|s| {
                                state::audit::sent_transaction(
                                    s,
                                    SubmittedBtcTransactionV2 {
                                        rune_id: rune_id.clone(),
                                        requests,
                                        txid,
                                        runes_utxos,
                                        btc_utxos,
                                        runes_change_output: req.runes_change_output,
                                        btc_change_output: req.btc_change_output,
                                        submitted_at: ic_cdk::api::time(),
                                        fee_per_vbyte: Some(fee_millisatoshi_per_vbyte),
                                    },
                                );
                            });
                        }
                        Err(err) => {
                            log!(
                                P0,
                                "[submit_pending_requests]: failed to send a bitcoin transaction: {}",
                                err
                            );
                        }
                    }
                }
                Err(err) => {
                    log!(
                        P0,
                        "[submit_pending_requests]: failed to sign a BTC transaction: {}",
                        err
                    );
                }
            }
        }
    }
}

fn finalization_time_estimate(min_confirmations: u32, network: Network) -> Duration {
    Duration::from_nanos(
        min_confirmations as u64
            * match network {
                Network::Mainnet => 10 * MIN_NANOS,
                Network::Testnet => MIN_NANOS,
                Network::Regtest => SEC_NANOS,
            },
    )
}

/// Returns finalized transactions from the list of `candidates` according to the
/// list of newly received UTXOs for the main customs account.
fn finalized_txs(
    candidates: &[state::SubmittedBtcTransactionV2],
    new_utxos: &[Utxo],
) -> Vec<state::SubmittedBtcTransactionV2> {
    candidates
        .iter()
        .filter_map(|tx| {
            new_utxos
                .iter()
                .any(|utxo| {
                    utxo.outpoint.vout == tx.btc_change_output.vout && utxo.outpoint.txid == tx.txid
                })
                .then_some(tx.clone())
        })
        .collect()
}

async fn finalize_rune_txs() {
    if state::read_state(|s| s.submitted_transactions.is_empty()) {
        return;
    }

    let ecdsa_public_key = updates::get_btc_address::init_ecdsa_public_key().await;
    let now = ic_cdk::api::time();

    // The list of transactions that are likely to be finalized, indexed by the transaction id.
    let mut maybe_finalized_transactions: BTreeMap<Txid, state::SubmittedBtcTransactionV2> =
        state::read_state(|s| {
            let wait_time = finalization_time_estimate(s.min_confirmations, s.btc_network);
            s.submitted_transactions
                .iter()
                .filter(|&req| (req.submitted_at + (wait_time.as_nanos() as u64) < now))
                .map(|req| (req.txid, req.clone()))
                .collect()
        });

    if maybe_finalized_transactions.is_empty() {
        return;
    }

    let main_chain_id = read_state(|s| s.chain_id.clone());
    let main_btc_address = (
        main_destination(main_chain_id.clone(), BTC_TOKEN.into()),
        address::main_bitcoin_address(&ecdsa_public_key, main_chain_id.clone(), BTC_TOKEN.into()),
    );

    let main_runes_addresses: Vec<(Destination, BitcoinAddress)> = maybe_finalized_transactions
        .iter()
        .map(|(_, tx)| {
            (
                main_destination(main_chain_id.clone(), tx.rune_id.to_string()),
                address::main_bitcoin_address(
                    &ecdsa_public_key,
                    main_chain_id.clone(),
                    tx.rune_id.to_string(),
                ),
            )
        })
        .collect();

    let (btc_network, min_confirmations) =
        state::read_state(|s| (s.btc_network, s.min_confirmations));

    let dest_btc_utxos =
        fetch_main_utxos(vec![main_btc_address], btc_network, min_confirmations).await;
    let dest_runes_utxos =
        fetch_main_utxos(main_runes_addresses.clone(), btc_network, min_confirmations).await;

    let new_btc_utxos = dest_btc_utxos
        .iter()
        .map(|(_, utxos)| utxos)
        .flatten()
        .map(|u| u.clone())
        .collect::<Vec<Utxo>>();

    // Transactions whose change outpoint is present in the newly fetched UTXOs
    // can be finalized. Note that all new customs transactions must have a
    // change output because customs always charges a fee for converting tokens.
    let confirmed_transactions: Vec<_> =
        state::read_state(|s| finalized_txs(&s.submitted_transactions, &new_btc_utxos));

    // It's possible that some transactions we considered lost or rejected became finalized in the
    // meantime. If that happens, we should stop waiting for replacement transactions to finalize.
    let unstuck_transactions: Vec<_> =
        state::read_state(|s| finalized_txs(&s.stuck_transactions, &new_btc_utxos));

    state::mutate_state(|s| {
        for (dest, utxos) in dest_btc_utxos {
            audit::add_utxos(s, dest, utxos, false);
        }
        for (dest, utxos) in dest_runes_utxos {
            audit::add_utxos(s, dest, utxos, true);
        }
        for tx in &confirmed_transactions {
            state::audit::confirm_transaction(s, &tx.txid);
            if tx.runes_change_output.value > 0 {
                let balance = RunesBalance {
                    rune_id: tx.runes_change_output.rune_id.clone(),
                    vout: tx.runes_change_output.vout,
                    amount: tx.runes_change_output.value,
                };
                audit::update_runes_balance(s, tx.txid, balance);
            }
            maybe_finalized_transactions.remove(&tx.txid);
        }
    });

    for tx in &confirmed_transactions {
        update_tx_hash_to_hub(tx).await;
    }

    for tx in &unstuck_transactions {
        update_tx_hash_to_hub(tx).await;
        state::read_state(|s| {
            if let Some(replacement_txid) = s.find_last_replacement_tx(&tx.txid) {
                maybe_finalized_transactions.remove(replacement_txid);
            }
        });
    }

    state::mutate_state(|s| {
        for tx in unstuck_transactions {
            log!(
                P0,
                "[finalize_requests]: finalized transaction {} assumed to be stuck",
                &tx.txid
            );
            state::audit::confirm_transaction(s, &tx.txid);
            // The change value of mint rune tx is 0.
            if tx.runes_change_output.value > 0 {
                let balance = RunesBalance {
                    rune_id: tx.runes_change_output.rune_id.clone(),
                    vout: tx.runes_change_output.vout,
                    amount: tx.runes_change_output.value,
                };
                audit::update_runes_balance(s, tx.txid, balance);
            }
        }
    });

    // Do not replace transactions if less than MIN_RESUBMISSION_DELAY passed since their
    // submission. This strategy works around short-term fee spikes.
    maybe_finalized_transactions
        .retain(|_txid, tx| tx.submitted_at + MIN_RESUBMISSION_DELAY.as_nanos() as u64 <= now);

    if maybe_finalized_transactions.is_empty() {
        // There are no transactions eligible for replacement.
        return;
    }

    // There are transactions that should have been finalized by now. Let's check whether the
    // Bitcoin network knows about them or they got lost in the meantime. Note that the Bitcoin
    // canister doesn't have access to the mempool, we can detect only transactions with at least
    // one confirmation.
    let main_utxos_zero_confirmations =
        fetch_main_utxos(main_runes_addresses, btc_network, 0).await;

    for (_, utxos) in main_utxos_zero_confirmations {
        for utxo in utxos {
            // This transaction got at least one confirmation, we don't need to replace it.
            maybe_finalized_transactions.remove(&utxo.outpoint.txid);
        }
    }

    if maybe_finalized_transactions.is_empty() {
        // All transactions we assumed to be stuck have at least one confirmation.
        // We shall finalize these transaction later.
        return;
    }

    // Found transactions that appear to be stuck: they might be sitting in the mempool, got
    // evicted from the mempool, or never reached it due to a temporary issue in the Bitcoin
    // integration.
    //
    // Let's resubmit these transactions.
    log!(
        P0,
        "[finalize_requests]: found {} stuck transactions: {}",
        maybe_finalized_transactions.len(),
        maybe_finalized_transactions
            .keys()
            .map(|txid| txid.to_string())
            .collect::<Vec<_>>()
            .join(","),
    );

    // We shall use the latest fee estimate for replacement transactions.
    let fee_per_vbyte = match estimate_fee_per_vbyte().await {
        Some(fee) => fee,
        None => return,
    };

    for (old_txid, submitted_tx) in maybe_finalized_transactions {
        let mut runes_utxos: BTreeSet<_> = submitted_tx.runes_utxos.iter().cloned().collect();
        let mut btc_utxos: BTreeSet<_> = submitted_tx.btc_utxos.iter().cloned().collect();

        let tx_fee_per_vbyte = match submitted_tx.fee_per_vbyte {
            Some(prev_fee) => {
                // Ensure that the fee is at least min relay fee higher than the previous
                // transaction fee to comply with BIP-125 (https://en.bitcoin.it/wiki/BIP_0125).
                fee_per_vbyte.max(prev_fee + MIN_RELAY_FEE_PER_VBYTE)
            }
            None => fee_per_vbyte,
        };

        let build_tx_req = new_build_tx_req(submitted_tx.requests.clone());

        let (unsigned_tx, runes_change, btc_change, used_runes_utxos, used_btc_utxos) =
            match build_unsigned_transaction(
                submitted_tx.rune_id,
                &mut runes_utxos,
                &mut btc_utxos,
                main_bitcoin_address(
                    &ecdsa_public_key,
                    main_chain_id.clone(),
                    submitted_tx.rune_id.to_string(),
                ),
                main_bitcoin_address(
                    &ecdsa_public_key,
                    main_chain_id.clone(),
                    String::from(BTC_TOKEN),
                ),
                build_tx_req,
                tx_fee_per_vbyte,
                true,
            ) {
                Ok(tx) => tx,
                // If it's impossible to build a new transaction, the fees probably became too high.
                // Let's ignore this transaction and wait for fees to go down.
                Err(err) => {
                    log!(
                        P1,
                        "[finalize_requests]: failed to rebuild stuck transaction {}: {:?}",
                        &submitted_tx.txid,
                        err
                    );
                    continue;
                }
            };

        let outpoint_dests = state::read_state(|s| filter_output_destinations(s, &unsigned_tx));

        assert!(
            runes_utxos.is_empty(),
            "build_unsigned_transaction didn't use all inputs"
        );
        assert_eq!(used_runes_utxos.len(), submitted_tx.runes_utxos.len());
        assert_eq!(used_btc_utxos.len(), submitted_tx.btc_utxos.len());

        let new_txid = unsigned_tx.txid();

        let maybe_signed_tx = sign_transaction(&outpoint_dests, unsigned_tx).await;

        let signed_tx = match maybe_signed_tx {
            Ok(tx) => tx,
            Err(err) => {
                log!(
                    P0,
                    "[finalize_requests]: failed to sign a BTC transaction: {}",
                    err
                );
                continue;
            }
        };

        match management::send_transaction(&signed_tx, btc_network).await {
            Ok(()) => {
                if old_txid == new_txid {
                    // DEFENSIVE: We should never take this branch because we increase fees for
                    // replacement transactions with each resubmission. However, since replacing a
                    // transaction with itself is not allowed, we still handle the transaction
                    // equality in case the fee computation rules change in the future.
                    log!(P0,
                        "[finalize_requests]: resent transaction {} with a new signature. TX bytes: {}",
                        &new_txid,
                        hex::encode(tx::encode_into(&signed_tx, Vec::new()))
                    );
                    continue;
                }
                log!(P0,
                    "[finalize_requests]: sent transaction {} to replace stuck transaction {}. TX bytes: {}",
                    &new_txid,
                    &old_txid,
                    hex::encode(tx::encode_into(&signed_tx, Vec::new()))
                );
                let new_tx = SubmittedBtcTransactionV2 {
                    rune_id: submitted_tx.rune_id,
                    requests: submitted_tx.requests,
                    runes_utxos: used_runes_utxos,
                    btc_utxos: used_btc_utxos,
                    txid: new_txid,
                    submitted_at: ic_cdk::api::time(),
                    runes_change_output: runes_change,
                    btc_change_output: btc_change,
                    fee_per_vbyte: Some(tx_fee_per_vbyte),
                };

                state::mutate_state(|s| {
                    state::audit::replace_transaction(s, old_txid, new_tx);
                });
            }
            Err(err) => {
                log!(P0, "[finalize_requests]: failed to send transaction bytes {} to replace stuck transaction {}: {}",
                    hex::encode(tx::encode_into(&signed_tx, Vec::new())),
                    &old_txid,
                    err,
                );
                continue;
            }
        }
    }
}

async fn finalize_gen_ticket_txs() {
    if state::read_state(|s| s.init_gen_ticket_requests.is_empty()) {
        return;
    }

    let now = ic_cdk::api::time();
    let (network, min_confirmations) = read_state(|s| (s.btc_network, s.min_confirmations));

    let maybe_finalized_transactions: Vec<GenTicketRequestV2> = state::read_state(|s| {
        let wait_time = finalization_time_estimate(min_confirmations, network);
        s.init_gen_ticket_requests
            .iter()
            .filter(|&(_, req)| (req.received_at + (wait_time.as_nanos() as u64) < now))
            .map(|(_, req)| req.clone())
            .collect()
    });

    for req in maybe_finalized_transactions {
        if management::get_utxos(
            network,
            &req.address,
            min_confirmations,
            management::CallSource::Custom,
        )
        .await
        .map_or_else(
            |err| {
                log!(
                    P0,
                    "failed to call get_utxos in finalize_gen_ticket_txs: {}",
                    err
                );
                return false;
            },
            |resp| {
                // Just include any utxo
                resp.utxos.contains(&req.new_utxos[0])
            },
        ) {
            mutate_state(|s| audit::confirm_generate_ticket_request(s, req.txid));
        }
    }
}

async fn update_tx_hash_to_hub(tx: &SubmittedBtcTransactionV2) {
    let hub_principal = read_state(|s| s.hub_principal);
    let ticket_ids = tx.requests.iter().map(|r| r.ticket_id.clone()).collect();
    if let Err(err) =
        hub::batch_update_tx_hash(hub_principal, ticket_ids, tx.txid.to_string()).await
    {
        log!(P0, "failed to update tx hash to hub: {}", err)
    }
}

fn new_build_tx_req(requests: Vec<RuneTxRequest>) -> BuildTxReq {
    if requests[0].action == TxAction::Mint {
        BuildTxReq::MintTxReq(requests[0].address.clone())
    } else {
        let outputs: Vec<_> = requests
            .iter()
            .map(|req| (req.address.clone(), req.amount))
            .collect();

        BuildTxReq::EdictTxReq(outputs)
    }
}

/// Builds the minimal OutPoint -> Account map required to sign a transaction.
fn filter_output_destinations(
    state: &state::CustomsState,
    unsigned_tx: &tx::UnsignedTransaction,
) -> BTreeMap<OutPoint, Destination> {
    unsigned_tx
        .inputs
        .iter()
        .map(|input| {
            (
                input.previous_output.clone(),
                state
                    .outpoint_destination
                    .get(&input.previous_output)
                    .unwrap_or_else(|| {
                        panic!(
                            "bug: missing account for output point {:?}",
                            input.previous_output
                        )
                    })
                    .clone(),
            )
        })
        .collect()
}

/// The algorithm greedily selects the smallest UTXO(s) with a value that is at least the given `target` in a first step.
///
/// If the customs manages more than [UTXOS_COUNT_THRESHOLD], it will then try to match the number of inputs with the
/// number of outputs + 2 (where the two additional outputs corresponds to the change output).
///
/// If there are no UTXOs matching the criteria, returns an empty vector.
///
/// PROPERTY: sum(u.value for u in available_set) ≥ target ⇒ !solution.is_empty()
/// POSTCONDITION: !solution.is_empty() ⇒ sum(u.value for u in solution) ≥ target
/// POSTCONDITION:  solution.is_empty() ⇒ available_utxos did not change.
fn utxos_selection<T, F, U>(
    target: T,
    available_utxos: &mut BTreeSet<U>,
    output_count: usize,
    get_value: F,
) -> Vec<U>
where
    F: Fn(&U) -> T + Copy,
    U: Ord + Clone,
    T: Copy + Default + Ord + PartialOrd + SaturatingSub + Sum,
{
    let mut input_utxos = greedy(target, available_utxos, get_value);

    if input_utxos.is_empty() {
        return vec![];
    }

    if available_utxos.len() > UTXOS_COUNT_THRESHOLD {
        while input_utxos.len() < output_count + 2 {
            if let Some(min_utxo) = available_utxos.iter().min_by_key(|u| get_value(u)) {
                input_utxos.push(min_utxo.clone());
                assert!(available_utxos.remove(&min_utxo.clone()));
            } else {
                break;
            }
        }
    }

    input_utxos
}

/// Selects a subset of UTXOs with the specified total target value and removes
/// the selected UTXOs from the available set.
///
/// If there are no UTXOs matching the criteria, returns an empty vector.
///
/// PROPERTY: sum(u.value for u in available_set) ≥ target ⇒ !solution.is_empty()
/// POSTCONDITION: !solution.is_empty() ⇒ sum(u.value for u in solution) ≥ target
/// POSTCONDITION:  solution.is_empty() ⇒ available_utxos did not change.
fn greedy<T, F, U>(target: T, available_utxos: &mut BTreeSet<U>, get_value: F) -> Vec<U>
where
    F: Fn(&U) -> T,
    U: Ord + Clone,
    T: Copy + Default + Ord + PartialOrd + SaturatingSub + Sum,
{
    let mut solution = vec![];
    let mut goal = target;
    while goal > T::default() {
        let utxo = match available_utxos.iter().max_by_key(|u| get_value(u)) {
            Some(max_utxo) if get_value(max_utxo) < goal => max_utxo.clone(),
            Some(_) => available_utxos
                .iter()
                .filter(|u| get_value(u) >= goal)
                .min_by_key(|u| get_value(u))
                .cloned()
                .expect("bug: there must be at least one UTXO matching the criteria"),
            None => {
                // Not enough available UTXOs to satisfy the request.
                for u in solution {
                    available_utxos.insert(u);
                }
                return vec![];
            }
        };
        goal = goal.saturating_sub(&get_value(&utxo));
        assert!(available_utxos.remove(&utxo));
        solution.push(utxo);
    }

    debug_assert!(
        solution.is_empty() || solution.iter().map(|u| get_value(u)).sum::<T>() >= target
    );

    solution
}

pub fn fake_sign(unsigned_tx: &tx::UnsignedTransaction) -> tx::SignedTransaction {
    tx::SignedTransaction {
        inputs: unsigned_tx
            .inputs
            .iter()
            .map(|unsigned_input| tx::SignedInput {
                previous_output: unsigned_input.previous_output.clone(),
                sequence: unsigned_input.sequence,
                signature: signature::EncodedSignature::fake(),
                pubkey: ByteBuf::from(vec![0u8; tx::PUBKEY_LEN]),
            })
            .collect(),
        outputs: unsigned_tx.outputs.clone(),
        lock_time: unsigned_tx.lock_time,
    }
}

/// Gathers ECDSA signatures for all the inputs in the specified unsigned
/// transaction.
///
/// # Panics
///
/// This function panics if the `output_account` map does not have an entry for
/// at least one of the transaction previous output points.
pub async fn sign_transaction(
    output_destinations: &BTreeMap<tx::OutPoint, Destination>,
    unsigned_tx: tx::UnsignedTransaction,
) -> Result<tx::SignedTransaction, call_error::CallError> {
    use crate::address::{derivation_path, derive_public_key};

    let mut signed_inputs = Vec::with_capacity(unsigned_tx.inputs.len());
    let sighasher = tx::TxSigHasher::new(&unsigned_tx);
    for input in &unsigned_tx.inputs {
        let outpoint = &input.previous_output;

        let destination = output_destinations
            .get(outpoint)
            .unwrap_or_else(|| panic!("bug: no account for outpoint {:?}", outpoint));

        let (key_name, ecdsa_public_key) =
            read_state(|s| s.get_ecdsa_key(destination.token.clone()));

        let path = derivation_path(destination);
        let pubkey = ByteBuf::from(derive_public_key(&ecdsa_public_key, destination).public_key);
        let pkhash = tx::hash160(&pubkey);

        let sighash = sighasher.sighash(input, &pkhash);

        let sec1_signature =
            management::sign_with_ecdsa(key_name.clone(), DerivationPath::new(path), sighash)
                .await?;

        signed_inputs.push(tx::SignedInput {
            signature: signature::EncodedSignature::from_sec1(&sec1_signature),
            pubkey,
            previous_output: outpoint.clone(),
            sequence: input.sequence,
        });
    }
    Ok(tx::SignedTransaction {
        inputs: signed_inputs,
        outputs: unsigned_tx.outputs,
        lock_time: unsigned_tx.lock_time,
    })
}

#[derive(Debug, PartialEq, Eq)]
pub enum BuildTxError {
    /// The customs does not have enough UTXOs to make the transfer
    /// Try again later after pending transactions have settled.
    NotEnoughFunds,
    NotEnoughGas,
}

pub enum BuildTxReq {
    EdictTxReq(Vec<(BitcoinAddress, u128)>),
    MintTxReq(BitcoinAddress),
}

/// Builds a transaction that transfer runes token to the specified destination accounts
/// using the Runes and BTC UTXOs that the customs owns. The customs pay the fee.
///
/// Sends the change back to the specified customs main address.
///
/// # Arguments
///
/// * `available_runes_utxos` - The set of all Runes UTXOs customs owns
/// * `available_btc_utxos` - The set of all BTC UTXOs customs owns
/// * `outputs` - The destination BTC addresses and respective amounts.
/// * `runes_main_address` - The BTC address of the customs's main account do absorb the Runes change.
/// * `btc_main_address` - The BTC address of the customs's main account do absorb the BTC change.
/// * `fee_per_vbyte` - The current 50th percentile of BTC fees, in millisatoshi/byte
/// * `is_resubmission` - A flag indicating whether to resubmit
///
/// # Panics
///
/// This function panics if the `outputs` vector is empty as it indicates a bug
/// in the caller's code.
pub fn build_unsigned_transaction(
    rune_id: RuneId,
    available_runes_utxos: &mut BTreeSet<RunesUtxo>,
    available_btc_utxos: &mut BTreeSet<Utxo>,
    runes_main_address: BitcoinAddress,
    btc_main_address: BitcoinAddress,
    req: BuildTxReq,
    fee_per_vbyte: u64,
    is_resubmission: bool,
) -> Result<
    (
        tx::UnsignedTransaction,
        RunesChangeOutput,
        BtcChangeOutput,
        Vec<RunesUtxo>,
        Vec<Utxo>,
    ),
    BuildTxError,
> {
    /// Having a sequence number lower than (0xffffffff - 1) signals the use of replacement by fee.
    /// It allows us to increase the fee of a transaction already sent to the mempool.
    /// The rbf option is used in `resubmit release token`.
    /// https://github.com/bitcoin/bips/blob/master/bip-0125.mediawiki
    const SEQUENCE_RBF_ENABLED: u32 = 0xfffffffd;

    let (stone, rune_change_output, rune_change_address, rune_utxos, outputs) = match req {
        BuildTxReq::EdictTxReq(outputs) => {
            assert!(!outputs.is_empty());

            let amount = outputs.iter().map(|(_, amount)| amount).sum::<u128>();
            let rune_utxos = utxos_selection(amount, available_runes_utxos, outputs.len(), |u| {
                if u.runes.rune_id.eq(&rune_id) {
                    u.runes.amount
                } else {
                    0
                }
            });

            if rune_utxos.is_empty() {
                return Err(BuildTxError::NotEnoughFunds);
            }

            let inputs_value = rune_utxos.iter().map(|u| u.runes.amount).sum::<u128>();
            debug_assert!(inputs_value >= amount);

            let burn_amount = outputs
                .iter()
                .filter(|(address, _)| matches!(address, BitcoinAddress::OpReturn(_)))
                .map(|(_, amount)| amount)
                .sum::<u128>();

            let outputs = outputs
                .iter()
                .filter(|(address, _)| !matches!(address, BitcoinAddress::OpReturn(_)))
                .map(|(address, amount)| (address.clone(), *amount))
                .collect::<Vec<(BitcoinAddress, u128)>>();

            let mut edicts = vec![];
            if burn_amount > 0 {
                edicts.push(Edict {
                    id: rune_id.into(),
                    amount: burn_amount,
                    output: 0,
                });
            }

            edicts.append(
                &mut outputs
                    .iter()
                    .enumerate()
                    .map(|(idx, (_, amount))| Edict {
                        id: rune_id.into(),
                        amount: *amount,
                        output: (idx + 2) as u32,
                    })
                    .collect::<Vec<Edict>>(),
            );

            let stone = Runestone { edicts, mint: None };

            let rune_change_output = state::RunesChangeOutput {
                rune_id,
                vout: 1,
                value: inputs_value - amount,
            };
            (
                stone,
                rune_change_output,
                runes_main_address,
                rune_utxos,
                outputs,
            )
        }
        BuildTxReq::MintTxReq(receiver) => {
            let stone = Runestone {
                mint: Some(rune_id),
                edicts: vec![],
            };

            let rune_change_output = state::RunesChangeOutput {
                rune_id,
                vout: 1,
                // Rune change output as output of mint rune receiver.
                // After the mint transaction is finalized, the user needs to call generate_ticket,
                // and the corresponding rune utxo will be placed in customs at that time.
                value: 0,
            };
            (stone, rune_change_output, receiver, vec![], vec![])
        }
    };

    // This guard returns the selected UTXOs back to the available_utxos set if
    // we fail to build the transaction.
    let rune_utxos_guard = guard(rune_utxos, |utxos| {
        for utxo in utxos {
            available_runes_utxos.insert(utxo);
        }
    });

    const MIN_OUTPUT_AMOUNT: u64 = 546;

    let mut tx_outputs = vec![
        tx::TxOut {
            value: 0,
            address: BitcoinAddress::OpReturn(stone.encipher()),
        },
        // Runes token change
        tx::TxOut {
            value: MIN_OUTPUT_AMOUNT,
            address: rune_change_address,
        },
    ];

    tx_outputs.append(
        &mut outputs
            .iter()
            .map(|(address, _)| tx::TxOut {
                address: address.clone(),
                value: MIN_OUTPUT_AMOUNT,
            })
            .collect(),
    );

    let mut tx_inputs = rune_utxos_guard
        .iter()
        .map(|utxo| tx::UnsignedInput {
            previous_output: utxo.raw.outpoint.clone(),
            value: utxo.raw.value,
            sequence: SEQUENCE_RBF_ENABLED,
        })
        .collect::<Vec<tx::UnsignedInput>>();

    // Initially assume two additional input utxos as source of transaction fees,
    // and one additional output as btc change output.
    let tx_vsize = tx_vsize_estimate(
        (rune_utxos_guard.len() + 2) as u64,
        (tx_outputs.len() + 1) as u64,
    );
    let fee: u64 = (tx_vsize as u64 * fee_per_vbyte) / 1000;
    // Additional MIN_OUTPUT_AMOUNT are used as the value of the outputs(two change output + multiple dest runes outputs).
    let non_op_return_outputs_sz = (outputs.len() + 2) as u64;
    // Select twise the fee to handle resubmissions.
    let select_fee = fee * 2 + MIN_OUTPUT_AMOUNT * non_op_return_outputs_sz;

    let mut input_btc_amount = rune_utxos_guard
        .iter()
        .map(|input| input.raw.value)
        .sum::<u64>();

    let mut btc_utxos: Vec<Utxo> = vec![];

    let selected_btc_amount = if is_resubmission {
        btc_utxos = available_btc_utxos.iter().map(|u| u.clone()).collect();
        available_btc_utxos.clear();
        btc_utxos.iter().map(|u| u.value).sum::<u64>()
    } else if input_btc_amount < select_fee {
        let target_fee = select_fee - input_btc_amount;

        btc_utxos = greedy(target_fee, available_btc_utxos, |u| u.value);
        if btc_utxos.is_empty() {
            log!(
                P0,
                "[select_btc_utxos]: target fee required: {}: available fee: {}",
                target_fee,
                available_btc_utxos.iter().map(|u| u.value).sum::<u64>()
            );
            return Err(BuildTxError::NotEnoughGas);
        }

        let btc_amount = btc_utxos.iter().map(|u| u.value).sum::<u64>();
        assert!(btc_amount >= target_fee);

        btc_amount
    } else {
        0
    };

    let btc_utxos_guard = guard(btc_utxos, |utxos| {
        for utxo in utxos {
            available_btc_utxos.insert(utxo);
        }
    });

    input_btc_amount += selected_btc_amount;
    tx_inputs.append(
        &mut btc_utxos_guard
            .iter()
            .map(|u| tx::UnsignedInput {
                previous_output: u.outpoint.clone(),
                value: u.value,
                sequence: SEQUENCE_RBF_ENABLED,
            })
            .collect::<Vec<tx::UnsignedInput>>(),
    );

    tx_outputs.push(tx::TxOut {
        address: btc_main_address,
        value: 0,
    });

    let mut unsigned_tx = tx::UnsignedTransaction {
        inputs: tx_inputs,
        outputs: tx_outputs,
        lock_time: 0,
    };

    // We need to recaculate the fee when the number of inputs and outputs is finalized.
    let real_fee = fake_sign(&unsigned_tx).vsize() as u64 * fee_per_vbyte / 1000;
    let btc_consumed = real_fee + MIN_OUTPUT_AMOUNT * non_op_return_outputs_sz;
    if input_btc_amount < btc_consumed {
        log!(
            P0,
            "input btc amount: {} greater than btc consumed: {}",
            input_btc_amount,
            btc_consumed,
        );
        return Err(BuildTxError::NotEnoughGas);
    }

    let btc_change_amount = input_btc_amount - btc_consumed + MIN_OUTPUT_AMOUNT;
    unsigned_tx.outputs.iter_mut().last().unwrap().value = btc_change_amount;
    let btc_change_out = BtcChangeOutput {
        vout: unsigned_tx.outputs.len() as u32 - 1,
        value: btc_change_amount,
    };

    Ok((
        unsigned_tx,
        rune_change_output,
        btc_change_out,
        ScopeGuard::into_inner(rune_utxos_guard),
        ScopeGuard::into_inner(btc_utxos_guard),
    ))
}

pub fn process_tx_task() {
    ic_cdk::spawn(async {
        let _guard = match crate::guard::TimerLogicGuard::new() {
            Some(guard) => guard,
            None => return,
        };
        submit_rune_txs().await;
        finalize_rune_txs().await;
        finalize_gen_ticket_txs().await;
    });
}

pub fn process_directive_msg_task() {
    ic_cdk::spawn(async {
        let _guard = match crate::guard::ProcessDirectiveMsgGuard::new() {
            Some(guard) => guard,
            None => return,
        };
        process_directive().await;
    });
}

pub fn process_ticket_msg_task() {
    ic_cdk::spawn(async {
        let _guard = match crate::guard::ProcessTicketMsgGuard::new() {
            Some(guard) => guard,
            None => return,
        };
        process_tickets().await;
    });
}

pub fn refresh_fee_task() {
    ic_cdk::spawn(async {
        let _ = estimate_fee_per_vbyte().await;
    });
}

/// Computes an estimate for the size of transaction (in vbytes) with the given number of inputs and outputs.
pub fn tx_vsize_estimate(input_count: u64, output_count: u64) -> u64 {
    // See
    // https://github.com/bitcoin/bips/blob/master/bip-0141.mediawiki
    // for the transaction structure and
    // https://bitcoin.stackexchange.com/questions/92587/calculate-transaction-fee-for-external-addresses-which-doesnt-belong-to-my-loca/92600#92600
    // for transaction size estimate.
    const INPUT_SIZE_VBYTES: u64 = 68;
    const OUTPUT_SIZE_VBYTES: u64 = 31;
    const TX_OVERHEAD_VBYTES: u64 = 11;

    input_count * INPUT_SIZE_VBYTES + output_count * OUTPUT_SIZE_VBYTES + TX_OVERHEAD_VBYTES
}

/// Computes an estimate for the release_token fee.
///
/// Arguments:
///   * `available_utxos` - the list of UTXOs available to the customs.
///   * `maybe_amount` - the withdrawal amount.
///   * `median_fee_millisatoshi_per_vbyte` - the median network fee, in millisatoshi per vbyte.
pub fn estimate_fee(
    rune_id: RuneId,
    available_utxos: &BTreeSet<RunesUtxo>,
    maybe_amount: Option<u128>,
    median_fee_millisatoshi_per_vbyte: u64,
) -> RedeemFee {
    const DEFAULT_INPUT_COUNT: u64 = 2;
    // One output for the caller and two for the btc change & runes change.
    const DEFAULT_OUTPUT_COUNT: u64 = 3;
    let input_count = match maybe_amount {
        Some(amount) => {
            // We simulate the algorithm that selects UTXOs for the
            // specified amount. If the withdrawal rate is low, we
            // should get the exact number of inputs that the customs
            // will use.
            let mut utxos = available_utxos.clone();
            let selected_utxos = utxos_selection(amount, &mut utxos, 1, |u| {
                if u.runes.rune_id.eq(&rune_id) {
                    u.runes.amount
                } else {
                    0
                }
            });

            if !selected_utxos.is_empty() {
                selected_utxos.len() as u64
            } else {
                DEFAULT_INPUT_COUNT
            }
        }
        None => DEFAULT_INPUT_COUNT,
    };

    let vsize = tx_vsize_estimate(input_count, DEFAULT_OUTPUT_COUNT);
    let bitcoin_fee = vsize * median_fee_millisatoshi_per_vbyte / 1000;
    RedeemFee { bitcoin_fee }
}
