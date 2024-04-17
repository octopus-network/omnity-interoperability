use candid::Principal;
use ic_stable_structures::Log;

use omnity_types::Directive;
use omnity_types::Factor;
use omnity_types::SeqKey;
use omnity_types::Ticket;

use std::cell::RefCell;

use crate::state::HubState;

use crate::memory::{init_event_log, Memory};
use crate::types::ChainTokenFactor;
use crate::types::ChainWithSeq;
use crate::types::TokenKey;
use crate::types::TokenMeta;
use omnity_types::ToggleState;
use serde::{Deserialize, Serialize};
const MAX_EVENTS_PER_QUERY: usize = 2000;
type EventLog = Log<Vec<u8>, Memory, Memory>;

thread_local! {
    // The event storage
    static EVENTS: RefCell<EventLog> =  RefCell::new(init_event_log())
}

pub struct EventIterator {
    buf: Vec<u8>,
    pos: u64,
}

impl Iterator for EventIterator {
    type Item = Event;

    fn next(&mut self) -> Option<Event> {
        EVENTS.with(|events| {
            let events = events.borrow();

            match events.read_entry(self.pos, &mut self.buf) {
                Ok(()) => {
                    self.pos = self.pos.saturating_add(1);
                    Some(decode_event(&self.buf))
                }
                Err(_) => None,
            }
        })
    }

    fn nth(&mut self, n: usize) -> Option<Event> {
        self.pos = self.pos.saturating_add(n as u64);
        self.next()
    }
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
    Init(Principal),

    #[serde(rename = "pre_upgrade")]
    PreUpgrade(Vec<u8>),

    #[serde(rename = "post_upgrade")]
    PostUpgrade(Vec<u8>),

    #[serde(rename = "added_chain")]
    AddedChain(ChainWithSeq),

    #[serde(rename = "updated_chain")]
    UpdatedChainCounterparties(ChainWithSeq),

    #[serde(rename = "added_token")]
    AddedToken(TokenMeta),

    #[serde(rename = "toggled_chain_state")]
    ToggledChainState {
        chain: ChainWithSeq,
        state: ToggleState,
    },

    #[serde(rename = "updated_fee")]
    UpdatedFee(Factor),

    #[serde(rename = "received_directive")]
    ReceivedDirective {
        dst_chain: ChainWithSeq,
        dire: Directive,
    },

    #[serde(rename = "received_ticket")]
    ReceivedTicket {
        dst_chain: ChainWithSeq,
        ticket: Ticket,
    },

    #[serde(rename = "added_token_position")]
    AddedTokenPosition { position: TokenKey, amount: u128 },

    #[serde(rename = "updated_token_position")]
    UpdatedTokenPosition { position: TokenKey, amount: u128 },
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
        Some(Event::Init(principal)) => {
            let mut hub_state = HubState::default();
            hub_state.owner = Some(principal.to_string());
            hub_state
        }
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
            Event::Init(principal) => {
                hub_state.owner = Some(principal.to_string());
            }
            Event::PreUpgrade(_) => {}
            Event::PostUpgrade(state_bytes) => {
                let state: HubState =
                    ciborium::de::from_reader(&*state_bytes).expect("failed to decode state");
                hub_state = state;
            }
            Event::AddedChain(chain) => {
                hub_state
                    .chains
                    .insert(chain.chain_id.to_string(), chain.clone());
                // update auth
                hub_state
                    .authorized_caller
                    .insert(chain.canister_id.to_string(), chain.chain_id.to_string());
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
                    ()
                }
                Factor::UpdateFeeTokenFactor(tf) => {
                    hub_state
                        .target_chain_factors
                        .iter()
                        .for_each(|(chain_id, _)| {
                            let token_key =
                                TokenKey::from(chain_id.to_string(), tf.fee_token.to_string());
                            let fee_factor = ChainTokenFactor {
                                dst_chain_id: chain_id.to_string(),
                                fee_token: tf.fee_token.to_string(),
                                fee_token_factor: tf.fee_token_factor,
                            };
                            hub_state.fee_token_factors.insert(token_key, fee_factor);
                        });
                    ()
                }
            },

            Event::ReceivedDirective { dst_chain, dire } => {
                //update chain info
                hub_state
                    .chains
                    .insert(dst_chain.chain_id.to_string(), dst_chain.clone());
                hub_state.dire_queue.insert(
                    SeqKey::from(
                        dst_chain.chain_id.to_string(),
                        dst_chain.latest_dire_seq.unwrap(),
                    ),
                    dire,
                );
            }
            Event::AddedTokenPosition { position, amount }
            | Event::UpdatedTokenPosition { position, amount } => {
                hub_state.token_position.insert(position, amount);
            }

            Event::ReceivedTicket { dst_chain, ticket } => {
                //update chain info
                hub_state
                    .chains
                    .insert(ticket.dst_chain.to_string(), dst_chain.clone());
                // add new ticket to queue
                hub_state.ticket_queue.insert(
                    SeqKey::from(
                        ticket.dst_chain.to_string(),
                        dst_chain.latest_ticket_seq.unwrap(),
                    ),
                    ticket.clone(),
                );
                //save ticket to ledger
                hub_state
                    .cross_ledger
                    .insert(ticket.ticket_id.to_string(), ticket.clone());
            }
        }
    }
    Ok(hub_state)
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::types::ChainWithSeq;
    use crate::types::TokenMeta;
    use candid::Principal;
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
        let event = Event::Init(Principal::anonymous());
        let bytes = encode_event(&event);
        let decoded_event = decode_event(&bytes);
        assert_eq!(event, decoded_event);
    }

    #[test]
    fn test_replay() {
        let events = vec![
            Event::Init(Principal::anonymous()),
            Event::AddedChain(ChainWithSeq {
                chain_id: "Bitcoin".to_string(),
                chain_type: ChainType::SettlementChain,
                chain_state: ChainState::Active,
                canister_id: "bkyz2-fmaaa-aaaaa-qaaaq-cai".to_string(),
                contract_address: None,
                counterparties: None,
                fee_token: None,
                latest_dire_seq: Some(0),
                latest_ticket_seq: Some(0),
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
                chain: ChainWithSeq {
                    chain_id: "Bitcoin".to_string(),
                    chain_type: ChainType::SettlementChain,
                    chain_state: ChainState::Deactive,
                    canister_id: "bkyz2-fmaaa-aaaaa-qaaaq-cai".to_string(),
                    contract_address: None,
                    counterparties: None,
                    fee_token: None,
                    latest_dire_seq: Some(0),
                    latest_ticket_seq: Some(0),
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
            Event::ReceivedDirective {
                dst_chain: ChainWithSeq {
                    chain_id: "Bitcoin".to_string(),
                    chain_type: ChainType::SettlementChain,
                    chain_state: ChainState::Active,
                    canister_id: "bkyz2-fmaaa-aaaaa-qaaaq-cai".to_string(),
                    contract_address: None,
                    counterparties: None,
                    fee_token: None,
                    latest_dire_seq: Some(0),
                    latest_ticket_seq: Some(0),
                },
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
                dst_chain: ChainWithSeq {
                    chain_id: "Bitcoin".to_string(),
                    chain_type: ChainType::SettlementChain,
                    chain_state: ChainState::Active,
                    canister_id: "bkyz2-fmaaa-aaaaa-qaaaq-cai".to_string(),
                    contract_address: None,
                    counterparties: None,
                    fee_token: None,
                    latest_dire_seq: Some(0),
                    latest_ticket_seq: Some(0),
                },
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

        println!("{:?}", hub_state.owner);
    }
}
