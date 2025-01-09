use std::cell::RefCell;
use std::collections::BTreeMap;
use std::time::Duration;

// use bitcoin::Address;
use candid::{CandidType, Principal};
use ic_canister_log::log;
use ic_ic00_types::DerivationPath;
use ic_stable_structures::writer::Writer;
use ic_stable_structures::StableBTreeMap;
use serde::{Deserialize, Serialize};

use omnity_types::ic_log::INFO;
use omnity_types::{Chain, ChainId, ChainState, Directive, Network, Seq, Ticket, TicketId, Token, TokenId};

use crate::constants::MIN_NANOS;
use crate::custom_to_dogecoin::SendTicketResult;
use crate::doge::chainparams::{chain_from_key_bits, ChainParams, KeyBits, MAIN_NET_DOGE};
use crate::doge::ecdsa::derive_public_key;
use crate::doge::script::Address;
use crate::doge::transaction::Transaction;
use crate::doge::script;
use crate::errors::CustomsError;
use crate::service::InitArgs;
use crate::stable_memory;
use crate::stable_memory::Memory;
use crate::types::{deserialize_hex, wrap_to_customs_error, Destination, ECDSAPublicKey, GenTicketStatus, LockTicketRequest, MultiRpcConfig, ReleaseTokenStatus, RpcConfig, Txid, Utxo};

thread_local! {
    static STATE: RefCell<Option<DogeState>> = const {RefCell::new(None)};
}

#[derive(Deserialize, Serialize)]
pub struct DogeState {
    pub admins: Vec<Principal>,
    pub doge_chain: KeyBits,
    pub doge_fee_rate: Option<u64>,
    pub min_confirmations: u32,
    #[serde(default)]
    pub min_deposit_amount: u64,
    pub ecdsa_key_name: String,
    pub ecdsa_public_key: Option<ECDSAPublicKey>,
    pub hub_principal: Principal,
    pub chain_id: String,
    pub tokens: BTreeMap<TokenId, Token>,
    pub counterparties: BTreeMap<ChainId, Chain>,
    pub chain_state: ChainState,
    pub next_ticket_seq: u64,
    pub next_directive_seq: u64,
    pub next_consume_ticket_seq: u64,

    #[serde(skip, default = "crate::stable_memory::init_unlock_tickets_queue")]
    pub tickets_queue: StableBTreeMap<Seq, Ticket, Memory>,
    pub flight_unlock_ticket_map: BTreeMap<Seq, SendTicketResult>,
    // pub finalized_unlock_ticket_map: BTreeMap<Seq, SendTicketResult>,
    pub ticket_id_seq_indexer: BTreeMap<TicketId, Seq>,

    #[serde(skip, default = "crate::stable_memory::init_unlock_ticket_results")]
    pub finalized_unlock_ticket_results_map: StableBTreeMap<Seq, SendTicketResult, Memory>,

    //lock tickets storage
    pub pending_lock_ticket_requests: BTreeMap<Txid, LockTicketRequest>,
    #[serde(skip, default = "crate::stable_memory::init_lock_ticket_requests")]
    pub finalized_lock_ticket_requests_map: StableBTreeMap<Txid, LockTicketRequest, Memory>,

    #[serde(skip, default = "crate::stable_memory::init_directives_queue")]
    pub directives_queue: StableBTreeMap<u64, Directive, Memory>,
    #[serde(skip)]
    pub is_timer_running: BTreeMap<String, bool>,

    pub deposited_utxo: Vec<(Utxo, Destination)>,
    // omnity fee
    #[serde(default)]
    pub fee_token: String,
    pub fee_collector: String,
    pub fee_token_factor: Option<u128>,
    pub target_chain_factor: BTreeMap<ChainId, u128>,

    // rpc
    // https://dashboard.tatum.io, use custom rpc method
    #[serde(default)]
    pub tatum_api_config: RpcConfig,

    #[serde(default)]
    pub default_doge_rpc_config: RpcConfig,

