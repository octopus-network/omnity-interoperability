use crate::auth::{is_admin, set_perms, Permission};
use crate::call_error::{CallError, Reason};
use candid::Principal;
use ic_cdk::{init, post_upgrade, pre_upgrade, query, update};
use ic_solana::types::TransactionStatus;

use crate::handler::ticket::{self, GenerateTicketError, GenerateTicketOk, GenerateTicketReq};
use crate::handler::{self, scheduler, sol_call};
use crate::lifecycle::{self, RouteArg, UpgradeArgs};
use crate::service::sol_call::solana_client;
use crate::state::{AccountInfo, AccountStatus, TokenResp};
use crate::types::TokenId;
use ic_solana::token::SolanaClient;
use ic_solana::token::TokenInfo;

use crate::service::ticket::MintTokenRequest;
use crate::state::MintAccount;
use crate::state::Owner;
use crate::state::{mutate_state, read_state, MintTokenStatus};
use crate::types::ChainState;
use crate::types::{Chain, ChainId, Ticket};
use ic_canister_log::export as export_logs;
use ic_canister_log::log;
use ic_solana::token::associated_account::get_associated_token_address_with_program_id;
use ic_solana::token::constants::token22_program_id;
use ic_solana::types::Pubkey;
use ic_solana::types::TransactionConfirmationStatus;
use std::str::FromStr;

use ic_canisters_http_types::{HttpRequest, HttpResponse, HttpResponseBuilder};
use ic_solana::logs::{DEBUG, ERROR};

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
    log!(DEBUG, "begin to execute pre_upgrade ...");
    scheduler::cancel_schedule();
    lifecycle::pre_upgrade();
    log!(DEBUG, "pre_upgrade end!");
}

#[post_upgrade]
fn post_upgrade(args: Option<RouteArg>) {
    log!(DEBUG, "begin to execute post_upgrade with :{:?}", args);
    let mut upgrade_arg: Option<UpgradeArgs> = None;
    if let Some(route_arg) = args {
        upgrade_arg = match route_arg {
            RouteArg::Upgrade(upgrade_args) => upgrade_args,
            RouteArg::Init(_) => panic!("expected Option<UpgradeArgs> got InitArgs."),
        };
    }

    lifecycle::post_upgrade(upgrade_arg);
    log!(DEBUG, "upgrade successfully!");
}

#[update(guard = "is_admin")]
pub fn start_schedule() {
    log!(DEBUG, "start schedule task ...");
    scheduler::start_schedule();
}

