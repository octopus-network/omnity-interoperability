use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

use candid::{CandidType, Principal};
pub use ic_btc_interface::OutPoint;
use ic_btc_interface::{Address, Network, Utxo};

use ic_cdk::api::call::RejectionCode;
use omnity_types::{TicketId, TokenId};
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;

use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

#[derive(candid::CandidType, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PushUtxosToAddress {
    pub utxos: BTreeMap<Address, Vec<Utxo>>,
}

#[derive(CandidType, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Txid([u8; 32]);

impl AsRef<[u8]> for Txid {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl From<Txid> for [u8; 32] {
    fn from(txid: Txid) -> Self {
        txid.0
    }
}

impl serde::Serialize for Txid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_bytes(&self.0)
    }
}

impl<'de> serde::de::Deserialize<'de> for Txid {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        struct TxidVisitor;

        impl<'de> serde::de::Visitor<'de> for TxidVisitor {
            type Value = Txid;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a 32-byte array")
            }

            fn visit_bytes<E>(self, value: &[u8]) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match TryInto::<[u8; 32]>::try_into(value) {
                    Ok(txid) => Ok(Txid(txid)),
                    Err(_) => Err(E::invalid_length(value.len(), &self)),
                }
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                use serde::de::Error;
                if let Some(size_hint) = seq.size_hint() {
                    if size_hint != 32 {
                        return Err(A::Error::invalid_length(size_hint, &self));
                    }
                }
                let mut bytes = [0u8; 32];
                let mut i = 0;
                while let Some(byte) = seq.next_element()? {
                    if i == 32 {
                        return Err(A::Error::invalid_length(i + 1, &self));
                    }

                    bytes[i] = byte;
                    i += 1;
                }
                if i != 32 {
                    return Err(A::Error::invalid_length(i, &self));
                }
                Ok(Txid(bytes))
            }
        }

        deserializer.deserialize_bytes(TxidVisitor)
    }
}

impl fmt::Display for Txid {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        // In Bitcoin, you display hash bytes in reverse order.
        //
        // > Due to historical accident, the tx and block hashes that bitcoin core
        // > uses are byte-reversed. I’m not entirely sure why. Maybe something
        // > like using openssl bignum to store hashes or something like that,
        // > then printing them as a number.
        // > -- Wladimir van der Laan
        //
        // Source: https://learnmeabitcoin.com/technical/txid
        for b in self.0.iter().rev() {
            write!(fmt, "{:02x}", *b)?
        }
        Ok(())
    }
}

impl From<[u8; 32]> for Txid {
    fn from(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}

impl TryFrom<&'_ [u8]> for Txid {
    type Error = core::array::TryFromSliceError;
    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        let txid: [u8; 32] = bytes.try_into()?;
        Ok(Txid(txid))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TxidFromStrError {
    InvalidChar(u8),
    InvalidLength { expected: usize, actual: usize },
}

impl fmt::Display for TxidFromStrError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::InvalidChar(c) => write!(f, "char {c} is not a valid hex"),
            Self::InvalidLength { expected, actual } => write!(
                f,
                "Bitcoin transaction id must be precisely {expected} characters, got {actual}"
            ),
        }
    }
}

impl FromStr for Txid {
    type Err = TxidFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        fn decode_hex_char(c: u8) -> Result<u8, TxidFromStrError> {
            match c {
                b'A'..=b'F' => Ok(c - b'A' + 10),
                b'a'..=b'f' => Ok(c - b'a' + 10),
                b'0'..=b'9' => Ok(c - b'0'),
                _ => Err(TxidFromStrError::InvalidChar(c)),
            }
        }
        if s.len() != 64 {
            return Err(TxidFromStrError::InvalidLength {
                expected: 64,
                actual: s.len(),
            });
        }
        let mut bytes = [0u8; 32];
        let chars = s.as_bytes();
        for i in 0..32 {
            bytes[31 - i] =
                (decode_hex_char(chars[2 * i])? << 4) | decode_hex_char(chars[2 * i + 1])?;
        }
        Ok(Self(bytes))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseRuneIdError;

impl fmt::Display for ParseRuneIdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        "provided rune_id was not valid".fmt(f)
    }
}

impl Error for ParseRuneIdError {
    fn description(&self) -> &str {
        "failed to parse rune_id"
    }
}

#[derive(
    candid::CandidType,
    Clone,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Copy,
    Default,
    Serialize,
    Deserialize,
)]
pub struct RuneId {
    pub block: u64,
    pub tx: u32,
}

