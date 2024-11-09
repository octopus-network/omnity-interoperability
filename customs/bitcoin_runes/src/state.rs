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

use crate::lifecycle::init::InitArgs;
use crate::lifecycle::upgrade::UpgradeArgs;
use crate::{address::BitcoinAddress, ECDSAPublicKey};
use crate::{
    destination::Destination,
    runestone::{Edict, Runestone},
};
use candid::{CandidType, Deserialize, Principal};
pub use ic_btc_interface::Network;
use ic_btc_interface::{OutPoint, Txid, Utxo};
use ic_canister_log::log;
use ic_utils_ensure::{ensure, ensure_eq};
use omnity_types::{
    rune_id::RuneId, Chain, ChainId, ChainState, TicketId, Token, TokenId, TxAction,
};
use serde::Serialize;
use omnity_types::ic_log::INFO;

/// The maximum number of finalized requests that we keep in the
/// history.
const MAX_FINALIZED_REQUESTS: usize = 10000;
const RICH_TOKEN: &str = "840000:846";

pub const BTC_TOKEN: &str = "BTC";
pub const RUNES_TOKEN: &str = "RUNES";
pub const PROD_KEY: &str = "key_1";

thread_local! {
    static __STATE: RefCell<Option<CustomsState>> = RefCell::default();
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

#[derive(candid::CandidType, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuneTxRequest {
    pub ticket_id: TicketId,
    pub action: TxAction,
    pub rune_id: RuneId,
    /// The amount to release token.
    pub amount: u128,
    /// The destination BTC address.
    pub address: BitcoinAddress,
    /// The time at which the customs accepted the request.
    pub received_at: u64,
}

impl From<ReleaseTokenRequest> for RuneTxRequest {
    fn from(value: ReleaseTokenRequest) -> Self {
        Self {
            ticket_id: value.ticket_id,
            action: if matches!(value.address, BitcoinAddress::OpReturn(_)) {
                TxAction::Burn
            } else {
                TxAction::Redeem
            },
            rune_id: value.rune_id,
            amount: value.amount,
            address: value.address,
            received_at: value.received_at,
        }
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
pub struct GenTicketRequestV2 {
    pub address: String,
    pub target_chain_id: String,
    pub receiver: String,
    pub token_id: TokenId,
    pub rune_id: RuneId,
    pub amount: u128,
    pub txid: Txid,
    pub new_utxos: Vec<Utxo>,
    pub received_at: u64,
}

impl From<GenTicketRequest> for GenTicketRequestV2 {
    fn from(value: GenTicketRequest) -> Self {
        Self {
            address: value.address,
            target_chain_id: value.target_chain_id,
            receiver: value.receiver,
            token_id: value.token_id,
            rune_id: value.rune_id,
            amount: value.amount,
            txid: value.txid,
            new_utxos: Default::default(),
            received_at: value.received_at,
        }
    }
}

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct RunesBalance {
    pub rune_id: RuneId,
    pub vout: u32,
    pub amount: u128,
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
    pub rune_id: RuneId,
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
    pub rune_id: RuneId,
    /// The original release token requests that initiated the transaction.
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubmittedBtcTransactionV2 {
    pub rune_id: RuneId,
    pub requests: Vec<RuneTxRequest>,
    pub txid: Txid,
    pub runes_utxos: Vec<RunesUtxo>,
    pub btc_utxos: Vec<Utxo>,
    pub submitted_at: u64,
    pub runes_change_output: RunesChangeOutput,
    pub btc_change_output: BtcChangeOutput,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fee_per_vbyte: Option<u64>,
}

/// The outcome of a release token request.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FinalizedStatus {
    /// The transaction that release token got enough confirmations.
    Confirmed(Txid),
}

/// The status of a Bitcoin transaction that the customs hasn't yet sent to the Bitcoin network.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum InFlightStatus {
    /// Awaiting signatures for transaction inputs.
    Signing,
    /// Awaiting the Bitcoin canister to accept the transaction.
    Sending { txid: Txid },
}

/// The status of a rune tx request.
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

#[derive(CandidType, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GenTicketStatus {
    /// The custom has no data for this request.
    /// The request is either invalid or too old.
    Unknown,
    /// The request is in the queue.
    Pending(GenTicketRequestV2),
    Confirmed(GenTicketRequestV2),
    Finalized(GenTicketRequestV2),
}

