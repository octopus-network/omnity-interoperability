use crate::*;

thread_local! {
    // TODO implement directives
    // TODO What is suitable size for TICKETS_QUERY_LIMIT?
    static TICKET_QUERY_LIMIT: RefCell<u32> = RefCell::new(10);
    static TICKET_SEQUENCE: RefCell<u64> = RefCell::new(0);
    static BROADCASTED_TXS: RefCell<Rc<HashMap<String, Ticket>>> = RefCell::new(Rc::new(HashMap::new()));
    static NONCE: RefCell<u64> = RefCell::new(0);
    static ACTIVE: RefCell<bool> = RefCell::new(false);
    // init on startup
    static TARGET_CHAIN_ID: RefCell<ChainId> = RefCell::new(ChainId::default());
    static HUB_ADDR: RefCell<Option<Principal>> = RefCell::new(None);
    static PORT_ADDR: RefCell<Option<Principal>> = RefCell::new(None);
}

pub(crate) fn ticket_query_limit() -> u32 {
    TICKET_QUERY_LIMIT.with(|limit| limit.borrow().clone())
}

pub(crate) fn ticket_sequence() -> u64 {
    TICKET_SEQUENCE.with(|seq| seq.borrow().clone())
}

pub(crate) fn hub_addr_or_error() -> Result<Principal> {
    HUB_ADDR.with(|port_addr| port_addr.borrow().clone())
        .ok_or(Error::RouteNotInitialized("PORT_ADDR".to_string()))
}

pub(crate) fn port_addr_or_error() -> Result<Principal> {
    PORT_ADDR.with(|port_addr| port_addr.borrow().clone())
        .ok_or(Error::RouteNotInitialized("PORT_ADDR".to_string()))
}

pub(crate) fn target_chain_id() -> ChainId {
    TARGET_CHAIN_ID.with(|id| id.borrow().clone())
}

pub(crate) fn set_ticket_sequence(seq: u64) {
    TICKET_SEQUENCE.with(|s| *s.borrow_mut() = seq);
}