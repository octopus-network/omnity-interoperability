use std::str::FromStr;

use crate::state::{audit, mutate_state, read_state};
use crate::{DEFAULT_MEMORY_LIMIT, ICRC2_WASM};
use candid::{CandidType, Deserialize, Nat};
use candid::{Encode, Principal};
use ic_cdk::api::management_canister::main::{
    create_canister, install_code, CanisterIdRecord, CanisterInstallMode, CreateCanisterArgument,
    InstallCodeArgument,
};
use ic_icrc1_ledger::{ArchiveOptions, InitArgsBuilder as LedgerInitArgsBuilder, LedgerArgument};
use icrc_ledger_types::icrc1::account::Account;
use omnity_types::Token;

#[derive(CandidType, Clone, Debug, Deserialize, PartialEq, Eq)]
pub enum AddNewTokenError {
    AleardyAdded(String),
    CreateLedgerErr(String),
}

pub async fn add_new_token(token: Token) -> Result<(), AddNewTokenError> {
    if read_state(|s| s.tokens.contains_key(&token.token_id)) {
        return Err(AddNewTokenError::AleardyAdded(token.token_id));
    }

    let record = install_icrc2_ledger(token.token_id.clone(), token.symbol.clone(), token.decimals)
        .await
        .map_err(AddNewTokenError::CreateLedgerErr)?;

    mutate_state(|s| {
        audit::add_token(s, token, record.canister_id);
    });
    Ok(())
}

async fn install_icrc2_ledger(
    token_name: String,
    token_symbol: String,
    token_decimal: u8,
) -> Result<CanisterIdRecord, String> {
    let create_canister_arg = CreateCanisterArgument { settings: None };
    let (canister_id_record,) = create_canister(create_canister_arg, 500_000_000_000)
        .await
        .map_err(|(_, reason)| reason)?;

    let owner: Principal = ic_cdk::id();
    let install_code_arg = InstallCodeArgument {
        mode: CanisterInstallMode::Install,
        canister_id: canister_id_record.canister_id.clone(),
        wasm_module: ICRC2_WASM.to_vec(),
        arg: Encode!(&LedgerArgument::Init(
            LedgerInitArgsBuilder::with_symbol_and_name(token_symbol, token_name)
                .with_decimals(token_decimal)
                .with_minting_account(Into::<Account>::into(owner))
                .with_transfer_fee(Nat::from_str("0").unwrap())
                .with_archive_options(ArchiveOptions {
                    // The number of blocks which, when exceeded, will trigger an archiving operation.
                    // If the speed of block production is 1 block per second, 
                    // it means 1000 seconds ≈ 16 minutes will trigger an archiving operation.
                    trigger_threshold: 1000,
                    // The number of blocks to archive when trigger threshold is exceeded.
                    num_blocks_to_archive: 1000,
                    // Allocate 1GB for raw blocks.
                    node_max_memory_size_bytes: Some(DEFAULT_MEMORY_LIMIT),
                    // The maximum number of blocks to return in a single get_transactions request.
                    max_message_size_bytes: None,
                    controller_id: owner.into(),
                    // default value: 0
                    cycles_for_archive_creation: None,
                    // default value: 2000
                    max_transactions_per_response: None,
                })
                .build()
        ))
        .unwrap(),
    };
    install_code(install_code_arg)
        .await
        .map_err(|(_, reason)| reason)?;

    Ok(canister_id_record)
}
