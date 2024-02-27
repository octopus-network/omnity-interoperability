//! State management module.
//!
//! The state is stored in the global thread-level variable `__STATE`.
//! This module provides utility functions to manage the state. Most
//! code should use those functions instead of touching `__STATE` directly.
use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet, VecDeque},
};

pub mod audit;
pub mod eventlog;

use crate::destination::Destination;
use crate::lifecycle::init::InitArgs;
use crate::lifecycle::upgrade::UpgradeArgs;
use crate::logs::P0;
use crate::{address::BitcoinAddress, ECDSAPublicKey};
use candid::{CandidType, Deserialize, Principal};
pub use ic_btc_interface::Network;
use ic_btc_interface::{OutPoint, Txid, Utxo};
use ic_canister_log::log;
use ic_utils_ensure::{ensure, ensure_eq};
use omnity_types::TicketId;
use serde::Serialize;

/// The maximum number of finalized BTC retrieval requests that we keep in the
/// history.
const MAX_FINALIZED_REQUESTS: usize = 100;

thread_local! {
    static __STATE: RefCell<Option<CustomsState>> = RefCell::default();
}

// A pending release token request
#[derive(candid::CandidType, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseTokenRequest {
    pub ticket_id: TicketId,
    pub runes_id: RunesId,
    /// The amount to release token.
    pub amount: u128,
    /// The destination BTC address.
    pub address: BitcoinAddress,
    /// The time at which the minter accepted the request.
    pub received_at: u64,
}

#[derive(candid::CandidType, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenTicketRequest {
    pub address: String,
    pub target_chain_id: String,
    pub receiver: String,
    pub runes_id: RunesId,
    pub amount: u128,
    pub tx_id: Txid,
}

pub type RunesId = u128;

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct RunesBalance {
    pub runes_id: RunesId,
    pub value: u128,
}

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct RunesUtxo {
    pub raw: Utxo,
    // A utxo is only bound to one runes token
    pub runes: RunesBalance,
}

/// A transaction output storing the custom's runes change.
#[derive(candid::CandidType, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunesChangeOutput {
    pub runes_id: RunesId,
    /// The index of the output in the transaction.
    pub vout: u32,
    /// The value of the output.
    pub value: u128,
}

/// A transaction output storing the custom's BTC change.
#[derive(candid::CandidType, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BtcChangeOutput {
    /// The index of the output in the transaction.
    pub vout: u32,
    /// The value of the output.
    pub value: u64,
}

/// Represents a transaction sent to the Bitcoin network.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubmittedBtcTransaction {
    pub runes_id: RunesId,
    /// The original retrieve_btc requests that initiated the transaction.
    pub requests: Vec<ReleaseTokenRequest>,
    /// The identifier of the unconfirmed transaction.
    pub txid: Txid,
    /// The list of Runes UTXOs we used in the transaction.
    pub runes_utxos: Vec<RunesUtxo>,
    /// The list of BTC UTXOs we used in the transaction.
    pub btc_utxos: Vec<Utxo>,
    /// The IC time at which we submitted the Bitcoin transaction.
    pub submitted_at: u64,
    /// The tx runes change output from the submitted transaction that the customs owns.
    pub runes_change_output: RunesChangeOutput,
    /// The tx btc change output from the submitted transaction that the customs owns.
    pub btc_change_output: BtcChangeOutput,
    /// Fee per vbyte in millisatoshi.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fee_per_vbyte: Option<u64>,
}

/// Pairs a retrieve_btc request with its outcome.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FinalizedTokenRetrieval {
    /// The original retrieve_btc request that initiated the transaction.
    pub request: ReleaseTokenRequest,
    /// The status of the finalized request.
    pub status: FinalizedStatus,
}

/// The outcome of a retrieve_btc request.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FinalizedStatus {
    /// The transaction that retrieves BTC got enough confirmations.
    Confirmed(Txid),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FinalizedTicket {
    pub request: GenTicketRequest,
    pub status: FinalizedTicketStatus,
}

/// The outcome of a generate_ticket request.
#[derive(candid::CandidType, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FinalizedTicketStatus {
    Invalid,
    Finalized,
}

