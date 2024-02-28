mod auth;
mod errors;
mod memory;
mod metrics;
mod signer;
mod utils;

use candid::types::principal::Principal;
use candid::CandidType;

use auth::auth;
use ic_cdk::{init, post_upgrade, pre_upgrade, query, update};
use ic_stable_structures::writer::Writer;
use ic_stable_structures::Memory;
use log::debug;
use omnity_types::{
    ChainCondition, ChainId, ChainInfo, ChainState, ChainType, DireQueue, Directive, Error, Fee,
    LockedToken, Proposal, Seq, StateAction, Ticket, TicketId, TicketQueue, TokenCondition,
    TokenId, TokenMetaData, Topic, TxAction, TxCondition,
};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap, HashSet};

// use utils::init_log;
use crate::signer::PublicKeyReply;
use crate::utils::Network;
pub type TotalAmount = u64;

thread_local! {
    static STATE: RefCell<HubState> = RefCell::new(HubState::default());
}

#[derive(CandidType, Deserialize, Serialize, Default, Clone, Debug)]
pub struct ChainInfoWithSeq {
    pub chain_name: ChainId,
    pub chain_type: ChainType,
    pub chain_state: ChainState,
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
    pub locked_tokens: HashMap<TokenId, HashMap<ChainId, TotalAmount>>,
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
    let caller = ic_cdk::api::caller();
    with_state_mut(|hs| hs.owner = Some(caller))
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
pub async fn validate_proposal(proposal: Proposal) -> Result<String, Error> {
    if !matches!(
        proposal,
        Proposal::AddChain(_)
            | Proposal::AddToken(_)
            | Proposal::ToggleChainState(_)
            | Proposal::UpdateFee(_)
    ) {
        return Err(Error::NotSupportedProposal);
    }
    match proposal {
        Proposal::AddChain(chain) => {
            if chain.chain_name.is_empty() {
                return Err(Error::ProposalError(
                    "Chain name can not be empty".to_string(),
                ));
            }

            if matches!(chain.chain_state, ChainState::Deactive) {
                return Err(Error::ProposalError(
                    "The status of the new chain state must be active".to_string(),
                ));
            }
            // check chain repetitive
            if with_state(|hub_state| hub_state.chains.contains_key(&chain.chain_name)) {
                return Err(Error::ProposalError(format!(
                    "The {} already exists",
                    chain.chain_name
                )));
            }

            Ok(format!("Tne AddChain proposal is: {}", chain))
        }
        Proposal::AddToken(token) => {
            if token.name.is_empty() || token.symbol.is_empty() || token.issue_chain.is_empty() {
                return Err(Error::ProposalError(
                    "Token id, token symbol or issue chain can not be empty".to_string(),
                ));
            }
            // check token repetitive
            if with_state(|hub_state| {
                hub_state
                    .tokens
                    .contains_key(&(token.issue_chain.clone(), token.name.clone()))
            }) {
                return Err(Error::ProposalError(format!(
                    "The {} already exists",
                    token.name
                )));
            }
            //check the issue chain must exsiting and not deactive!
            let _ = with_state(|hub_state| {
                match hub_state.chains.get(&token.issue_chain) {
                    Some(chain) => {
                        if matches!(chain.chain_state, ChainState::Deactive) {
                            return Err(Error::ProposalError(format!(
                                "The {} is deactive",
                                token.issue_chain
                            )));
                        }
                    }
                    None => {
                        return Err(Error::ProposalError(format!(
                            "The {} not exists",
                            token.issue_chain
                        )))
                    }
                }

                Ok(())
            });

            Ok(format!("The AddToken proposal is: {}", token))
        }
        Proposal::ToggleChainState(toggle_state) => {
            if toggle_state.chain_id.is_empty() {
                return Err(Error::ProposalError(
                    "Chain id can not be empty".to_string(),
                ));
            }
            if !matches!(
                toggle_state.action,
                StateAction::Activate | StateAction::Deactivate
            ) {
                return Err(Error::ProposalError("Not support chain state".to_string()));
            }

            let _ = with_state(|hub_state| {
                match hub_state.chains.get(&toggle_state.chain_id) {
                    Some(chain) => {
                        //If the state and action are consistent, there is no need to switch
                        if (matches!(chain.chain_state, ChainState::Active)
                            && matches!(toggle_state.action, StateAction::Activate))
                            || (matches!(chain.chain_state, ChainState::Deactive)
                                && matches!(toggle_state.action, StateAction::Deactivate))
                        {
                            return Err(Error::ProposalError(format!(
                                "The {} is no need to switch",
                                toggle_state.chain_id
                            )));
                        }
                    }
                    None => {
                        return Err(Error::ProposalError(format!(
                            "The {} not exists",
                            toggle_state.chain_id
                        )))
                    }
                }

                Ok(())
            });

            Ok(format!(
                "The ToggleChainStatus proposal is: {}",
                toggle_state
            ))
        }
        Proposal::UpdateFee(fee) => {
            if fee.fee_token.is_empty() {
                return Err(Error::ProposalError(
                    "The Quote token can not be empty".to_string(),
                ));
            };
            //check the issue chain must exsiting and not deactive!
            let _ = with_state(|hub_state| {
                match hub_state.chains.get(&fee.dst_chain_id) {
                    Some(chain) => {
                        if matches!(chain.chain_state, ChainState::Deactive) {
                            return Err(Error::ProposalError("The chain is deactive".to_string()));
                        }
                    }
                    None => return Err(Error::ProposalError("The chain not exists".to_string())),
                }

                Ok(())
            });
            Ok(format!("The UpdateFee proposal is: {}", fee))
        }
    }
}

/// build directive based on proposal, this method will be called by sns
/// add chain / add token /change chain status / update fee
#[update(guard = "auth")]
pub async fn build_directive(proposal: Proposal) -> Result<(), Error> {
    ic_cdk::println!("build directive for :{:?}", proposal);
    match proposal {
        Proposal::AddChain(chain) => {
            with_state_mut(|hub_state| {
                let mut new_chain = ChainInfoWithSeq {
                    chain_name: chain.chain_name.clone(),
                    chain_type: chain.chain_type.clone(),
                    chain_state: chain.chain_state.clone(),
                    latest_dire_seq: 0,
                    latest_ticket_seq: 0,
                };

                // build directives
                match chain.chain_type {
                    ChainType::SettlementChain => (),

                    ChainType::ExecutionChain => {
                        for (dst_chain_name, dst_chain_info) in hub_state.chains.iter_mut() {
                            //check: chain state != deactive
                            if matches!(dst_chain_info.chain_state, ChainState::Deactive) {
                                continue;
                            }
                            // build directive for exsiting chain
                            hub_state
                                .dire_queue
                                .entry(dst_chain_name.to_string())
                                .and_modify(|dires| {
                                    // increases the new chain seq
                                    dst_chain_info.latest_dire_seq += 1;
                                    dires.insert(
                                        dst_chain_info.latest_dire_seq,
                                        Directive::AddChain(chain.clone()),
                                    );
                                })
                                .or_insert_with(|| {
                                    let mut dires = BTreeMap::new();
                                    dires.insert(0u64, Directive::AddChain(chain.clone()));
                                    dires
                                });

                            // build directive for new chain except new chain self
                            if dst_chain_name.ne(&new_chain.chain_name) {
                                let new_dst_chain_info = ChainInfo {
                                    chain_name: dst_chain_name.to_string(),
                                    chain_type: dst_chain_info.chain_type.clone(),
                                    chain_state: dst_chain_info.chain_state.clone(),
                                };
                                hub_state
                                    .dire_queue
                                    .entry(new_chain.chain_name.clone())
                                    .and_modify(|dires| {
                                        // increases the new chain seq
                                        new_chain.latest_dire_seq += 1;
                                        dires.insert(
                                            new_chain.latest_dire_seq,
                                            Directive::AddChain(new_dst_chain_info.clone()),
                                        );
                                    })
                                    .or_insert_with(|| {
                                        let mut dires = BTreeMap::new();
                                        dires.insert(0u64, Directive::AddChain(new_dst_chain_info));
                                        dires
                                    });
                            }
                        }
                    }
                }

                // save new chain
                hub_state
                    .chains
                    .insert(chain.chain_name.clone(), new_chain.clone());
            });
            //TODO: build `add token` directive for new chain ?
        }
        Proposal::AddToken(token) => {
            with_state_mut(|hub_state| {
                // save token info
                hub_state.tokens.insert(
                    (token.clone().issue_chain, token.clone().name),
                    token.clone(),
                );

                // build directive
                for (dst_chain_name, dst_chain_info) in hub_state.chains.iter_mut() {
                    //check: chain state !=Deactive
                    if matches!(dst_chain_info.chain_state, ChainState::Deactive) {
                        continue;
                    }
                    //TODO: except the token`s issue chain ?
                    hub_state
                        .dire_queue
                        .entry(dst_chain_name.to_string())
                        .and_modify(|dires| {
                            dst_chain_info.latest_dire_seq += 1;
                            dires.insert(
                                dst_chain_info.latest_dire_seq,
                                Directive::AddToken(token.clone()),
                            );
                        })
                        .or_insert_with(|| {
                            let mut dires = BTreeMap::new();
                            dires.insert(0, Directive::AddToken(token.clone()));
                            dires
                        });
                }
            });
        }
        Proposal::ToggleChainState(toggle_status) => {
            with_state_mut(|hub_state| {
                if let Some(dst_chain) = hub_state.chains.get_mut(&toggle_status.chain_id) {
                    //change dst chain status
                    match toggle_status.action {
                        StateAction::Activate => dst_chain.chain_state = ChainState::Active,
                        StateAction::Deactivate => dst_chain.chain_state = ChainState::Deactive,
                    }

                    // build directive
                    for (dst_chain, dst_chain_info) in hub_state.chains.iter_mut() {
                        if dst_chain.ne(&toggle_status.chain_id) {
                            //check: chain state !=Deactive
                            if matches!(dst_chain_info.chain_state, ChainState::Deactive) {
                                continue;
                            }
                            hub_state
                                .dire_queue
                                .entry(dst_chain.to_string())
                                .and_modify(|dires| {
                                    dst_chain_info.latest_dire_seq += 1;
                                    dires.insert(
                                        dst_chain_info.latest_dire_seq,
                                        Directive::ToggleChainState(toggle_status.clone()),
                                    );
                                })
                                .or_insert_with(|| {
                                    let mut dires = BTreeMap::new();
                                    dires.insert(
                                        0,
                                        Directive::ToggleChainState(toggle_status.clone()),
                                    );
                                    dires
                                });
                        }
                    }
                }
            });
        }
        Proposal::UpdateFee(fee) => {
            with_state_mut(|hub_state| {
                if let Some(dst_chain) = hub_state.chains.get_mut(&fee.dst_chain_id) {
                    // save fee info
                    hub_state
                        .fees
                        .entry((dst_chain.chain_name.clone(), fee.clone().fee_token))
                        .and_modify(|f| *f = fee.clone())
                        .or_insert(fee.clone());

                    // build `update fee` directive for dst chain
                    hub_state
                        .dire_queue
                        .entry(dst_chain.chain_name.clone().to_string())
                        .and_modify(|dires| {
                            // increase seq
                            dst_chain.latest_dire_seq += 1;
                            dires.insert(
                                dst_chain.latest_dire_seq,
                                Directive::UpdateFee(fee.clone()),
                            );
                        })
                        .or_insert_with(|| {
                            let mut dires = BTreeMap::new();
                            // seq is zero
                            dires.insert(0, Directive::UpdateFee(fee.clone()));
                            dires
                        });
                }
            });
        }
    }
    Ok(())
}

/// check and build update fee directive and push it to the directive queue
#[update(guard = "auth")]
pub async fn update_fee(fee: Fee) -> Result<(), Error> {
    // check proposal
    validate_proposal(Proposal::UpdateFee(fee.clone())).await?;
    //  build directive
    build_directive(Proposal::UpdateFee(fee)).await?;

    Ok(())
}

/// query directives for chain id,this method will be called by route and custom
#[query(guard = "auth")]
pub async fn query_directives(
    chain_id: ChainId,
    from: u64,
    num: u64,
) -> Result<Vec<(Seq, Directive)>, Error> {
    let end = from + num;
    // asset(start <= end)
    if from > end {
        return Err(Error::CustomError(format!(
            "Query range error, from({}) > from + num({})",
            from, end
        )));
    }

    with_state(|hub_state| {
        match hub_state.dire_queue.get(&chain_id) {
            Some(d) => {
                let mut directives: Vec<(u64, Directive)> = Vec::new();
                for (&seq, &ref dire) in d.range(from..end) {
                    directives.push((seq, dire.clone()));
                }
                //TODO: remove the directive for the chain id ?
                // hub_state.dire_queue.remove(&chain_id);
                Ok(directives)
            }
            None => Err(Error::NotFoundChain(chain_id)),
        }
    })
}

/// query directives for chain id filter by topic,this method will be called by route and custom
#[query(guard = "auth")]
pub async fn query_directive(
    chain_id: ChainId,
    topic: Option<Topic>,
    from: u64,
    num: u64,
) -> Result<Vec<(Seq, Directive)>, Error> {
    let end = from + num;
    with_state(|hub_state| match hub_state.dire_queue.get(&chain_id) {
        Some(d) => {
            let mut directives: Vec<(u64, Directive)> = Vec::new();
            if let Some(topic) = topic {
                match topic {
                    Topic::AddChain(chain_type) => {
                        if let Some(dst_chain_type) = chain_type {
                            for (&seq, &ref dire) in d.range(from..end) {
                                if let Directive::AddChain(chain_info) = dire {
                                    if dst_chain_type == chain_info.chain_type {
                                        directives.push((seq, dire.clone()));
                                    }
                                }
                            }
                        } else {
                            for (&seq, &ref dire) in d.range(from..end) {
                                if matches!(dire, Directive::AddChain(_)) {
                                    directives.push((seq, dire.clone()));
                                }
                            }
                        }
                    }
                    Topic::AddToken(token_id) => {
                        if let Some(token) = token_id {
                            for (&seq, &ref dire) in d.range(from..end) {
                                if let Directive::AddToken(token_meta) = dire {
                                    if token_meta.name.eq(&token) {
                                        directives.push((seq, dire.clone()));
                                    }
                                }
                            }
                        } else {
                            for (&seq, &ref dire) in d.range(from..end) {
                                if matches!(dire, Directive::AddToken(_)) {
                                    directives.push((seq, dire.clone()));
                                }
                            }
                        }
                    }
                    Topic::UpdateFee(token_id) => {
                        if let Some(token) = token_id {
                            for (&seq, &ref dire) in d.range(from..end) {
                                if let Directive::UpdateFee(fee) = dire {
                                    if fee.fee_token.eq(&token) {
                                        directives.push((seq, dire.clone()));
                                    }
                                }
                            }
                        } else {
                            for (&seq, &ref dire) in d.range(from..end) {
                                if matches!(dire, Directive::UpdateFee(_)) {
                                    directives.push((seq, dire.clone()));
                                }
                            }
                        }
                    }
                    Topic::ActivateChain => {
                        for (&seq, &ref dire) in d.range(from..end) {
                            if let Directive::ToggleChainState(toggle_state) = dire {
                                if toggle_state.action == StateAction::Activate {
                                    directives.push((seq, dire.clone()));
                                }
                            }
                        }
                    }
                    Topic::DeactivateChain => {
                        for (&seq, &ref dire) in d.range(from..end) {
                            if let Directive::ToggleChainState(toggle_state) = dire {
                                if toggle_state.action == StateAction::Deactivate {
                                    directives.push((seq, dire.clone()));
                                }
                            }
                        }
                    }
                }
            }

            Ok(directives)
        }
        None => Err(Error::NotFoundChain(chain_id)),
    })
}

/// check the ticket availability
pub async fn check_ticket(ticket: &Ticket) -> Result<(), Error> {
    let _ = with_state(|hub_state| {
        // check chain and status
        match hub_state.chains.get(&ticket.src_chain) {
            Some(chain) => {
                if matches!(chain.chain_state, ChainState::Deactive) {
                    return Err(Error::CustomError(format!(
                        "The {} is deactive",
                        ticket.src_chain
                    )));
                }
            }
            None => {
                return Err(Error::CustomError(format!(
                    "The {} not exists",
                    ticket.src_chain
                )))
            }
        }

        match hub_state.chains.get(&ticket.dst_chain) {
            Some(chain) => {
                if matches!(chain.chain_state, ChainState::Deactive) {
                    return Err(Error::CustomError(format!(
                        "The {} is deactive",
                        ticket.dst_chain
                    )));
                }
            }
            None => {
                return Err(Error::CustomError(format!(
                    "The {} not exists",
                    ticket.dst_chain
                )))
            }
        }

        // check token
        if hub_state
            .tokens
            .get(&(ticket.src_chain.clone(), ticket.token.clone()))
            .is_none()
        {
            return Err(Error::CustomError(format!(
                "The {} is not exists",
                ticket.token
            )));
        }
        // TODO: check sender`s token balance availability

        Ok(())
    });
    Ok(())
}

/// check and push ticket into queue
#[update(guard = "auth")]
pub async fn send_ticket(ticket: Ticket) -> Result<(), Error> {
    // checke ticket avalidate
    check_ticket(&ticket).await?;

    // build tickets
    with_state_mut(|hub_state| {
        if let Some(chain) = hub_state.chains.get_mut(&ticket.dst_chain) {
            hub_state
                .ticket_queue
                .entry(ticket.dst_chain.clone())
                .and_modify(|tickets| {
                    //increase seq
                    chain.latest_ticket_seq += 1;
                    tickets.insert(chain.latest_ticket_seq, ticket.clone());
                })
                .or_insert_with(|| {
                    let mut tickets = BTreeMap::new();
                    // seq is 0
                    tickets.insert(chain.latest_ticket_seq, ticket.clone());
                    tickets
                });
        }
    });

    // keep amount
    match ticket.action {
        TxAction::Transfer => with_state_mut(|hub_state| {
            hub_state
                .cross_ledger
                .transfers
                .insert(ticket.clone().ticket_id, ticket.clone());
        }),
        TxAction::Redeem => with_state_mut(|hub_state| {
            hub_state
                .cross_ledger
                .redeems
                .insert(ticket.clone().ticket_id, ticket.clone());
        }),
    }

    // count locked token
    with_state_mut(|hub_state| {
        hub_state
            .locked_tokens
            .entry(ticket.clone().token)
            .and_modify(|chain_tokens| {
                chain_tokens
                    .entry(ticket.clone().src_chain)
                    .and_modify(|total_amount| {
                        //TODO: covert ticket amount into number and handle the token decimal
                        let amount: u64 = ticket.amount.parse().unwrap();
                        *total_amount += amount
                    })
                    .or_insert_with(|| {
                        let amount: u64 = ticket.amount.parse().unwrap();
                        amount
                    });
            })
            .or_insert_with(|| {
                //TODO: covert ticket amount into number and handle the token decimal
                let mut locked_tokens = HashMap::new();
                let total_amount: u64 = ticket.amount.parse().unwrap();

                locked_tokens.insert(ticket.src_chain, total_amount);
                locked_tokens
            });
    });

    Ok(())
}

/// query tickets for chain id,this method will be called by route and custom
#[query(guard = "auth")]
pub async fn query_tickets(
    chain_id: ChainId,
    from: u64,
    num: u64,
) -> Result<Vec<(Seq, Ticket)>, Error> {
    //TODO: check from and num
    let end = from + num;
    with_state(|hub_state| {
        match hub_state.ticket_queue.get(&chain_id) {
            Some(t) => {
                let mut tickets: Vec<(u64, Ticket)> = Vec::new();
                for (&seq, &ref ticket) in t.range(from..end) {
                    tickets.push((seq, ticket.clone()));
                }

                //TODO: remove the tickets for the chain id
                // hub_state.ticket_queue.remove(&chain_id);
                Ok(tickets)
            }
            None => Err(Error::NotFoundChain(chain_id)),
        }
    })
}

ic_cdk::export_candid!();

#[cfg(test)]
mod tests {

