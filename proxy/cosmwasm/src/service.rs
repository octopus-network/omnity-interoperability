use crate::*;
use candid::Nat;
use external::{
    ckbtc,
    custom::{generate_ticket, TARGET_CHAIN_ID, TOKEN_ID},
};
use ic_canisters_http_types::{HttpRequest, HttpResponse};
use ic_cdk::post_upgrade;
use icrc_ledger_types::icrc1::account::Account;
use itertools::Itertools;
use omnity_types::log::{init_log, StableLogWriter};
use state::{
    extend_ticket_records, get_settings, get_ticket_records, get_utxo_records, init_stable_log, insert_utxo_records, MintedUtxo, Settings, TicketRecord, UtxoRecord
};
use utils::nat_to_u128;

#[ic_cdk::init]
pub async fn init(args: lifecycle::init::InitArgs) {
    lifecycle::init::init(args);

    init_log(Some(init_stable_log()));
}

#[query]
pub fn get_identity_by_osmosis_account_id(
    osmosis_account_id: String,
) -> std::result::Result<Account, String> {
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
) -> std::result::Result<String, String> {
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

#[query(hidden = true)]
fn http_request(req: HttpRequest) -> HttpResponse {
    StableLogWriter::http_request(req)
}

#[update]
pub async fn update_settings(
    ckbtc_ledger_principal: Option<Principal>,
    ckbtc_minter_principal: Option<Principal>,
    icp_customs_principal: Option<Principal>,
) {
    state::mutate_settings(|settings| {
        if ckbtc_ledger_principal.is_some() {
            settings.ckbtc_ledger_principal = ckbtc_ledger_principal.unwrap();
        }
        if ckbtc_minter_principal.is_some() {
            settings.ckbtc_minter_principal = ckbtc_minter_principal.unwrap();
        }
        if icp_customs_principal.is_some() {
            settings.icp_customs_principal = icp_customs_principal.unwrap();
        }
    });
}

#[update]
pub async fn generate_ticket_from_subaccount(
    osmosis_account_id: String,
) -> std::result::Result<TicketId, String> {
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

    let ticket_id = generate_ticket(
        TOKEN_ID.to_string(),
        TARGET_CHAIN_ID.to_string(),
        nat_to_u128(balance - Nat::from(2_u8) * Nat::from(10_u8)),
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
pub async fn trigger_update_balance(osmosis_account_id: String) -> std::result::Result<(), String> {
    let address_data = AddressData::try_from(osmosis_account_id.as_str())
        .map_err(|e| Errors::AccountIdParseError(osmosis_account_id.clone(), e.to_string()))
        .map_err(|e| e.to_string())?;

    let subaccount: Subaccount = address_data.into();

    let result = ckbtc::update_balance(UpdateBalanceArgs {
        owner: None,
        subaccount: Some(subaccount.clone()),
    })
    .await
    .map_err(|e| e.to_string())?;

    log::info!(
        "osmosis account id: {} ,update_balance result: {:?}",
        osmosis_account_id,
        result
    );

    let minted_success_utxo_status = result
        .iter()
        .filter_map(|e| match e {
            UtxoStatus::Minted {
                block_index,
                minted_amount,
                utxo,
            } => Some(UtxoRecord {
                minted_utxo: MintedUtxo {
                    block_index: block_index.clone(),
                    minted_amount: minted_amount.clone(),
                    utxo: utxo.clone(),
                },
                ticket_id: None,
            }),
            _ => None,
        })
        .collect_vec();

    let minted_success_amount = minted_success_utxo_status
        .iter()
        .map(|e| e.minted_utxo.minted_amount)
        .sum::<u64>();
    let block_index_set = minted_success_utxo_status
        .iter()
        .map(|e| e.minted_utxo.block_index)
        .collect::<std::collections::HashSet<u64>>();

    let mut utxo_record_list = get_utxo_records(osmosis_account_id.clone());
    utxo_record_list.extend(minted_success_utxo_status.clone());
    insert_utxo_records(osmosis_account_id.clone(), utxo_record_list);

    approve_ckbtc_for_icp_custom(Some(subaccount), minted_success_amount.into())
        .await
        .map_err(|e| e.to_string())?;

    let ticket_id = generate_ticket(
        TOKEN_ID.to_string(),
        TARGET_CHAIN_ID.to_string(),
        nat_to_u128(minted_success_amount - Nat::from(2_u8) * Nat::from(10_u8)),
        subaccount,
    )
    .await
    .map_err(|e| e.to_string())?;

    log::info!(
        "osmosis account id: {} ,generate_ticket result: {:?}",
        osmosis_account_id,
        ticket_id
    );

    let mut utxo_record_list = get_utxo_records(osmosis_account_id.clone());
    utxo_record_list.iter_mut().for_each(|e| {
        if block_index_set.contains(&e.minted_utxo.block_index) {
            e.ticket_id = Some(ticket_id.clone());
        }
    });
    insert_utxo_records(osmosis_account_id.clone(), utxo_record_list.clone());

    let belong_ticket_utxos = utxo_record_list
        .iter()
        .filter(|e| e.ticket_id.is_some())
        .map(|e| e.minted_utxo.clone())
        .collect_vec(); 

    extend_ticket_records(osmosis_account_id, vec![TicketRecord {
        ticket_id: ticket_id.clone(),
        minted_utxos: belong_ticket_utxos,
    }]);
    // insert_ticket_records(ticket_id, utxo_record_list);

    Ok(())
}

#[post_upgrade]
fn post_upgrade() {
    init_log(Some(init_stable_log()));
}

ic_cdk::export_candid!();
