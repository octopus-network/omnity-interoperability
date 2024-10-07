use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::str::FromStr;
use std::time::Duration;

use bitcoin::Address;
use candid::{CandidType, Principal};
use ic_btc_interface::{Network, Txid};
use ic_ic00_types::DerivationPath;
use ic_stable_structures::writer::Writer;
use ic_stable_structures::StableBTreeMap;
use serde::{Deserialize, Serialize};

use omnity_types::ChainState::Active;
use omnity_types::ChainType::ExecutionChain;
use omnity_types::{Chain, ChainId, ChainState, Directive, Seq, Ticket, TicketId, Token, TokenId};

use crate::bitcoin::{main_bitcoin_address, ECDSAPublicKey};
use crate::constants::{MIN_NANOS, SEC_NANOS};
use crate::custom_to_bitcoin::SendTicketResult;
use crate::ord::builder::Utxo;
use crate::service::InitArgs;
use crate::stable_memory;
use crate::stable_memory::Memory;
use crate::types::{GenTicketStatus, LockTicketRequest, ReleaseTokenStatus};

thread_local! {
    static STATE: RefCell<Option<Brc20State >> = RefCell::new(None);
}

#[derive(Deserialize, Serialize)]
pub struct Brc20State {
    pub admins: Vec<Principal>,
    pub min_confirmations: u8,
    pub btc_network: Network,
    pub ecdsa_key_name: String,
    pub ecdsa_public_key: Option<ECDSAPublicKey>,
    pub deposit_addr: Option<String>,
    pub deposit_pubkey: Option<String>,
    pub indexer_principal: Principal,
    pub hub_principal: Principal,
    pub chain_id: String,
    pub reveal_utxo_index: BTreeSet<String>,
    pub tokens: BTreeMap<TokenId, Token>,
    pub counterparties: BTreeMap<ChainId, Chain>,

    pub chain_state: ChainState,
    pub next_ticket_seq: u64,
    pub next_directive_seq: u64,
    pub next_consume_ticket_seq: u64,
    pub next_consume_directive_seq: u64,

    //unlock tickets storage
    #[serde(skip, default = "crate::stable_memory::init_unlock_tickets_queue")]
    pub tickets_queue: StableBTreeMap<u64, Ticket, Memory>,
    pub flight_unlock_ticket_map: BTreeMap<Seq, SendTicketResult>,
    pub finalized_unlock_ticket_map: BTreeMap<Seq, SendTicketResult>,
    pub ticket_id_seq_indexer: BTreeMap<TicketId, Seq>,
    //lock tickets storage
    pub pending_lock_ticket_requests: BTreeMap<Txid, LockTicketRequest>,
    pub finalized_lock_ticket_requests: BTreeMap<Txid, LockTicketRequest>,

    #[serde(skip, default = "crate::stable_memory::init_directives_queue")]
    pub directives_queue: StableBTreeMap<u64, Directive, Memory>,
    #[serde(skip)]
    pub is_timer_running: BTreeMap<String, bool>,
    pub deposit_addr_utxo: Vec<Utxo>,
}

#[derive(Serialize, Deserialize, CandidType, Clone)]
pub struct StateProfile {
    pub admins: Vec<Principal>,
    pub min_confirmations: u8,
    pub btc_network: Network,
    pub ecdsa_key_name: String,
    pub ecdsa_public_key: Option<ECDSAPublicKey>,
    pub deposit_addr: Option<String>,
    pub deposit_pubkey: Option<String>,
    pub indexer_principal: Principal,
    pub hub_principal: Principal,
    pub chain_id: String,
    pub tokens: BTreeMap<TokenId, Token>,
    pub counterparties: BTreeMap<ChainId, Chain>,
    pub finalized_mint_token_requests: BTreeMap<TicketId, String>,
    pub chain_state: ChainState,
    pub next_ticket_seq: u64,
    pub next_directive_seq: u64,
    pub next_consume_ticket_seq: u64,
    pub next_consume_directive_seq: u64,
    pub pending_gen_ticket_requests: BTreeMap<Txid, LockTicketRequest>,
    pub finalized_gen_ticket_requests: BTreeMap<Txid, LockTicketRequest>,
}

