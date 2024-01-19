use crate::destination::Destination;
use crate::logs::{P0, P1};
use crate::memo::MintMemo;
use crate::state::{mutate_state, read_state, RunesUtxo};
use crate::tasks::{schedule_now, TaskType};
use candid::{CandidType, Deserialize, Nat, Principal};
use ic_btc_interface::{GetUtxosError, GetUtxosResponse, OutPoint, Txid, Utxo};
use ic_canister_log::log;
use ic_ckbtc_kyt::Error as KytError;
use icrc_ledger_client_cdk::{CdkRuntime, ICRC1Client};
use icrc_ledger_types::icrc1::account::Account;
use icrc_ledger_types::icrc1::transfer::Memo;
use icrc_ledger_types::icrc1::transfer::{TransferArg, TransferError};
use num_traits::ToPrimitive;
use serde::Serialize;

use super::gen_boarding_pass::GenBoardingPassError;
use super::get_btc_address::init_ecdsa_public_key;

use crate::{
    guard::{balance_update_guard, GuardError},
    management::{fetch_utxo_alerts, get_utxos, CallError, CallSource},
    state,
    tx::{DisplayAmount, DisplayOutpoint},
    updates::get_btc_address,
};

/// The argument of the [update_balance] endpoint.
#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct UpdateBalanceArgs {
    pub target_chain_id: String,
    pub receiver: String,
}

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct FinalizeBoardingPassArgs {
    pub tx_id: Txid,
    pub runes_utxos: Vec<RunesUtxo>,
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

pub enum ErrorCode {
    ConfigurationError = 1,
}

#[derive(CandidType, Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct PendingUtxo {
    pub outpoint: OutPoint,
    pub value: u64,
    pub confirmations: u32,
}

#[derive(CandidType, Clone, Debug, Deserialize, PartialEq, Eq)]
pub enum FinalizeBoardingPassError {
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

impl From<GuardError> for FinalizeBoardingPassError {
    fn from(e: GuardError) -> Self {
        match e {
            GuardError::AlreadyProcessing => Self::AlreadyProcessing,
            GuardError::TooManyConcurrentRequests => {
                Self::TemporarilyUnavailable("too many concurrent requests".to_string())
            }
        }
    }
}

impl From<GetUtxosError> for FinalizeBoardingPassError {
    fn from(e: GetUtxosError) -> Self {
        Self::GenericError {
            error_code: ErrorCode::ConfigurationError as u64,
            error_message: format!("failed to get UTXOs from the Bitcoin canister: {}", e),
        }
    }
}

impl From<TransferError> for FinalizeBoardingPassError {
    fn from(e: TransferError) -> Self {
        Self::GenericError {
            error_code: ErrorCode::ConfigurationError as u64,
            error_message: format!("failed to mint tokens on the ledger: {:?}", e),
        }
    }
}

impl From<CallError> for FinalizeBoardingPassError {
    fn from(e: CallError) -> Self {
        Self::TemporarilyUnavailable(e.to_string())
    }
}

pub async fn finalize_boarding_pass(
    args: FinalizeBoardingPassArgs,
) -> Result<(), GenBoardingPassError> {
    state::read_state(|s| s.mode.is_transport_available_for())
        .map_err(GenBoardingPassError::TemporarilyUnavailable)?;

    let req = read_state(|s| {
        match s
            .pending_gen_boarding_pass_requests
            .iter()
            .find(|req| (req.tx_id == args.tx_id))
        {
            Some(req) => Ok(req.clone()),
            None => Err(GenBoardingPassError::PendingReqNotFound),
        }
    })?;

    for utxo in &args.runes_utxos {
        if !state::read_state(|s| {
            s.outpoint_destination.contains_key(&OutPoint {
                txid: args.tx_id,
                vout: utxo.vout,
            })
        }) {
            return Err(GenBoardingPassError::UtxoNotFound);
        }
    }

    mutate_state(|s| {
        for utxo in args.runes_utxos {
            s.available_runes_utxos.insert(utxo);
        }
    });

    // TODO invoke hub to generate landing pass
    Ok(())
}