    use super::*;
    use crypto::digest::Digest;
    use crypto::sha3::Sha3;
    use omnity_types::{
        ChainInfo, ChainType, Fee, Proposal, StateAction, Ticket, ToggleState, TokenMetaData,
        TxAction,
    };

    use std::time::{SystemTime, UNIX_EPOCH};
    use uuid::Uuid;

    async fn add_chain() {
        let chain_info = ChainInfo {
            chain_name: "Bitcoin".to_string(),
            chain_type: ChainType::SettlementChain,
            chain_state: ChainState::Active,
        };
        let add_chain = Proposal::AddChain(chain_info);
        let _ = build_directive(add_chain).await;

        let chain_info = ChainInfo {
            chain_name: "Ethereum".to_string(),
            chain_type: ChainType::SettlementChain,
            chain_state: ChainState::Active,
        };
        let add_chain = Proposal::AddChain(chain_info);
        let _ = build_directive(add_chain).await;

        let chain_info = ChainInfo {
            chain_name: "Near".to_string(),
            chain_type: ChainType::ExecutionChain,
            chain_state: ChainState::Active,
        };
        let add_chain = Proposal::AddChain(chain_info);
        let _ = build_directive(add_chain).await;

        let chain_info = ChainInfo {
            chain_name: "Otto".to_string(),
            chain_type: ChainType::ExecutionChain,
            chain_state: ChainState::Active,
        };
        let add_chain = Proposal::AddChain(chain_info);
        let _ = build_directive(add_chain).await;
    }

