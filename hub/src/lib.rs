mod auth;
mod errors;
mod memory;
mod signer;
mod utils;

use candid::types::principal::Principal;
use candid::CandidType;

use auth::auth;
use ic_cdk::{init, post_upgrade, pre_upgrade, update};
use ic_stable_structures::writer::Writer;
use ic_stable_structures::Memory;
use log::debug;
use omnity_types::{
    Action, ChainId, ChainInfo, ChainType, DireQueue, Directive, Error, Fee, Proposal, Seq, Status,
    Ticket, TicketId, TicketQueue, TokenId, TokenMetaData,
};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::ops::Bound::Included;
// use utils::init_log;
use crate::signer::PublicKeyReply;
use crate::utils::Network;

thread_local! {
    static STATE: RefCell<HubState> = RefCell::new(HubState::default());
}

#[derive(CandidType, Deserialize, Serialize, Default, Clone, Debug)]
pub struct ChainInfoWithSeq {
    pub chain_name: ChainId,
    pub chain_type: ChainType,
    pub chain_status: Status,
    pub latest_dire_seq: Seq,
    pub latest_ticket_seq: Seq,
    // Optional: settlement chain export contract address
    // pub export_address: Option<String>,
    // Optional: execution chain port contract address
    // pub port_address: Option<String>,
}

#[derive(CandidType, Deserialize, Serialize, Default, Debug)]
struct CrossLedger {
    pub transfers: HashMap<TicketId, Ticket>,
    pub redeems: HashMap<TicketId, Ticket>,
}

#[derive(CandidType, Deserialize, Serialize, Default, Debug)]
struct HubState {
    pub chains: HashMap<ChainId, ChainInfoWithSeq>,
    pub tokens: HashMap<(ChainId, TokenId), TokenMetaData>,
    pub fees: HashMap<(ChainId, TokenId), Fee>,
    pub cross_ledger: CrossLedger,
    pub dire_queue: DireQueue,
    pub ticket_queue: TicketQueue,
    pub owner: Option<Principal>,
    pub whitelist: HashSet<Principal>,
}

/// A helper method to read the state.
///
/// Precondition: the state is already initialized.
fn with_state<R>(f: impl FnOnce(&HubState) -> R) -> R {
    STATE.with(|cell| f(&cell.borrow()))
}
/// A helper method to mutate the state.
///
/// Precondition: the state is already initialized.
fn with_state_mut<R>(f: impl FnOnce(&mut HubState) -> R) -> R {
    STATE.with(|cell| f(&mut cell.borrow_mut()))
}

// A helper method to set the state.
//
// Precondition: the state is _not_ initialized.
fn set_state(state: HubState) {
    STATE.with(|cell| *cell.borrow_mut() = state);
}
#[init]
fn init() {
    // init_log()
}

#[pre_upgrade]
fn pre_upgrade() {
    debug!("begin to handle pre_update state ...");

    // Serialize the state.
    let mut state_bytes = vec![];
    with_state(|state| ciborium::ser::into_writer(state, &mut state_bytes))
        .expect("failed to encode state");

    // Write the length of the serialized bytes to memory, followed by the
    // by the bytes themselves.
    let len = state_bytes.len() as u32;
    let mut memory = memory::get_upgrades_memory();
    let mut writer = Writer::new(&mut memory, 0);
    writer
        .write(&len.to_le_bytes())
        .expect("failed to save config len");
    writer.write(&state_bytes).expect("failed to save config");
}

#[post_upgrade]
fn post_upgrade() {
    let memory = memory::get_upgrades_memory();

    // Read the length of the state bytes.
    let mut state_len_bytes = [0; 4];
    memory.read(0, &mut state_len_bytes);
    let state_len = u32::from_le_bytes(state_len_bytes) as usize;

    // Read the bytes
    let mut state_bytes = vec![0; state_len];
    memory.read(4, &mut state_bytes);

    // Deserialize and set the state.
    let state: HubState = ciborium::de::from_reader(&*state_bytes).expect("failed to decode state");
    set_state(state);
}

/// validate directive ,this method will be called by sns
#[update(guard = "auth")]
pub fn validate_proposal(proposal: Proposal) -> Result<String, Error> {
    match proposal {
        Proposal::AddChain(chain) => {
            if chain.chain_name.is_empty() {
                return Err(Error::ProposalError(
                    "Chain name can not be empty".to_string(),
                ));
            }
            match chain.chain_state {
                Status::Reinstate | Status::Suspend => {
                    return Err(Error::ProposalError(
                        "The status of the new chain state must be active".to_string(),
                    ))
                }
                _ => ..,
            };

            Ok(format!("Tne AddChain proposal is: {}", chain))
        }
        Proposal::AddToken(token) => {
            if token.name.is_empty() || token.symbol.is_empty() || token.issue_chain.is_empty() {
                return Err(Error::ProposalError(
                    "Token id, token symbol or issue chain can not be empty".to_string(),
                ));
            }
            Ok(format!("The AddToken proposal is: {}", token))
        }
        Proposal::ChangeChainStatus(statue) => {
            if statue.chain.is_empty() {
                return Err(Error::ProposalError(
                    "Chain id can not be empty".to_string(),
                ));
            }
            match statue.state {
                Status::Active | Status::Reinstate | Status::Suspend => ..,
                _ => {
                    return Err(Error::ProposalError(
                        "The chain state not match".to_string(),
                    ))
                }
            };

            Ok(format!("the ChangeChainStatus proposal is: {}", statue))
        }
        Proposal::UpdateFee(fee) => {
            if fee.fee_token.is_empty() {
                return Err(Error::ProposalError(
                    "The Quote token can not be empty".to_string(),
                ));
            };
            Ok(format!("The UpdateFee proposal is: {}", fee))
        }

        _ => Err(Error::NotSupportedProposal),
    }
}

