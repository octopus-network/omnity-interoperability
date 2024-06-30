use bitcoin_mock::types::{
    mutate_state, read_state, Args, CallError, FinalizedStatus, GenTicketRequest,
    GenerateTicketArgs, PushUtxosToAddress, Reason, ReleaseTokenStatus, RuneId, TimerLogicGuard,
    Txid, DEFAULT_TIP_HEIGHT, MAX_FINALIZED_REQUESTS, TOKEN_ID,
};

use candid::{candid_method, Principal};
use ic_btc_interface::{
    GetCurrentFeePercentilesRequest, GetUtxosRequest, GetUtxosResponse, MillisatoshiPerByte,
    Network, Utxo, UtxosFilterInRequest,
};
use ic_cdk::api::management_canister::bitcoin::{BitcoinNetwork, SendTransactionRequest};
use ic_cdk::query;
use ic_cdk_macros::{init, update};
use ic_cdk_timers::set_timer_interval;
use log::info;
use omnity_types::log::init_log;
use omnity_types::{Directive, Seq, Ticket, TicketId, Topic};
use serde_bytes::ByteBuf;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

use std::str::FromStr;
use std::time::Duration;

// We use 12 as the default tip height to mint all
// the utxos with height 1 in the customs.

pub const INTERVAL_QUERY_DIRECTIVE: u64 = 5;
pub const INTERVAL_QUERY_TICKET: u64 = 5;
pub const DIRE_CHAIN_ID: &str = "eICP";
pub const TICKET_CHAIN_ID: &str = "Bitcoin";

pub async fn query_directives(
    hub_principal: Principal,
    method: String,
    offset: u64,
    limit: u64,
) -> Result<Vec<(Seq, Directive)>, CallError> {
    // let (hub_principal, query_directive) =
    //     read_state(|s| (s.hub_principal, s.directive_method.to_string()));
    // let offset = 0u64;
    // let limit = 12u64;
    let resp: (Result<Vec<(Seq, Directive)>, omnity_types::Error>,) = ic_cdk::api::call::call(
        hub_principal,
        &method,
        (Some(DIRE_CHAIN_ID), None::<Option<Topic>>, offset, limit),
    )
    .await
    .map_err(|(code, message)| CallError {
        method: method.to_string(),
        reason: Reason::from_reject(code, message),
    })?;

    let data = resp.0.map_err(|err| CallError {
        method: method,
        reason: Reason::CanisterError(err.to_string()),
    })?;

    Ok(data)
}

fn handle_directive() {
    ic_cdk::spawn(async {
        let _guard = match TimerLogicGuard::new("FETCH_HUB_DIRECTIVE".to_string()) {
            Some(guard) => guard,
            None => return,
        };
        let (hub_principal, query_directive) =
            read_state(|s| (s.hub_principal, s.directive_method.to_string()));
        let offset = 0u64;
        let limit = 12u64;
        match query_directives(hub_principal, query_directive.to_string(), offset, limit).await {
            Ok(directives) => {
                info!("{} result : {:?}", query_directive, directives);
            }
            Err(err) => {
                info!(" failed to {}, err: {:?}", query_directive, err);
            }
        }
    })
}
pub async fn query_tickets(
    hub_principal: Principal,
    method: String,
    offset: u64,
    limit: u64,
) -> Result<Vec<(Seq, Ticket)>, CallError> {
    // let (hub_principal, query_ticket) =
    //     read_state(|s| (s.hub_principal, s.ticket_method.to_string()));
    // let offset = 0u64;
    // let limit = 6u64;
    let resp: (Result<Vec<(Seq, Ticket)>, omnity_types::Error>,) = ic_cdk::api::call::call(
        hub_principal,
        &method,
        (Some(TICKET_CHAIN_ID), offset, limit),
    )
    .await
    .map_err(|(code, message)| CallError {
        method: method.to_string(),
        reason: Reason::from_reject(code, message),
    })?;
    let data = resp.0.map_err(|err| CallError {
        method: method,
        reason: Reason::CanisterError(err.to_string()),
    })?;
    Ok(data)
}
fn handle_tickets() {
    ic_cdk::spawn(async {
        let _guard = match TimerLogicGuard::new("FETCH_HUB_TICKET".to_string()) {
            Some(guard) => guard,
            None => return,
        };
        let (hub_principal, query_ticket) =
            read_state(|s| (s.hub_principal, s.ticket_method.to_string()));
        let offset = 0u64;
        let limit = 6u64;
        match query_tickets(hub_principal, query_ticket.to_string(), offset, limit).await {
            Ok(tickets) => {
                info!("{} result : {:?}", query_ticket, tickets);
            }
            Err(err) => {
                info!(" failed to {}, err: {:?}", query_ticket, err);
            }
        }
    })
}