    async fn add_token() {
        let token = TokenMetaData {
            name: "BTC".to_string(),
            symbol: "BTC".to_owned(),
            issue_chain: "Bitcion".to_string(),
            decimals: 18,
            icon: None,
        };
        let add_token = Proposal::AddToken(token);
        let _ = build_directive(add_token).await;

        let token = TokenMetaData {
            name: "ETH".to_string(),
            symbol: "ETH".to_owned(),
            issue_chain: "Ethereum".to_string(),
            decimals: 18,
            icon: None,
        };
        let add_token = Proposal::AddToken(token);
        let _ = build_directive(add_token).await;

        let token = TokenMetaData {
            name: "OCT".to_string(),
            symbol: "OCT".to_owned(),
            issue_chain: "Near".to_string(),
            decimals: 18,
            icon: None,
        };

        let add_token = Proposal::AddToken(token);
        let _ = build_directive(add_token).await;

        let token = TokenMetaData {
            name: "OTTO".to_string(),
            symbol: "OTTO".to_owned(),
            issue_chain: "Otto".to_string(),
            decimals: 18,
            icon: None,
        };
        let add_token = Proposal::AddToken(token);
        let _ = build_directive(add_token).await;
    }

    fn get_timestamp() -> u64 {
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        since_the_epoch.as_millis() as u64
    }
    #[test]
    fn hash() {
        let mut hasher = Sha3::keccak256();
        hasher.input_str("Hi,Omnity");
        let hex = hasher.result_str();
        println!("{}", hex);
    }
    #[tokio::test]
    async fn test_validate_proposal() {
        let chain_info = ChainInfo {
            chain_name: "Bitcoin".to_string(),
            chain_type: ChainType::SettlementChain,
            chain_state: ChainState::Active,
        };
        let add_chain = Proposal::AddChain(chain_info);
        let result = validate_proposal(add_chain).await;
        println!("Proposal::AddChain(chain_info) result:{:?}", result);
        assert!(result.is_ok());

        let token = TokenMetaData {
            name: "Octopus".to_string(),
            symbol: "OCT".to_owned(),
            issue_chain: "Near".to_string(),
            decimals: 18,
            icon: None,
        };
        let add_token = Proposal::AddToken(token);
        let result = validate_proposal(add_token).await;
        println!("Proposal::AddToken(token) result:{:?}", result);
        assert!(result.is_ok());

        let chain_state = ToggleState {
            chain_id: "Bitcoin".to_string(),
            action: StateAction::Deactivate,
        };
        let change_state = Proposal::ToggleChainState(chain_state);
        let result = validate_proposal(change_state).await;
        println!(
            "Proposal::ChangeChainState(chain_state) result:{:?}",
            result
        );
        assert!(result.is_ok());

        let fee = Fee {
            dst_chain_id: "Bitcoin".to_string(),
            fee_token: "OCT".to_string(),
            factor: 18,
        };

        let update_fee = Proposal::UpdateFee(fee);
        let result = validate_proposal(update_fee).await;
        println!("Proposal::UpdateFee(fee) result:{:?}", result);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_build_chain_directive() {
        // add chain
        add_chain().await;

        // print all directives
        with_state(|hs| {
            for (chain_id, dires) in hs.dire_queue.iter() {
                println!(
                    "chain id: {:}, chain info: {:?}, chain dires: {:?}",
                    chain_id,
                    hs.chains.get(chain_id),
                    dires
                );
            }
        })
    }

    #[tokio::test]
    async fn test_build_token_directive() {
        // add chain
        add_chain().await;
        // add token
        add_token().await;

        // print all tokens
        with_state(|hs| {
            for (token_key, token) in hs.tokens.iter() {
                println!("token key: {:?}, : token meta data: {:?}", token_key, token);
            }
        });

        // print all directives
        with_state(|hs| {
            for (chain_id, dires) in hs.dire_queue.iter() {
                println!(
                    "chain id: {:}, chain info: {:?}, chain dires: {:?}",
                    chain_id,
                    hs.chains.get(chain_id),
                    dires
                );
            }
        })
    }

    #[tokio::test]
    async fn test_build_chain_state_directive() {
        // add chain
        add_chain().await;
        // add token
        add_token().await;

        // change chain state
        let chain_state = ToggleState {
            chain_id: "Otto".to_string(),
            action: StateAction::Deactivate,
        };
        let chang_chain_state = Proposal::ToggleChainState(chain_state);
        let _ = build_directive(chang_chain_state).await;

        // print chain info and directives
        with_state(|hs| {
            for (chain_id, dires) in hs.dire_queue.iter() {
                println!(
                    "chain id: {:}, chain info: {:?}, chain dires: {:?}",
                    chain_id,
                    hs.chains.get(chain_id),
                    dires
                );
            }
        })
    }

    #[tokio::test]
    async fn test_update_fee() {
        // add chain
        add_chain().await;
        // add token
        add_token().await;

        // change chain state
        let fee = Fee {
            dst_chain_id: "Near".to_string(),
            fee_token: "OTTO".to_string(),
            factor: 12,
        };

        // let update_fee = Proposal::UpdateFee(fee);
        // let _ = build_directive(update_fee).await;
        let _ = update_fee(fee).await;

        // print fee info
        with_state(|hs| {
            for (fee_key, fee) in hs.fees.iter() {
                println!("fee key: {:?}, fee: {:?}", fee_key, fee);
            }
        });

        // query directives for chain id
        let chaid_ids = ["Bitcoin", "Ethereum", "Near", "Otto"];
        for chain_id in chaid_ids {
            let result = query_directives(chain_id.to_string(), 0, 5).await;
            println!("query_directives for {:} dires: {:?}", chain_id, result);
            assert!(result.is_ok());
        }
    }

    #[tokio::test]
    async fn test_check_ticket() {
        assert!(true);
    }

    #[tokio::test]
    async fn test_send_ticket() {
        // add chain
        add_chain().await;
        // add token
        add_token().await;

        // build `transfer` ticket
        let current_timestamp = get_timestamp();
        let ticket = Ticket {
            ticket_id: Uuid::new_v4().to_string(),
            created_time: current_timestamp,
            src_chain: "Bitcoin".to_string(),
            dst_chain: "Near".to_string(),
            action: TxAction::Transfer,
            token: "ODR".to_string(),
            amount: 88888.to_string(),
            sender: "sdsdfsyiesdfsdfds".to_string(),
            receiver: "sdfsdfsdffdrytrrr".to_string(),
            memo: None,
        };
        let _ = send_ticket(ticket).await;

        // build `redeem` ticket
        let current_timestamp = get_timestamp();
        let ticket = Ticket {
            ticket_id: Uuid::new_v4().to_string(),
            created_time: current_timestamp,
            src_chain: "Near".to_string(),
            dst_chain: "Bitcoin".to_string(),
            action: TxAction::Redeem,
            token: "WODR".to_string(),
            amount: 88888.to_string(),
            sender: "sdfsdfsdffdrytrrr".to_string(),
            receiver: "sdsdfsyiesdfsdfds".to_string(),
            memo: None,
        };
        let _ = send_ticket(ticket).await;

        // print tickets queue
        with_state(|hs| {
            for (dst_chain, tickets) in hs.ticket_queue.iter() {
                println!("dst chain: {:?}, tickets: {:?}", dst_chain, tickets);
            }
        });

        // query tickets for chain id
        let chaid_ids = ["Bitcoin", "Ethereum", "Near", "Otto"];
        for chain_id in chaid_ids {
            let result = query_tickets(chain_id.to_string(), 0, 5).await;
            println!("query tickets for {:} tickets: {:?}", chain_id, result);
            assert!(result.is_ok());
        }
    }
}