    #[serde(default)]
    pub multi_rpc_config: MultiRpcConfig,
    // #[serde(default)]
    // pub fee_payment_address: String,
    #[serde(skip, default = "crate::stable_memory::init_deposit_fee_tx_set")]
    pub deposit_fee_tx_set: StableBTreeMap<String, (), Memory>,
    #[serde(default)]
    pub fee_payment_utxo: Vec<Utxo>,
}

#[derive(Serialize, Deserialize, CandidType, Clone)]
pub struct StateProfile {
    pub admins: Vec<Principal>,
    pub doge_chain: KeyBits,
    pub doge_fee_rate: Option<u64>,
    pub min_confirmations: u32,
    pub min_deposit_amount: u64,
    pub ecdsa_key_name: String,
    pub ecdsa_public_key: Option<ECDSAPublicKey>,
    pub hub_principal: Principal,
    pub chain_id: String,
    pub tokens: BTreeMap<TokenId, Token>,
    pub counterparties: BTreeMap<ChainId, Chain>,
    pub chain_state: ChainState,
    pub next_ticket_seq: u64,
    pub next_directive_seq: u64,
    pub next_consume_ticket_seq: u64,
    pub flight_unlock_ticket_map: BTreeMap<Seq, SendTicketResult>,
    pub pending_lock_ticket_requests: BTreeMap<String, LockTicketRequest>,

    pub deposited_utxo: Vec<(Utxo, Destination)>,

    pub fee_token: String,
    pub fee_collector: String,
    pub fee_token_factor: Option<u128>,
    pub target_chain_factor: BTreeMap<ChainId, u128>,

    pub tatum_rpc_config: RpcConfig,
    pub multi_rpc_config: MultiRpcConfig,
    pub fee_payment_utxo: Vec<Utxo>,

}

impl From<&DogeState> for StateProfile {
    fn from(value: &DogeState) -> Self {
        StateProfile {
            admins: value.admins.clone(),
            doge_chain: value.doge_chain,
            doge_fee_rate: value.doge_fee_rate,
            min_confirmations: value.min_confirmations,
            min_deposit_amount: value.min_deposit_amount,
            ecdsa_key_name: value.ecdsa_key_name.clone(),
            ecdsa_public_key: value.ecdsa_public_key.clone(),
            hub_principal: value.hub_principal,
            chain_id: value.chain_id.clone(),
            tokens: value.tokens.clone(),
            counterparties: value.counterparties.clone(),
            chain_state: value.chain_state.clone(),
            next_ticket_seq: value.next_ticket_seq,
            next_directive_seq: value.next_directive_seq,
            next_consume_ticket_seq: value.next_consume_ticket_seq,
            pending_lock_ticket_requests: value.pending_lock_ticket_requests.iter().map(|(k, v)| (k.to_string(), v.clone())).collect(),
            flight_unlock_ticket_map: value.flight_unlock_ticket_map.clone(),
            deposited_utxo: value.deposited_utxo.clone(),
            fee_token: value.fee_token.clone(),
            fee_collector: value.fee_collector.clone(),
            fee_token_factor: value.fee_token_factor,
            target_chain_factor: value.target_chain_factor.clone(),
            tatum_rpc_config: value.tatum_api_config.clone(),
            multi_rpc_config: value.multi_rpc_config.clone(),
            fee_payment_utxo: value.fee_payment_utxo.clone(),
        }
    }
}

