use crate::auth::Permission;
use crate::lifecycle::init::InitArgs;
use crate::lifecycle::upgrade::UpgradeArgs;
use crate::state::HubState;
use ic_stable_structures::Log;
use omnity_types::Directive;
use omnity_types::Factor;
use omnity_types::SeqKey;
use omnity_types::Ticket;
use omnity_types::Topic;

use std::cell::RefCell;

use crate::memory::{init_event_log, Memory};

use omnity_types::hub_types::{ChainMeta, ChainTokenFactor, Subscribers, TokenKey, TokenMeta};

use omnity_types::ToggleState;
use serde::{Deserialize, Serialize};
const MAX_EVENTS_PER_QUERY: usize = 2000;
type EventLog = Log<Vec<u8>, Memory, Memory>;

thread_local! {
    // The event storage
    static EVENTS: RefCell<EventLog> =  RefCell::new(init_event_log())
}

/// Encodes an event into a byte array.
fn encode_event(event: &Event) -> Vec<u8> {
    let mut buf = Vec::new();
    ciborium::ser::into_writer(event, &mut buf).expect("failed to encode a customs event");
    buf
}

/// # Panics
///
/// This function panics if the event decoding fails.
fn decode_event(buf: &[u8]) -> Event {
    ciborium::de::from_reader(buf).expect("failed to decode a customs event")
}

/// Returns an iterator over all customs events.
pub fn events(args: GetEventsArg) -> Vec<Event> {
    EVENTS.with(|events| {
        events
            .borrow()
            .iter()
            .skip(args.start as usize)
            .take(MAX_EVENTS_PER_QUERY.min(args.length as usize))
            .map(|bytes| decode_event(&bytes))
            .collect()
    })
}

/// Returns the current number of events in the log.
pub fn count_events() -> u64 {
    EVENTS.with(|events| events.borrow().len())
}

/// Records a new customs event.
pub fn record_event(event: &Event) {
    let bytes = encode_event(event);
    EVENTS.with(|events| {
        events
            .borrow()
            .append(&bytes)
            .expect("failed to append an entry to the event log")
    });
}

#[derive(candid::CandidType, Deserialize)]
pub struct GetEventsArg {
    pub start: u64,
    pub length: u64,
}

#[derive(candid::CandidType, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Event {
    #[serde(rename = "init")]
    Init(InitArgs),

    #[serde(rename = "upgrade")]
    Upgrade(UpgradeArgs),

    #[serde(rename = "added_chain")]
    UpdatedChain(ChainMeta),

    #[serde(rename = "updated_chain")]
    UpdatedChainCounterparties(ChainMeta),

    #[serde(rename = "Subscribed_topic")]
    SubDirectives { topic: Topic, subs: Subscribers },

    #[serde(rename = "Unsubscribed_topic")]
    UnSubDirectives { topic: Topic, sub: String },

    #[serde(rename = "added_token")]
    AddedToken(TokenMeta),

    #[serde(rename = "toggled_chain_state")]
    ToggledChainState {
        chain: ChainMeta,
        state: ToggleState,
    },

    #[serde(rename = "updated_fee")]
    UpdatedFee(Factor),

    #[serde(rename = "saved_directive")]
    SavedDirective(Directive),
    #[serde(rename = "deleted_directive")]
    DeletedDirective(SeqKey),

    #[serde(rename = "published_directive")]
    PubedDirective { seq_key: SeqKey, dire: Directive },

    #[serde(rename = "received_ticket")]
    ReceivedTicket { seq_key: SeqKey, ticket: Ticket },

    #[serde(rename = "pending_ticket")]
    PendingTicket { ticket: Ticket },

    #[serde(rename = "finaize_ticket")]
    FinalizeTicket { ticket_id: String },

    #[serde(rename = "added_token_position")]
    AddedTokenPosition { position: TokenKey, amount: u128 },

    #[serde(rename = "updated_token_position")]
    UpdatedTokenPosition { position: TokenKey, amount: u128 },

    #[serde(rename = "resubmit_ticket")]
    ResubmitTicket { ticket_id: String, timestamp: u64 },

    #[serde(rename = "updated_tx_hash")]
    UpdatedTxHash { ticket_id: String, tx_hash: String },
}