/// The status of a Bitcoin transaction that the minter hasn't yet sent to the Bitcoin network.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum InFlightStatus {
    /// Awaiting signatures for transaction inputs.
    Signing,
    /// Awaiting the Bitcoin canister to accept the transaction.
    Sending { txid: Txid },
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
    Sending(Txid),
    /// Awaiting for confirmations on the transaction satisfying this request.
    Submitted(Txid),
    /// Confirmed a transaction satisfying this request.
    Confirmed(Txid),
}

#[derive(CandidType, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GenTicketStatus {
    /// The custom has no data for this request.
    /// The request is either invalid or too old.
    Unknown,
    /// The request is in the queue.
    Pending(GenTicketRequest),
    Invalid,
    Finalized,
}

/// Controls which operations the minter can perform.
#[derive(candid::CandidType, Clone, Debug, PartialEq, Eq, serde::Deserialize, Serialize)]
pub enum Mode {
    /// Custom's state is read-only.
    ReadOnly,
    /// Transport operations are restricted.
    TransportRestricted,
    /// Release operations are restricted.
    ReleaseRestricted,
    /// No restrictions on the custom interactions.
    GeneralAvailability,
}

impl Mode {
    /// Returns Ok if the transport operation is avaliable.
    pub fn is_transport_available_for(&self) -> Result<(), String> {
        match self {
            Self::GeneralAvailability | Self::ReleaseRestricted => Ok(()),
            Self::ReadOnly | Self::TransportRestricted => {
                Err("transport operations are restricted".to_string())
            }
        }
    }

    /// Returns Ok if the release operation is avaliable.
    pub fn is_release_available_for(&self) -> Result<(), String> {
        match self {
            Self::GeneralAvailability | Self::TransportRestricted => Ok(()),
            Self::ReadOnly | Self::ReleaseRestricted => {
                Err("release operations are restricted".to_string())
            }
        }
    }
}

impl Default for Mode {
    fn default() -> Self {
        Self::GeneralAvailability
    }
}

/// Indicates that fee distribution overdrafted.
#[derive(Clone, Copy, Debug)]
pub struct Overdraft(pub u64);

/// The state of the Bitcoin Customs.
///
/// Every piece of state of the Customs should be stored as field of this struct.
#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, Serialize)]
pub struct CustomsState {
    /// The bitcoin network that the minter will connect to
    pub btc_network: Network,

    /// The name of the [EcdsaKeyId]. Use "dfx_test_key" for local replica and "test_key_1" for
    /// a testing key for testnet and mainnet
    pub ecdsa_key_name: String,

    /// The Minter ECDSA public key
    pub ecdsa_public_key: Option<ECDSAPublicKey>,

    /// The minimum number of confirmations on the Bitcoin chain.
    pub min_confirmations: u32,

    /// Maximum time of nanoseconds that a transaction should spend in the queue
    /// before being sent.
    pub max_time_in_queue_nanos: u64,

    pub generate_ticket_counter: u64,

    pub release_token_counter: u64,

    pub pending_gen_ticket_requests: BTreeMap<Txid, GenTicketRequest>,

    pub finalized_gen_ticket_requests: VecDeque<FinalizedTicket>,

    // Start index of query tickets from hub
    pub next_release_ticket_index: u64,

    /// Release_token requests that are waiting to be served, sorted by
    /// received_at.
    pub pending_release_token_requests: BTreeMap<RunesId, Vec<ReleaseTokenRequest>>,

    /// The identifiers of retrieve_btc requests which we're currently signing a
    /// transaction or sending to the Bitcoin network.
    pub requests_in_flight: BTreeMap<TicketId, InFlightStatus>,

    /// BTC transactions waiting for finalization.
    pub submitted_transactions: Vec<SubmittedBtcTransaction>,

    /// Transactions that likely didn't make it into the mempool.
    pub stuck_transactions: Vec<SubmittedBtcTransaction>,

    /// Maps ID of a stuck transaction to the ID of the corresponding replacement transaction.
    pub replacement_txid: BTreeMap<Txid, Txid>,

    /// Maps ID of a replacement transaction to the ID of the corresponding stuck transaction.
    pub rev_replacement_txid: BTreeMap<Txid, Txid>,