/// build directive based on proposal, this method will be called by sns
/// add chain / add token /change chain status / update fee
#[update(guard = "auth")]
pub async fn build_directive(proposal: Proposal) -> Result<(), Error> {
    match proposal {
        Proposal::AddChain(chain) => {
            with_state_mut(|hub_state| {
                let new_chain = ChainInfoWithSeq {
                    chain_name: chain.chain_name.clone(),
                    chain_type: chain.chain_type.clone(),
                    chain_status: chain.chain_state.clone(),
                    latest_dire_seq: 0,
                    latest_ticket_seq: 0,
                };
                // save new chain
                hub_state
                    .chains
                    .insert(chain.chain_name.clone(), new_chain.clone());

                // build directives
                match chain.chain_type {
                    ChainType::SettlementChain => (),

                    ChainType::ExecutionChain => {
                        // add directive for existing chain
                        let _ = hub_state
                            .chains
                            .iter_mut()
                            .filter(|(&ref chain_id, _)| *chain_id != new_chain.chain_name.clone())
                            .map(|(chain_id, existing_chain)| {
                                // increases the new chain seq
                                hub_state
                                    .dire_queue
                                    .entry(chain_id.to_string())
                                    .and_modify(|dires| {
                                        existing_chain.latest_dire_seq += 1;
                                        dires.insert(
                                            existing_chain.latest_dire_seq,
                                            Directive::AddChain(chain.clone()),
                                        );
                                    })
                                    .or_insert_with(|| {
                                        let mut dires = BTreeMap::new();
                                        dires.insert(0u64, Directive::AddChain(chain.clone()));
                                        dires
                                    });
                            });

                        // add directive for new chain
                        let mut new_chain_seq: u64 = new_chain.latest_dire_seq;

                        let _ = hub_state
                            .chains
                            .iter_mut()
                            .filter(|(&ref id, _)| *id != chain.chain_name.clone())
                            .map(|(_existing_chain_id, existing_chain)| {
                                let dest_chain = ChainInfo {
                                    chain_name: existing_chain.chain_name.clone(),
                                    chain_type: existing_chain.chain_type.clone(),
                                    chain_state: existing_chain.chain_status.clone(),
                                };

                                hub_state
                                    .dire_queue
                                    .entry(new_chain.chain_name.clone())
                                    .and_modify(|dires| {
                                        // increases the new chain seq
                                        new_chain_seq += 1;
                                        dires.insert(
                                            new_chain_seq,
                                            Directive::AddChain(dest_chain.clone()),
                                        );
                                    })
                                    .or_insert_with(|| {
                                        let mut dires = BTreeMap::new();
                                        dires.insert(0u64, Directive::AddChain(dest_chain.clone()));
                                        dires
                                    });
                            });
                        // update the new chain latest seq
                        hub_state
                            .chains
                            .get_mut(&new_chain.chain_name)
                            .and_then(|new_chain| {
                                new_chain.latest_dire_seq = new_chain_seq;
                                Some(new_chain.latest_dire_seq)
                            });
                    }
                }
            });
            //TODO: build `add token` directive for existing token;
        }
        Proposal::AddToken(token) => {
            with_state_mut(|hub_state| {
                hub_state.chains.iter_mut().map(|(chain_id, chain_info)| {
                    // chain_dire_seq = chain.latest_dire_seq;
                    hub_state
                        .dire_queue
                        .entry(chain_id.to_string())
                        .and_modify(|dires| {
                            chain_info.latest_dire_seq += 1;
                            dires.insert(
                                chain_info.latest_dire_seq,
                                Directive::AddToken(token.clone()),
                            );
                        })
                        .or_insert_with(|| {
                            let mut dires = BTreeMap::new();
                            dires.insert(0, Directive::AddToken(token.clone()));
                            dires
                        });
                    // save token info
                    hub_state
                        .tokens
                        .insert((chain_id.to_string(), token.name), token)
                });
            });
        }
        Proposal::ChangeChainStatus(status) => {
            with_state_mut(|hub_state| {
                let mut chain_dire_seq = 0;

                if let Some(chain) = hub_state.chains.get_mut(&status.chain) {
                    chain.latest_dire_seq += 1;
                    chain_dire_seq = chain.latest_dire_seq;
                    //save chain status
                    chain.chain_status = status;
                }

                hub_state
                    .dire_queue
                    .entry(status.clone().chain)
                    .and_modify(|dires| {
                        dires.insert(chain_dire_seq, Directive::ChangeChainStatus(status.clone()));
                    })
                    .or_insert_with(|| {
                        let mut dires = BTreeMap::new();
                        dires.insert(chain_dire_seq, Directive::ChangeChainStatus(status.clone()));
                        dires
                    });
            });
        }
        Proposal::UpdateFee(fee) => {
            with_state_mut(|hub_state| {
                let mut chain_dire_seq = 0;

                if let Some(chain) = hub_state.chains.get_mut(&fee.dst_chain) {
                    chain.latest_dire_seq += 1;
                    chain_dire_seq = chain.latest_dire_seq;
                }
                //save fee in hub
                hub_state
                    .fees
                    .entry((fee.dst_chain, fee.fee_token))
                    .and_modify(|fee| fee = fee)
                    .or_insert(fee);

                hub_state
                    .dire_queue
                    .entry(fee.dst_chain.clone())
                    .and_modify(|dires| {
                        dires.insert(chain_dire_seq, Directive::UpdateFee(fee.clone()));
                    })
                    .or_insert_with(|| {
                        let mut dires = BTreeMap::new();
                        dires.insert(chain_dire_seq, Directive::UpdateFee(fee.clone()));
                        dires
                    });
            });
        }
    }
    Ok(())
}