#[derive(Debug)]
pub enum ReplayLogError {
    /// There are no events in the event log.
    EmptyLog,
    /// The event log is inconsistent.
    InconsistentLog(String),
}

/// Reconstructs the hub state from an event log.
pub fn replay(mut events: impl Iterator<Item = Event>) -> Result<HubState, ReplayLogError> {
    let mut hub_state = match events.next() {
        Some(Event::Init(args)) => HubState::from(args),
        Some(evt) => {
            return Err(ReplayLogError::InconsistentLog(format!(
                "The first event is not Init: {:?}",
                evt
            )))
        }
        None => return Err(ReplayLogError::EmptyLog),
    };

    for event in events {
        match event {
            Event::Init(args) => {
         /*       hub_state
                    .caller_perms
                    .insert(args.admin.to_string(), Permission::Update);
                hub_state.admin = args.admin;*/
            }
            Event::Upgrade(args) => {
             //   hub_state.upgrade(args);
            }
            Event::UpdatedChain(chain) => {
                hub_state
                    .chains
                    .insert(chain.chain_id.to_string(), chain.clone());
                // update auth
                let _ = hub_state.update_auth(&chain);
            }
            Event::UpdatedChainCounterparties(chain) => {
                hub_state
                    .chains
                    .insert(chain.chain_id.to_string(), chain.clone());
            }
            Event::AddedToken(token) => {
                hub_state.tokens.insert(token.token_id.to_string(), token);
            }
            Event::ToggledChainState { chain, state } => {
                hub_state.chains.insert(state.chain_id.to_string(), chain);
            }
            Event::UpdatedFee(fee) => match fee {
                Factor::UpdateTargetChainFactor(cf) => {
                    hub_state
                        .target_chain_factors
                        .insert(cf.target_chain_id, cf.target_chain_factor);
                }
                Factor::UpdateFeeTokenFactor(tf) => {
                    hub_state
                        .target_chain_factors
                        .iter()
                        .for_each(|(chain_id, _)| {
                            let token_key =
                                TokenKey::from(chain_id.to_string(), tf.fee_token.to_string());
                            let fee_factor = ChainTokenFactor {
                                target_chain_id: chain_id.to_string(),
                                fee_token: tf.fee_token.to_string(),
                                fee_token_factor: tf.fee_token_factor,
                            };
                            hub_state.fee_token_factors.insert(token_key, fee_factor);
                        });
                }
            },

            Event::AddedTokenPosition { position, amount }
            | Event::UpdatedTokenPosition { position, amount } => {
                hub_state.token_position.insert(position, amount);
            }
            Event::PendingTicket { ticket } => {
                hub_state
                    .pending_tickets
                    .insert(ticket.ticket_id.to_string(), ticket.clone());
            }
            Event::FinalizeTicket { ticket_id } => {
                hub_state.pending_tickets.remove(&ticket_id);
            }
            Event::ReceivedTicket { seq_key, ticket } => {
                hub_state
                    .ticket_seq
                    .insert(seq_key.chain_id.to_string(), seq_key.seq);
                // add new ticket to queue
                hub_state
                    .ticket_map
                    .insert(seq_key, ticket.ticket_id.to_string());
                //save ticket to ledger
                hub_state
                    .cross_ledger
                    .insert(ticket.ticket_id.to_string(), ticket.clone());
            }
            Event::SubDirectives { topic, subs } => {
                hub_state.topic_subscribers.insert(topic, subs);
            }
            Event::UnSubDirectives { topic, sub } => {
                if let Some(mut subscribers) = hub_state.topic_subscribers.get(&topic) {
                    if subscribers.subs.remove(&sub) {
                        hub_state
                            .topic_subscribers
                            .insert(topic.clone(), subscribers);
                    }
                }
            }
            Event::SavedDirective(dire) => {
                hub_state.directives.insert(dire.hash(), dire);
            }
            Event::DeletedDirective(seq_key) => {
                hub_state.dire_map.remove(&seq_key);
            }
            Event::PubedDirective { seq_key, dire } => {
                hub_state
                    .directive_seq
                    .insert(seq_key.chain_id.to_string(), seq_key.seq);
                hub_state.dire_map.insert(seq_key, dire);
            }
            Event::ResubmitTicket {
                ticket_id: _,
                timestamp,
            } => hub_state.last_resubmit_ticket_time = timestamp,

            Event::UpdatedTxHash { ticket_id, tx_hash } => {
                hub_state.tx_hashes.insert(ticket_id, tx_hash);
            }
        }
    }
    Ok(hub_state)
}

