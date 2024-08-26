use candid::Principal;
use omnity_types::{Chain, ChainId, TicketId, Token, TokenId};
use serde::Serialize;
use std::{borrow::Cow, cell::RefCell};

use crate::lifecycle::init::InitArgs;
use ic_stable_structures::{
    memory_manager::{MemoryId, MemoryManager, VirtualMemory}, storable::Bound, DefaultMemoryImpl, Storable
};
use ic_stable_structures::{Cell, StableBTreeMap};

type InnerMemory = DefaultMemoryImpl;
pub type Memory = VirtualMemory<InnerMemory>;

const TOKEN_MEMORY_ID: MemoryId = MemoryId::new(1);
const TOKEN_PRINCIPAL_MEMORY_ID: MemoryId = MemoryId::new(2);
const COUNTERPARTIES_MEMORY_ID: MemoryId = MemoryId::new(3);
const FINALIZED_MINT_TOKEN_REQUESTS_MEMORY_ID: MemoryId = MemoryId::new(4);
const STATE_MEMORY_ID: MemoryId = MemoryId::new(5);

thread_local! {
    static __STATE: RefCell<Option<CustomsState>> = RefCell::default();

    static MEMORY_MANAGER: RefCell<MemoryManager<InnerMemory>> =
    RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    static TOKENS: RefCell<StableBTreeMap<TokenId, Token, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(TOKEN_MEMORY_ID)),
        )
    );

    static TOKEN_PRINCIPALS: RefCell<StableBTreeMap<TokenId, Principal, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(TOKEN_PRINCIPAL_MEMORY_ID)),
        )
    );

    static COUNTERPARTIES: RefCell<StableBTreeMap<ChainId, Chain, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(COUNTERPARTIES_MEMORY_ID)),
        )
    );

    static FINALIZED_MINT_TOKEN_REQUESTS: RefCell<StableBTreeMap<TicketId, u64, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(FINALIZED_MINT_TOKEN_REQUESTS_MEMORY_ID))
        )
    );

    static STATE: RefCell<Cell<Option<CustomsState>, Memory>> = RefCell::new(
        Cell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(STATE_MEMORY_ID)),
            None
        ).expect("Failed to init route state")
    );

}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, Serialize)]
pub struct CustomsState {
    pub chain_id: String,

    pub hub_principal: Principal,

    // Next index of query tickets from hub
    pub next_ticket_seq: u64,

    // Next index of query directives from hub
    pub next_directive_seq: u64,

    // pub tokens: BTreeMap<TokenId, (Token, Principal)>,

    // pub counterparties: BTreeMap<ChainId, Chain>,

    // pub finalized_mint_token_requests: BTreeMap<TicketId, u64>,

    pub ckbtc_ledger_principal: Principal,

    pub icp_token_id: Option<TokenId>,

    pub ckbtc_token_id: Option<TokenId>,

    #[serde(skip)]
    pub is_timer_running: bool,
}

impl Storable for CustomsState {
    const BOUND: Bound = Bound::Unbounded;

    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        let mut bytes = vec![];
        let _ = ciborium::ser::into_writer(self, &mut bytes);
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        let dire = ciborium::de::from_reader(bytes.as_ref()).expect("failed to decode Chain");
        dire
    }
}

impl From<InitArgs> for CustomsState {
    fn from(args: InitArgs) -> Self {
        Self {
            chain_id: args.chain_id,
            hub_principal: args.hub_principal,
            next_ticket_seq: 0,
            next_directive_seq: 0,
            // tokens: Default::default(),
            // counterparties: Default::default(),
            // finalized_mint_token_requests: Default::default(),
            is_timer_running: false,
            ckbtc_ledger_principal: args.ckbtc_ledger_principal,
            icp_token_id: None,
            ckbtc_token_id: None,
        }
    }
}

pub fn insert_finalized_mint_token_request(ticket_id: TicketId, block_index: u64) {
    FINALIZED_MINT_TOKEN_REQUESTS.with(|f| {
        f.borrow_mut().insert(ticket_id, block_index);
    });
}

pub fn get_finalized_mint_token_request(ticket_id: &TicketId) -> Option<u64> {
    FINALIZED_MINT_TOKEN_REQUESTS.with(|f| f.borrow().get(ticket_id).to_owned())
}

pub fn insert_counterparty(chain: Chain) {
    COUNTERPARTIES.with(|c| {
        c.borrow_mut().insert(chain.chain_id.clone(), chain);
    });
}

pub fn get_counterparty(chain_id: &ChainId) -> Option<Chain> {
    COUNTERPARTIES.with(|c| c.borrow().get(chain_id).to_owned())
}

pub fn get_chain_list() -> Vec<Chain> {
    COUNTERPARTIES.with(|c| {
        c.borrow()
            .iter()
            .map(|(_, chain)| chain.clone())
            .collect()
    })
}

pub fn get_token_list() -> Vec<Token> {
    TOKENS.with(|t| {
        t.borrow()
            .iter()
            .map(|(_, token)| token.clone())
            .collect()
    })
}

pub fn insert_token(token: Token, principal: Principal) {
    TOKEN_PRINCIPALS.with(|t| {
        t.borrow_mut().insert(token.token_id.clone(), principal);
    });

    TOKENS.with(|t| {
        t.borrow_mut().insert(token.token_id.clone(), token.clone());
    });
}

pub fn get_token(token_id: &TokenId) -> Option<Token> {
    TOKENS.with(|t| 
        t.borrow().get(token_id).to_owned())
}

pub fn get_token_principal(token_id: &TokenId) -> Option<Principal> {
    TOKEN_PRINCIPALS.with(|t| 
        t.borrow().get(token_id).to_owned())
}

pub fn mutate_state<F, R>(f: F) -> R
where
    F: FnOnce(&mut CustomsState) -> R,
{
    let mut route_state = STATE.with(|r| {
        r.borrow_mut()
            .get()
            .clone()
            .expect("Failed to mutate route state")
    });
    let r = f(&mut route_state);
    STATE
        .with(|r| r.borrow_mut().set(Some(route_state)))
        .expect("Failed to set route state");

    r
}

/// Read (part of) the current state using `f`.
///
/// Panics if there is no state.
pub fn read_state<F, R>(f: F) -> R
where
    F: FnOnce(&CustomsState) -> R,
{
    STATE.with(|r| {
        f(r.borrow()
            .get()
            .as_ref()
            .expect("Failed to read route state"))
    })
}

pub fn set_state(state: CustomsState) {
    STATE.with(|r| {
        r.borrow_mut().set(Some(state)).expect("Failed to set route state");
    });
}

pub fn is_ckbtc(token_id: &TokenId)->bool {
    read_state(|state| {
        state.ckbtc_token_id.as_ref().map_or(false, |id| id == token_id)
    })
}

pub fn is_icp(token_id: &TokenId)->bool{
    read_state(|state| {
        state.icp_token_id.as_ref().map_or(false, |id| id == token_id)
    })
}

// pub fn replace_state(state: CustomsState) {
//     __STATE.with(|s| {
//         *s.borrow_mut() = Some(state);
//     });
// }