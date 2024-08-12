use crate::auth::{is_admin, set_perms, Permission};
use crate::call_error::{CallError, Reason};
use candid::Principal;
use ic_cdk::{init, post_upgrade, pre_upgrade, query, update};
use ic_solana::types::TransactionStatus;

use crate::handler::ticket::{self, GenerateTicketError, GenerateTicketOk, GenerateTicketReq};
use crate::handler::{self, scheduler, sol_call};
use crate::state::TokenResp;
use ic_solana::token::TokenInfo;

use crate::types::TokenId;

use crate::lifecycle::{self, RouteArg, UpgradeArgs};

use crate::state::AssociatedTokenAccount;
use crate::state::Owner;
use crate::state::TokenMint;
use crate::state::{mutate_state, read_state, MintTokenStatus};
use crate::types::ChainState;
use crate::types::{Chain, ChainId, Ticket};
use ic_canister_log::export as export_logs;
use ic_canister_log::log;

use ic_canisters_http_types::{HttpRequest, HttpResponse, HttpResponseBuilder};
use ic_solana::logs::{ERROR, INFO};

#[init]
fn init(args: RouteArg) {
    match args {
        RouteArg::Init(args) => {
            lifecycle::init(args);
        }
        RouteArg::Upgrade(_) => {
            panic!("expected InitArgs got UpgradeArgs");
        }
    }
}

#[pre_upgrade]
fn pre_upgrade() {
    log!(INFO, "begin to execute pre_upgrade ...");
    scheduler::cancel_schedule();
    lifecycle::pre_upgrade();
    log!(INFO, "pre_upgrade end!");
}

#[post_upgrade]
fn post_upgrade(args: Option<RouteArg>) {
    log!(INFO, "begin to execute post_upgrade with :{:?}", args);
    let mut upgrade_arg: Option<UpgradeArgs> = None;
    if let Some(route_arg) = args {
        upgrade_arg = match route_arg {
            RouteArg::Upgrade(upgrade_args) => upgrade_args,
            RouteArg::Init(_) => panic!("expected Option<UpgradeArgs> got InitArgs."),
        };
    }

    lifecycle::post_upgrade(upgrade_arg);
    log!(INFO, "upgrade successfully!");
}

#[update(guard = "is_admin")]
pub fn start_schedule() {
    log!(INFO, "start schedule task ...");
    scheduler::start_schedule();
}

#[update(guard = "is_admin")]
pub fn cancel_schedule() {
    log!(INFO, "cancel schedule task ...");
    scheduler::cancel_schedule();
}

#[update(guard = "is_admin")]
pub async fn update_schnorr_info(id: Principal, key_name: String) {
    mutate_state(|s| {
        s.schnorr_canister = id;
        s.schnorr_key_name = key_name;
    })
}

#[update]
pub async fn signer() -> Result<String, String> {
    let pk = sol_call::eddsa_public_key().await?;
    Ok(pk.to_string())
}

#[update]
pub async fn get_fee_account() -> String {
    read_state(|s| s.fee_account.to_string())
}

#[update(guard = "is_admin")]
pub async fn resend_tickets() -> Result<(), GenerateTicketError> {
    let tickets_sz = read_state(|s| s.failed_tickets.len());
    while !read_state(|s| s.failed_tickets.is_empty()) {
        let ticket = mutate_state(|rs| rs.failed_tickets.pop()).unwrap();

        let hub_principal = read_state(|s| (s.hub_principal));
        if let Err(err) = handler::ticket::send_ticket(hub_principal, ticket.to_owned())
            .await
            .map_err(|err| GenerateTicketError::SendTicketErr(format!("{}", err)))
        {
            mutate_state(|state| {
                state.failed_tickets.push(ticket.to_owned());
            });
            log!(ERROR, "failed to resend ticket: {}", ticket.ticket_id);
            return Err(err);
        }
    }
    log!(INFO, "successfully resend {} tickets", tickets_sz);
    Ok(())
}

#[query]
fn get_chain_list() -> Vec<Chain> {
    read_state(|s| {
        s.counterparties
            .iter()
            .filter(|(_, chain)| matches!(chain.chain_state, ChainState::Active))
            .map(|(_, chain)| chain.clone())
            .collect()
    })
}

#[query]
fn get_token_list() -> Vec<TokenResp> {
    read_state(|s| {
        s.tokens
            .iter()
            .filter(|(token_id, _)| s.token_mint_map.contains_key(&token_id.to_string()))
            .map(|(_, token)| token.to_owned().into())
            .collect()
    })
}

#[query]
fn get_tickets_from_queue() -> Vec<(u64, Ticket)> {
    read_state(|s| {
        s.tickets_queue
            .iter()
            .map(|(seq, ticket)| (seq, ticket))
            .collect()
    })
}