    /// Finalized release_token requests for which we received enough confirmations.
    pub finalized_release_token_requests: VecDeque<FinalizedTokenRetrieval>,

    /// The total number of finalized requests.
    pub finalized_requests_count: u64,

    /// The set of Runes UTXOs unused in pending transactions.
    pub available_runes_utxos: BTreeSet<RunesUtxo>,

    /// The set of BTC UTXOs unused in pending transactions.
    pub available_fee_utxos: BTreeSet<Utxo>,

    /// The mapping from output points to the utxo.
    pub outpoint_utxos: BTreeMap<OutPoint, Utxo>,

    /// The mapping from output points to the destination to which they
    /// belong.
    pub outpoint_destination: BTreeMap<OutPoint, Destination>,

    /// The map of known destinations to their utxos.
    pub utxos_state_destinations: BTreeMap<Destination, BTreeSet<Utxo>>,

    pub hub_principal: Principal,

    /// Process one timer event at a time.
    #[serde(skip)]
    pub is_timer_running: bool,

    /// The mode in which the minter runs.
    pub mode: Mode,

    pub last_fee_per_vbyte: Vec<u64>,
}

impl CustomsState {
    pub fn reinit(
        &mut self,
        InitArgs {
            btc_network,
            ecdsa_key_name,
            max_time_in_queue_nanos,
            min_confirmations,
            mode,
            hub_principal,
        }: InitArgs,
    ) {
        self.btc_network = btc_network.into();
        self.ecdsa_key_name = ecdsa_key_name;
        self.max_time_in_queue_nanos = max_time_in_queue_nanos;
        self.mode = mode;
        self.hub_principal = hub_principal;
        if let Some(min_confirmations) = min_confirmations {
            self.min_confirmations = min_confirmations;
        }
    }

    pub fn upgrade(
        &mut self,
        UpgradeArgs {
            max_time_in_queue_nanos,
            min_confirmations,
            mode,
            hub_principal,
        }: UpgradeArgs,
    ) {
        if let Some(max_time_in_queue_nanos) = max_time_in_queue_nanos {
            self.max_time_in_queue_nanos = max_time_in_queue_nanos;
        }
        if let Some(min_conf) = min_confirmations {
            if min_conf < self.min_confirmations {
                self.min_confirmations = min_conf;
            } else {
                log!(
                    P0,
                    "Didn't increase min_confirmations to {} (current value: {})",
                    min_conf,
                    self.min_confirmations
                );
            }
        }
        if let Some(mode) = mode {
            self.mode = mode;
        }
        if let Some(hub_principal) = hub_principal {
            self.hub_principal = hub_principal;
        }
    }

    pub fn validate_config(&self) {
        if self.ecdsa_key_name.is_empty() {
            ic_cdk::trap("ecdsa_key_name is not set");
        }
    }

    pub fn check_invariants(&self) -> Result<(), String> {
        for utxo in self.available_runes_utxos.iter() {
            ensure!(
                self.outpoint_destination.contains_key(&utxo.raw.outpoint),
                "the output_account map is missing an entry for {:?}",
                utxo.raw.outpoint
            );

            ensure!(
                self.utxos_state_destinations
                    .iter()
                    .any(|(_, utxos)| utxos.contains(&utxo.raw)),
                "available utxo {:?} does not belong to any destination",
                utxo
            );
        }

        for (dest, utxos) in self.utxos_state_destinations.iter() {
            for utxo in utxos.iter() {
                ensure_eq!(
                    self.outpoint_destination.get(&utxo.outpoint),
                    Some(dest),
                    "missing outpoint destination for {:?}",
                    utxo.outpoint
                );
            }
        }

        for (_, requests) in &self.pending_release_token_requests {
            for (l, r) in requests.iter().zip(requests.iter().skip(1)) {
                ensure!(
                    l.received_at <= r.received_at,
                    "pending retrieve_btc requests are not sorted by receive time"
                );
            }
        }

        for tx in &self.stuck_transactions {
            ensure!(
                self.replacement_txid.contains_key(&tx.txid),
                "stuck transaction {} does not have a replacement id",
                &tx.txid,
            );
        }

        for (old_txid, new_txid) in &self.replacement_txid {
            ensure!(
                self.stuck_transactions
                    .iter()
                    .any(|tx| &tx.txid == old_txid),
                "not found stuck transaction {}",
                old_txid,
            );

            ensure!(
                self.submitted_transactions
                    .iter()
                    .chain(self.stuck_transactions.iter())
                    .any(|tx| &tx.txid == new_txid),
                "not found replacement transaction {}",
                new_txid,
            );
        }

        ensure_eq!(
            self.replacement_txid.len(),
            self.rev_replacement_txid.len(),
            "direct and reverse TX replacement links don't match"
        );
        for (old_txid, new_txid) in &self.replacement_txid {
            ensure_eq!(
                self.rev_replacement_txid.get(new_txid),
                Some(old_txid),
                "no back link for {} -> {} TX replacement",
                old_txid,
                new_txid,
            );
        }

        Ok(())
    }

