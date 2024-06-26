use crate::guard::{release_token_guard, GuardError};
use crate::state::{audit, ReleaseTokenStatus};
use crate::{
    address::{BitcoinAddress, ParseAddressError},
    state::{mutate_state, read_state, ReleaseTokenRequest},
};
use candid::{CandidType, Deserialize};
use omnity_types::{TicketId, TokenId, TxAction};

const MAX_CONCURRENT_PENDING_REQUESTS: usize = 1000;

/// The arguments of the [release_token] endpoint.
#[derive(CandidType, Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct ReleaseTokenArgs {
    pub ticket_id: TicketId,
    pub token_id: TokenId,
    pub action: TxAction,
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

    UnsupportedToken(String),

    /// The bitcoin address is not valid.
    MalformedAddress(String),

    InvalidTxAction,

    /// There are too many concurrent requests, retry later.
    TemporarilyUnavailable(String),
}

impl From<ParseAddressError> for ReleaseTokenError {
    fn from(e: ParseAddressError) -> Self {
        Self::MalformedAddress(e.to_string())
    }
}

impl From<GuardError> for ReleaseTokenError {
    fn from(e: GuardError) -> Self {
        match e {
            GuardError::TooManyConcurrentRequests => {
                Self::TemporarilyUnavailable("too many concurrent requests".to_string())
            }
        }
    }
}

pub async fn release_token(args: ReleaseTokenArgs) -> Result<(), ReleaseTokenError> {
    let rune_id = read_state(|s| {
        if let Some((rune_id, _)) = s.tokens.get(&args.token_id) {
            Ok(*rune_id)
        } else {
            Err(ReleaseTokenError::UnsupportedToken(args.token_id.clone()))
        }
    })?;

    let _guard = release_token_guard()?;

    let btc_network = read_state(|s| s.btc_network);

    let parsed_address = match args.action {
        TxAction::Redeem => BitcoinAddress::parse(&args.address, btc_network)?,
        TxAction::Burn => BitcoinAddress::OpReturn(vec![]),
        TxAction::Transfer => {
            return Err(ReleaseTokenError::InvalidTxAction);
        }
    };

    if read_state(|s| {
        s.count_incomplete_release_token_requests() >= MAX_CONCURRENT_PENDING_REQUESTS
    }) {
        return Err(ReleaseTokenError::TemporarilyUnavailable(
            "too many pending release_token requests".to_string(),
        ));
    }

    read_state(|s| match s.release_token_status(&args.ticket_id) {
        ReleaseTokenStatus::Pending
        | ReleaseTokenStatus::Signing
        | ReleaseTokenStatus::Sending(_)
        | ReleaseTokenStatus::Submitted(_) => Err(ReleaseTokenError::AlreadyProcessing),
        ReleaseTokenStatus::Confirmed(_) => Err(ReleaseTokenError::AlreadyProcessed),
        ReleaseTokenStatus::Unknown => Ok(()),
    })?;

    let request = ReleaseTokenRequest {
        ticket_id: args.ticket_id.clone(),
        rune_id,
        amount: args.amount,
        address: parsed_address,
        received_at: ic_cdk::api::time(),
    };

    mutate_state(|s| audit::accept_release_token_request(s, request));

    assert_eq!(
        crate::state::ReleaseTokenStatus::Pending,
        read_state(|s| s.release_token_status(&args.ticket_id))
    );
    Ok(())
}
