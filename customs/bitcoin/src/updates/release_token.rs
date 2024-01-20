use super::get_btc_address::init_ecdsa_public_key;
use crate::memo::BurnMemo;
use crate::tasks::{schedule_now, TaskType};
use crate::{
    address::{account_to_bitcoin_address, BitcoinAddress},
    state::{self, mutate_state, read_state, RetrieveBtcRequest, Token},
};
use candid::{CandidType, Deserialize};
use icrc_ledger_types::icrc1::account::Account;

const MAX_CONCURRENT_PENDING_REQUESTS: usize = 1000;

/// The arguments of the [release_token] endpoint.
#[derive(CandidType, Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct ReleaseTokenArgs {
    pub token: Token,
    // amount to retrieve in satoshi
    pub amount: u64,

    // address where to send bitcoins
    pub address: String,
}

#[derive(CandidType, Clone, Debug, Deserialize, PartialEq, Eq)]
pub enum ReleaseTokenError {
    /// There is another request for this principal.
    AlreadyProcessing,

    /// The withdrawal amount is too low.
    AmountTooLow(u64),

    /// The bitcoin address is not valid.
    MalformedAddress(String),

    /// The withdrawal account does not hold the requested ckBTC amount.
    InsufficientFunds { balance: u64 },

    /// The caller didn't approve enough funds for spending.
    InsufficientAllowance { allowance: u64 },

    /// There are too many concurrent requests, retry later.
    TemporarilyUnavailable(String),

    /// A generic error reserved for future extensions.
    GenericError {
        error_message: String,
        /// See the [ErrorCode] enum above for the list of possible values.
        error_code: u64,
    },
}

pub async fn release_token(args: ReleaseTokenArgs) -> Result<(), ReleaseTokenError> {
    state::read_state(|s| s.mode.is_release_available_for())
        .map_err(ReleaseTokenError::TemporarilyUnavailable)?;

    let ecdsa_public_key = init_ecdsa_public_key().await;
    let main_address = account_to_bitcoin_address(
        &ecdsa_public_key,
        &Account {
            owner: ic_cdk::id(),
            subaccount: None,
        },
    );

    if args.address == main_address.display(state::read_state(|s| s.btc_network)) {
        ic_cdk::trap("illegal release token target");
    }

    let (min_amount, btc_network) = read_state(|s| (s.release_min_amount, s.btc_network));
    if args.amount < min_amount {
        return Err(ReleaseTokenError::AmountTooLow(min_amount));
    }
    let parsed_address = BitcoinAddress::parse(&args.address, btc_network)?;
    if read_state(|s| s.count_incomplete_retrieve_btc_requests() >= MAX_CONCURRENT_PENDING_REQUESTS)
    {
        return Err(ReleaseTokenError::TemporarilyUnavailable(
            "too many pending retrieve_btc requests".to_string(),
        ));
    }

    let request = RetrieveBtcRequest {
        amount: args.amount,
        address: parsed_address,
        block_index,
        received_at: ic_cdk::api::time(),
    };

    mutate_state(|s| state::audit::accept_retrieve_btc_request(s, request));

    assert_eq!(
        crate::state::RetrieveBtcStatus::Pending,
        read_state(|s| s.retrieve_btc_status(block_index))
    );

    schedule_now(TaskType::ProcessLogic);

    Ok(())
}