    // public for only for tests
    pub(crate) fn add_utxos(&mut self, destination: Destination, utxos: Vec<Utxo>, is_runes: bool) {
        if utxos.is_empty() {
            return;
        }

        let bucket = self
            .utxos_state_destinations
            .entry(destination.clone())
            .or_default();

        for utxo in &utxos {
            self.outpoint_destination
                .insert(utxo.outpoint.clone(), destination.clone());
            self.outpoint_utxos
                .insert(utxo.outpoint.clone(), utxo.clone());
            bucket.insert(utxo.clone());
            if !is_runes {
                self.available_fee_utxos.insert(utxo.clone());
            }
        }

        #[cfg(debug_assertions)]
        self.check_invariants()
            .expect("state invariants are violated");
    }

    pub(crate) fn update_runes_balance(&mut self, outpoint: OutPoint, balance: RunesBalance) {
        assert!(self.outpoint_utxos.contains_key(&outpoint));
        if let Some(utxo) = self.outpoint_utxos.get(&outpoint) {
            assert!(self.available_runes_utxos.insert(RunesUtxo {
                raw: utxo.clone(),
                runes: balance,
            }));
        }
    }

    pub fn generate_ticket_status(&self, tx_id: Txid) -> GenTicketStatus {
        if let Some(req) = self.pending_gen_ticket_requests.get(&tx_id) {
            return GenTicketStatus::Pending(req.clone());
        }
        match self
            .finalized_gen_ticket_requests
            .iter()
            .find(|req| req.request.tx_id == tx_id)
            .map(|r| r.status.clone())
        {
            Some(FinalizedTicketStatus::Finalized) => GenTicketStatus::Finalized,
            Some(FinalizedTicketStatus::Invalid) => GenTicketStatus::Invalid,
            None => GenTicketStatus::Unknown,
        }
    }

    /// Returns the status of the release_token request with the specified
    /// identifier.
    pub fn release_token_status(&self, ticket_id: &TicketId) -> ReleaseTokenStatus {
        if self
            .pending_release_token_requests
            .iter()
            .any(|(_, reqs)| reqs.iter().any(|req| req.ticket_id.eq(ticket_id)))
        {
            return ReleaseTokenStatus::Pending;
        }

        if let Some(status) = self.requests_in_flight.get(ticket_id).cloned() {
            return match status {
                InFlightStatus::Signing => ReleaseTokenStatus::Signing,
                InFlightStatus::Sending { txid } => ReleaseTokenStatus::Sending(txid),
            };
        }

        if let Some(txid) = self.submitted_transactions.iter().find_map(|tx| {
            (tx.requests.iter().any(|r| r.ticket_id.eq(ticket_id))).then_some(tx.txid)
        }) {
            return ReleaseTokenStatus::Submitted(txid);
        }

        match self
            .finalized_release_token_requests
            .iter()
            .find(|finalized_request| finalized_request.request.ticket_id.eq(ticket_id))
            .map(|final_req| final_req.status.clone())
        {
            Some(FinalizedStatus::Confirmed(txid)) => return ReleaseTokenStatus::Confirmed(txid),
            None => (),
        }

        ReleaseTokenStatus::Unknown
    }