#[update]
async fn get_latest_blockhash() -> Result<String, CallError> {
    use crate::service::sol_call::solana_client;
    let client = solana_client().await;
    let block_hash = client
        .get_latest_blockhash()
        .await
        .map_err(|err| CallError {
            method: "get_latest_blockhash".to_string(),
            reason: Reason::CanisterError(err.to_string()),
        })?;
    Ok(block_hash.to_string())
}

#[update]
async fn get_transaction(signature: String) -> Result<String, CallError> {
    use crate::service::sol_call::solana_client;
    let client = solana_client().await;
    client
        .query_transaction(signature)
        .await
        .map_err(|err| CallError {
            method: "get_transaction".to_string(),
            reason: Reason::CanisterError(err.to_string()),
        })
}

#[update]
async fn get_signature_status(
    signatures: Vec<String>,
) -> Result<Vec<TransactionStatus>, CallError> {
    sol_call::get_signature_status(signatures).await
}

#[update(guard = "is_admin")]
pub async fn handle_mint_token() {
    ticket::handle_mint_token().await;
}

#[update(guard = "is_admin")]
pub async fn create_mint(req: TokenInfo) -> Result<String, CallError> {
    sol_call::create_mint_account(req).await
}

#[query]
pub async fn query_token_mint(token_id: TokenId) -> Option<TokenMint> {
    read_state(|s| s.token_mint_map.get(&token_id).cloned())
}

#[update(guard = "is_admin")]
pub async fn get_or_create_aossicated_account(
    owner: String,
    token_mint: String,
) -> Result<String, CallError> {
    sol_call::get_or_create_ata(owner, token_mint).await
}

#[query]
pub async fn query_aossicated_account(
    owner: Owner,
    token_mint: TokenMint,
) -> Option<AssociatedTokenAccount> {
    read_state(|s| s.associated_account.get(&(owner, token_mint)).cloned())
}

#[update(guard = "is_admin")]
pub async fn mint_to(
    aossicated_account: String,
    amount: u64,
    token_mint: String,
) -> Result<String, CallError> {
    sol_call::mint_to(aossicated_account, amount, token_mint).await
}

#[update(guard = "is_admin")]
pub async fn update_token_metadata(
    token_mint: String,
    req: TokenInfo,
) -> Result<String, CallError> {
    sol_call::update_token_metadata(token_mint, req).await
}

#[update(guard = "is_admin")]
pub async fn transfer_to(to_account: String, amount: u64) -> Result<String, CallError> {
    sol_call::transfer_to(to_account, amount).await
}

#[query]
fn mint_token_status(ticket_id: String) -> MintTokenStatus {
    read_state(|s| {
        s.finalized_mint_token_requests.get(&ticket_id).map_or(
            MintTokenStatus::Unknown,
            |signature| MintTokenStatus::Finalized {
                signature: signature.to_string(),
            },
        )
    })
}

// redeem fee = gas fee + service fee
// the service fee,there is 3 solutions
// s2e: free; e2s: 2$; e2e: 1$
#[query]
pub fn get_redeem_fee(chain_id: ChainId) -> Option<u128> {
    read_state(|s| s.get_fee(chain_id))
}

#[update]
async fn generate_ticket(args: GenerateTicketReq) -> Result<GenerateTicketOk, GenerateTicketError> {
    ticket::generate_ticket(args).await
}

#[update(guard = "is_admin")]
pub async fn set_permissions(caller: Principal, perm: Permission) {
    set_perms(caller.to_string(), perm)
}

#[query(hidden = true)]
fn http_request(req: HttpRequest) -> HttpResponse {
    if ic_cdk::api::data_certificate().is_none() {
        ic_cdk::trap("update call rejected");
    }

    if req.path() == "/logs" {
        use serde_json;
        use std::str::FromStr;

        let max_skip_timestamp = match req.raw_query_param("time") {
            Some(arg) => match u64::from_str(arg) {
                Ok(value) => value,
                Err(_) => {
                    return HttpResponseBuilder::bad_request()
                        .with_body_and_content_length("failed to parse the 'time' parameter")
                        .build()
                }
            },
            None => 0,
        };

        let mut entries = vec![];
        for entry in export_logs(&ic_solana::logs::INFO_BUF) {
            entries.push(entry);
        }
        for entry in export_logs(&ic_solana::logs::DEBUG_BUF) {
            entries.push(entry);
        }
        for entry in export_logs(&ic_solana::logs::ERROR_BUF) {
            entries.push(entry);
        }
        for entry in export_logs(&ic_solana::logs::TRACE_HTTP_BUF) {
            entries.push(entry);
        }
        entries.retain(|entry| entry.timestamp >= max_skip_timestamp);
        HttpResponseBuilder::ok()
            .header("Content-Type", "application/json; charset=utf-8")
            .with_body_and_content_length(serde_json::to_string(&entries).unwrap_or_default())
            .build()
    } else {
        HttpResponseBuilder::not_found().build()
    }
}

// Enable Candid export
ic_cdk::export_candid!();
