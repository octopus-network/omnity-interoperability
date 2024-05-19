use candid::CandidType;
use ic_stable_structures::storable::Bound;
use ic_stable_structures::Storable;
use log::info;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::HashMap;

use crate::memory::{self, Memory};

use crate::metrics::with_metrics_mut;
use crate::state::{set_state, HubState};
use crate::types::{Amount, ChainMeta, ChainTokenFactor, Subscribers, TokenKey, TokenMeta};
use candid::Principal;

use ic_stable_structures::StableBTreeMap;

use omnity_types::{
    ChainId, Directive, Seq, SeqKey, Ticket, TicketId, TicketStatus, TicketType, TokenId, Topic,
    TxAction,
};

use omnity_types::{Account, Timestamp};

#[derive(
    CandidType, Deserialize, Serialize, Default, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
pub struct PreTicket {
    pub ticket_id: TicketId,
    pub ticket_type: TicketType,
    pub ticket_time: Timestamp,
    pub src_chain: ChainId,
    pub dst_chain: ChainId,
    pub action: TxAction,
    pub token: TokenId,
    pub amount: String,
    pub sender: Option<Account>,
    pub receiver: Account,
    pub memo: Option<Vec<u8>>,
}

impl Storable for PreTicket {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        let mut bytes = vec![];
        let _ = ciborium::ser::into_writer(self, &mut bytes);
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        let ticket = ciborium::de::from_reader(bytes.as_ref()).expect("failed to decode TokenKey");
        ticket
    }

    const BOUND: Bound = Bound::Unbounded;
}

#[derive(Deserialize, Serialize)]
pub struct PreHubState {
    #[serde(skip, default = "memory::init_chain")]
    pub chains: StableBTreeMap<ChainId, ChainMeta, Memory>,
    #[serde(skip, default = "memory::init_token")]
    pub tokens: StableBTreeMap<TokenId, TokenMeta, Memory>,
    #[serde(skip, default = "memory::init_chain_factor")]
    pub target_chain_factors: StableBTreeMap<ChainId, u128, Memory>,
    #[serde(skip, default = "memory::init_token_factor")]
    pub fee_token_factors: StableBTreeMap<TokenKey, ChainTokenFactor, Memory>,
    #[serde(skip, default = "memory::init_directive")]
    pub directives: StableBTreeMap<String, Directive, Memory>,
    #[serde(skip, default = "memory::init_dire_queue")]
    pub dire_queue: StableBTreeMap<SeqKey, Directive, Memory>,
    #[serde(skip, default = "memory::init_subs")]
    pub topic_subscribers: StableBTreeMap<Topic, Subscribers, Memory>,
    #[serde(skip, default = "memory::init_ticket_queue")]
    pub ticket_queue: StableBTreeMap<SeqKey, Ticket, Memory>,
    #[serde(skip, default = "memory::init_token_position")]
    pub token_position: StableBTreeMap<TokenKey, Amount, Memory>,
    #[serde(skip, default = "memory::init_ledger")]
    pub cross_ledger: StableBTreeMap<TicketId, PreTicket, Memory>,
    pub directive_seq: HashMap<String, Seq>,
    pub ticket_seq: HashMap<String, Seq>,
    pub admin: Principal,
    pub authorized_caller: HashMap<String, ChainId>,
    pub last_resubmit_ticket_time: u64,
}

pub fn migrate(pre_state: PreHubState) {
    info!(" Begine to mirate ...");
    let mut new_ledger: StableBTreeMap<TicketId, Ticket, Memory> =
        StableBTreeMap::init(memory::get_ledger_v2_memory());

    for (k, v) in pre_state.cross_ledger.iter() {
        info!("the ticket before migration: {:?} ", v);
        let new_ticket = Ticket {
            ticket_id: v.ticket_id,
            ticket_type: v.ticket_type,
            ticket_time: v.ticket_time,
            src_chain: v.src_chain,
            dst_chain: v.dst_chain,
            action: v.action,
            token: v.token,
            amount: v.amount,
            sender: v.sender,
            receiver: v.receiver,
            memo: v.memo,
            status: TicketStatus::WaitingForConfirmByDest,
        };
        info!("the ticket after migration: {:?} ", new_ticket);
        new_ledger.insert(k, new_ticket.clone());
        //update ticket meric
        with_metrics_mut(|metrics| metrics.update_ticket_metric(new_ticket));
    }
    let new_stat = HubState {
        chains: pre_state.chains,
        tokens: pre_state.tokens,
        target_chain_factors: pre_state.target_chain_factors,
        fee_token_factors: pre_state.fee_token_factors,
        directives: pre_state.directives,
        dire_queue: pre_state.dire_queue,
        topic_subscribers: pre_state.topic_subscribers,
        ticket_queue: pre_state.ticket_queue,
        token_position: pre_state.token_position,
        cross_ledger: new_ledger,
        directive_seq: pre_state.directive_seq,
        ticket_seq: pre_state.ticket_seq,
        admin: pre_state.admin,
        authorized_caller: pre_state.authorized_caller,
        last_resubmit_ticket_time: pre_state.last_resubmit_ticket_time,
    };
    set_state(new_stat);

    info!(" Migration Done!");

    //TODO: remove pre ledger
}