    /// Returns true if the pending requests queue has enough requests to form a
    /// batch or there are old enough requests to form a batch.
    pub fn can_form_a_batch(&self, runes_id: &RunesId, min_pending: usize, now: u64) -> bool {
        match self.pending_release_token_requests.get(runes_id) {
            Some(requests) => {
                if requests.len() >= min_pending {
                    return true;
                }
                match requests.first() {
                    Some(req) => self.max_time_in_queue_nanos < now.saturating_sub(req.received_at),
                    None => false,
                }
            }
            None => false,
        }
    }

    /// Forms a batch of retrieve_btc requests that the minter can fulfill.
    pub fn build_batch(&mut self, runes_id: &RunesId, max_size: usize) -> Vec<ReleaseTokenRequest> {
        assert!(self.pending_release_token_requests.contains_key(runes_id));

        let available_utxos_value = self
            .available_runes_utxos
            .iter()
            .filter(|u| u.runes.runes_id.eq(runes_id))
            .map(|u| u.runes.value)
            .sum::<u128>();
        let mut batch = vec![];
        let mut tx_amount = 0;
        let requests = self
            .pending_release_token_requests
            .entry(*runes_id)
            .or_default();
        for req in std::mem::take(requests) {
            if available_utxos_value < req.amount + tx_amount || batch.len() >= max_size {
                // Put this request back to the queue until we have enough liquid UTXOs.
                requests.push(req);
            } else {
                tx_amount += req.amount;
                batch.push(req.clone());
            }
        }

        batch
    }

    /// Returns the total number of all release_token requests that we haven't
    /// finalized yet.
    pub fn count_incomplete_release_token_requests(&self) -> usize {
        self.pending_release_token_requests.len()
            + self.requests_in_flight.len()
            + self
                .submitted_transactions
                .iter()
                .map(|tx| tx.requests.len())
                .sum::<usize>()
    }

    /// Returns true if there is a pending retrieve_btc request with the given
    /// identifier.
    fn has_pending_request(&self, ticket_id: &TicketId) -> bool {
        self.pending_release_token_requests
            .iter()
            .any(|(_, reqs)| reqs.iter().any(|req| req.ticket_id.eq(ticket_id)))
    }

    fn forget_utxo(&mut self, utxo: &Utxo) {
        if let Some(destination) = self.outpoint_destination.remove(&utxo.outpoint) {
            self.outpoint_utxos.remove(&utxo.outpoint);
            let last_utxo = match self.utxos_state_destinations.get_mut(&destination) {
                Some(utxo_set) => {
                    utxo_set.remove(utxo);
                    utxo_set.is_empty()
                }
                None => false,
            };
            if last_utxo {
                self.utxos_state_destinations.remove(&destination);
            }
        }
    }

    pub(crate) fn finalize_transaction(&mut self, txid: &Txid) {
        let finalized_tx = if let Some(pos) = self
            .submitted_transactions
            .iter()
            .position(|tx| &tx.txid == txid)
        {
            self.submitted_transactions.swap_remove(pos)
        } else if let Some(pos) = self
            .stuck_transactions
            .iter()
            .position(|tx| &tx.txid == txid)
        {
            self.stuck_transactions.swap_remove(pos)
        } else {
            ic_cdk::trap(&format!(
                "Attempted to finalized a non-existent transaction {}",
                txid
            ));
        };

        for utxo in finalized_tx.runes_utxos.iter() {
            self.forget_utxo(&utxo.raw);
        }
        for utxo in finalized_tx.btc_utxos.iter() {
            self.forget_utxo(utxo);
        }
        self.finalized_requests_count += finalized_tx.requests.len() as u64;
        for request in finalized_tx.requests {
            self.push_finalized_release_token(FinalizedTokenRetrieval {
                request,
                status: FinalizedStatus::Confirmed(*txid),
            });
        }

        self.cleanup_tx_replacement_chain(txid);
    }

