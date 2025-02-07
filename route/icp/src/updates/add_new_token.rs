use crate::state::{audit, mutate_state, read_state};
use crate::{BLOCK_HOLE_ADDRESS, FEE_COLLECTOR_SUB_ACCOUNT, ICRC2_WASM};
use candid::{CandidType, Deserialize};
use candid::{Encode, Principal};
use ic_cdk::api::management_canister::main::{
    create_canister, install_code, CanisterIdRecord, CanisterInstallMode, CanisterSettings,
    CreateCanisterArgument, InstallCodeArgument,
};
use ic_icrc1_ledger::{
    ArchiveOptions, InitArgsBuilder as LedgerInitArgsBuilder, LedgerArgument, UpgradeArgs,
};
use icrc_ledger_types::icrc::generic_metadata_value::MetadataValue;
use icrc_ledger_types::icrc1::account::Account;
use omnity_types::Token;

#[derive(CandidType, Clone, Debug, Deserialize, PartialEq, Eq)]
pub enum AddNewTokenError {
    AlreadyAdded(String),
    CreateLedgerErr(String),
}

pub async fn add_new_token(token: Token) -> Result<(), AddNewTokenError> {
    if read_state(|s| s.tokens.contains_key(&token.token_id)) {
        mutate_state(|s| {
            s.tokens.insert(token.token_id.clone(), token);
        });
        return Ok(())
    }

    let record = install_icrc2_ledger(
        token.name.clone(),
        token.symbol.clone(),
        token.decimals,
        token.icon.clone(),
    )
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
    token_icon: Option<String>,
) -> Result<CanisterIdRecord, String> {
    let create_canister_arg = CreateCanisterArgument {
        settings: Some(CanisterSettings {
            controllers: Some(vec![
                ic_cdk::id(),
                Principal::from_text(BLOCK_HOLE_ADDRESS).unwrap(),
            ]),
            compute_allocation: None,
            memory_allocation: None,
            freezing_threshold: None,
            reserved_cycles_limit: None,
        }),
    };

    if ic_cdk::api::canister_balance128() < 3_000_000_000_000 {
        return Err("Insufficient cycles for create token ledger canister".to_string());
    }

    let (canister_id_record,) = create_canister(create_canister_arg, 3_000_000_000_000)
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
                .with_transfer_fee(
                    10_u128
                )
                .with_fee_collector_account(Account {
                    owner,
                    subaccount: Some(FEE_COLLECTOR_SUB_ACCOUNT.clone())
                })
                .with_metadata_entry(
                    "icrc1:logo",
                    MetadataValue::Text(token_icon.unwrap_or_default())
                )
                .with_archive_options(ArchiveOptions {
                    trigger_threshold: 1000,
                    num_blocks_to_archive: 1000,
                    node_max_memory_size_bytes: None,
                    max_message_size_bytes: None,
                    controller_id: owner.into(),
                    cycles_for_archive_creation: None,
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

pub async fn upgrade_icrc2_ledger(
    canister_id: Principal,
    upgrade_args: UpgradeArgs,
) -> Result<(), String> {
    let install_code_arg = InstallCodeArgument {
        mode: CanisterInstallMode::Upgrade,
        canister_id: canister_id,
        wasm_module: ICRC2_WASM.to_vec(),
        arg: Encode!(&LedgerArgument::Upgrade(Some(upgrade_args))).unwrap(),
    };
    install_code(install_code_arg)
        .await
        .map_err(|(_, reason)| reason)
}