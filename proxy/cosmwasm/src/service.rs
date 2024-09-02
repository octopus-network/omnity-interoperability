use crate::*;
use candid::Nat;
use external::{
    ckbtc,
    custom::{generate_ticket, TARGET_CHAIN_ID, TOKEN_ID},
};
use ic_cdk::post_upgrade;
use icrc_ledger_types::icrc1::{account::Account, transfer::BlockIndex};
use state::{
    contains_executed_transaction_index, get_btc_transport_records, get_ckbtc_ledger_principal, get_icp_custom_principal, insert_btc_transport_records, insert_executed_transaction_index, set_ckbtc_minter_principal, BtcTransportInfo, BtcTransportRecord
};
use utils::{nat_to_u128, nat_to_u64};

#[ic_cdk::init]
pub async fn init(args: lifecycle::init::InitArgs) {
    lifecycle::init::init(args);
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

#[query]
pub fn query_btc_transport_info(osmosis_account_id: String) -> Vec<BtcTransportRecord> {
    get_btc_transport_records(osmosis_account_id)
}

#[ic_cdk::update]
pub fn set_trigger_principal(principal: Principal) -> std::result::Result<(), String> {
    assert!(
        ic_cdk::api::is_controller(&ic_cdk::caller()),
        "Caller is not controller"
    );
    state::set_trigger_principal(principal);
    Ok(())
}

#[ic_cdk::query]
pub async fn query_status() -> (Principal, Principal) {
    (get_icp_custom_principal(), get_ckbtc_ledger_principal())
}

// #[ic_cdk::update]
// pub async fn trigger_generate_ticket(
//     subaccount: Subaccount,
//     amount: u64,
// ) -> std::result::Result<String, String> {
//     let a = Nat::from(amount);

//     approve_ckbtc_for_icp_custom(Some(subaccount.clone()), a)
//         .await
//         .map_err(|e| e.to_string())?;

//     let ticket_id = generate_ticket(
//         TOKEN_ID.to_string(),
//         TARGET_CHAIN_ID.to_string(),
//         nat_to_u128(amount - Nat::from(2_u8) * Nat::from(10_u32)),
//         subaccount,
//     )
//     .await
//     .map_err(|e| e.to_string())?;

//     Ok(ticket_id)
// }

#[ic_cdk::update]
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

    // let minted_success_utxo = result
    // .iter()
    // .filter_map(|e| {
    //     if matches!(e, UtxoStatus::Minted { .. }) {
    //         Some(e)
    //     } else {
    //         None
    //     }
    // }).collect();
    let mut minted_success_utxo_status: Vec<BtcTransportRecord> = vec![];
    let mut minted_success_amount = 0_u64;
    for e in result {
        match e {
            UtxoStatus::ValueTooSmall(_) | UtxoStatus::Tainted(_) | UtxoStatus::Checked(_) => {
                continue;
            }
            UtxoStatus::Minted {
                block_index,
                minted_amount,
                utxo,
            } => {
                minted_success_utxo_status.push(
                    BtcTransportRecord {
                        block_index: block_index,
                        minted_amount: minted_amount,
                        utxo: utxo,
                        ticket_id: None
                    }
                );
                minted_success_amount += minted_amount;
            }
        }
    }

    // let mut btc_transport_record_list = get_btc_transport_records(osmosis_account_id);
    // btc_transport_record_list.extend(minted_success_utxo_status);
    // insert_btc_transport_records(osmosis_account_id, btc_transport_record_list);

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

    let mut btc_transport_record_list = get_btc_transport_records(osmosis_account_id.clone());
    btc_transport_record_list.extend(minted_success_utxo_status.iter().map(|e| {
        BtcTransportRecord {
            block_index: e.block_index,
            minted_amount: e.minted_amount,
            utxo: e.utxo.clone(),
            ticket_id: Some(ticket_id.clone())
        }
    }));
    insert_btc_transport_records(osmosis_account_id, btc_transport_record_list);

    // todo save ticket contained utxos

    // todo should save transported utxo
    // insert_executed_transaction_index(nat_to_u64(block_index), ticket_id);


    Ok(())
}

#[ic_cdk::update]
pub async fn trigger_transaction(block_index: BlockIndex) -> std::result::Result<(), String> {
    assert_eq!(
        ic_cdk::api::caller(),
        state::get_trigger_principal(),
        "Caller is not trigger principal"
    );
    assert!(
        !contains_executed_transaction_index(nat_to_u64(block_index.clone())),
        "Transaction already executed"
    );

    let transaction_response = get_ckbtc_transaction(block_index.clone())
        .await
        .map_err(|e| e.to_string())?;
    assert_eq!(
        transaction_response.transactions.len(),
        1,
        "Expected 1 transaction, got {}",
        transaction_response.transactions.len()
    );

    let transaction = transaction_response.transactions[0].clone();
    assert!(
        transaction.transfer.is_some(),
        "Expected transfer transaction, got {:?}",
        transaction
    );

    let transfer = transaction.transfer.unwrap();
    assert_eq!(
        transfer.to.owner,
        ic_cdk::api::id(),
        "Transaction not for this canister"
    );

    approve_ckbtc_for_icp_custom(transfer.to.subaccount.clone(), transfer.amount.clone())
        .await
        .map_err(|e| e.to_string())?;

    let ticket_id = generate_ticket(
        TOKEN_ID.to_string(),
        TARGET_CHAIN_ID.to_string(),
        nat_to_u128(transfer.amount - Nat::from(2_u8) * transfer.fee.unwrap_or(Nat::from(0_u8))),
        transfer.to.subaccount.unwrap(),
    )
    .await
    .map_err(|e| e.to_string())?;

    insert_executed_transaction_index(nat_to_u64(block_index), ticket_id);

    Ok(())
}

#[post_upgrade]
fn post_upgrade() {
    let p = Principal::from_text("mqygn-kiaaa-aaaar-qaadq-cai").unwrap();
    set_ckbtc_minter_principal(p);
}

ic_cdk::export_candid!();
