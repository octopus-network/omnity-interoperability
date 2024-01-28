use crate::guard::release_token_guard;
use crate::state::{ReleaseId, ReleaseTokenStatus};
use crate::tasks::{schedule_now, TaskType};
use crate::{
    address::{BitcoinAddress, ParseAddressError},
    state::{self, mutate_state, read_state, ReleaseTokenRequest, RunesId},
};
use candid::{CandidType, Deserialize};

const MAX_CONCURRENT_PENDING_REQUESTS: usize = 1000;

/// The arguments of the [release_token] endpoint.
#[derive(CandidType, Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct ReleaseTokenArgs {
    pub release_id: ReleaseId,
    pub rune_id: RunesId,
    // amount to retrieve
    pub amount: u128,
    // address where to send tokens
    pub address: String,
}

#[derive(CandidType, Clone, Debug, Deserialize, PartialEq, Eq)]
pub enum ReleaseTokenError {
    /// There is another request for this principal.
    AlreadyProcessing,

    AlreadyProcessed,

    /// The withdrawal amount is too low.
    AmountTooLow(u128),

    /// The bitcoin address is not valid.
    MalformedAddress(String),

    /// There are too many concurrent requests, retry later.
    TemporarilyUnavailable(String),

    /// A generic error reserved for future extensions.
    GenericError {
        error_message: String,
        /// See the [ErrorCode] enum above for the list of possible values.
        error_code: u64,
    },
}

impl From<ParseAddressError> for ReleaseTokenError {
    fn from(e: ParseAddressError) -> Self {
        Self::MalformedAddress(e.to_string())
    }
}

pub async fn release_token(args: ReleaseTokenArgs) -> Result<(), ReleaseTokenError> {
    state::read_state(|s| s.mode.is_release_available_for())
        .map_err(ReleaseTokenError::TemporarilyUnavailable)?;

    let _guard = release_token_guard();

    let (min_amount, btc_network) = read_state(|s| (s.release_min_amount, s.btc_network));
    if args.amount < min_amount {
        return Err(ReleaseTokenError::AmountTooLow(min_amount));
    }

    let parsed_address = BitcoinAddress::parse(&args.address, btc_network)?;
    if read_state(|s| s.count_incomplete_retrieve_btc_requests() >= MAX_CONCURRENT_PENDING_REQUESTS)
    {
        return Err(ReleaseTokenError::TemporarilyUnavailable(
            "too many pending release_token requests".to_string(),
        ));
    }

    read_state(|s| match s.release_token_status(&args.release_id) {
        ReleaseTokenStatus::Pending
        | ReleaseTokenStatus::Signing
        | ReleaseTokenStatus::Sending(_)
        | ReleaseTokenStatus::Submitted(_) => Err(ReleaseTokenError::AlreadyProcessing),
        ReleaseTokenStatus::Confirmed(_) => Err(ReleaseTokenError::AlreadyProcessed),
        ReleaseTokenStatus::Unknown => Ok(()),
    })?;

    let request = ReleaseTokenRequest {
        release_id: args.release_id.clone(),
        runes_id: args.rune_id,
        amount: args.amount,
        address: parsed_address,
        received_at: ic_cdk::api::time(),
    };

    mutate_state(|s| state::audit::accept_release_token_request(s, request));

    assert_eq!(
        crate::state::ReleaseTokenStatus::Pending,
        read_state(|s| s.release_token_status(&args.release_id))
    );

    schedule_now(TaskType::ProcessLogic);

    Ok(())
}
