use bitcoin_customs::lifecycle::upgrade::UpgradeArgs;
use bitcoin_customs::lifecycle::{self, init::CustomArg};
use bitcoin_customs::metrics::encode_metrics;
use bitcoin_customs::queries::{EstimateFeeArgs, GetGenTicketReqsArgs, RedeemFee};
use bitcoin_customs::state::{
    mutate_state, read_state, GenTicketRequestV2, GenTicketStatus, ReleaseTokenStatus,
};
use bitcoin_customs::storage::record_event;
use bitcoin_customs::updates::generate_ticket::{GenerateTicketArgs, GenerateTicketError};
use bitcoin_customs::updates::update_btc_utxos::UpdateBtcUtxosErr;
use bitcoin_customs::updates::{
    self,
    get_btc_address::GetBtcAddressArgs,
    update_runes_balance::{UpdateRunesBalanceArgs, UpdateRunesBalanceError},
};
use bitcoin_customs::{
    process_directive_msg_task, process_ticket_msg_task, process_tx_task, refresh_fee_task,
    CustomsInfo, TokenResp, FEE_ESTIMATE_DELAY, INTERVAL_PROCESSING, INTERVAL_QUERY_DIRECTIVES,
};
use bitcoin_customs::{
    state::eventlog::{Event, GetEventsArg},
    storage,
};
use candid::Principal;
use ic_btc_interface::{Txid, Utxo};
use ic_canister_log::log;
use ic_canisters_http_types::{HttpRequest, HttpResponse, HttpResponseBuilder};
use ic_cdk::api::management_canister::http_request::{self, TransformArgs};
use ic_cdk_macros::{init, post_upgrade, query, update};
use ic_cdk_timers::set_timer_interval;
use omnity_types::{Chain, ChainId};
use std::cmp::max;
use std::ops::Bound::{Excluded, Unbounded};
use std::str::FromStr;
use bitcoin_customs::state::eventlog::Event::UpdateFeeCollector;
use omnity_types::ic_log::INFO;

#[init]
fn init(args: CustomArg) {
    match args {
        CustomArg::Init(args) => {
            storage::record_event(&Event::Init(args.clone()));
            lifecycle::init::init(args);

            set_timer_interval(INTERVAL_PROCESSING, process_tx_task);
            set_timer_interval(INTERVAL_PROCESSING, process_ticket_msg_task);
            set_timer_interval(INTERVAL_QUERY_DIRECTIVES, process_directive_msg_task);
            set_timer_interval(FEE_ESTIMATE_DELAY, refresh_fee_task);

            #[cfg(feature = "self_check")]
            ok_or_die(check_invariants())
        }
        CustomArg::Upgrade(_) => {
            panic!("expected InitArgs got UpgradeArgs");
        }
    }
}

#[cfg(feature = "self_check")]
fn ok_or_die(result: Result<(), String>) {
    if let Err(msg) = result {
        ic_cdk::println!("{}", msg);
        ic_cdk::trap(&msg);
    }
}

/// Checks that customs state internally consistent.
#[cfg(feature = "self_check")]
fn check_invariants() -> Result<(), String> {
    use bitcoin_customs::state::eventlog::replay;

    read_state(|s| {
        s.check_invariants()?;

        let events: Vec<_> = storage::events().collect();
        let recovered_state = replay(events.clone().into_iter())
            .unwrap_or_else(|e| panic!("failed to replay log {:?}: {:?}", events, e));

        recovered_state.check_invariants()?;

        // A running timer can temporarily violate invariants.
        if !s.is_timer_running {
            s.check_semantically_eq(&recovered_state)?;
        }

        Ok(())
    })
}

#[cfg(feature = "self_check")]
#[update]
async fn refresh_fee_percentiles() {
    let _ = bitcoin_customs::estimate_fee_per_vbyte().await;
}

fn check_postcondition<T>(t: T) -> T {
    #[cfg(feature = "self_check")]
    ok_or_die(check_invariants());
    t
}

#[post_upgrade]
fn post_upgrade(custom_arg: Option<CustomArg>) {
    let mut upgrade_arg: Option<UpgradeArgs> = None;
    if let Some(custom_arg) = custom_arg {
        upgrade_arg = match custom_arg {
            CustomArg::Upgrade(upgrade_args) => upgrade_args,
            CustomArg::Init(_) => panic!("expected Option<UpgradeArgs> got InitArgs."),
        };
    }
    lifecycle::upgrade::post_upgrade(upgrade_arg);

    set_timer_interval(INTERVAL_PROCESSING, process_tx_task);
    set_timer_interval(INTERVAL_PROCESSING, process_ticket_msg_task);
    set_timer_interval(INTERVAL_QUERY_DIRECTIVES, process_directive_msg_task);
    set_timer_interval(FEE_ESTIMATE_DELAY, refresh_fee_task);
}