    fn cleanup_tx_replacement_chain(&mut self, confirmed_txid: &Txid) {
        let mut txids_to_remove = BTreeSet::new();

        // Collect transactions preceding the confirmed transaction.
        let mut to_edge = *confirmed_txid;
        while let Some(from_edge) = self.replacement_txid.remove(&to_edge) {
            debug_assert_eq!(self.rev_replacement_txid.get(&from_edge), Some(&to_edge));
            self.rev_replacement_txid.remove(&from_edge);
            txids_to_remove.insert(from_edge);
            to_edge = from_edge;
        }

        // Collect transactions replacing the confirmed transaction.
        let mut from_edge = *confirmed_txid;
        while let Some(to_edge) = self.rev_replacement_txid.remove(&from_edge) {
            debug_assert_eq!(self.replacement_txid.get(&to_edge), Some(&from_edge));
            txids_to_remove.insert(to_edge);
            from_edge = to_edge;
        }

        for txid in &txids_to_remove {
            self.replacement_txid.remove(txid);
            self.rev_replacement_txid.remove(txid);
        }

        if txids_to_remove.is_empty() {
            return;
        }

        self.submitted_transactions
            .retain(|tx| !txids_to_remove.contains(&tx.txid));
        self.stuck_transactions
            .retain(|tx| !txids_to_remove.contains(&tx.txid));
    }

    pub(crate) fn longest_resubmission_chain_size(&self) -> usize {
        self.submitted_transactions
            .iter()
            .map(|tx| {
                let mut txid = &tx.txid;
                let mut len = 0;
                while let Some(older_txid) = self.rev_replacement_txid.get(txid) {
                    len += 1;
                    txid = older_txid;
                }
                len
            })
            .max()
            .unwrap_or_default()
    }

    /// Replaces a stuck transaction with a newly sent transaction.
    pub(crate) fn replace_transaction(&mut self, old_txid: &Txid, mut tx: SubmittedBtcTransaction) {
        assert_ne!(old_txid, &tx.txid);
        assert_eq!(
            self.replacement_txid.get(old_txid),
            None,
            "replacing the same transaction twice is not allowed"
        );
        for req in tx.requests.iter() {
            assert!(!self.has_pending_request(&req.ticket_id));
        }

        let new_txid = tx.txid;
        let pos = self
            .submitted_transactions
            .iter()
            .position(|tx| &tx.txid == old_txid)
            .expect("BUG: attempted to replace an unknown transaction");

        std::mem::swap(&mut self.submitted_transactions[pos], &mut tx);
        // tx points to the old transaction now.
        debug_assert_eq!(&tx.txid, old_txid);

        self.stuck_transactions.push(tx);
        self.replacement_txid.insert(*old_txid, new_txid);
        self.rev_replacement_txid.insert(new_txid, *old_txid);
    }

    /// Returns the identifier of the most recent replacement transaction for the given stuck
    /// transaction id.
    pub fn find_last_replacement_tx(&self, txid: &Txid) -> Option<&Txid> {
        let mut last = self.replacement_txid.get(txid)?;
        while let Some(newer_txid) = self.replacement_txid.get(last) {
            last = newer_txid;
        }
        Some(last)
    }

    /// Removes a pending retrieve_btc request with the specified block index.
    fn remove_pending_request(&mut self, ticket_id: TicketId) -> Option<ReleaseTokenRequest> {
        for (_, requests) in &mut self.pending_release_token_requests {
            match requests.iter().position(|req| req.ticket_id == ticket_id) {
                Some(pos) => return Some(requests.remove(pos)),
                None => {}
            }
        }
        None
    }

    /// Marks the specified retrieve_btc request as in-flight.
    ///
    /// # Panics
    ///
    /// This function panics if there is a pending retrieve_btc request with the
    /// same identifier.
    pub fn push_in_flight_request(&mut self, ticket_id: TicketId, status: InFlightStatus) {
        assert!(!self.has_pending_request(&ticket_id));

        self.requests_in_flight.insert(ticket_id, status);
    }

    /// Returns a retrieve_btc requests back to the pending queue.
    ///
    /// # Panics
    ///
    /// This function panics if there is a pending retrieve_btc request with the
    /// same identifier.
    pub fn push_from_in_flight_to_pending_requests(&mut self, requests: Vec<ReleaseTokenRequest>) {
        for req in requests.iter() {
            assert!(!self.has_pending_request(&req.ticket_id));
            self.requests_in_flight.remove(&req.ticket_id);

            let bucket = self
                .pending_release_token_requests
                .entry(req.runes_id)
                .or_default();
            bucket.push(req.clone());
            bucket.sort_by_key(|r| r.received_at);
        }
    }

