use crate::address::destination_to_bitcoin_address;
use crate::destination::Destination;
use crate::guard::{release_token_guard, GuardError};
use crate::state::{audit, ReleaseTokenStatus, RuneTxRequest, RUNES_TOKEN};
use crate::updates::get_btc_address::init_ecdsa_public_key;
use crate::{
    address::{BitcoinAddress, ParseAddressError},
    state::{mutate_state, read_state},
};
use candid::{CandidType, Deserialize};
use omnity_types::{TicketId, TokenId, TxAction};

const MAX_CONCURRENT_PENDING_REQUESTS: usize = 1000;

/// The arguments of the [release_token] endpoint.
#[derive(CandidType, Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct RuneTxArgs {
    pub ticket_id: TicketId,
    pub token_id: TokenId,
    pub src_chain: String,
    pub action: TxAction,
    // amount to retrieve
    pub amount: u128,
    pub receiver: String,
}

#[derive(CandidType, Clone, Debug, Deserialize, PartialEq, Eq)]
pub enum GenRuneTxReqError {
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

impl From<ParseAddressError> for GenRuneTxReqError {
    fn from(e: ParseAddressError) -> Self {
        Self::MalformedAddress(e.to_string())
    }
}

impl From<GuardError> for GenRuneTxReqError {
    fn from(e: GuardError) -> Self {
        match e {
            GuardError::TooManyConcurrentRequests => {
                Self::TemporarilyUnavailable("too many concurrent requests".to_string())
            }
            GuardError::KeyIsHandling => {
                Self::TemporarilyUnavailable("request key is handling".to_string())
            }
        }
    }
}

pub async fn generate_rune_tx_request(args: RuneTxArgs) -> Result<(), GenRuneTxReqError> {
    let rune_id = read_state(|s| {
        if let Some((rune_id, _)) = s.tokens.get(&args.token_id) {
            Ok(*rune_id)
        } else {
            Err(GenRuneTxReqError::UnsupportedToken(args.token_id.clone()))
        }
    })?;

    let _guard = release_token_guard()?;

    let btc_network = read_state(|s| s.btc_network);

    let parsed_address = match args.action {
        TxAction::Redeem => BitcoinAddress::parse(&args.receiver, btc_network)?,
        TxAction::Burn => BitcoinAddress::OpReturn(vec![]),
        TxAction::Mint => destination_to_bitcoin_address(
            &init_ecdsa_public_key().await,
            &Destination {
                target_chain_id: args.src_chain,
                receiver: args.receiver,
                token: Some(RUNES_TOKEN.into()),
            },
        ),
        TxAction::Transfer => {
            return Err(GenRuneTxReqError::InvalidTxAction);
        }
        TxAction::RedeemIcpChainKeyAssets(_) => {
            return Err(GenRuneTxReqError::InvalidTxAction);
        }
    };

    if read_state(|s| s.count_incomplete_rune_tx_requests() >= MAX_CONCURRENT_PENDING_REQUESTS) {
        return Err(GenRuneTxReqError::TemporarilyUnavailable(
            "too many pending release_token requests".to_string(),
        ));
    }

    read_state(|s| match s.rune_tx_status(&args.ticket_id) {
        ReleaseTokenStatus::Pending
        | ReleaseTokenStatus::Signing
        | ReleaseTokenStatus::Sending(_)
        | ReleaseTokenStatus::Submitted(_) => Err(GenRuneTxReqError::AlreadyProcessing),
        ReleaseTokenStatus::Confirmed(_) => Err(GenRuneTxReqError::AlreadyProcessed),
        ReleaseTokenStatus::Unknown => Ok(()),
    })?;

    let request = RuneTxRequest {
        ticket_id: args.ticket_id.clone(),
        action: args.action,
        rune_id,
        amount: args.amount,
        address: parsed_address,
        received_at: ic_cdk::api::time(),
    };

    mutate_state(|s| audit::accept_rune_tx_request(s, request));

    assert_eq!(
        crate::state::ReleaseTokenStatus::Pending,
        read_state(|s| s.rune_tx_status(&args.ticket_id))
    );
    Ok(())
}
