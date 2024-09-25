use std::time::Duration;

use crate::*;
use business::update_balance::{process_update_balance_jobs, update_balance_and_generate_ticket};
use external::{ckbtc, custom::generate_ticket};
use ic_canisters_http_types::{HttpRequest, HttpResponse};
use ic_cdk::post_upgrade;
use ic_cdk_timers::set_timer_interval;
use icrc_ledger_types::icrc1::account::Account;
use state::{
    extend_ticket_records, get_settings, get_ticket_records, get_utxo_records,
    mutate_settings, Settings, TicketRecord, UtxoRecord,
};
use std::result::Result;
use utils::nat_to_u128;

pub fn is_controller() -> Result<(), String> {
    if ic_cdk::api::is_controller(&ic_cdk::caller()) {
        Ok(())
    } else {
        Err("caller is not controller".to_string())
    }
}

#[ic_cdk::init]
pub async fn init(args: lifecycle::init::InitArgs) {
    lifecycle::init::init(args);

    set_timer_interval(
        Duration::from_secs(5 * 60),
        process_update_balance_jobs,
    );
}

#[query]
pub fn get_identity_by_osmosis_account_id(
    osmosis_account_id: String,
) -> Result<Account, String> {
    let address_data = AddressData::try_from(osmosis_account_id.as_str())
        .map_err(|e| Errors::AccountIdParseError(osmosis_account_id.clone(), e.to_string()))
        .map_err(|e| e.to_string())?;

    Ok(Account {
        owner: ic_cdk::api::id(),
        subaccount: Some(address_data.into()),
    })
}

#[update]
pub async fn get_btc_mint_address(
    osmosis_account_id: String,
) -> Result<String, String> {
    let address_data = AddressData::try_from(osmosis_account_id.as_str())
        .map_err(|e| Errors::AccountIdParseError(osmosis_account_id.clone(), e.to_string()))
        .map_err(|e| e.to_string())?;

    get_btc_address(GetBtcAddressArgs {
        owner: None,
        subaccount: Some(address_data.into()),
    })
    .await
    .map_err(|e| e.to_string())
}

#[query]
pub fn query_utxo_records(osmosis_account_id: String) -> Vec<UtxoRecord> {
    get_utxo_records(osmosis_account_id)
}

#[query]
pub fn query_ticket_records(osmosis_account_id: String) -> Vec<TicketRecord> {
    get_ticket_records(osmosis_account_id)
}

#[query]
pub async fn query_settings() -> Settings {
    get_settings()
}

#[query(guard = "is_controller")]
pub async fn query_scheduled_osmosis_account_id_list() -> Vec<String> {
    state::get_scheduled_osmosis_account_id_list()
}

#[query(hidden = true)]
fn http_request(req: HttpRequest) -> HttpResponse {
    if ic_cdk::api::data_certificate().is_none() {
        ic_cdk::trap("update call rejected");
    }
    omnity_types::ic_log::http_request(req)
}

#[update(guard = "is_controller")]
pub async fn update_settings(
    ckbtc_ledger_principal: Option<Principal>,
    ckbtc_minter_principal: Option<Principal>,
    icp_customs_principal: Option<Principal>,
    token_id: Option<String>,
    target_chain_id: Option<String>,
) {
    state::mutate_settings(|settings| {
        if let Some(ckbtc_ledger_principal) = ckbtc_ledger_principal {
            settings.ckbtc_ledger_principal = ckbtc_ledger_principal;
        }
        if let Some(ckbtc_minter_principal) = ckbtc_minter_principal {
            settings.ckbtc_minter_principal = ckbtc_minter_principal;
        }
        if let Some(icp_customs_principal) = icp_customs_principal {
            settings.icp_customs_principal = icp_customs_principal;
        }
        if let Some(token_id) = token_id {
            settings.token_id = token_id;
        }
        if let Some(target_chain_id) = target_chain_id {
            settings.target_chain_id = target_chain_id;
        }
    });
}

#[update]
pub async fn generate_ticket_from_subaccount(
    osmosis_account_id: String,
) -> Result<TicketId, String> {
    let address_data = AddressData::try_from(osmosis_account_id.as_str())
        .map_err(|e| Errors::AccountIdParseError(osmosis_account_id.clone(), e.to_string()))
        .map_err(|e| e.to_string())?;

    let subaccount: Subaccount = address_data.into();

    let balance = ckbtc::balance_of(Account {
        owner: ic_cdk::api::id(),
        subaccount: Some(subaccount.clone()),
    })
    .await
    .map_err(|e| e.to_string())?;

    approve_ckbtc_for_icp_custom(Some(subaccount), balance.clone())
        .await
        .map_err(|e| e.to_string())?;

    let setting = get_settings();

    let ticket_amount = nat_to_u128(balance)
        .and_then(|u| u.checked_sub(2 * CKBTC_FEE as u128).ok_or(Errors::CustomError("overflow".to_string())))
        .map_err(|e| e.to_string())?;
    let ticket_id = generate_ticket(
        setting.token_id,
        setting.target_chain_id,
        ticket_amount,
        subaccount,
    )
    .await
    .map_err(|e| e.to_string())?;

    extend_ticket_records(osmosis_account_id, vec![TicketRecord {
            ticket_id: ticket_id.clone(),
            minted_utxos: vec![],
        }]);
    Ok(ticket_id)
}

#[update]
pub async fn update_balance_after_finalization(osmosis_account_id: String) {

    mutate_settings(|s| {
        s.update_balances_jobs.push(UpdateBalanceJob::new(osmosis_account_id.clone()))
    });

    log!(INFO, "Created update balance job for osmosis account id: {}", osmosis_account_id);

}

#[update(guard = "is_controller")]
pub async fn trigger_update_balance(osmosis_account_id: String) -> Result<TicketId, String> {
    update_balance_and_generate_ticket(osmosis_account_id).await
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