/// The state of the Bitcoin Customs.
///
/// Every piece of state of the Customs should be stored as field of this struct.
#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, Serialize)]
pub struct CustomsState {
    /// The bitcoin network that the customs will connect to
    pub btc_network: Network,

    pub chain_id: String,

    /// The name of the [EcdsaKeyId]. Use "dfx_test_key" for local replica and "test_key_1" for
    /// a testing key for testnet and mainnet
    pub ecdsa_key_name: String,

    /// The Customs ECDSA public key
    pub ecdsa_public_key: Option<ECDSAPublicKey>,

    pub prod_ecdsa_public_key: Option<ECDSAPublicKey>,

    /// The minimum number of confirmations on the Bitcoin chain.
    pub min_confirmations: u32,

    /// Maximum time of nanoseconds that a transaction should spend in the queue
    /// before being sent.
    pub max_time_in_queue_nanos: u64,

    pub generate_ticket_counter: u64,

    pub release_token_counter: u64,

    /// The transaction has just entered the memory pool
    /// or has not reached sufficient confirmation.
    pub pending_gen_ticket_requests: BTreeMap<Txid, GenTicketRequestV2>,

    /// The transaction needs to wait for runes oracle to update the runes balance.
    pub confirmed_gen_ticket_requests: BTreeMap<Txid, GenTicketRequestV2>,

    pub finalized_gen_ticket_requests: VecDeque<GenTicketRequestV2>,

    /// rune tx requests that are waiting to be served, sorted by
    /// received_at.
    pub pending_rune_tx_requests: BTreeMap<RuneId, Vec<RuneTxRequest>>,

    /// Finalized rune tx requests for which we received enough confirmations.
    pub finalized_rune_tx_requests: BTreeMap<TicketId, FinalizedStatus>,

    /// The identifiers of rune tx requests which we're currently signing a
    /// transaction or sending to the Bitcoin network.
    pub requests_in_flight: BTreeMap<TicketId, InFlightStatus>,

    /// BTC transactions waiting for finalization.
    pub submitted_transactions: Vec<SubmittedBtcTransactionV2>,

    /// Transactions that likely didn't make it into the mempool.
    pub stuck_transactions: Vec<SubmittedBtcTransactionV2>,

    /// Maps ID of a stuck transaction to the ID of the corresponding replacement transaction.
    pub replacement_txid: BTreeMap<Txid, Txid>,

    /// Maps ID of a replacement transaction to the ID of the corresponding stuck transaction.
    pub rev_replacement_txid: BTreeMap<Txid, Txid>,

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

    pub counterparties: BTreeMap<ChainId, Chain>,

    pub tokens: BTreeMap<TokenId, (RuneId, Token)>,

    // Next index of query tickets from hub
    pub next_ticket_seq: u64,

    // Next index of query directives from hub
    pub next_directive_seq: u64,

    pub hub_principal: Principal,

    pub runes_oracles: BTreeSet<Principal>,

    pub rpc_url: Option<String>,

    /// Process one timer event at a time.
    #[serde(skip)]
    pub is_timer_running: bool,

    #[serde(skip)]
    pub is_process_directive_msg: bool,

    #[serde(skip)]
    pub is_process_ticket_msg: bool,

    /// The mode in which the customs runs.
    pub chain_state: ChainState,

    pub last_fee_per_vbyte: Vec<u64>,

    #[serde(default)]
    pub fee_token_factor: Option<u128>,

    #[serde(default)]
    pub target_chain_factor: BTreeMap<ChainId, u128>,

    #[serde(default)]
    pub fee_collector_address: String,
}