/// check fee validate
/// build update fee directive and push it to the directive queue
/// 构建更新fee指令时，为所有支持该计费token的执行链，构建更新费指令；
#[update(guard = "auth")]
pub async fn update_fee(fee: Fee) -> Result<(), Error> {
    //TODO: check fee validate
    //  build directive

    let directive = Proposal::UpdateFee(fee);
    build_directive(directive);

    Ok(())
}

/// query directives for chain id,this method calls by route and custom
#[update(guard = "auth")]
pub async fn query_directives(
    chain_id: ChainId,
    start: u64,
    end: u64,
) -> Result<Option<BTreeMap<Seq, Directive>>, Error> {
    with_state_mut(|hub_state| {
        match hub_state.dire_queue.get(&chain_id) {
            Some(d) => {
                let mut directives: BTreeMap<u64, Directive> = BTreeMap::new();
                for (&seq, &ref dire) in d.range((Included(start), Included(end))) {
                    directives.insert(seq, dire.clone());
                }
                // remove the directive for the chain id
                // hub_state.dire_queue.remove(&chain_id);
                Ok(Some(directives))
            }
            None => Ok(None),
        }
    })
}

/// check the ticket availability
/// check chain and status
/// check token and amount
pub async fn check_ticket(_t: &Ticket) -> Result<(), Error> {
    Ok(())
}

/// check and push ticket into queue
#[update(guard = "auth")]
pub async fn send_ticket(ticket: Ticket) -> Result<(), Error> {
    // checke ticket avalidate
    check_ticket(&ticket).await?;

    // build tickets
    with_state_mut(|hub_state| {
        let mut chain_ticket_seq = 0;

        if let Some(chain) = hub_state.chains.get_mut(&ticket.dst_chain) {
            chain.latest_ticket_seq += 1;
            chain_ticket_seq = chain.latest_ticket_seq;
        }

        hub_state
            .ticket_queue
            .entry(ticket.dst_chain.clone())
            .and_modify(|tickets| {
                tickets.insert(chain_ticket_seq, ticket.clone());
            })
            .or_insert_with(|| {
                let mut tickets = BTreeMap::new();
                tickets.insert(chain_ticket_seq, ticket.clone());
                tickets
            });
    });
    // keep amount
    match ticket.action {
        Action::Transfer => with_state_mut(|hub_state| {
            hub_state
                .cross_ledger
                .transfers
                .insert(ticket.clone().ticket_id, ticket.clone());
        }),
        Action::Redeem => with_state_mut(|hub_state| {
            hub_state
                .cross_ledger
                .redeems
                .insert(ticket.clone().ticket_id, ticket);
        }),
    }

    Ok(())
}

/// query tickets for chain id,this method calls by route and custom
#[update(guard = "auth")]
pub fn query_tickets(
    chain_id: ChainId,
    start: u64,
    end: u64,
) -> Result<Option<BTreeMap<Seq, Ticket>>, Error> {
    with_state_mut(|hub_state| {
        match hub_state.ticket_queue.get(&chain_id) {
            Some(t) => {
                let mut tickets: BTreeMap<u64, Ticket> = BTreeMap::new();
                for (&seq, &ref ticket) in t.range((Included(start), Included(end))) {
                    tickets.insert(seq, ticket.clone());
                }
                // remove the tickets for the chain id
                // hub_state.ticket_queue.remove(&chain_id);
                Ok(Some(tickets))
            }
            None => Ok(None),
        }
    })
}

ic_cdk::export_candid!();

#[cfg(test)]
mod tests {
    // use super::*;
    use crypto::digest::Digest;
    use crypto::sha3::Sha3;
    #[test]
    fn hash() {
        let mut hasher = Sha3::keccak256();
        hasher.input_str("Hi,Omnity");
        let hex = hasher.result_str();
        println!("{}", hex);
    }
}