#[update(guard = "is_admin")]
pub fn cancel_schedule() {
    log!(DEBUG, "cancel schedule task ...");
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
pub async fn sign(msg: String) -> Result<String, String> {
    let signature = sol_call::sign(msg).await?;
    Ok(signature)
}

#[update]
pub async fn get_fee_account() -> String {
    read_state(|s| s.fee_account.to_string())
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
            .filter(|(token_id, _)| s.token_mint_accounts.contains_key(&token_id.to_string()))
            .map(|(_, token)| token.to_owned().into())
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
pub async fn derive_mint_account(req: TokenInfo) -> Result<String, CallError> {
    let sol_client = solana_client().await;

    let mint_account = SolanaClient::derive_account(
        sol_client.schnorr_canister.clone(),
        sol_client.chainkey_name.clone(),
        req.name.to_string(),
    )
    .await;

    Ok(mint_account.to_string())
}

#[update(guard = "is_admin")]
pub async fn get_account_info(account: String) -> Result<Option<String>, CallError> {
    let sol_client = solana_client().await;

    // query account info from solana
    let account_info = sol_client
        .get_account_info(account.to_string())
        .await
        .map_err(|e| CallError {
            method: "[service::get_account_info] get_account_info".to_string(),
            reason: Reason::CanisterError(e.to_string()),
        })?;
    log!(
        DEBUG,
        "[service::get_account_info] {} account_info from solana : {:?} ",
        account.to_string(),
        account_info,
    );
    Ok(account_info)
}

#[query]
pub async fn query_mint_account(token_id: TokenId) -> Option<AccountInfo> {
    read_state(|s| s.token_mint_accounts.get(&token_id).cloned())
}

#[query]
pub async fn query_mint_address(token_id: TokenId) -> Option<String> {
    read_state(|s| {
        s.token_mint_accounts
            .get(&token_id)
            .map(|mint_account| mint_account.account.to_string())
    })
}

#[update(guard = "is_admin")]
pub async fn create_mint_account(req: TokenInfo) -> Result<AccountInfo, CallError> {
    let sol_client = solana_client().await;

    let mint_account = SolanaClient::derive_account(
        sol_client.schnorr_canister.clone(),
        sol_client.chainkey_name.clone(),
        req.name.to_string(),
    )
    .await;
    log!(
        DEBUG,
        "[service::create_mint] mint_account from schonnor chainkey: {:?} ",
        mint_account.to_string(),
    );

    let mut mint_account_info = if let Some(account_info) =
        read_state(|s| s.token_mint_accounts.get(&req.token_id).cloned())
    {
        account_info
    } else {
        let new_account_info = AccountInfo {
            account: mint_account.to_string(),
            retry: 0,
            signature: None,
            status: AccountStatus::Unknown,
        };

        new_account_info
    };

    log!(
        DEBUG,
        "[service::create_mint] mint_account_info from solana route: {:?} ",
        mint_account.to_string(),
    );

    if matches!(mint_account_info.status, AccountStatus::Confirmed) {
        return Ok(mint_account_info);
    }

    // only execute creating for new mint account
    if matches!(mint_account_info.status, AccountStatus::Unknown)
        && matches!(mint_account_info.signature, None)
    {
        let signature = sol_call::create_mint_account(mint_account, req.clone()).await?;
        log!(
            DEBUG,
            "[[service::create_mint] create_mint_account signature: {:?} ",
            signature.to_string(),
        );

        // update signature
        mint_account_info.signature = Some(signature);
        mint_account_info.retry += 1;
        mutate_state(|s| {
            s.token_mint_accounts
                .insert(req.token_id.to_string(), mint_account_info.clone())
        });
    }

    // query mint account from solana
    let mint_account_on_chain = sol_client
        .get_account_info(mint_account.to_string())
        .await
        .map_err(|e| CallError {
            method: "[service::create_mint] get_account_info".to_string(),
            reason: Reason::CanisterError(e.to_string()),
        })?;
    log!(
        DEBUG,
        "[service::create_mint] {} mint_account_info from solana : {:?} ",
        mint_account.to_string(),
        mint_account_on_chain,
    );
    match mint_account_on_chain {
        None => {
            log!(
                DEBUG,
             "[service::create_mint] not found mint_account_info from solana for {:?} , pls check the mint_account info and retry",
             mint_account.to_string(),
         );
        }
        Some(_) => {
            mint_account_info.status = AccountStatus::Confirmed;
            //update account info
            mutate_state(|s| {
                s.token_mint_accounts
                    .insert(req.token_id.to_string(), mint_account_info.clone())
            });
        }
    }

    Ok(mint_account_info)
}

#[update(guard = "is_admin")]
pub async fn derive_aossicated_account(
    owner: String,
    token_mint: String,
) -> Result<String, CallError> {
    let to_account_pk = Pubkey::from_str(&owner).expect("Invalid to_account address");
    let token_mint_pk = Pubkey::from_str(&token_mint).expect("Invalid token mint address");
    let associated_account = get_associated_token_address_with_program_id(
        &to_account_pk,
        &token_mint_pk,
        &token22_program_id(),
    );
    log!(
        DEBUG,
        "[ticket::create_associated_account] get_associated_token_address_with_program_id : {:?}",
        associated_account
    );

    Ok(associated_account.to_string())
}

#[query]
pub async fn query_aossicated_account(
    owner: Owner,
    token_mint: MintAccount,
) -> Option<AccountInfo> {
    read_state(|s| s.associated_accounts.get(&(owner, token_mint)).cloned())
}
#[query]
pub async fn query_aossicated_account_address(
    owner: Owner,
    token_mint: MintAccount,
) -> Option<String> {
    read_state(|s| {
        s.associated_accounts
            .get(&(owner, token_mint))
            .map(|ata| ata.account.to_string())
    })
}

#[update(guard = "is_admin")]
pub async fn create_aossicated_account(
    owner: String,
    token_mint: String,
) -> Result<AccountInfo, CallError> {
    let to_account_pk = Pubkey::from_str(&owner).expect("Invalid to_account address");
    let token_mint_pk = Pubkey::from_str(&token_mint).expect("Invalid token mint address");
    let associated_account = get_associated_token_address_with_program_id(
        &to_account_pk,
        &token_mint_pk,
        &token22_program_id(),
    );
    log!(
        DEBUG,
        "[ticket::create_associated_account] get_associated_token_address_with_program_id : {:?}",
        associated_account
    );
    let mut ata_info = if let Some(account_info) = read_state(|s| {
        s.associated_accounts
            .get(&(owner.to_string(), token_mint.to_string()))
            .cloned()
    }) {
        account_info
    } else {
        let new_account_info = AccountInfo {
            account: associated_account.to_string(),
            retry: 0,
            signature: None,
            status: AccountStatus::Unknown,
        };

        new_account_info
    };
    log!(
        DEBUG,
        "[ticket::create_associated_account] ata_info from solana route : {:?}",
        associated_account
    );

    if matches!(ata_info.status, AccountStatus::Confirmed) {
        return Ok(ata_info);
    }

    // only execute creating for new ata
    if matches!(ata_info.status, AccountStatus::Unknown) && matches!(ata_info.signature, None) {
        let signature = sol_call::create_ata(owner.to_string(), token_mint.to_string()).await?;
        log!(
            DEBUG,
            "[[service::create_aossicated_account] create_aossicated_account signature: {:?} ",
            signature.to_string(),
        );
        // update signature
        ata_info.signature = Some(signature);
        ata_info.retry += 1;
        mutate_state(|s| {
            s.associated_accounts.insert(
                (owner.to_string(), token_mint.to_string()),
                ata_info.clone(),
            )
        });
    }

    // query mint account from solana
    let sol_client = solana_client().await;
    let ata_on_chain = sol_client
        .get_account_info(associated_account.to_string())
        .await
        .map_err(|e| CallError {
            method: "[service::create_aossicated_account] get_account_info".to_string(),
            reason: Reason::CanisterError(e.to_string()),
        })?;
    log!(
        DEBUG,
        "[service::create_aossicated_account] {} mint_account_info from solana : {:?} ",
        associated_account.to_string(),
        ata_on_chain,
    );
    match ata_on_chain {
        None => {
            log!(
                DEBUG,
                "[service::create_mint] not found ata info from solana for {:?} , pls check the mint_account info and retry",
                associated_account.to_string(),
            );
        }
        Some(_) => {
            ata_info.status = AccountStatus::Confirmed;
            //update account info
            mutate_state(|s| {
                s.associated_accounts.insert(
                    (owner.to_string(), token_mint.to_string()),
                    ata_info.clone(),
                )
            });
        }
    }
    Ok(ata_info)
}

#[query]
pub async fn mint_token_status(ticket_id: String) -> Result<MintTokenStatus, CallError> {
    let req = read_state(|s| s.mint_token_requests.get(&ticket_id).cloned());
    match req {
        None => Err(CallError {
            method: "[service::mint_token_status] mint_token_status".to_string(),
            reason: Reason::CanisterError(format!(
                "Not found ticket({}) MintTokenStatus",
                ticket_id.to_string()
            )),
        }),

        Some(req) => Ok(req.status),
    }
}

#[query]
pub async fn mint_token_req(ticket_id: String) -> Result<MintTokenRequest, CallError> {
    let req = read_state(|s| s.mint_token_requests.get(&ticket_id).cloned());
    match req {
        None => Err(CallError {
            method: "[service::mint_token_req] mint_token_req".to_string(),
            reason: Reason::CanisterError(format!(
                "Not found ticket({}) mint token request",
                ticket_id.to_string()
            )),
        }),

        Some(req) => Ok(req),
    }
}

#[update(guard = "is_admin")]
pub async fn mint_token(req: MintTokenRequest) -> Result<MintTokenStatus, CallError> {
    let mut req =
        if let Some(req) = read_state(|s| s.mint_token_requests.get(&req.ticket_id).cloned()) {
            req
        } else {
            log!(
                DEBUG,
                "[service::mint_to] not found mint token req for ticket: {:?} ",
                req.ticket_id
            );
            req
        };
    log!(DEBUG, "[service::mint_to] mint token request: {:?} ", req);

    match req.status {
        MintTokenStatus::Unknown | MintTokenStatus::TxFailed { .. } => {
            match req.signature.clone() {
                None => {
                    // new mint req
                    let sig = sol_call::mint_to(
                        req.associated_account.clone(),
                        req.amount,
                        req.token_mint.clone(),
                    )
                    .await?;

                    // update signature
                    req.signature = Some(sig.to_string());
                    mutate_state(|s| {
                        s.update_mint_token_req(req.ticket_id.to_owned(), req.clone())
                    });
                    // query get_signature_status
                    let tx_status_vec =
                        sol_call::get_signature_status(vec![sig.to_string()]).await?;
                    tx_status_vec.first().map(|tx_status| {
                        log!(
                            DEBUG,
                            "[service::mint_to] {} status : {:?} ",
                            sig.to_string(),
                            tx_status,
                        );
                        if let Some(status) = &tx_status.confirmation_status {
                            if matches!(status, TransactionConfirmationStatus::Finalized) {
                                // update req status
                                req.status = MintTokenStatus::Finalized {
                                    signature: sig.to_string(),
                                };
                                mutate_state(|s| {
                                    s.update_mint_token_req(req.ticket_id.to_string(), req.clone())
                                });
                            }
                        }
                    });
                }
                Some(sig) => {
                    // let sig = req.signature.clone().unwrap().to_string();
                    let tx_status_vec =
                        sol_call::get_signature_status(vec![sig.to_string()]).await?;
                    tx_status_vec.first().map(|tx_status| {
                        log!(
                            DEBUG,
                            "[service::mint_to] {} status : {:?} ",
                            sig.to_string(),
                            tx_status,
                        );
                        if let Some(status) = &tx_status.confirmation_status {
                            if matches!(status, TransactionConfirmationStatus::Finalized) {
                                // update req status
                                req.status = MintTokenStatus::Finalized {
                                    signature: sig.to_string(),
                                };
                                mutate_state(|s| {
                                    s.update_mint_token_req(req.ticket_id.to_string(), req.clone())
                                });
                            }
                        }
                    });
                }
            }
        }
        MintTokenStatus::Finalized { .. } => {
            log!(DEBUG, "[service::mint_to] {:?} already finalized !", req);
            // remove ticket from quene,if exsits
            remove_ticket_from_quene(req.ticket_id.to_string()).await;
            return Ok(req.status);
        }
    }

    Ok(req.status)
}

#[query]
fn get_ticket_from_queue(ticket_id: String) -> Option<(u64, Ticket)> {
    read_state(|s| {
        s.tickets_queue
            .iter()
            .find(|(_seq, ticket)| ticket.ticket_id.eq(&ticket_id))
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

#[update(guard = "is_admin")]
pub async fn remove_ticket_from_quene(ticket_id: String) -> Option<Ticket> {
    mutate_state(|s| {
        let ticket = s
            .tickets_queue
            .iter()
            .find(|(_seq, ticket)| ticket.ticket_id.eq(&ticket_id));

        match ticket {
            None => None,
            Some((seq, _ticket)) => s.tickets_queue.remove(&seq),
        }
    })
}

#[update(guard = "is_admin")]
pub async fn update_token_metadata(req: TokenInfo) -> Result<String, CallError> {
    // token_mint must be exists
    match read_state(|s| s.token_mint_accounts.get(&req.token_id).cloned()) {
        None => {
            return Err(CallError {
                method: "[service::update_token_metadata] update_token_metadata".to_string(),
                reason: Reason::CanisterError(format!(
                    "{} token mint account not exists!",
                    req.token_id
                )),
            });
        }
        Some(account_info) => {
            let signature =
                sol_call::update_token_metadata(account_info.account, req.clone()).await?;
            log!(
                DEBUG,
                "[service::update_token_metadata] update_token_metadata signature: {:?} ",
                signature.to_string(),
            );
            //TODO: check signature status
            // update update_token_metadata result to state
            mutate_state(|s| {
                // update the token info
                s.tokens.get_mut(&req.token_id).map(|token| {
                    token.name = req.name;
                    token.symbol = req.symbol;
                    token.decimals = req.decimals;
                    token.icon = Some(req.uri);
                });
            });

            // remove update_token req from queue
            if let Some(_update_token) =
                read_state(|s| s.update_token_queue.get(&req.token_id).cloned())
            {
                // update update_token_metadata result to state
                mutate_state(|s| {
                    // remove the updated token from queue
                    s.update_token_queue.remove(&req.token_id)
                });
            }
            Ok(signature)
        }
    }
}

#[update(guard = "is_admin")]
pub async fn transfer_to(to_account: String, amount: u64) -> Result<String, CallError> {
    sol_call::transfer_to(to_account, amount).await
}

#[query]
pub fn get_redeem_fee(chain_id: ChainId) -> Option<u128> {
    read_state(|s| s.get_fee(chain_id))
}

#[update]
async fn generate_ticket(args: GenerateTicketReq) -> Result<GenerateTicketOk, GenerateTicketError> {
    ticket::generate_ticket(args).await
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
    log!(DEBUG, "successfully resend {} tickets", tickets_sz);
    Ok(())
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

        for entry in export_logs(&ic_solana::logs::ERROR_BUF) {
            entries.push(entry);
        }
        for entry in export_logs(&ic_solana::logs::DEBUG_BUF) {
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
