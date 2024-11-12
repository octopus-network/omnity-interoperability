use std::time::Duration;

use candid::Principal;
use ic_canisters_http_types::{HttpRequest, HttpResponse};
use ic_cdk::{post_upgrade, query, update};
use ic_cdk_timers::set_timer_interval;
use icrc_ledger_types::icrc1::account::Subaccount;
use omnity_types::TicketId;
use crate::{business::update_balance::{process_update_balance_jobs, update_balance_and_generate_ticket}, external::{ckbtc::{self, approve_ckbtc_for_icp_custom, get_btc_address, GetBtcAddressArgs, CKBTC_FEE}, custom::generate_ticket}, lifecycle, state::{self, extend_ticket_records, get_state, get_ticket_records, get_utxo_records, mutate_state, State}, types::{OmnityAccount, TicketRecord, UpdateBalanceJob, UtxoRecord}, utils::nat_to_u128, Errors};
use icrc_ledger_types::icrc1::account::Account as IcrcAccount;
pub use omnity_types::ic_log::{INFO, ERROR};
pub use ic_canister_log::log;

pub fn is_controller() -> std::result::Result<(), String> {
    if ic_cdk::api::is_controller(&ic_cdk::caller()) {
        Ok(())
    } else {
        Err("caller is not controller".to_string())
    }
}

#[ic_cdk::init]
pub async fn init(args: lifecycle::InitArgs) {
    lifecycle::init(args);

    set_timer_interval(
        Duration::from_secs(5 * 60),
        process_update_balance_jobs,
    );
}

#[update]
pub async fn query_btc_mint_address_by_omnity_account(
    omnity_account_id: OmnityAccount
) -> crate::errors::Result<String> {
    let get_btc_address_args = GetBtcAddressArgs {
        owner: None,
        subaccount: Some(omnity_account_id.get_mapping_subaccount()),
    };
    get_btc_address(
        get_btc_address_args
    )
    .await
}

#[query]
pub fn query_utxo_records(omnity_account_id: OmnityAccount) -> Vec<UtxoRecord> {
    get_utxo_records(omnity_account_id)
}

#[query]
pub fn query_ticket_records(omnity_account_id: OmnityAccount) -> Vec<TicketRecord> {
    get_ticket_records(omnity_account_id)
}

#[query(guard = "is_controller")]
pub fn query_state() -> crate::errors::Result<State> {
    Ok(get_state())
}

#[query(hidden = true)]
fn http_request(req: HttpRequest) -> HttpResponse {
    omnity_types::ic_log::http_request(req)
}


#[update(guard = "is_controller")]
pub async fn update_settings(
    ckbtc_ledger_principal: Option<Principal>,
    ckbtc_minter_principal: Option<Principal>,
    icp_customs_principal: Option<Principal>,
    token_id: Option<String>,
) {
    state::mutate_state(|state| {
        if let Some(ckbtc_ledger_principal) = ckbtc_ledger_principal {
            state.ckbtc_ledger_principal = ckbtc_ledger_principal;
        }
        if let Some(ckbtc_minter_principal) = ckbtc_minter_principal {
            state.ckbtc_minter_principal = ckbtc_minter_principal;
        }
        if let Some(icp_customs_principal) = icp_customs_principal {
            state.icp_customs_principal = icp_customs_principal;
        }
        if let Some(token_id) = token_id {
            state.token_id = token_id;
        }
    });
}

#[update]
pub async fn generate_ticket_from_subaccount(
    omnity_account: OmnityAccount,
) -> Result<TicketId, String> {
    let subaccount: Subaccount = omnity_account.get_mapping_subaccount();

    let balance = ckbtc::balance_of(IcrcAccount {
        owner: ic_cdk::api::id(),
        subaccount: Some(subaccount.clone()),
    })
    .await
    .map_err(|e| e.to_string())?;

    approve_ckbtc_for_icp_custom(Some(subaccount), balance.clone())
        .await
        .map_err(|e| e.to_string())?;

    let state = get_state();

    let ticket_amount = nat_to_u128(balance)
        .and_then(|u| u.checked_sub(2 * CKBTC_FEE as u128).ok_or(Errors::CustomError("overflow".to_string())))
        .map_err(|e| e.to_string())?;
    let ticket_id = generate_ticket(
        state.token_id,
        ticket_amount,
        omnity_account.clone(),
    )
    .await
    .map_err(|e| e.to_string())?;

    extend_ticket_records(omnity_account, vec![TicketRecord {
            ticket_id: ticket_id.clone(),
            minted_utxos: vec![],
        }]);
    Ok(ticket_id)
}

#[update]
pub async fn update_balance_after_finalization(omnity_account: OmnityAccount) {

    mutate_state(|s| {
        s.update_balances_jobs.push(UpdateBalanceJob::new(omnity_account.clone()))
    });

    log!(INFO, "Created update balance job for omnity account: {:?}", omnity_account);
}

#[update(guard = "is_controller")]
pub async fn trigger_update_balance(omnity_account: OmnityAccount) -> Result<TicketId, String> {
    update_balance_and_generate_ticket(omnity_account).await
}

#[post_upgrade]
fn post_upgrade() {
    set_timer_interval(
        Duration::from_secs(5 * 60),
        process_update_balance_jobs,
    );

    log!(INFO, "Finish Upgrade current version: {}", env!("CARGO_PKG_VERSION"));
}

ic_cdk::export_candid!();