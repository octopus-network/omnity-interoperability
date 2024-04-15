use candid::Principal;
use ic_cdk::api::{call::CallResult, management_canister::main::CanisterIdRecord};

use crate::call_error::CallError;

async fn stop_icrc_ledger(icrc_ledger_id: Principal) -> CallResult<((),)> {
    assert_eq!(ic_cdk::caller(), ic_cdk::api::id());
    let args = CanisterIdRecord {
        canister_id: icrc_ledger_id,
    };

    ic_cdk::api::call::call(Principal::management_canister(), "stop_canister", (args,)).await
}

pub async fn canister_status(args: CanisterId) -> CallResult<(CanisterStatus,)> {
    let canister_status = call(Principal::management_canister(), "canister_status", (args,)).await;
    return canister_status;
}