    /// Push back a retrieve_btc request to the ordered queue.
    ///
    /// # Panics
    ///
    /// This function panics if the new request breaks the request ordering in
    /// the queue.
    pub fn push_back_pending_request(&mut self, request: ReleaseTokenRequest) {
        let bucket = self
            .pending_release_token_requests
            .entry(request.runes_id)
            .or_default();
        if let Some(last_req) = bucket.last() {
            assert!(last_req.received_at <= request.received_at);
        }
        bucket.push(request);
    }

    /// Records a BTC transaction as submitted and updates statuses of all
    /// requests involved.
    ///
    /// # Panics
    ///
    /// This function panics if there is a pending retrieve_btc request with the
    /// same identifier as one of the request used for the transaction.
    pub fn push_submitted_transaction(&mut self, tx: SubmittedBtcTransaction) {
        for req in tx.requests.iter() {
            assert!(!self.has_pending_request(&req.ticket_id));
            self.requests_in_flight.remove(&req.ticket_id);
        }
        self.submitted_transactions.push(tx);
    }

    /// Marks the specified retrieve_btc request as finalized.
    ///
    /// # Panics
    ///
    /// This function panics if there is a pending retrieve_btc request with the
    /// same identifier.
    fn push_finalized_release_token(&mut self, req: FinalizedTokenRetrieval) {
        assert!(!self.has_pending_request(&req.request.ticket_id));

        if self.finalized_release_token_requests.len() >= MAX_FINALIZED_REQUESTS {
            self.finalized_release_token_requests.pop_front();
        }
        self.finalized_release_token_requests.push_back(req)
    }

    fn push_finalized_ticket(&mut self, req: FinalizedTicket) {
        assert!(!self
            .pending_gen_ticket_requests
            .contains_key(&req.request.tx_id));

        if self.finalized_gen_ticket_requests.len() >= MAX_FINALIZED_REQUESTS {
            self.finalized_gen_ticket_requests.pop_front();
        }
        self.finalized_gen_ticket_requests.push_back(req)
    }

    /// Filters out known UTXOs of the given destination from the given UTXO list.
    pub fn new_utxos_for_destination(
        &self,
        mut utxos: Vec<Utxo>,
        destination: &Destination,
        tx_id: Option<Txid>,
    ) -> Vec<Utxo> {
        let maybe_existing_utxos = self.utxos_state_destinations.get(destination);
        utxos.retain(|utxo| {
            !maybe_existing_utxos
                .map(|utxos| utxos.contains(utxo))
                .unwrap_or(false)
                && tx_id.map_or(true, |t| utxo.outpoint.txid == t)
        });
        utxos
    }