#[update]
async fn get_btc_address(args: GetBtcAddressArgs) -> String {
    updates::get_btc_address::get_btc_address(args).await
}

#[update]
async fn get_main_btc_address(token: String) -> String {
    updates::get_main_btc_address(token).await
}

#[query]
fn release_token_status(ticket_id: String) -> ReleaseTokenStatus {
    read_state(|s| s.rune_tx_status(&ticket_id))
}

#[query]
fn generate_ticket_status(ticket_id: String) -> GenTicketStatus {
    let txid = match Txid::from_str(&ticket_id) {
        Ok(txid) => txid,
        Err(_) => {
            return GenTicketStatus::Unknown;
        }
    };
    read_state(|s| s.generate_ticket_status(txid))
}

/// The function name needs to be changed to get_confirmed_gen_ticket_requests,
/// but considering that it will affect runes oracle, it will be retained temporarily.
#[query]
fn get_pending_gen_ticket_requests(args: GetGenTicketReqsArgs) -> Vec<GenTicketRequestV2> {
    let start = args.start_txid.map_or(Unbounded, |txid| Excluded(txid));
    let count = max(50, args.max_count) as usize;
    read_state(|s| {
        s.confirmed_gen_ticket_requests
            .range((start, Unbounded))
            .take(count)
            .map(|(_, req)| req.clone())
            .collect()
    })
}

pub fn is_runes_oracle() -> Result<(), String> {
    let caller = ic_cdk::api::caller();
    read_state(|s| {
        if !s.runes_oracles.contains(&caller) {
            Err("Not runes principal!".into())
        } else {
            log!(
                INFO,
                "[is_runes_oracle]: got update_runes_balance from runes oracle: {}",
                caller
            );
            Ok(())
        }
    })
}

pub fn is_controller() -> Result<(), String> {
    if ic_cdk::api::is_controller(&ic_cdk::caller()) {
        Ok(())
    } else {
        Err("caller is not controller".to_string())
    }
}

#[update(guard = "is_runes_oracle")]
async fn update_runes_balance(args: UpdateRunesBalanceArgs) -> Result<(), UpdateRunesBalanceError> {
    check_postcondition(updates::update_runes_balance(args).await)
}

#[update]
async fn update_btc_utxos() -> Result<Vec<Utxo>, UpdateBtcUtxosErr> {
    check_postcondition(updates::update_btc_utxos().await)
}

#[update]
async fn generate_ticket(args: GenerateTicketArgs) -> Result<(), GenerateTicketError> {
    check_postcondition(updates::generate_ticket(args).await)
}

#[query]
fn get_runes_oracles() -> Vec<Principal> {
    read_state(|s| s.runes_oracles.iter().cloned().collect())
}

#[update(guard = "is_controller")]
fn set_runes_oracle(oracle: Principal) {
    record_event(&Event::AddedRunesOracle { principal: oracle });
    mutate_state(|s| s.runes_oracles.insert(oracle));
}

#[update(guard = "is_controller")]
fn remove_runes_oracle(oracle: Principal) {
    if !read_state(|s| s.runes_oracles.contains(&oracle)) {
        return;
    }
    record_event(&Event::RemovedRunesOracle { principal: oracle });
    mutate_state(|s| s.runes_oracles.remove(&oracle));
}

#[update(guard = "is_controller")]
fn update_rpc_url(url: String) {
    record_event(&Event::UpdatedRpcURL {
        rpc_url: url.clone(),
    });
    mutate_state(|s| s.rpc_url = Some(url));
}

#[update]
async fn get_canister_status() -> ic_cdk::api::management_canister::main::CanisterStatusResponse {
    ic_cdk::api::management_canister::main::canister_status(
        ic_cdk::api::management_canister::main::CanisterIdRecord {
            canister_id: ic_cdk::id(),
        },
    )
    .await
    .expect("failed to fetch canister status")
    .0
}

#[query]
fn estimate_redeem_fee(arg: EstimateFeeArgs) -> RedeemFee {
    read_state(|s| {
        bitcoin_customs::estimate_fee(
            arg.rune_id,
            &s.available_runes_utxos,
            arg.amount,
            s.last_fee_per_vbyte[50],
        )
    })
}

#[query]
fn get_platform_fee(target_chain: ChainId) -> (Option<u128>, Option<String>) {
    read_state(|s| {
        s.get_transfer_fee_info(&target_chain)
    })
}

#[update(guard = "is_controller")]
pub fn set_fee_collector(addr: String) {
    mutate_state(|s|s.fee_collector_address = addr.clone());
    record_event(&UpdateFeeCollector {addr});
}