fn schedule_jobs() {
    set_timer_interval(
        Duration::from_secs(INTERVAL_QUERY_DIRECTIVE),
        handle_directive,
    );
    set_timer_interval(Duration::from_secs(INTERVAL_QUERY_TICKET), handle_tickets);
}

#[init]
fn init(args: Args) {
    init_log(None);
    let network = args.network.unwrap_or(Network::Regtest);

    mutate_state(|s| {
        s.network = network;
        s.fee_percentiles = [0; 100].into();
        s.is_available = true;
        s.utxo_to_address = BTreeMap::new();
        s.address_to_utxos = BTreeMap::new();
        s.mempool = BTreeSet::new();
        s.tip_height = DEFAULT_TIP_HEIGHT;
        s.pending_gen_ticket_requests = Default::default();
        s.pending_release_token_requests = Default::default();
        s.finalized_release_token_requests = BTreeMap::new();
        s.finalized_gen_ticket_requests = VecDeque::with_capacity(MAX_FINALIZED_REQUESTS);
        s.is_timer_running = BTreeMap::new();
        s.hub_principal = args.hub_principal;
        s.directive_method = args.directive_method;
        s.ticket_method = args.ticket_method;
    });
    schedule_jobs()
}

#[candid_method(update)]
#[update]
fn set_tip_height(tip_height: u32) {
    mutate_state(|s| s.tip_height = tip_height);
}

#[candid_method(update)]
#[update]
fn bitcoin_get_utxos(utxos_request: GetUtxosRequest) -> GetUtxosResponse {
    read_state(|s| {
        assert_eq!(utxos_request.network, s.network.into());

        let mut utxos = s
            .address_to_utxos
            .get(&utxos_request.address)
            .cloned()
            .unwrap_or_default()
            .iter()
            .cloned()
            .collect::<Vec<Utxo>>();

        if let Some(UtxosFilterInRequest::MinConfirmations(min_confirmations)) =
            utxos_request.filter
        {
            utxos.retain(|u| s.tip_height + 1 >= u.height + min_confirmations);
        }

        GetUtxosResponse {
            utxos,
            tip_block_hash: vec![],
            tip_height: s.tip_height,
            // TODO Handle pagination.
            next_page: None,
        }
    })
}

#[candid_method(update)]
#[update]
fn push_utxos_to_address(req: PushUtxosToAddress) {
    mutate_state(|s| {
        for (address, utxos) in &req.utxos {
            for utxo in utxos {
                s.utxo_to_address.insert(utxo.clone(), address.clone());
                s.address_to_utxos
                    .entry(address.clone())
                    .or_default()
                    .insert(utxo.clone());
            }
        }
    });
}

#[candid_method(update)]
#[update]
fn remove_utxo(utxo: Utxo) {
    let address = read_state(|s| s.utxo_to_address.get(&utxo).cloned().unwrap());
    mutate_state(|s| {
        s.utxo_to_address.remove(&utxo);
        s.address_to_utxos
            .get_mut(&address)
            .expect("utxo not found at address")
            .remove(&utxo);
    });
}