#[cfg(test)]
mod tests {
    use super::*;

    use candid::Principal;
    use omnity_types::hub_types::{ChainMeta, TokenMeta};
    use omnity_types::Chain;
    use omnity_types::ChainState;
    use omnity_types::Directive;
    use omnity_types::Factor;
    use omnity_types::Ticket;
    use omnity_types::ToggleState;
    use omnity_types::{ChainType, TargetChainFactor, TicketType, ToggleAction, TxAction};
    use std::collections::HashMap;
    use uuid::Uuid;

    #[test]
    fn test_encode_decode_event() {
        let event = Event::Init(InitArgs {
            admin: Principal::anonymous(),
        });
        let bytes = encode_event(&event);
        let decoded_event = decode_event(&bytes);
        assert_eq!(event, decoded_event);
    }

    #[test]
    fn test_replay() {
        let events = vec![
            Event::Init(InitArgs {
                admin: Principal::anonymous(),
            }),
            Event::UpdatedChain(ChainMeta {
                chain_id: "Bitcoin".to_string(),
                chain_type: ChainType::SettlementChain,
                chain_state: ChainState::Active,
                canister_id: "bkyz2-fmaaa-aaaaa-qaaaq-cai".to_string(),
                contract_address: None,
                counterparties: None,
                fee_token: None,
            }),
            Event::AddedToken(TokenMeta {
                token_id: "Bitcoin-RUNES-WTF".to_string(),
                name: "BTC".to_owned(),
                symbol: "BTC".to_owned(),
                issue_chain: "Bitcoin".to_string(),
                decimals: 18,
                icon: None,
                metadata: HashMap::new(),
                dst_chains: vec![],
            }),
            Event::ToggledChainState {
                chain: ChainMeta {
                    chain_id: "Bitcoin".to_string(),
                    chain_type: ChainType::SettlementChain,
                    chain_state: ChainState::Deactive,
                    canister_id: "bkyz2-fmaaa-aaaaa-qaaaq-cai".to_string(),
                    contract_address: None,
                    counterparties: None,
                    fee_token: None,
                },
                state: ToggleState {
                    chain_id: "Bitcoin".to_string(),
                    action: ToggleAction::Deactivate,
                },
            },
            Event::UpdatedFee(Factor::UpdateTargetChainFactor(TargetChainFactor {
                target_chain_id: "Bitcoin".to_string(),
                target_chain_factor: 1000,
            })),
            Event::PubedDirective {
                seq_key: SeqKey::from("Bitcoin".to_string(), 0),
                dire: Directive::AddChain(Chain {
                    chain_id: "Bitcoin".to_string(),
                    chain_type: ChainType::SettlementChain,
                    chain_state: ChainState::Active,
                    canister_id: "bkyz2-fmaaa-aaaaa-qaaaq-cai".to_string(),
                    contract_address: None,
                    counterparties: None,
                    fee_token: None,
                }),
            },
            Event::ReceivedTicket {
                seq_key: SeqKey::from("Bitcoin".to_string(), 0),
                ticket: Ticket {
                    ticket_id: Uuid::new_v4().to_string(),
                    ticket_type: TicketType::Normal,
                    ticket_time: 0,
                    src_chain: "Bitcoin".to_string(),
                    dst_chain: "EVM-Arbitrum".to_string(),
                    action: TxAction::Transfer,
                    token: "Bitcoin-RUNES-WTF".to_string(),
                    amount: "1000".to_string(),
                    sender: None,
                    receiver: Principal::anonymous().to_string(),
                    memo: None,
                },
            },
        ];

        let hub_state = replay(events.into_iter()).unwrap();

        println!("{:?}", hub_state.admin);
    }
}