#[query]
fn get_customs_info() -> CustomsInfo {
    read_state(|s| CustomsInfo {
        min_confirmations: s.min_confirmations,
        chain_state: s.chain_state.clone(),
        next_ticket_seq: s.next_ticket_seq,
        next_directive_seq: s.next_directive_seq,
        hub_principal: s.hub_principal,
        runes_oracles: s.runes_oracles.clone(),
        rpc_url: s.rpc_url.clone(),
        last_fee_per_vbyte: s.last_fee_per_vbyte.clone(),
        fee_token_factor: s.fee_token_factor.clone(),
        target_chain_factor: s.target_chain_factor.clone(),
        fee_collector_address: s.fee_collector_address.clone(),
        btc_network: s.btc_network,
        chain_id: s.chain_id.clone(),
        ecdsa_key_name: s.ecdsa_key_name.clone(),
        ecdsa_public_key: s.ecdsa_public_key.clone(),
        prod_ecdsa_public_key: s.prod_ecdsa_public_key.clone(),
        max_time_in_queue_nanos: s.max_time_in_queue_nanos,
        generate_ticket_counter: s.generate_ticket_counter,
        release_token_counter: s.release_token_counter,
    })
}

#[query]
fn get_chain_list() -> Vec<Chain> {
    read_state(|s| {
        s.counterparties
            .iter()
            .map(|(_, chain)| chain.clone())
            .collect()
    })
}

#[query]
fn get_token_list() -> Vec<TokenResp> {
    read_state(|s| {
        s.tokens
            .iter()
            .map(|(_, (rune_id, token))| TokenResp {
                token_id: token.token_id.clone(),
                symbol: token.symbol.clone(),
                decimals: token.decimals,
                icon: token.icon.clone(),
                rune_id: rune_id.to_string(),
            })
            .collect()
    })
}

#[query(hidden = true)]
fn http_request(req: HttpRequest) -> HttpResponse {
    if ic_cdk::api::data_certificate().is_none() {
        ic_cdk::trap("update call rejected");
    }

    if req.path() == "/metrics" {
        let mut writer =
            ic_metrics_encoder::MetricsEncoder::new(vec![], ic_cdk::api::time() as i64 / 1_000_000);

        match encode_metrics(&mut writer) {
            Ok(()) => HttpResponseBuilder::ok()
                .header("Content-Type", "text/plain; version=0.0.4")
                .with_body_and_content_length(writer.into_inner())
                .build(),
            Err(err) => {
                HttpResponseBuilder::server_error(format!("Failed to encode metrics: {}", err))
                    .build()
            }
        }
    } else if req.path() == "/logs" {
        omnity_types::ic_log::http_request(req)
    } else {
        HttpResponseBuilder::not_found().build()
    }
}

#[query]
fn get_events(args: GetEventsArg) -> Vec<Event> {
    const MAX_EVENTS_PER_QUERY: usize = 2000;

    storage::events()
        .skip(args.start as usize)
        .take(MAX_EVENTS_PER_QUERY.min(args.length as usize))
        .collect()
}

#[query]
fn transform(raw: TransformArgs) -> http_request::HttpResponse {
    http_request::HttpResponse {
        status: raw.response.status.clone(),
        body: raw.response.body.clone(),
        headers: vec![],
        ..Default::default()
    }
}

#[cfg(feature = "self_check")]
#[query]
fn self_check() -> Result<(), String> {
    check_invariants()
}

#[query(hidden = true)]
fn __get_candid_interface_tmp_hack() -> &'static str {
    include_str!(env!("BITCOIN_CUSTOMS_DID_PATH"))
}

fn main() {}

/// Checks the real candid interface against the one declared in the did file
#[test]
fn check_candid_interface_compatibility() {
    use candid_parser::utils::{service_equal, CandidSource};

    fn source_to_str(source: &CandidSource) -> String {
        match source {
            CandidSource::File(f) => std::fs::read_to_string(f).unwrap_or_else(|_| "".to_string()),
            CandidSource::Text(t) => t.to_string(),
        }
    }

    fn check_service_equal(new_name: &str, new: CandidSource, old_name: &str, old: CandidSource) {
        let new_str = source_to_str(&new);
        let old_str = source_to_str(&old);
        match service_equal(new, old) {
            Ok(_) => {}
            Err(e) => {
                eprintln!(
                    "{} is not compatible with {}!\n\n\
            {}:\n\
            {}\n\n\
            {}:\n\
            {}\n",
                    new_name, old_name, new_name, new_str, old_name, old_str
                );
                panic!("{:?}", e);
            }
        }
    }

    candid::export_service!();

    let new_interface = __export_service();

    // check the public interface against the actual one
    let old_interface = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("bitcoin_customs.did");

    check_service_equal(
        "actual ledger candid interface",
        candid_parser::utils::CandidSource::Text(&new_interface),
        "declared candid interface in bitcoin_customs.did file",
        candid_parser::utils::CandidSource::File(old_interface.as_path()),
    );
}

// Enable Candid export
ic_cdk::export_candid!();