impl From<&Brc20State> for StateProfile {
    fn from(value: &Brc20State) -> Self {
        StateProfile {
            admins: value.admins.clone(),
            min_confirmations: value.min_confirmations,
            btc_network: value.btc_network,
            ecdsa_key_name: value.ecdsa_key_name.clone(),
            ecdsa_public_key: value.ecdsa_public_key.clone(),
            deposit_addr: value.deposit_addr.clone(),
            deposit_pubkey: value.deposit_pubkey.clone(),
            indexer_principal: value.indexer_principal,
            hub_principal: value.hub_principal,
            chain_id: value.chain_id.clone(),
            tokens: value.tokens.clone(),
            counterparties: value.counterparties.clone(),
            finalized_mint_token_requests: Default::default(),
            chain_state: value.chain_state.clone(),
            next_ticket_seq: value.next_ticket_seq,
            next_directive_seq: value.next_directive_seq,
            next_consume_ticket_seq: value.next_consume_ticket_seq,
            next_consume_directive_seq: value.next_consume_directive_seq,
            pending_gen_ticket_requests: value.pending_lock_ticket_requests.clone(),
            finalized_gen_ticket_requests: value.finalized_lock_ticket_requests.clone(),
        }
    }
}

impl Brc20State {
    pub fn init(args: InitArgs) -> anyhow::Result<Self> {
        let btc_network = match args.network {
            omnity_types::Network::Local => ic_btc_interface::Network::Testnet,
            omnity_types::Network::Testnet => ic_btc_interface::Network::Testnet,
            omnity_types::Network::Mainnet => ic_btc_interface::Network::Mainnet,
        };
        let mut ret = Brc20State {
            admins: args.admins,
            hub_principal: args.hub_principal,
            chain_id: args.chain_id,
            reveal_utxo_index: Default::default(),
            tokens: Default::default(),
            counterparties: Default::default(),
            chain_state: ChainState::Active,
            ecdsa_key_name: args.network.key_id().name,
            flight_unlock_ticket_map: BTreeMap::default(),
            ecdsa_public_key: None,
            deposit_addr: None,
            deposit_pubkey: None,
            next_ticket_seq: 0,
            next_directive_seq: 0,
            next_consume_ticket_seq: 0,
            next_consume_directive_seq: 0,
            tickets_queue: StableBTreeMap::init(crate::stable_memory::get_unlock_tickets_memory()),
            directives_queue: StableBTreeMap::init(crate::stable_memory::get_directives_memory()),
            is_timer_running: Default::default(),
            pending_lock_ticket_requests: Default::default(),
            finalized_lock_ticket_requests: Default::default(),
            btc_network,
            indexer_principal: args.indexer_principal,
            deposit_addr_utxo: vec![],
            min_confirmations: 4,
            finalized_unlock_ticket_map: Default::default(),
            ticket_id_seq_indexer: Default::default(),
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
        let state: Brc20State =
            ciborium::de::from_reader(&*state_bytes).expect("failed to decode state");
        replace_state(state);
    }

    pub fn pull_tickets(&self, from: usize, limit: usize) -> Vec<(Seq, Ticket)> {
        self.tickets_queue
            .iter()
            .skip(from)
            .take(limit)
            .map(|(seq, t)| (seq, t.clone()))
            .collect()
    }

    pub fn pull_directives(&self, from: usize, limit: usize) -> Vec<(Seq, Directive)> {
        self.directives_queue
            .iter()
            .skip(from)
            .take(limit)
            .map(|(seq, d)| (seq, d.clone()))
            .collect()
    }

    pub fn generate_ticket_status(&self, tx_id: Txid) -> GenTicketStatus {
        if let Some(req) = self.pending_lock_ticket_requests.get(&tx_id) {
            return GenTicketStatus::Pending(req.clone());
        }

        match self
            .finalized_lock_ticket_requests
            .iter()
            .find(|req| req.1.txid == tx_id)
        {
            Some(req) => GenTicketStatus::Finalized(req.1.clone()),
            None => GenTicketStatus::Unknown,
        }
    }

    pub fn unlock_tx_status(&self, ticket_id: &TicketId) -> ReleaseTokenStatus {
        let seq = self.ticket_id_seq_indexer.get(ticket_id).cloned();

        if seq.is_none() {
            return ReleaseTokenStatus::Unknown;
        }
        let seq = seq.unwrap();
        if let Some(status) = self.flight_unlock_ticket_map.get(&seq).cloned() {
            if status.success {
                let txid = status.txs[2].txid();
                return ReleaseTokenStatus::Submitted(txid.to_string());
            } else {
                return ReleaseTokenStatus::Unknown;
            }
        }
        match self.finalized_unlock_ticket_map.get(&seq) {
            Some(tx) => {
                let txid = tx.txs[2].txid();
                return ReleaseTokenStatus::Confirmed(txid.to_string());
            }
            None => (),
        }
        ReleaseTokenStatus::Pending
    }
}

pub async fn init_ecdsa_public_key() -> ECDSAPublicKey {
    if let Some(pub_key) = read_state(|s| s.ecdsa_public_key.clone()) {
        return pub_key;
    };
    let key_name = read_state(|s| s.ecdsa_key_name.clone());
    let pub_key = crate::management::ecdsa_public_key(key_name, DerivationPath::new(vec![]))
        .await
        .unwrap_or_else(|e| ic_cdk::trap(&format!("failed to retrieve ECDSA public key: {e}")));
    mutate_state(|s| {
        s.ecdsa_public_key = Some(pub_key.clone());
        s.deposit_pubkey = Some(hex::encode(pub_key.public_key.clone()));
        let address = main_bitcoin_address(&pub_key.clone());
        let deposit_addr = address.display(s.btc_network);
        s.deposit_addr = Some(deposit_addr);
    });
    pub_key
}

pub fn deposit_addr() -> Address {
    let r = read_state(|s| s.deposit_addr.clone().unwrap());
    Address::from_str(&r).unwrap().assume_checked()
}

pub fn bitcoin_network() -> bitcoin::Network {
    let n = read_state(|s| s.btc_network);
    match n {
        Network::Mainnet => bitcoin::Network::Bitcoin,
        Network::Testnet => bitcoin::Network::Testnet,
        Network::Regtest => bitcoin::Network::Regtest,
    }
}

pub fn finalization_time_estimate(
    min_confirmations: u8,
    network: ic_btc_interface::Network,
) -> Duration {
    Duration::from_nanos(
        min_confirmations as u64
            * match network {
                ic_btc_interface::Network::Mainnet => 7 * MIN_NANOS,
                ic_btc_interface::Network::Testnet => MIN_NANOS,
                ic_btc_interface::Network::Regtest => SEC_NANOS,
            },
    )
}

pub fn deposit_pubkey() -> String {
    read_state(|s| s.deposit_pubkey.clone().unwrap())
}

pub fn mutate_state<F, R>(f: F) -> R
where
    F: FnOnce(&mut Brc20State) -> R,
{
    STATE.with(|s| f(s.borrow_mut().as_mut().expect("State not initialized!")))
}

pub fn read_state<F, R>(f: F) -> R
where
    F: FnOnce(&Brc20State) -> R,
{
    STATE.with(|s| f(s.borrow().as_ref().expect("State not initialized!")))
}

/// Replaces the current state.
pub fn replace_state(state: Brc20State) {
    STATE.with(|s| {
        *s.borrow_mut() = Some(state);
    });
}
