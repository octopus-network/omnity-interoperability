use candid::Nat;
use ic_btc_interface::{OutPoint, Utxo};
use icrc_ledger_types::{
    icrc1::account::{Account, Subaccount},
    icrc2::approve::ApproveArgs,
    icrc3::transactions::{GetTransactionsRequest, GetTransactionsResponse},
};

use crate::*;

pub const CKBTC_FEE: u64 = 10;

pub async fn balance_of(owner: Account) -> Result<Nat> {
    let settings = state::get_settings();
    let client = ICRC1Client {
        runtime: CdkRuntime,
        ledger_canister_id: settings.ckbtc_ledger_principal,
    };
    let result = client
        .balance_of(owner)
        .await
        .map_err(|(code, msg)| {
            Errors::CanisterCallError(
                settings.ckbtc_ledger_principal.to_string(),
                "balance_of".to_string(),
                format!("{:?}", code),
                msg,
            )
        })?;
    Ok(result)
}

pub async fn approve_ckbtc_for_icp_custom(
    subaccount: Option<Subaccount>,
    amount: Nat,
) -> Result<()> {
    let settings = state::get_settings();
    let ckbtc_ledger_principal = settings.ckbtc_ledger_principal.clone();
    let client = ICRC1Client {
        runtime: CdkRuntime,
        ledger_canister_id: ckbtc_ledger_principal,
    };
    let spender = Account {
        owner: settings.icp_customs_principal,
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
    let ckbtc_ledger_principal = state::get_settings().ckbtc_ledger_principal;
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

pub async fn update_balance(args: UpdateBalanceArgs)-> Result<Vec<UtxoStatus>> {
    let settings = state::get_settings();
    let result: (std::result::Result<Vec<UtxoStatus>, UpdateBalanceError>,) =
    ic_cdk::api::call::call(settings.ckbtc_minter_principal, "update_balance", (args.clone(),))
        .await
        .map_err(|(code, msg)| {
            Errors::CanisterCallError(
                settings.ckbtc_minter_principal.to_string(),
                "get_transactions".to_string(),
                format!("{:?}", code),
                msg,
            )
        })?;

    result.0.map_err(|e| 
        Errors::CkBtcUpdateBalanceError(
            format!("{:?}", args).to_string(),
            format!("{:?}",e).to_string()
        )
    )
} 

pub async fn get_btc_address(args: GetBtcAddressArgs) -> Result<String> {
    let ckbtc_minter_principal = state::get_settings().ckbtc_minter_principal;
    let address: (String,) = ic_cdk::api::call::call(ckbtc_minter_principal, "get_btc_address", (args,))
        .await
        .map_err(|(code, msg)| {
            Errors::CanisterCallError(
                ckbtc_minter_principal.to_string(),
                "get_btc_address".to_string(),
                format!("{:?}", code),
                msg,
            )
        })?;
    Ok(address.0)
}


#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct GetBtcAddressArgs {
    pub owner: Option<Principal>,
    pub subaccount: Option<Subaccount>,
}

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct UpdateBalanceArgs {
    /// The owner of the account on the ledger.
    /// The minter uses the caller principal if the owner is None.
    pub owner: Option<Principal>,
    /// The desired subaccount on the ledger, if any.
    pub subaccount: Option<Subaccount>,
}

/// The outcome of UTXO processing.
#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum UtxoStatus {
    /// The UTXO value does not cover the KYT check cost.
    ValueTooSmall(Utxo),
    /// The KYT check found issues with the deposited UTXO.
    Tainted(Utxo),
    /// The deposited UTXO passed the KYT check, but the minter failed to mint ckBTC on the ledger.
    /// The caller should retry the [update_balance] call.
    Checked(Utxo),
    /// The minter accepted the UTXO and minted ckBTC tokens on the ledger.
    Minted {
        /// The MINT transaction index on the ledger.
        block_index: u64,
        /// The minted amount (UTXO value minus fees).
        minted_amount: u64,
        /// The UTXO that caused the balance update.
        utxo: Utxo,
    },
}

#[derive(CandidType, Clone, Debug, Deserialize, PartialEq, Eq)]
pub enum UpdateBalanceError {
    /// The minter experiences temporary issues, try the call again later.
    TemporarilyUnavailable(String),
    /// There is a concurrent [update_balance] invocation from the same caller.
    AlreadyProcessing,
    /// The minter didn't discover new UTXOs with enough confirmations.
    NoNewUtxos {
        /// If there are new UTXOs that do not have enough
        /// confirmations yet, this field will contain the number of
        /// confirmations as observed by the minter.
        current_confirmations: Option<u32>,
        /// The minimum number of UTXO confirmation required for the minter to accept a UTXO.
        required_confirmations: u32,
        /// List of utxos that don't have enough confirmations yet to be processed.
        pending_utxos: Option<Vec<PendingUtxo>>,
    },
    GenericError {
        error_code: u64,
        error_message: String,
    },
}

#[derive(CandidType, Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct PendingUtxo {
    pub outpoint: OutPoint,
    pub value: u64,
    pub confirmations: u32,
}
