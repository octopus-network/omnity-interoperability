use candid::Principal;
use ic_cdk::api::{
    call::{call, CallResult},
    management_canister::main::{CanisterIdRecord, CanisterStatusResponse},
};

pub async fn stop_icrc_canister(icrc_canister_id: Principal) -> CallResult<((),)> {
    let args = CanisterIdRecord {
        canister_id: icrc_canister_id,
    };

    call(Principal::management_canister(), "stop_canister", (args,)).await
}

pub async fn start_icrc_canister(icrc_canister_id: Principal) -> CallResult<((),)> {
    let args = CanisterIdRecord {
        canister_id: icrc_canister_id,
    };

    call(Principal::management_canister(), "start_canister", (args,)).await
}

pub async fn delete_icrc_canister(icrc_canister_id: Principal) -> CallResult<((),)> {
    let args = CanisterIdRecord {
        canister_id: icrc_canister_id,
    };
    call(Principal::management_canister(), "delete_canister", (args,)).await
}

pub async fn icrc_canister_status(icrc_canister_id: Principal) -> CallResult<(CanisterStatusResponse,)> {
    let args = CanisterIdRecord {
        canister_id: icrc_canister_id,
    };
    call(Principal::management_canister(), "canister_status", (args,)).await
}
