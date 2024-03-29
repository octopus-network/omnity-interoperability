use crate::state::{audit, mutate_state, read_state};
use crate::ICRC2_WASM;
use candid::{CandidType, Deserialize};
use candid::{Encode, Principal};
use ic_cdk::api::management_canister::main::{
    create_canister, install_code, CanisterIdRecord, CanisterInstallMode, CanisterSettings,
    CreateCanisterArgument, InstallCodeArgument,
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

    let record =
        install_icrc2_ledger(token.token_id.clone(), token.symbol.clone(), token.decimals)
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
    let create_canister_arg = CreateCanisterArgument {
        settings: Some(CanisterSettings {
            controllers: Some(vec![ic_cdk::id()]),
            compute_allocation: Some(0_u32.into()),
            memory_allocation: Some(4096000_u128.into()),
            freezing_threshold: Some(10000_u128.into()),
            reserved_cycles_limit: None,
        }),
    };
    let (canister_id_record,) = create_canister(create_canister_arg, 100_000_000_000).await.unwrap();

    let owner: Principal = ic_cdk::id();
    let install_code_arg = InstallCodeArgument {
        mode: CanisterInstallMode::Install,
        canister_id: canister_id_record.canister_id.clone(),
        wasm_module: ICRC2_WASM.to_vec(),
        arg: Encode!(&LedgerArgument::Init(
            LedgerInitArgsBuilder::with_symbol_and_name(token_symbol, token_name)
                .with_decimals(token_decimal)
                .with_minting_account(Into::<Account>::into(owner))
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