impl DogeState {
    pub fn init(args: InitArgs) -> anyhow::Result<Self> {
        let ret = DogeState {
            admins: args.admins,
            doge_fee_rate: Option::None,
            doge_chain: MAIN_NET_DOGE,
            hub_principal: args.hub_principal,
            chain_id: args.chain_id,
            // reveal_utxo_index: Default::default(),
            tokens: Default::default(),
            counterparties: Default::default(),
            chain_state: ChainState::Active,
            ecdsa_key_name: Network::Mainnet.key_id().name,
            flight_unlock_ticket_map: BTreeMap::default(),
            ecdsa_public_key: None,
            next_ticket_seq: 0,
            next_directive_seq: 0,
            next_consume_ticket_seq: 0,
            tickets_queue: StableBTreeMap::init(crate::stable_memory::get_unlock_tickets_memory()),
            directives_queue: StableBTreeMap::init(crate::stable_memory::get_directives_memory()),
            is_timer_running: Default::default(),
            pending_lock_ticket_requests: Default::default(),
            // finalized_lock_ticket_requests: Default::default(),
            // deposit_addr_utxo: vec![],
            fee_collector: "".to_string(),
            fee_token_factor: None,
            min_confirmations: 4,
            min_deposit_amount: 0,
            // finalized_unlock_ticket_map: Default::default(),
            ticket_id_seq_indexer: Default::default(),
            target_chain_factor: Default::default(),
            fee_token: args.fee_token,
            deposited_utxo: vec![],
            tatum_api_config: RpcConfig::default(),
            default_doge_rpc_config: RpcConfig::default(),
            multi_rpc_config: MultiRpcConfig::default(),
            // fee_payment_address: String::default(),
            deposit_fee_tx_set: StableBTreeMap::init(crate::stable_memory::get_deposit_tx_memory()),
            fee_payment_utxo: vec![],
            finalized_unlock_ticket_results_map: StableBTreeMap::init(crate::stable_memory::get_unlock_ticket_results_memory()),
            finalized_lock_ticket_requests_map: StableBTreeMap::init(crate::stable_memory::get_lock_ticket_requests_memory()),
        };
        Ok(ret)
    }

    pub fn pre_upgrade(&self) {
        let mut state_bytes = vec![];
        let _ = ciborium::ser::into_writer(self, &mut state_bytes);
        let len = state_bytes.len() as u32;
        let mut memory = crate::stable_memory::get_upgrade_stash_memory();
        let mut writer = Writer::new(&mut memory, 0);
        writer
            .write(&len.to_le_bytes())
            .expect("failed to save hub state len");
        writer
            .write(&state_bytes)
            .expect("failed to save hub state");
    }

    pub fn post_upgrade() {
        use ic_stable_structures::Memory;
        let memory = stable_memory::get_upgrade_stash_memory();
        // Read the length of the state bytes.
        let mut state_len_bytes = [0; 4];
        memory.read(0, &mut state_len_bytes);
        let state_len = u32::from_le_bytes(state_len_bytes) as usize;
        let mut state_bytes = vec![0; state_len];
        memory.read(4, &mut state_bytes);
        let state: DogeState =
            ciborium::de::from_reader(&*state_bytes).expect("failed to decode state");
        replace_state(state);
        log!(INFO, "post upgradge sucessed!!");
    }

    pub fn get_transfer_fee_info(
        &self,
        target_chain_id: &ChainId,
    ) -> (Option<u128>, Option<String>) {
        if target_chain_id.ne("Ethereum") {
            return (None, None);
        }
        let fee = self.fee_token_factor.and_then(|f| {
            self.target_chain_factor
                .get(target_chain_id)
                .map(|factor| f * factor)
        });
        (fee, fee.map(|_| self.fee_collector.clone()))
    }

    pub fn generate_ticket_status(&self, tx_id: &Txid) -> GenTicketStatus {
        if let Some(req) = self.pending_lock_ticket_requests.get(tx_id) {
            return GenTicketStatus::Pending(req.clone());
        }

        if let Some(req) = self.finalized_lock_ticket_requests_map.get(tx_id) {
            return GenTicketStatus::Finalized(req.clone());
        } else {
            return GenTicketStatus::Unknown;
        }
        
        // match self
        //     .finalized_lock_ticket_requests
        //     .iter()
        //     .find(|req| req.1.txid == tx_id.clone().into())
        // {
        //     Some(req) => GenTicketStatus::Finalized(req.1.clone()),
        //     None => GenTicketStatus::Unknown,
        // }
    }