impl CustomsState {
    pub fn get_transfer_fee_info(&self, target_chain_id: &ChainId) -> (Option<u128>, Option<String>) {
        if target_chain_id.ne("Ethereum") {
            return (None, None);
        }
        let fee = self.fee_token_factor.and_then(|f| {
            self.target_chain_factor.get(target_chain_id).map(|factor| f * factor)
        });
        (fee, fee.map(|_|self.fee_collector_address.clone()))
    }

    pub fn reinit(
        &mut self,
        InitArgs {
            btc_network,
            ecdsa_key_name,
            max_time_in_queue_nanos,
            min_confirmations,
            chain_state,
            hub_principal,
            runes_oracle_principal,
            chain_id,
        }: InitArgs,
    ) {
        self.btc_network = btc_network.into();
        self.ecdsa_key_name = ecdsa_key_name;
        self.max_time_in_queue_nanos = max_time_in_queue_nanos;
        self.chain_state = chain_state;
        self.hub_principal = hub_principal;
        self.runes_oracles = BTreeSet::from_iter(vec![runes_oracle_principal]);
        self.chain_id = chain_id;
        if let Some(min_confirmations) = min_confirmations {
            self.min_confirmations = min_confirmations;
        }
    }

    pub fn upgrade(
        &mut self,
        UpgradeArgs {
            max_time_in_queue_nanos,
            min_confirmations,
            chain_state,
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
                    INFO,
                    "Didn't increase min_confirmations to {} (current value: {})",
                    min_conf,
                    self.min_confirmations
                );
            }
        }
        if let Some(chain_state) = chain_state {
            self.chain_state = chain_state;
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

        for (_, requests) in &self.pending_rune_tx_requests {
            for (l, r) in requests.iter().zip(requests.iter().skip(1)) {
                ensure!(
                    l.received_at <= r.received_at,
                    "pending rune tx requests are not sorted by receive time"
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

    pub fn get_ecdsa_key(&self, token: Option<String>) -> (String, ECDSAPublicKey) {
        let pub_key = self
            .ecdsa_public_key
            .clone()
            .expect("the ECDSA public key must be initialized");
        if cfg!(feature = "non_prod") {
            return (self.ecdsa_key_name.clone(), pub_key);
        }
        let prod_pub_key = self
            .prod_ecdsa_public_key
            .clone()
            .expect("the ECDSA public key must be initialized");
        match token {
            // the token field in the destination of user deposit address are all None in previous version
            None => (self.ecdsa_key_name.clone(), pub_key),
            Some(token) => {
                // main change address in previous version
                if token == RICH_TOKEN || token == BTC_TOKEN {
                    (self.ecdsa_key_name.clone(), pub_key)
                } else {
                    (PROD_KEY.into(), prod_pub_key)
                }
            }
        }
    }

    // public for only for tests
    pub(crate) fn add_utxos(&mut self, destination: Destination, utxos: Vec<Utxo>, is_runes: bool) {
        if utxos.is_empty() {
            return;
        }

        let bucket: &mut BTreeSet<Utxo> = self
            .utxos_state_destinations
            .entry(destination.clone())
            .or_default();

        for utxo in &utxos {
            // It is possible that utxo has been added via update_btc_utxos.
            if self.outpoint_utxos.contains_key(&utxo.outpoint) {
                continue;
            }
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

    pub(crate) fn update_runes_balance(&mut self, txid: Txid, balance: RunesBalance) {
        let outpoint = OutPoint {
            txid,
            vout: balance.vout,
        };
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
        if let Some(req) = self.confirmed_gen_ticket_requests.get(&tx_id) {
            return GenTicketStatus::Confirmed(req.clone());
        }
        match self
            .finalized_gen_ticket_requests
            .iter()
            .find(|req| req.txid == tx_id)
        {
            Some(req) => GenTicketStatus::Finalized(req.clone()),
            None => GenTicketStatus::Unknown,
        }
    }

    /// Returns the status of the rune tx request with the specified
    /// identifier.
    pub fn rune_tx_status(&self, ticket_id: &TicketId) -> ReleaseTokenStatus {
        if self
            .pending_rune_tx_requests
            .iter()
            .any(|(_, reqs)| reqs.iter().any(|req| req.ticket_id.eq(ticket_id)))
        {
            return ReleaseTokenStatus::Pending;
        }

        if let Some(status) = self.requests_in_flight.get(ticket_id).cloned() {
            return match status {
                InFlightStatus::Signing => ReleaseTokenStatus::Signing,
                InFlightStatus::Sending { txid } => ReleaseTokenStatus::Sending(txid.to_string()),
            };
        }

        if let Some(txid) = self.submitted_transactions.iter().find_map(|tx| {
            (tx.requests.iter().any(|r| r.ticket_id.eq(ticket_id))).then_some(tx.txid)
        }) {
            return ReleaseTokenStatus::Submitted(txid.to_string());
        }

        match self.finalized_rune_tx_requests.get(ticket_id) {
            Some(FinalizedStatus::Confirmed(txid)) => {
                return ReleaseTokenStatus::Confirmed(txid.to_string())
            }
            None => (),
        }

        ReleaseTokenStatus::Unknown
    }

    /// Returns true if the pending requests queue has enough requests to form a
    /// batch or there are old enough requests to form a batch.
    pub fn can_form_a_batch(&self, rune_id: RuneId, min_pending: usize, now: u64) -> bool {
        match self.pending_rune_tx_requests.get(&rune_id) {
            Some(requests) => {
                if requests.iter().any(|req| req.action == TxAction::Mint) {
                    return true;
                }
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

    /// Forms a batch of rune tx requests that the customs can fulfill.
    pub fn build_batch(&mut self, rune_id: RuneId, max_size: usize) -> Vec<RuneTxRequest> {
        assert!(self.pending_rune_tx_requests.contains_key(&rune_id));

        let available_utxos_value = self
            .available_runes_utxos
            .iter()
            .filter(|u| u.runes.rune_id.eq(&rune_id))
            .map(|u| u.runes.amount)
            .sum::<u128>();
        let mut batch = vec![];
        let mut tx_amount = 0;
        let requests = self.pending_rune_tx_requests.entry(rune_id).or_default();

        if let Some(pos) = requests.iter().position(|req| req.action == TxAction::Mint) {
            let req = requests.remove(pos);
            batch.push(req);
            return batch;
        }

        let mut edicts = vec![];
        for req in std::mem::take(requests) {
            edicts.push(Edict {
                id: req.rune_id.into(),
                amount: req.amount,
                output: 0,
            });
            // Maybe there is a better optimized version.
            let script = Runestone {
                edicts: edicts.clone(),
                mint: None,
            }
            .encipher();
            if script.len() > 82
                || available_utxos_value < req.amount + tx_amount
                || batch.len() >= max_size
            {
                // Put this request back to the queue until we have enough liquid UTXOs.
                requests.push(req);
                edicts.pop();
            } else {
                tx_amount += req.amount;
                batch.push(req.clone());
            }
        }

        batch
    }

    /// Returns the total number of all rune tx requests that we haven't
    /// finalized yet.
    pub fn count_incomplete_rune_tx_requests(&self) -> usize {
        self.pending_rune_tx_requests.len()
            + self.requests_in_flight.len()
            + self
                .submitted_transactions
                .iter()
                .map(|tx| tx.requests.len())
                .sum::<usize>()
    }

    /// Returns true if there is a pending rune tx request with the given
    /// identifier.
    fn has_pending_request(&self, ticket_id: &TicketId) -> bool {
        self.pending_rune_tx_requests
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
            self.push_finalized_rune_tx(request.ticket_id, FinalizedStatus::Confirmed(*txid));
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
    pub(crate) fn replace_transaction(
        &mut self,
        old_txid: &Txid,
        mut tx: SubmittedBtcTransactionV2,
    ) {
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

    /// Removes a pending release_token request with the specified block index.
    fn remove_pending_request(&mut self, ticket_id: TicketId) -> Option<RuneTxRequest> {
        for (_, requests) in &mut self.pending_rune_tx_requests {
            match requests.iter().position(|req| req.ticket_id == ticket_id) {
                Some(pos) => return Some(requests.remove(pos)),
                None => {}
            }
        }
        None
    }

    /// Marks the specified release_token request as in-flight.
    ///
    /// # Panics
    ///
    /// This function panics if there is a pending release_token request with the
    /// same identifier.
    pub fn push_in_flight_request(&mut self, ticket_id: TicketId, status: InFlightStatus) {
        assert!(!self.has_pending_request(&ticket_id));

        self.requests_in_flight.insert(ticket_id, status);
    }

    /// Returns a release_token requests back to the pending queue.
    ///
    /// # Panics
    ///
    /// This function panics if there is a pending release_token request with the
    /// same identifier.
    pub fn push_from_in_flight_to_pending_requests(&mut self, requests: Vec<RuneTxRequest>) {
        for req in requests.iter() {
            assert!(!self.has_pending_request(&req.ticket_id));
            self.requests_in_flight.remove(&req.ticket_id);

            let bucket = self
                .pending_rune_tx_requests
                .entry(req.rune_id)
                .or_default();
            bucket.push(req.clone());
            bucket.sort_by_key(|r| r.received_at);
        }
    }

    /// Push back a release token request to the ordered queue.
    ///
    /// # Panics
    ///
    /// This function panics if the new request breaks the request ordering in
    /// the queue.
    pub fn push_back_pending_request(&mut self, request: RuneTxRequest) {
        let bucket = self
            .pending_rune_tx_requests
            .entry(request.rune_id)
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
    /// This function panics if there is a pending release_token request with the
    /// same identifier as one of the request used for the transaction.
    pub fn push_submitted_transaction(&mut self, tx: SubmittedBtcTransactionV2) {
        for req in tx.requests.iter() {
            assert!(!self.has_pending_request(&req.ticket_id));
            self.requests_in_flight.remove(&req.ticket_id);
        }
        self.submitted_transactions.push(tx);
    }

    /// Marks the specified release_token request as finalized.
    ///
    /// # Panics
    ///
    /// This function panics if there is a pending release_token request with the
    /// same identifier.
    fn push_finalized_rune_tx(&mut self, ticket_id: TicketId, status: FinalizedStatus) {
        assert!(!self.has_pending_request(&ticket_id));

        self.finalized_rune_tx_requests.insert(ticket_id, status);
    }

    fn push_finalized_ticket(&mut self, req: GenTicketRequestV2) {
        assert!(!self.confirmed_gen_ticket_requests.contains_key(&req.txid));

        if self.finalized_gen_ticket_requests.len() >= MAX_FINALIZED_REQUESTS {
            self.finalized_gen_ticket_requests.pop_front();
        }
        self.finalized_gen_ticket_requests.push_back(req)
    }

    /// Filters out known UTXOs from the given UTXO list.
    pub fn new_utxos(&self, mut utxos: Vec<Utxo>, tx_id: Option<Txid>) -> Vec<Utxo> {
        utxos.retain(|utxo| {
            !self.outpoint_utxos.contains_key(&utxo.outpoint)
                && tx_id.map_or(true, |t| utxo.outpoint.txid == t)
        });
        utxos
    }

    /// Checks whether the internal state of the customs matches the other state
    /// semantically (the state holds the same data, but maybe in a slightly
    /// different form).
    pub fn check_semantically_eq(&self, other: &Self) -> Result<(), String> {
        ensure_eq!(
            self.btc_network,
            other.btc_network,
            "btc_network does not match"
        );
        ensure_eq!(self.chain_id, other.chain_id, "chain_id does not match");
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
            self.max_time_in_queue_nanos,
            other.max_time_in_queue_nanos,
            "max_time_in_queue_nanos does not match"
        );
        ensure_eq!(
            self.pending_gen_ticket_requests,
            other.pending_gen_ticket_requests,
            "pending_gen_ticket_requests do not match"
        );
        ensure_eq!(
            self.confirmed_gen_ticket_requests,
            other.confirmed_gen_ticket_requests,
            "pending_gen_ticket_requests do not match"
        );
        ensure_eq!(
            self.finalized_gen_ticket_requests,
            other.finalized_gen_ticket_requests,
            "pending_gen_ticket_requests do not match"
        );
        ensure_eq!(
            self.next_ticket_seq,
            other.next_ticket_seq,
            "next_ticket_seq do not match"
        );
        ensure_eq!(
            self.next_directive_seq,
            other.next_directive_seq,
            "next_directive_seq do not match"
        );
        ensure_eq!(
            self.finalized_rune_tx_requests,
            other.finalized_rune_tx_requests,
            "finalized_requests do not match"
        );
        ensure_eq!(
            self.requests_in_flight,
            other.requests_in_flight,
            "requests_in_flight do not match"
        );
        ensure_eq!(
            self.available_fee_utxos,
            other.available_fee_utxos,
            "available_fee_utxos do not match"
        );
        ensure_eq!(
            self.available_runes_utxos,
            other.available_runes_utxos,
            "available_utxos do not match"
        );
        ensure_eq!(
            self.outpoint_utxos,
            other.outpoint_utxos,
            "outpoint_utxos do not match"
        );
        ensure_eq!(
            self.outpoint_destination,
            other.outpoint_destination,
            "outpoint_destination do not match"
        );
        ensure_eq!(
            self.utxos_state_destinations,
            other.utxos_state_destinations,
            "utxos_state_addresses do not match"
        );
        ensure_eq!(
            self.counterparties,
            other.counterparties,
            "counterparties do not match"
        );
        ensure_eq!(self.tokens, other.tokens, "tokens do not match");
        ensure_eq!(
            self.hub_principal,
            other.hub_principal,
            "hub_principal does not match"
        );
        ensure_eq!(
            self.runes_oracles,
            other.runes_oracles,
            "runes_oracles does not match"
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
            self.pending_rune_tx_requests.len(),
            other.pending_rune_tx_requests.len(),
            "size of pending_release_token_requests do not match"
        );
        for (rune_id, requests) in &self.pending_rune_tx_requests {
            let my_requests = as_sorted_vec(requests.iter().cloned(), |r| r.ticket_id.clone());
            match other.pending_rune_tx_requests.get(rune_id) {
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
            chain_id: args.chain_id,
            ecdsa_key_name: args.ecdsa_key_name,
            ecdsa_public_key: None,
            prod_ecdsa_public_key: None,
            min_confirmations: args
                .min_confirmations
                .unwrap_or(crate::lifecycle::init::DEFAULT_MIN_CONFIRMATIONS),
            max_time_in_queue_nanos: args.max_time_in_queue_nanos,
            generate_ticket_counter: 0,
            release_token_counter: 0,
            pending_gen_ticket_requests: Default::default(),
            confirmed_gen_ticket_requests: Default::default(),
            finalized_gen_ticket_requests: VecDeque::with_capacity(MAX_FINALIZED_REQUESTS),
            pending_rune_tx_requests: Default::default(),
            finalized_rune_tx_requests: BTreeMap::new(),
            requests_in_flight: Default::default(),
            submitted_transactions: Default::default(),
            replacement_txid: Default::default(),
            rev_replacement_txid: Default::default(),
            stuck_transactions: Default::default(),
            finalized_requests_count: 0,
            available_runes_utxos: Default::default(),
            available_fee_utxos: Default::default(),
            outpoint_utxos: Default::default(),
            outpoint_destination: Default::default(),
            utxos_state_destinations: Default::default(),
            counterparties: Default::default(),
            tokens: Default::default(),
            next_ticket_seq: 0,
            next_directive_seq: 0,
            is_timer_running: false,
            is_process_directive_msg: false,
            is_process_ticket_msg: false,
            chain_state: args.chain_state,
            hub_principal: args.hub_principal,
            runes_oracles: BTreeSet::from_iter(vec![args.runes_oracle_principal]),
            rpc_url: None,
            last_fee_per_vbyte: vec![1; 100],
            fee_token_factor: None,
            target_chain_factor: Default::default(),
            fee_collector_address: "".to_string(),
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
