use candid::Principal;
// use ic_canisters_http_types::{HttpRequest, HttpResponse};

use ic_canisters_http_types::{HttpRequest, HttpResponse};
use ic_cdk::{init, post_upgrade, pre_upgrade, query, update};

use ic_log::writer::Logs;
use log::info;
use omnity_types::log::{init_log, LoggerConfigService, StableLogWriter};
use omnity_types::{Chain, ChainId};
use solana_route::auth::{is_admin, set_perms, Permission};
use solana_route::event::{Event, GetEventsArg};
use solana_route::handler::ticket::{
    self, GenerateTicketError, GenerateTicketOk, GenerateTicketReq,
};
use solana_route::handler::{self, scheduler::start_schedule};
use solana_route::state::TokenResp;

// use omnity_types::Network;
use solana_route::event;
use solana_route::lifecycle::{self, RouteArg, UpgradeArgs};
use solana_route::memory::init_stable_log;
use solana_route::state::{management_canister, mutate_state, read_state, MintTokenStatus};
use solana_route::threshold_schnorr::{
    ManagementCanisterSchnorrPublicKeyReply, ManagementCanisterSchnorrPublicKeyRequest,
    PublicKeyReply, SchnorrAlgorithm, SchnorrKeyIds,
};

#[init]
fn init(args: RouteArg) {
    init_log(Some(init_stable_log()));
    match args {
        RouteArg::Init(args) => {
            event::record_event(&Event::Init(args.clone()));
            lifecycle::init(args);
        }
        RouteArg::Upgrade(_) => {
            panic!("expected InitArgs got UpgradeArgs");
        }
    }
    start_schedule()
}

#[pre_upgrade]
fn pre_upgrade() {
    info!("begin to execute pre_upgrade ...");
    lifecycle::pre_upgrade();
    info!("pre_upgrade end!");
}

#[post_upgrade]
fn post_upgrade(args: Option<RouteArg>) {
    init_log(Some(init_stable_log()));
    info!("begin to execute post_upgrade with :{:?}", args);
    let mut upgrade_arg: Option<UpgradeArgs> = None;
    if let Some(route_arg) = args {
        upgrade_arg = match route_arg {
            RouteArg::Upgrade(upgrade_args) => upgrade_args,
            RouteArg::Init(_) => panic!("expected Option<UpgradeArgs> got InitArgs."),
        };
    }
    lifecycle::post_upgrade(upgrade_arg);

    start_schedule();
    info!("upgrade successfully!");
}

// just for test or dev
#[update(guard = "is_admin")]
async fn update_schnorr_canister_id(id: Principal) -> Result<(), String> {
    mutate_state(|s| {
        s.schnorr_canister = id;
    });
    Ok(())
}

// TODO: match network for schnorr_key_id
#[update(guard = "is_admin")]
async fn public_key(algorithm: SchnorrAlgorithm) -> Result<PublicKeyReply, String> {
    let request = ManagementCanisterSchnorrPublicKeyRequest {
        canister_id: None,
        derivation_path: vec![ic_cdk::api::caller().as_slice().to_vec()],
        key_id: SchnorrKeyIds::TestKeyLocalDevelopment.to_key_id(algorithm),
    };

    let (res,): (ManagementCanisterSchnorrPublicKeyReply,) =
        ic_cdk::call(management_canister(), "schnorr_public_key", (request,))
            .await
            .map_err(|e| format!("schnorr_public_key failed {}", e.1))?;

    Ok(PublicKeyReply {
        public_key_hex: hex::encode(&res.public_key),
    })
}

#[update(guard = "is_admin")]
pub async fn resend_tickets() -> Result<(), GenerateTicketError> {
    let tickets_sz = read_state(|s| s.failed_tickets.len());
    while !read_state(|s| s.failed_tickets.is_empty()) {
        let ticket = mutate_state(|rs| rs.failed_tickets.pop()).unwrap();

        let hub_principal = read_state(|s| (s.hub_principal));
        if let Err(err) = handler::ticket::send_ticket(hub_principal, ticket.clone())
            .await
            .map_err(|err| GenerateTicketError::SendTicketErr(format!("{}", err)))
        {
            mutate_state(|state| {
                state.failed_tickets.push(ticket.clone());
            });
            log::error!("failed to resend ticket: {}", ticket.ticket_id);
            return Err(err);
        }
    }
    log::info!("successfully resend {} tickets", tickets_sz);
    Ok(())
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
            .map(|(_, token)| token.clone().into())
            .collect()
    })
}

#[query]
fn mint_token_status(ticket_id: String) -> MintTokenStatus {
    read_state(|s| {
        s.finalized_mint_token_requests
            .get(&ticket_id)
            .map_or(MintTokenStatus::Unknown, |&block_index| {
                MintTokenStatus::Finalized { block_index }
            })
    })
}

#[query]
pub fn get_redeem_fee(chain_id: ChainId) -> Option<u128> {
    read_state(|s| {
        s.target_chain_factor
            .get(&chain_id)
            .map_or(None, |target_chain_factor| {
                s.fee_token_factor
                    .map(|fee_token_factor| target_chain_factor * fee_token_factor)
            })
    })
}

#[update]
async fn generate_ticket(args: GenerateTicketReq) -> Result<GenerateTicketOk, GenerateTicketError> {
    ticket::generate_ticket(args).await
}

#[update(guard = "is_admin")]
pub async fn set_logger_filter(filter: String) {
    LoggerConfigService::default().set_logger_filter(&filter);
}

#[update(guard = "is_admin")]
pub async fn set_permissions(caller: Principal, perm: Permission) {
    set_perms(caller.to_string(), perm)
}

#[query]
pub fn get_log_records(offset: usize, limit: usize) -> Logs {
    log::debug!("collecting {limit} log records");
    ic_log::take_memory_records(limit, offset)
}

#[query(hidden = true)]
fn http_request(req: HttpRequest) -> HttpResponse {
    StableLogWriter::http_request(req)
}

#[query]
fn get_events(args: GetEventsArg) -> Vec<Event> {
    const MAX_EVENTS_PER_QUERY: usize = 2000;

    event::events()
        .skip(args.start as usize)
        .take(MAX_EVENTS_PER_QUERY.min(args.length as usize))
        .collect()
}

fn main() {}

// Enable Candid export
ic_cdk::export_candid!();