    /// Checks whether the internal state of the minter matches the other state
    /// semantically (the state holds the same data, but maybe in a slightly
    /// different form).
    pub fn check_semantically_eq(&self, other: &Self) -> Result<(), String> {
        ensure_eq!(
            self.btc_network,
            other.btc_network,
            "btc_network does not match"
        );
        ensure_eq!(
            self.ecdsa_key_name,
            other.ecdsa_key_name,
            "ecdsa_key_name does not match"
        );
        ensure_eq!(
            self.min_confirmations,
            other.min_confirmations,
            "min_confirmations does not match"
        );
        ensure_eq!(
            self.finalized_release_token_requests,
            other.finalized_release_token_requests,
            "finalized_requests do not match"
        );
        ensure_eq!(
            self.requests_in_flight,
            other.requests_in_flight,
            "requests_in_flight do not match"
        );
        ensure_eq!(
            self.available_runes_utxos,
            other.available_runes_utxos,
            "available_utxos do not match"
        );
        ensure_eq!(
            self.utxos_state_destinations,
            other.utxos_state_destinations,
            "utxos_state_addresses do not match"
        );

        let my_txs = as_sorted_vec(self.submitted_transactions.iter().cloned(), |tx| tx.txid);
        let other_txs = as_sorted_vec(other.submitted_transactions.iter().cloned(), |tx| tx.txid);
        ensure_eq!(my_txs, other_txs, "submitted_transactions do not match");

        ensure_eq!(
            self.stuck_transactions,
            other.stuck_transactions,
            "stuck_transactions do not match"
        );

        ensure_eq!(
            self.pending_release_token_requests.len(),
            other.pending_release_token_requests.len(),
            "size of pending_release_token_requests do not match"
        );
        for (runes_id, requests) in &self.pending_release_token_requests {
            let my_requests = as_sorted_vec(requests.iter().cloned(), |r| r.ticket_id.clone());
            match other.pending_release_token_requests.get(&runes_id) {
                Some(requests) => {
                    let other_requests =
                        as_sorted_vec(requests.iter().cloned(), |r| r.ticket_id.clone());
                    ensure_eq!(
                        my_requests,
                        other_requests,
                        "pending_release_token_requests do not match"
                    );
                }
                None => return Err(String::from("pending_release_token_requests do not match")),
            }
        }

        ensure_eq!(
            self.replacement_txid,
            other.replacement_txid,
            "replacement_txid maps do not match"
        );

        ensure_eq!(
            self.rev_replacement_txid,
            other.rev_replacement_txid,
            "rev_replacement_txid maps do not match"
        );

        Ok(())
    }
}

fn as_sorted_vec<T, K: Ord>(values: impl Iterator<Item = T>, key: impl Fn(&T) -> K) -> Vec<T> {
    let mut v: Vec<_> = values.collect();
    v.sort_by_key(key);
    v
}

impl From<InitArgs> for CustomsState {
    fn from(args: InitArgs) -> Self {
        Self {
            btc_network: args.btc_network.into(),
            ecdsa_key_name: args.ecdsa_key_name,
            ecdsa_public_key: None,
            min_confirmations: args
                .min_confirmations
                .unwrap_or(crate::lifecycle::init::DEFAULT_MIN_CONFIRMATIONS),
            max_time_in_queue_nanos: args.max_time_in_queue_nanos,
            generate_ticket_counter: 0,
            release_token_counter: 0,
            pending_gen_ticket_requests: Default::default(),
            next_release_ticket_index: 0,
            pending_release_token_requests: Default::default(),
            requests_in_flight: Default::default(),
            submitted_transactions: Default::default(),
            replacement_txid: Default::default(),
            rev_replacement_txid: Default::default(),
            stuck_transactions: Default::default(),
            finalized_release_token_requests: VecDeque::with_capacity(MAX_FINALIZED_REQUESTS),
            finalized_gen_ticket_requests: VecDeque::with_capacity(MAX_FINALIZED_REQUESTS),
            finalized_requests_count: 0,
            available_runes_utxos: Default::default(),
            available_fee_utxos: Default::default(),
            outpoint_utxos: Default::default(),
            outpoint_destination: Default::default(),
            utxos_state_destinations: Default::default(),
            is_timer_running: false,
            mode: args.mode,
            hub_principal: args.hub_principal,
            last_fee_per_vbyte: vec![1; 100],
        }
    }
}

/// Take the current state.
///
/// After calling this function the state won't be initialized anymore.
/// Panics if there is no state.
pub fn take_state<F, R>(f: F) -> R
where
    F: FnOnce(CustomsState) -> R,
{
    __STATE.with(|s| f(s.take().expect("State not initialized!")))
}

/// Mutates (part of) the current state using `f`.
///
/// Panics if there is no state.
pub fn mutate_state<F, R>(f: F) -> R
where
    F: FnOnce(&mut CustomsState) -> R,
{
    __STATE.with(|s| f(s.borrow_mut().as_mut().expect("State not initialized!")))
}

/// Read (part of) the current state using `f`.
///
/// Panics if there is no state.
pub fn read_state<F, R>(f: F) -> R
where
    F: FnOnce(&CustomsState) -> R,
{
    __STATE.with(|s| f(s.borrow().as_ref().expect("State not initialized!")))
}

/// Replaces the current state.
pub fn replace_state(state: CustomsState) {
    __STATE.with(|s| {
        *s.borrow_mut() = Some(state);
    });
}