impl Display for RuneId {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.block, self.tx,)
    }
}

impl FromStr for RuneId {
    type Err = ParseRuneIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (height, index) = s.split_once(':').ok_or_else(|| ParseRuneIdError)?;

        Ok(Self {
            block: height.parse().map_err(|_| ParseRuneIdError)?,
            tx: index.parse().map_err(|_| ParseRuneIdError)?,
        })
    }
}

#[derive(candid::CandidType, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenTicketRequest {
    pub address: String,
    pub target_chain_id: String,
    pub receiver: String,
    pub token_id: TokenId,
    pub rune_id: RuneId,
    pub amount: u128,
    pub txid: Txid,
    pub received_at: u64,
}

#[derive(candid::CandidType, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FinalizedStatus {
    /// The transaction that release token got enough confirmations.
    Confirmed(Txid),
}

#[derive(candid::CandidType, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BitcoinAddress {
    /// Pay to witness public key hash address.
    /// See BIP-173.
    #[serde(rename = "p2wpkh_v0")]
    P2wpkhV0([u8; 20]),
    /// Pay to witness script hash address.
    /// See BIP-141.
    #[serde(rename = "p2wsh_v0")]
    P2wshV0([u8; 32]),
    /// Pay to taproot address.
    /// See BIP-341.
    #[serde(rename = "p2tr_v1")]
    P2trV1([u8; 32]),
    /// Pay to public key hash address.
    #[serde(rename = "p2pkh")]
    P2pkh([u8; 20]),
    /// Pay to script hash address.
    #[serde(rename = "p2sh")]
    P2sh([u8; 20]),
    /// Pay to OP_RETURN
    OpReturn(Vec<u8>),
}

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct GenerateTicketArgs {
    pub target_chain_id: String,
    pub receiver: String,
    pub rune_id: String,
    pub amount: u128,
    pub txid: String,
}

// A pending release token request
#[derive(candid::CandidType, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseTokenRequest {
    pub ticket_id: TicketId,
    pub rune_id: RuneId,
    /// The amount to release token.
    pub amount: u128,
    /// The destination BTC address.
    pub address: BitcoinAddress,
    /// The time at which the customs accepted the request.
    pub received_at: u64,
}
#[derive(Debug, PartialEq, Eq)]
pub enum GuardError {
    TooManyConcurrentRequests,
}

#[derive(CandidType, Clone, Debug, Deserialize, PartialEq, Eq)]
pub enum GenerateTicketError {
    TemporarilyUnavailable(String),
    AlreadySubmitted,
    AleardyProcessed,
    NoNewUtxos,
    InvalidRuneId(String),
    InvalidTxId,
    UnsupportedChainId(String),
    UnsupportedToken(String),
}

impl From<GuardError> for GenerateTicketError {
    fn from(e: GuardError) -> Self {
        match e {
            GuardError::TooManyConcurrentRequests => {
                Self::TemporarilyUnavailable("too many concurrent requests".to_string())
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallError {
    pub method: String,
    pub reason: Reason,
}

impl CallError {
    /// Returns the name of the method that resulted in this error.
    pub fn method(&self) -> &str {
        &self.method
    }

    /// Returns the failure reason.
    pub fn reason(&self) -> &Reason {
        &self.reason
    }
}

impl fmt::Display for CallError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            fmt,
            "management call '{}' failed: {}",
            self.method, self.reason
        )
    }
}

/// The result of a Call.
///
/// Errors on the IC have two components; a Code and a message associated with it.
pub type CallResult<R> = Result<R, (RejectionCode, String)>;

#[derive(Debug, Clone, PartialEq, Eq)]
/// The reason for the management call failure.
pub enum Reason {
    /// Failed to send a signature request because the local output queue is
    /// full.
    QueueIsFull,
    /// The canister does not have enough cycles to submit the request.
    OutOfCycles,
    /// The call failed with an error.
    CanisterError(String),
    /// The management canister rejected the signature request (not enough
    /// cycles, the ECDSA subnet is overloaded, etc.).
    Rejected(String),
}

impl fmt::Display for Reason {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::QueueIsFull => write!(fmt, "the canister queue is full"),
            Self::OutOfCycles => write!(fmt, "the canister is out of cycles"),
            Self::CanisterError(msg) => write!(fmt, "canister error: {}", msg),
            Self::Rejected(msg) => {
                write!(fmt, "the management canister rejected the call: {}", msg)
            }
        }
    }
}

