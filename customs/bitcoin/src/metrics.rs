use crate::state;
use std::cell::Cell;

thread_local! {
    pub static GET_UTXOS_CLIENT_CALLS: Cell<u64> = Cell::default();
    pub static GET_UTXOS_CUSTOM_CALLS: Cell<u64> = Cell::default();
}

pub fn encode_metrics(
    metrics: &mut ic_metrics_encoder::MetricsEncoder<Vec<u8>>,
) -> std::io::Result<()> {
    const WASM_PAGE_SIZE_IN_BYTES: f64 = 65536.0;

    metrics.encode_gauge(
        "bitcoin_customs_stable_memory_bytes",
        ic_cdk::api::stable::stable_size() as f64 * WASM_PAGE_SIZE_IN_BYTES,
        "Size of the stable memory allocated by this canister.",
    )?;

    let cycle_balance = ic_cdk::api::canister_balance128() as f64;

    metrics.encode_gauge(
        "bitcoin_customs_cycle_balance",
        cycle_balance,
        "Cycle balance on this canister.",
    )?;

    metrics
        .gauge_vec("cycle_balance", "Cycle balance on this canister.")?
        .value(&[("canister", "bitcoin_customs")], cycle_balance)?;

    metrics
        .gauge_vec(
            "bitcioin_customs_generate_ticket_request_count",
            "Total count of generate ticket requests, by status.",
        )?
        .value(
            &[("status", "pending")],
            state::read_state(|s| s.pending_gen_ticket_requests.len()) as f64,
        )?
        .value(
            &[("status", "finalized")],
            state::read_state(|s| s.finalized_gen_ticket_requests.len()) as f64,
        )?;

    metrics
        .gauge_vec(
            "bitcoin_customs_release_token_request_count",
            "Total count of incomplete release token requests, by status.",
        )?
        .value(
            &[("status", "pending")],
            state::read_state(|s| s.pending_release_token_requests.len()) as f64,
        )?
        .value(
            &[("status", "signing")],
            state::read_state(|s| {
                s.requests_in_flight
                    .values()
                    .filter(|v| matches!(v, state::InFlightStatus::Signing))
                    .count()
            }) as f64,
        )?
        .value(
            &[("status", "sending")],
            state::read_state(|s| {
                s.requests_in_flight
                    .values()
                    .filter(|v| matches!(*v, state::InFlightStatus::Sending { .. }))
                    .count()
            }) as f64,
        )?
        .value(
            &[("status", "submitted")],
            state::read_state(|s| {
                s.submitted_transactions
                    .iter()
                    .map(|tx| tx.requests.len())
                    .sum::<usize>()
            }) as f64,
        )?;

    metrics
        .gauge_vec(
            "bitcoin_customs_btc_transaction_count",
            "Total count of non-finalized btc transaction, by status.",
        )?
        .value(
            &[("status", "submitted")],
            state::read_state(|s| s.submitted_transactions.len() as f64),
        )?
        .value(
            &[("status", "stuck")],
            state::read_state(|s| s.stuck_transactions.len() as f64),
        )?;

    metrics.encode_gauge(
        "bitcoin_customs_longest_resubmission_chain_size",
        state::read_state(|s| s.longest_resubmission_chain_size() as f64),
        "The length of the longest active transaction resubmission chain.",
    )?;

    metrics.encode_gauge(
        "bitcoin_customs_stored_finalized_requests",
        state::read_state(|s| s.finalized_release_token_requests.len()) as f64,
        "Total number of finalized release_token requests the customs keeps in memory.",
    )?;

    metrics.encode_counter(
        "bitcoin_customs_finalized_requests",
        state::read_state(|s| s.finalized_requests_count) as f64,
        "Total number of finalized release_token requests.",
    )?;

    metrics.encode_gauge(
        "bitcoin_customs_min_confirmations",
        state::read_state(|s| s.min_confirmations) as f64,
        "Min number of confirmations on BTC network",
    )?;

    metrics.encode_gauge(
        "bitcoin_customs_runes_utxos_available",
        state::read_state(|s| s.available_runes_utxos.len()) as f64,
        "Total number of Runes UTXOs the customs can use for release_token requests.",
    )?;

    metrics.encode_gauge(
        "bitcoin_customs_btc_utxos_available",
        state::read_state(|s| s.available_fee_utxos.len()) as f64,
        "Total number of BTC UTXOs the customs can use for release_token requests.",
    )?;

    metrics
        .counter_vec(
            "bitcoin_customs_get_utxos_calls",
            "Number of get_utxos calls the customs issued, labeled by source.",
        )?
        .value(
            &[("source", "client")],
            GET_UTXOS_CLIENT_CALLS.with(|cell| cell.get()) as f64,
        )?
        .value(
            &[("source", "customs")],
            GET_UTXOS_CUSTOM_CALLS.with(|cell| cell.get()) as f64,
        )?;

    metrics.encode_gauge(
        "bitcoin_customs_managed_addresses_count",
        state::read_state(|s| s.utxos_state_destinations.len()) as f64,
        "Total number of customs addresses owning UTXOs.",
    )?;

    metrics.encode_gauge(
        "bitcoin_customs_outpoint_count",
        state::read_state(|s| s.outpoint_destination.len()) as f64,
        "Total number of outputs the customs has to remember.",
    )?;

    metrics.encode_gauge(
        "bitcoin_customs_median_fee_per_vbyte",
        state::read_state(|s| s.last_fee_per_vbyte[50]) as f64,
        "Median Bitcoin transaction fee per vbyte in Satoshi.",
    )?;

    Ok(())
}
