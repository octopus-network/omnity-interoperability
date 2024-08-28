use crate::*;
use candid::Nat;
use external::custom::{generate_ticket, TARGET_CHAIN_ID, TOKEN_ID};
use icrc_ledger_types::icrc1::{account::Account, transfer::BlockIndex};
use state::{contains_executed_transaction_index, insert_executed_transaction_index};
use utils::{nat_to_u128, nat_to_u64};

#[ic_cdk::init]
pub async fn init(args: lifecycle::init::InitArgs) {
    lifecycle::init::init(args);
}

#[query]
pub fn get_identity_by_osmosis_account_id(osmosis_account_id: String) -> std::result::Result<Account, String> {
    let address_data = AddressData::try_from(osmosis_account_id.as_str())
        .map_err(|e| Errors::AccountIdParseError(osmosis_account_id.clone(), e.to_string())).map_err(|e| e.to_string())?;

    Ok(Account {
        owner: ic_cdk::api::id(),
        subaccount: Some(address_data.into()),
    })
}

#[ic_cdk::update]
pub fn set_trigger_principal(principal: Principal) -> std::result::Result<(), String> {
    assert!(ic_cdk::api::is_controller(&ic_cdk::caller()), "Caller is not controller");
    state::set_trigger_principal(principal);
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

    let transaction_response = get_ckbtc_transaction(block_index.clone()).await.map_err(|e| e.to_string())?;
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

    approve_ckbtc_for_icp_custom(transfer.to.subaccount.clone(), transfer.amount.clone()).await.map_err(|e| e.to_string())?;

    let ticket_id = generate_ticket(
        TOKEN_ID.to_string(),
        TARGET_CHAIN_ID.to_string(),
        nat_to_u128(transfer.amount - Nat::from(2_u8) * transfer.fee.unwrap_or(Nat::from(0_u8))),
        transfer.to.subaccount.unwrap(),
    )
    .await.map_err(|e| e.to_string())?;

    insert_executed_transaction_index(nat_to_u64(block_index), ticket_id);

    Ok(())
}

ic_cdk::export_candid!();