impl Reason {
    pub fn from_reject(reject_code: RejectionCode, reject_message: String) -> Self {
        match reject_code {
            RejectionCode::CanisterReject => Self::Rejected(reject_message),
            _ => Self::CanisterError(reject_message),
        }
    }
}

/// The status of a release_token request.
#[derive(candid::CandidType, Clone, Debug, PartialEq, Eq, Deserialize)]
pub enum ReleaseTokenStatus {
    /// The custom has no data for this request.
    /// The request id is either invalid or too old.
    Unknown,
    /// The request is in the batch queue.
    Pending,
    /// Waiting for a signature on a transaction satisfy this request.
    Signing,
    /// Sending the transaction satisfying this request.
    Sending(String),
    /// Awaiting for confirmations on the transaction satisfying this request.
    Submitted(String),
    /// Confirmed a transaction satisfying this request.
    Confirmed(String),
}

#[derive(candid::CandidType, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Args {
    pub hub_principal: Principal,
    pub directive_method: String,
    pub ticket_method: String,
    pub network: Option<Network>,
}

#[must_use]
pub struct TimerLogicGuard(String);

impl TimerLogicGuard {
    pub fn new(task_name: String) -> Option<Self> {
        mutate_state(|s| {
            let running = s
                .is_timer_running
                .get(&task_name)
                .cloned()
                .unwrap_or_default();
            if running {
                return None;
            }
            s.is_timer_running.insert(task_name.clone(), true);
            Some(TimerLogicGuard(task_name))
        })
    }
}

impl Drop for TimerLogicGuard {
    fn drop(&mut self) {
        mutate_state(|s| s.is_timer_running.remove(&self.0));
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct State {
    pub fee_percentiles: Vec<u64>,
    // The network used in the bitcoin canister.
    pub network: Network,
    // Is the bitcoin canister available.
    pub is_available: bool,
    pub address_to_utxos: BTreeMap<Address, BTreeSet<Utxo>>,
    pub utxo_to_address: BTreeMap<Utxo, Address>,
    // Pending transactions.
    pub mempool: BTreeSet<ByteBuf>,
    pub tip_height: u32,

    pub pending_gen_ticket_requests: BTreeMap<Txid, GenTicketRequest>,

    pub finalized_gen_ticket_requests: VecDeque<GenTicketRequest>,

    /// Release_token requests that are waiting to be served, sorted by
    /// received_at.
    pub pending_release_token_requests: BTreeMap<RuneId, Vec<ReleaseTokenRequest>>,

    /// Finalized release_token requests for which we received enough confirmations.
    pub finalized_release_token_requests: BTreeMap<TicketId, FinalizedStatus>,
    /// Process one timer event at a time.

    #[serde(skip)]
    pub is_timer_running: BTreeMap<String, bool>,
    pub hub_principal: Principal,
    pub directive_method: String,
    pub ticket_method: String,
}
pub const DEFAULT_TIP_HEIGHT: u32 = 12;
pub const MAX_FINALIZED_REQUESTS: usize = 10000;
pub const TOKEN_ID: &str = "Bitcoin-runes-HOPE•YOU•GET•RICH";

impl Default for State {
    fn default() -> Self {
        State {
            fee_percentiles: [0; 100].into(),
            network: Network::Mainnet,
            is_available: true,
            address_to_utxos: BTreeMap::new(),
            utxo_to_address: BTreeMap::new(),
            mempool: BTreeSet::new(),
            tip_height: DEFAULT_TIP_HEIGHT,
            pending_gen_ticket_requests: Default::default(),
            pending_release_token_requests: Default::default(),
            finalized_release_token_requests: BTreeMap::new(),
            finalized_gen_ticket_requests: VecDeque::with_capacity(MAX_FINALIZED_REQUESTS),
            is_timer_running: BTreeMap::new(),
            hub_principal: Principal::anonymous(),
            directive_method: "query_directives".to_string(),
            ticket_method: "query_tickets".to_string(),
        }
    }
}

pub fn mutate_state<F, R>(f: F) -> R
where
    F: FnOnce(&mut State) -> R,
{
    STATE.with(|s| f(&mut s.borrow_mut()))
}

pub fn read_state<F, R>(f: F) -> R
where
    F: FnOnce(&State) -> R,
{
    STATE.with(|s| f(&s.borrow()))
}

thread_local! {
    static STATE: RefCell<State> = RefCell::default();
}