#[candid_method(update)]
#[update]
fn bitcoin_get_current_fee_percentiles(
    _: GetCurrentFeePercentilesRequest,
) -> Vec<MillisatoshiPerByte> {
    read_state(|s| s.fee_percentiles.clone())
}

#[candid_method(update)]
#[update]
fn set_fee_percentiles(fee_percentiles: Vec<MillisatoshiPerByte>) {
    mutate_state(|s| s.fee_percentiles = fee_percentiles);
}

#[candid_method(update)]
#[update]
fn bitcoin_send_transaction(transaction: SendTransactionRequest) {
    mutate_state(|s| {
        let cdk_network = match transaction.network {
            BitcoinNetwork::Mainnet => Network::Mainnet,
            BitcoinNetwork::Testnet => Network::Testnet,
            BitcoinNetwork::Regtest => Network::Regtest,
        };
        assert_eq!(cdk_network, s.network);
        if s.is_available {
            s.mempool.insert(ByteBuf::from(transaction.transaction));
        }
    })
}

#[candid_method(update)]
#[update]
fn change_availability(is_available: bool) {
    mutate_state(|s| s.is_available = is_available);
}

#[candid_method(update)]
#[update]
fn get_mempool() -> Vec<ByteBuf> {
    read_state(|s| s.mempool.iter().cloned().collect::<Vec<ByteBuf>>())
}

#[candid_method(update)]
#[update]
fn reset_mempool() {
    mutate_state(|s| s.mempool = BTreeSet::new());
}

#[update]
pub fn generate_ticket(args: GenerateTicketArgs) {
    println!("received generate_ticket: {:?}", args);
    let rune_id = RuneId::from_str(&args.rune_id).unwrap();
    let token_id = TOKEN_ID.to_owned();
    let txid = Txid::from_str(&args.txid).unwrap();

    let request = GenTicketRequest {
        address: "bc1qmh0chcr9f73a3ynt90k0w8qsqlydr4a6espnj6".to_owned(),
        target_chain_id: args.target_chain_id,
        receiver: args.receiver,
        token_id,
        rune_id,
        amount: args.amount,
        txid,
        received_at: ic_cdk::api::time(),
    };

    mutate_state(|s| {
        s.pending_gen_ticket_requests.insert(request.txid, request);
    })
}

//mock: ticket be finalized
#[update]
pub fn mock_finalized_ticket(ticket_id: TicketId) {
    let txid = Txid::from_str(&ticket_id).unwrap();
    mutate_state(|s| {
        s.pending_gen_ticket_requests.remove(&txid);
    })
}

#[query]
fn get_pending_gen_ticket_size() -> u64 {
    let size = read_state(|s| s.pending_gen_ticket_requests.len() as u64);
    size
}

#[query]
fn get_pending_gen_tickets(from_seq: usize, limit: usize) -> Vec<GenTicketRequest> {
    read_state(|s| {
        s.pending_gen_ticket_requests
            .iter()
            .skip(from_seq)
            .take(limit)
            .map(|(_, req)| req.to_owned())
            .collect::<Vec<_>>()
    })
}

#[update]
fn mock_finalized_release_token(ticket_id: TicketId, status: FinalizedStatus) {
    mutate_state(|s| {
        s.finalized_release_token_requests.insert(ticket_id, status);
    })
}

#[query]
fn release_token_status(ticket_id: String) -> ReleaseTokenStatus {
    read_state(|s| {
        match s.finalized_release_token_requests.get(&ticket_id) {
            Some(FinalizedStatus::Confirmed(txid)) => {
                return ReleaseTokenStatus::Confirmed(txid.to_string())
            }
            None => (),
        }

        ReleaseTokenStatus::Unknown
    })
}

fn main() {}
ic_cdk::export_candid!();
