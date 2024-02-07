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
    Action, ChainId, ChainInfo, ChainType, DireQueue, Directive, Error, Fee, Proposal, Seq, State,
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
    pub chain_state: State,
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
            | Proposal::ChangeChainState(_)
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

            if matches!(chain.chain_state, State::Reinstate | State::Suspend) {
                return Err(Error::ProposalError(
                    "The status of the new chain state must be active".to_string(),
                ));
            }
            //TODO: check repetitive
            Ok(format!("Tne AddChain proposal is: {}", chain))
        }
        Proposal::AddToken(token) => {
            if token.name.is_empty() || token.symbol.is_empty() || token.issue_chain.is_empty() {
                return Err(Error::ProposalError(
                    "Token id, token symbol or issue chain can not be empty".to_string(),
                ));
            }
            //TODO: check the issue chain must exsiting and not suspend!
             //TODO: check repetitive
            Ok(format!("The AddToken proposal is: {}", token))
        }
        Proposal::ChangeChainState(chain_state) => {
            if chain_state.chain_id.is_empty() {
                return Err(Error::ProposalError(
                    "Chain id can not be empty".to_string(),
                ));
            }
            //TODO:dst chain must be exsiting!
            if !matches!(
                chain_state.state,
                State::Active | State::Reinstate | State::Suspend
            ) {
                return Err(Error::ProposalError("Not support chain state".to_string()));
            }
            Ok(format!(
                "the ChangeChainStatus proposal is: {}",
                chain_state
            ))
        }
        Proposal::UpdateFee(fee) => {
            if fee.fee_token.is_empty() {
                return Err(Error::ProposalError(
                    "The Quote token can not be empty".to_string(),
                ));
            };
            //TODO: check the issue chain must exsiting and not suspend!
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
                            //check: chain state !=suspend
                            if matches!(dst_chain_info.chain_state, State::Suspend) {
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
            //TODO: build `add token` directive for new chain;
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
                    //check: chain state !=suspend
                    if matches!(dst_chain_info.chain_state, State::Suspend) {
                        continue;
                    }
                    //TODO: except the token`s issue chain
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
        Proposal::ChangeChainState(change_status) => {
            with_state_mut(|hub_state| {
                if let Some(dst_chain) = hub_state.chains.get_mut(&change_status.chain_id) {
                    //change dst chain status
                    dst_chain.chain_state = change_status.clone().state;

                    // build directive
                    for (dst_chain, dst_chain_info) in hub_state.chains.iter_mut() {
                        if dst_chain.ne(&change_status.chain_id) {
                            //check: chain state !=suspend
                            if matches!(dst_chain_info.chain_state, State::Suspend) {
                                continue;
                            }
                            hub_state
                                .dire_queue
                                .entry(dst_chain.to_string())
                                .and_modify(|dires| {
                                    dst_chain_info.latest_dire_seq += 1;
                                    dires.insert(
                                        dst_chain_info.latest_dire_seq,
                                        Directive::ChangeChainState(change_status.clone()),
                                    );
                                })
                                .or_insert_with(|| {
                                    let mut dires = BTreeMap::new();
                                    dires.insert(
                                        0,
                                        Directive::ChangeChainState(change_status.clone()),
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

/// check fee validate
/// build update fee directive and push it to the directive queue
/// 构建更新fee指令时，为所有支持该计费token的执行链，构建更新费指令；
#[update(guard = "auth")]
pub async fn update_fee(fee: Fee) -> Result<(), Error> {
    //TODO: check fee validate
    // fee.dst_chain must be execution chain

    //  build directive
    let directive = Proposal::UpdateFee(fee);
    build_directive(directive).await?;

    Ok(())
}

/// query directives for chain id,this method calls by route and custom
#[update(guard = "auth")]
pub async fn query_directives(
    chain_id: ChainId,
    start: u64,
    end: u64,
) -> Result<Option<BTreeMap<Seq, Directive>>, Error> {
    //TODO: check start and end is validate!
    // asset(start <= end)

    with_state_mut(|hub_state| {
        match hub_state.dire_queue.get(&chain_id) {
            Some(d) => {
                let mut directives: BTreeMap<u64, Directive> = BTreeMap::new();
                for (&seq, &ref dire) in d.range((Included(start), Included(end))) {
                    directives.insert(seq, dire.clone());
                }
                //TODO: remove the directive for the chain id
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
pub async fn query_tickets(
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
                //TODO: remove the tickets for the chain id
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

    use super::*;
    use crypto::digest::Digest;
    use crypto::sha3::Sha3;
    use omnity_types::{
        Action, ChainInfo, ChainState, ChainType, Fee, Proposal, State, Ticket, TokenMetaData,
    };

    use std::time::{SystemTime, UNIX_EPOCH};
    use uuid::Uuid;

    async fn add_chain() {
        let chain_info = ChainInfo {
            chain_name: "Bitcoin".to_string(),
            chain_type: ChainType::SettlementChain,
            chain_state: State::Active,
        };
        let add_chain = Proposal::AddChain(chain_info);
        let _ = build_directive(add_chain).await;

        let chain_info = ChainInfo {
            chain_name: "Ethereum".to_string(),
            chain_type: ChainType::SettlementChain,
            chain_state: State::Active,
        };
        let add_chain = Proposal::AddChain(chain_info);
        let _ = build_directive(add_chain).await;

        let chain_info = ChainInfo {
            chain_name: "Near".to_string(),
            chain_type: ChainType::ExecutionChain,
            chain_state: State::Active,
        };
        let add_chain = Proposal::AddChain(chain_info);
        let _ = build_directive(add_chain).await;

        let chain_info = ChainInfo {
            chain_name: "Otto".to_string(),
            chain_type: ChainType::ExecutionChain,
            chain_state: State::Active,
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
            chain_state: State::Active,
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

        let chain_state = ChainState {
            chain_id: "Bitcoin".to_string(),
            state: State::Suspend,
        };
        let change_state = Proposal::ChangeChainState(chain_state);
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
        let chain_state = ChainState {
            chain_id: "Otto".to_string(),
            state: State::Suspend,
        };
        let chang_chain_state = Proposal::ChangeChainState(chain_state);
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
            action: Action::Transfer,
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
            action: Action::Redeem,
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
