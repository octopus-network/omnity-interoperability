use ic_cdk::api::management_canister::main::{
    create_canister, install_code, CanisterIdRecord, CanisterInstallMode, CanisterSettings,
    CreateCanisterArgument, InstallCodeArgument,
};
use icrc_ledger_types::icrc1::account::Account;
use omnity_types::{Chain, ToggleState, Token};

use crate::{call_error::CallError, ICRC2_WASM};
use candid::{Decode, Encode, Principal};
use ic_icrc1_ledger::{ArchiveOptions, InitArgs as LedgerInitArgs, InitArgsBuilder as LedgerInitArgsBuilder, LedgerArgument};

use super::{mutate_state, RouteState};

pub fn add_chain(chain: Chain) {
    mutate_state(|state| state.counterparties.insert(chain.chain_id.clone(), chain));
}

pub async fn add_token(token: Token) {
    let canister_id_record = create_new_icrc2_canister(
        token.decimals.clone(), 
        token.token_id.clone(), 
        token.symbol.clone()
    ).await.unwrap();

    let token_id = token.token_id.clone();

    mutate_state(|state| {
        state.tokens.insert(token_id.clone(), token);
        state
            .token_ledgers
            .insert(token_id.clone(), canister_id_record.canister_id);
    })
}

pub fn toggle_chain_state(toggle: ToggleState) {
    mutate_state(|state| {
        if let Some(chain) = state.counterparties.get_mut(&toggle.chain_id) {
            chain.chain_state = toggle.action.into();
        }
    });
}

async fn create_new_icrc2_canister(token_decimal: u8, token_name: String, token_symbol: String ) -> Result<CanisterIdRecord, CallError> {
    let create_canister_arg = CreateCanisterArgument {
        settings: Some(CanisterSettings {
            controllers: Some(vec![ic_cdk::id()]),
            compute_allocation: Some(0_u32.into()),
            memory_allocation: Some(10000_u128.into()),
            freezing_threshold: Some(10000_u128.into()),
            reserved_cycles_limit: None,
        }),
    };
    let (canister_id_record,) = create_canister(create_canister_arg, 0).await.unwrap();

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
            .build())
        ).unwrap(),
    };
    install_code(install_code_arg).await.unwrap();

    Ok(canister_id_record)
}
