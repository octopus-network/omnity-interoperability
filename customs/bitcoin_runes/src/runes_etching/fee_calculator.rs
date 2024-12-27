use anyhow::anyhow;
use candid::Principal;
use ic_ledger_types::{account_balance, AccountBalanceArgs, AccountIdentifier, DEFAULT_SUBACCOUNT, MAINNET_LEDGER_CANISTER_ID, Memo, Subaccount, Tokens, TransferArgs};
use crate::runes_etching::constants::POSTAGE;
use crate::runes_etching::Utxo;
use crate::state::mutate_state;

pub const INPUT_SIZE_VBYTES: u64 = 74;
pub const OUTPUT_SIZE_VBYTES: u64 = 31;
pub const TX_OVERHEAD_VBYTES: u64 = 21;
pub const FIXED_COMMIT_TX_VBYTES: u64 =
    OUTPUT_SIZE_VBYTES*2 + TX_OVERHEAD_VBYTES;
const ICP_TRANSFER_FEE: u64 = 10_000;

pub fn select_utxos(fee_rate: u64, fixed_size: u64) -> anyhow::Result<Vec<Utxo>> {
    let mut selected_utxos: Vec<Utxo> = vec![];
    let mut selected_amount = 0u64;
    let mut estimate_size = fixed_size;
    mutate_state(|s| loop {
        if selected_amount >= fee_rate * estimate_size + POSTAGE * 2 {
            return Ok(selected_utxos);
        }
        let u = s.etching_fee_utxos.pop();
        match u {
            None => {
                return Err(anyhow!("InsufficientFunds"));
            }
            Some(utxo) => {
                selected_amount += utxo.amount.to_sat();
                selected_utxos.push(utxo);
                estimate_size += INPUT_SIZE_VBYTES;
            }
        }
    })
}

pub async fn charge_fee(fee_amount: u64) -> anyhow::Result<()> {
    let subaccount = principal_to_subaccount(&ic_cdk::caller());
    let balance = ic_balance_of(&subaccount).await?.e8s();
    if balance < fee_amount {
        return Err(anyhow!( format!("InsufficientFee: required: {}, provided: {}", fee_amount, balance)));
    }

    transfer_fee(&subaccount, balance).await?;
    Ok(())
}

pub fn principal_to_subaccount(principal_id: &Principal) -> Subaccount {
    let mut subaccount = [0; std::mem::size_of::<Subaccount>()];
    let principal_id = principal_id.as_slice();
    subaccount[0] = principal_id.len().try_into().unwrap();
    subaccount[1..1 + principal_id.len()].copy_from_slice(principal_id);

    Subaccount(subaccount)
}

async fn ic_balance_of(subaccount: &Subaccount) -> anyhow::Result<Tokens> {
    let account_identifier = AccountIdentifier::new(&ic_cdk::api::id(), &subaccount);
    let balance_args = AccountBalanceArgs {
        account: account_identifier,
    };
    account_balance(MAINNET_LEDGER_CANISTER_ID, balance_args)
        .await
        .map_err(|(_, reason)| anyhow!(reason))
}

async fn transfer_fee(subaccount: &Subaccount, fee_amount: u64) -> anyhow::Result<()> {
    let transfer_args = TransferArgs {
        memo: Memo(0),
        amount: Tokens::from_e8s(fee_amount - ICP_TRANSFER_FEE),
        fee: Tokens::from_e8s(ICP_TRANSFER_FEE),
        from_subaccount: Some(subaccount.clone()),
        to: AccountIdentifier::new(&ic_cdk::api::id(), &DEFAULT_SUBACCOUNT),
        created_at_time: None,
    };

    ic_ledger_types::transfer(MAINNET_LEDGER_CANISTER_ID, transfer_args)
        .await
        .map_err(|(_, reason)| anyhow!(reason))?
        .map_err(|err| anyhow!(err.to_string()))?;

    Ok(())
}