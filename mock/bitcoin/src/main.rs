use candid::{candid_method, CandidType};
use ic_btc_interface::{
    Address, GetCurrentFeePercentilesRequest, GetUtxosRequest, GetUtxosResponse,
    MillisatoshiPerByte, Network, Utxo, UtxosFilterInRequest,
};
use ic_cdk::api::management_canister::bitcoin::{BitcoinNetwork, SendTransactionRequest};
use ic_cdk::query;
use ic_cdk_macros::{init, update};
use omnity_types::{TicketId, TokenId};
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

// We use 12 as the default tip height to mint all
// the utxos with height 1 in the customs.
const DEFAULT_TIP_HEIGHT: u32 = 12;
const MAX_FINALIZED_REQUESTS: usize = 10000;

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
}

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

#[init]
fn init(network: Option<Network>) {
    let network = network.unwrap_or(Network::Regtest);

    STATE.with(|s| {
        let state = State {
            network,
            fee_percentiles: [0; 100].into(),
            is_available: true,
            utxo_to_address: BTreeMap::new(),
            address_to_utxos: BTreeMap::new(),
            mempool: BTreeSet::new(),
            tip_height: DEFAULT_TIP_HEIGHT,
            pending_gen_ticket_requests: Default::default(),
            pending_release_token_requests: Default::default(),
            finalized_release_token_requests: BTreeMap::new(),
            finalized_gen_ticket_requests: VecDeque::with_capacity(MAX_FINALIZED_REQUESTS),
        };
        *s.borrow_mut() = state;
    });
}

#[candid_method(update)]
#[update]
fn set_tip_height(tip_height: u32) {
    mutate_state(|s| s.tip_height = tip_height);
}

#[candid_method(update)]
#[update]
fn bitcoin_get_utxos(utxos_request: GetUtxosRequest) -> GetUtxosResponse {
    read_state(|s| {
        assert_eq!(utxos_request.network, s.network.into());

        let mut utxos = s
            .address_to_utxos
            .get(&utxos_request.address)
            .cloned()
            .unwrap_or_default()
            .iter()
            .cloned()
            .collect::<Vec<Utxo>>();

        if let Some(UtxosFilterInRequest::MinConfirmations(min_confirmations)) =
            utxos_request.filter
        {
            utxos.retain(|u| s.tip_height + 1 >= u.height + min_confirmations);
        }

        GetUtxosResponse {
            utxos,
            tip_block_hash: vec![],
            tip_height: s.tip_height,
            // TODO Handle pagination.
            next_page: None,
        }
    })
}

#[candid_method(update)]
#[update]
fn push_utxos_to_address(req: bitcoin_mock::PushUtxosToAddress) {
    mutate_state(|s| {
        for (address, utxos) in &req.utxos {
            for utxo in utxos {
                s.utxo_to_address.insert(utxo.clone(), address.clone());
                s.address_to_utxos
                    .entry(address.clone())
                    .or_default()
                    .insert(utxo.clone());
            }
        }
    });
}

#[candid_method(update)]
#[update]
fn remove_utxo(utxo: Utxo) {
    let address = read_state(|s| s.utxo_to_address.get(&utxo).cloned().unwrap());
    mutate_state(|s| {
        s.utxo_to_address.remove(&utxo);
        s.address_to_utxos
            .get_mut(&address)
            .expect("utxo not found at address")
            .remove(&utxo);
    });
}

#[candid_method(update)]
#[update]
fn bitcoin_get_current_fee_percentiles(
    _: GetCurrentFeePercentilesRequest,
) -> Vec<MillisatoshiPerByte> {
    read_state(|s| s.fee_percentiles.clone())
}

#[candid_method(update)]
#[update]
fn set_fee_percentiles(fee_percentiles: Vec<MillisatoshiPerByte>) {
    mutate_state(|s| s.fee_percentiles = fee_percentiles);
}

#[candid_method(update)]
#[update]
fn bitcoin_send_transaction(transaction: SendTransactionRequest) {
    mutate_state(|s| {
        let cdk_network = match transaction.network {
            BitcoinNetwork::Mainnet => Network::Mainnet,
            BitcoinNetwork::Testnet => Network::Testnet,
            BitcoinNetwork::Regtest => Network::Regtest,
        };
        assert_eq!(cdk_network, s.network);
        if s.is_available {
            s.mempool.insert(ByteBuf::from(transaction.transaction));
        }
    })
}

#[candid_method(update)]
#[update]
fn change_availability(is_available: bool) {
    mutate_state(|s| s.is_available = is_available);
}

#[candid_method(update)]
#[update]
fn get_mempool() -> Vec<ByteBuf> {
    read_state(|s| s.mempool.iter().cloned().collect::<Vec<ByteBuf>>())
}

#[candid_method(update)]
#[update]
fn reset_mempool() {
    mutate_state(|s| s.mempool = BTreeSet::new());
}

#[update]
pub fn generate_ticket(args: GenerateTicketArgs) {
    println!("received generate_ticket: {:?}",args);
    let rune_id = RuneId::from_str(&args.rune_id).unwrap();
    let token_id = args.rune_id.clone();
    let txid = Txid::from_str(&args.txid).unwrap();

    let request = GenTicketRequest {
        address: "bc1qmh0chcr9f73a3ynt90k0w8qsqlydr4a6espnj6".to_owned(),
        target_chain_id: args.target_chain_id,
        receiver: args.receiver,
        token_id,
        rune_id,
        amount: args.amount,
        txid,
        received_at: ic_cdk::api::time(),
    };

    mutate_state(|s| {
        s.pending_gen_ticket_requests.insert(request.txid, request);
    })
}

//mock: ticket be finalized
#[update]
pub fn mock_finalized_ticket(ticket_id: TicketId) {
    let txid = Txid::from_str(&ticket_id).unwrap();
    mutate_state(|s| {
        s.pending_gen_ticket_requests.remove(&txid);
    })
}

#[query]
fn get_pending_gen_ticket_size() -> u64 {
    let size = read_state(|s| s.pending_gen_ticket_requests.len() as u64);
    size
}

#[query]
fn get_pending_gen_tickets(from_seq: usize, limit: usize) -> Vec<GenTicketRequest> {
    read_state(|s| {
        s.pending_gen_ticket_requests
            .iter()
            .skip(from_seq)
            .take(limit)
            .map(|(_, req)| req.to_owned())
            .collect::<Vec<_>>()
    })
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

#[update]
fn mock_finalized_release_token(ticket_id: TicketId, status: FinalizedStatus) {
    mutate_state(|s| {
        s.finalized_release_token_requests.insert(ticket_id, status);
    })
}

#[query]
fn release_token_status(ticket_id: String) -> ReleaseTokenStatus {
    read_state(|s| {
        match s.finalized_release_token_requests.get(&ticket_id) {
            Some(FinalizedStatus::Confirmed(txid)) => {
                return ReleaseTokenStatus::Confirmed(txid.to_string())
            }
            None => (),
        }

        ReleaseTokenStatus::Unknown
    })
}

fn main() {}
ic_cdk::export_candid!();
