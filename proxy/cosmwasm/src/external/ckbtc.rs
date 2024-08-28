use candid::Nat;
use icrc_ledger_types::{
    icrc1::account::{Account, Subaccount},
    icrc2::approve::ApproveArgs,
    icrc3::transactions::{GetTransactionsRequest, GetTransactionsResponse},
};

use crate::*;

pub async fn approve_ckbtc_for_icp_custom(
    subaccount: Option<Subaccount>,
    amount: Nat,
) -> Result<()> {
    let ckbtc_ledger_principal = state::get_ckbtc_ledger_principal();
    let client = ICRC1Client {
        runtime: CdkRuntime,
        ledger_canister_id: ckbtc_ledger_principal,
    };
    let spender = Account {
        owner: state::get_icp_custom_principal(),
        subaccount: None,
    };
    let approve_args = ApproveArgs {
        from_subaccount: subaccount,
        spender: spender,
        amount: amount,
        expected_allowance: None,
        expires_at: None,
        fee: None,
        memo: None,
        created_at_time: None,
    };
    client
        .approve(approve_args)
        .await
        .map_err(|(code, msg)| {
            Errors::CanisterCallError(
                ckbtc_ledger_principal.to_string(),
                "approve".to_string(),
                format!("{:?}", code),
                msg,
            )
        })?
        .map_err(|e| Errors::CustomError(format!("{:?}", e)))?;

    Ok(())
}

pub async fn get_ckbtc_transaction(block_index: BlockIndex) -> Result<GetTransactionsResponse> {
    let ckbtc_ledger_principal = state::get_ckbtc_ledger_principal();
    let request = GetTransactionsRequest {
        start: block_index,
        length: 1_u8.into(),
    };
    let result: (Result<GetTransactionsResponse>,) =
        ic_cdk::api::call::call(ckbtc_ledger_principal, "get_transactions", (request,))
            .await
            .map_err(|(code, msg)| {
                Errors::CanisterCallError(
                    ckbtc_ledger_principal.to_string(),
                    "get_transactions".to_string(),
                    format!("{:?}", code),
                    msg,
                )
            })?;

    result.0
}

// pub async fn get_account_transactions(
//     arg: GetAccountTransactionsArgs,
// ) -> Result<GetAccountTransactionsResult> {
//     let ckbtc_index_principal = state::get_ckbtc_ledger_principal();
//     let result: (std::result::Result<GetAccountTransactionsResult, Errors>,) =
//         ic_cdk::api::call::call(ckbtc_index_principal, "get_account_transactions", (arg,))
//             .await
//             .map_err(|(code, message)| {
//                 Errors::CanisterCallError(
//                     ckbtc_index_principal.to_string(),
//                     "get_account_transactions".to_string(),
//                     format!("{:?}", code),
//                     message,
//                 )
//             })?;

//     result.0.map_err(|e| Errors::CustomError(e.to_string()))
// }