    pub fn unlock_tx_status(&self, ticket_id: &TicketId) -> ReleaseTokenStatus {
        let seq = self.ticket_id_seq_indexer.get(ticket_id).cloned();

        if seq.is_none() {
            return ReleaseTokenStatus::Unknown;
        }
        let seq = seq.unwrap();
        if let Some(status) = self.flight_unlock_ticket_map.get(&seq).cloned() {
            if status.success {
                let txid = status.txid;
                return ReleaseTokenStatus::Submitted(txid.to_string());
            } else {
                return ReleaseTokenStatus::Unknown;
            }
        }
        if let Some(tx) = self.finalized_unlock_ticket_results_map.get(&seq) {
            let txid = tx.txid.to_string();
            return ReleaseTokenStatus::Confirmed(txid);
        }
        ReleaseTokenStatus::Pending
    }

    pub fn chain_params(&self) -> &'static ChainParams {
        chain_from_key_bits(self.doge_chain)
    }

    pub fn get_address(&self, dest: Destination) -> Result<(Address, Vec<u8>), CustomsError> {
        let pk = self
            .ecdsa_public_key
            .clone()
            .ok_or(CustomsError::ECDSAPublicKeyNotFound)?;

        let pk = derive_public_key(&pk, dest.derivation_path());
        Ok((script::p2pkh_address(&pk.public_key, self.chain_params())?, pk.public_key))
    }

    pub fn save_utxo(&mut self, ticket_request: LockTicketRequest) -> Result<(), CustomsError> {
        let transaction: Transaction = deserialize_hex(&ticket_request.transaction_hex).map_err(wrap_to_customs_error)?;
        let destination = Destination::new(ticket_request.target_chain_id.clone(), ticket_request.receiver.clone(), None);
        // let first_tx_out = transaction.output.first().cloned().ok_or(CustomsError::CustomError("transaction output is empty".to_string()))?;
        // let receiver = first_tx_out.get_mainnet_address().ok_or(CustomsError::CustomError("first output receiver address is empty".to_string()))?;
        let destination_address = self.get_address(destination.clone())?.0.to_string();
        for (i, tx_out) in transaction.output.iter().enumerate() {
            if let Some(tx_out_address) = tx_out.get_mainnet_address() {
                if tx_out_address == destination_address {
                    self.deposited_utxo.push(
                        (Utxo {
                            txid: ticket_request.txid.clone(),
                            vout: i as u32,
                            value: tx_out.value,
                        },
                        destination.clone(),
                    ));
                }
            }
        }

        Ok(())
    }
}

pub async fn init_ecdsa_public_key() -> Result<ECDSAPublicKey, CustomsError> {
    if let Some(pub_key) = read_state(|s| s.ecdsa_public_key.clone()) {
        return Ok(pub_key);
    };
    let key_name = read_state(|s| s.ecdsa_key_name.clone());
    let pub_key = crate::management::ecdsa_public_key(key_name, DerivationPath::new(vec![]))
        .await
        .unwrap_or_else(|e| ic_cdk::trap(&format!("failed to retrieve ECDSA public key: {e}")));
    mutate_state(|s| {
        s.ecdsa_public_key = Some(pub_key.clone());
        // s.deposit_pubkey = Some(bytes_to_hex(pub_key.public_key.as_slice()));
        // let doge_main_addr = script::p2pkh_address(&pub_key.public_key, s.chain_params())?;
        // let address = pubkey_to_doge_address(&pub_key.clone());
        // let deposit_addr = doge_main_addr.to_string();
        // s.deposit_addr = Some(deposit_addr);

        Ok(())
    })?;
    Ok(pub_key)
}

pub fn finalization_time_estimate(
    min_confirmations: u32,
) -> Duration {
    Duration::from_nanos(
        (min_confirmations + 1) as u64 * 1 * MIN_NANOS
    )
}

pub fn mutate_state<F, R>(f: F) -> R
where
    F: FnOnce(&mut DogeState) -> R,
{
    STATE.with(|s| f(s.borrow_mut().as_mut().expect("State not initialized!")))
}

pub fn read_state<F, R>(f: F) -> R
where
    F: FnOnce(&DogeState) -> R,
{
    STATE.with(|s| f(s.borrow().as_ref().expect("State not initialized!")))
}

/// Replaces the current state.
pub fn replace_state(state: DogeState) {
    STATE.with(|s| {
        *s.borrow_mut() = Some(state);
    });
}
