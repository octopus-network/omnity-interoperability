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
    Action, Chain, ChainInfo, ChainStatus, DireQueue, Directive, Error, Fee, Proposal, Seq, Ticket,
    TicketId, TicketQueue, Token, TokenMetaData,
};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap, HashSet};

// use utils::init_log;
use crate::signer::PublicKeyReply;
use crate::utils::Network;

thread_local! {
    static STATE: RefCell<HubState> = RefCell::new(HubState::default());
}

#[derive(CandidType, Deserialize, Serialize, Default, Debug)]
struct CrossLedger {
    pub transfers: HashMap<TicketId, Ticket>,
    pub redeems: HashMap<TicketId, Ticket>,
}

#[derive(CandidType, Deserialize, Serialize, Default, Debug)]
struct HubState {
    pub chains: HashMap<Chain, ChainInfo>,
    pub tokens: HashMap<(Chain, Token), TokenMetaData>,
    pub fees: HashMap<(Chain, Token), Fee>,
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
                ChainStatus::Reinstate | ChainStatus::Suspend => {
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
                ChainStatus::Active | ChainStatus::Reinstate | ChainStatus::Suspend => ..,
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
/// 1. add chain
///  如果增加的是结算链，只需要在 hub 保存结算链信息即可，无需中继的其他链执行此指令；
///  如果增加的是执行链，需要分别为所有目标链构建新增链指令，然后放入队列，等待 route 中继执行；
///  todo：新增链需要考虑，是否为已经存在的token，构建新增 token 的指令？
/// 2. add token
///  需要为所有非发行链构建新增 token 指令，然后放入队列，等待route 中继执行；
/// 3. change chain status
///  需要通知所有支持向该链的转账的目标链，变更此链的状态；
#[update(guard = "auth")]
pub async fn build_directive(proposal: Proposal) -> Result<(), Error> {
    match proposal {
        Proposal::AddChain(chain) => todo!(),
        Proposal::AddToken(token) => todo!(),
        Proposal::ChangeChainStatus(statue) => todo!(),
        Proposal::UpdateFee(fee) => {
            todo!()
        }
    }
    Ok(())
}

/// check fee validate
/// build update fee directive and push it to the directive queue
/// 构建更新fee指令时，为所有支持该计费token的执行链，构建更新费指令；
#[update(guard = "auth")]
pub async fn update_fee(_fee: Fee) -> Result<(), Error> {
    // check fee validate
    // call build_directive
    Ok(())
}

/// route 或者 custom 查询与自身相关的指令信息；
/// 指令队列自动清理已经轮询过的跟chain id相关的指令；
#[update(guard = "auth")]
pub async fn query_directives(
    chain_id: Chain,
    start: u64,
    end: u64,
) -> Result<Option<HashMap<Seq, Directive>>, Error> {
    with_state_mut(|hub_state| {
        match hub_state.dire_queue.get(&chain_id) {
            Some(d) => {
                // clone
                let diretives = d.clone();
                // remove the directive for the chain id
                hub_state.dire_queue.remove(&chain_id);
                Ok(Some(diretives))
            }
            None => Ok(None),
        }
    })
}

/// check the ticket availability
/// check chain and status
/// check token and amount
pub async fn check_ticket(_t: Ticket) -> Result<(), Error> {
    Ok(())
}

/// 检查ticket 有效性，并将其放入目标链队列中
#[update(guard = "auth")]
pub async fn send_ticket(ticket: Ticket) -> Result<(), Error> {
    check_ticket(ticket).await?;
    Ok(())
}

/// route 或者 custom 查询与自身相关的tickets；
/// ticket队列自动清理已经查询过的跟chain id相关的tickets;
#[update(guard = "auth")]
pub fn query_tickets(
    chain_id: Chain,
    start: u64,
    end: u64,
) -> Result<Option<HashMap<Seq, Ticket>>, Error> {
    with_state_mut(|hub_state| {
        match hub_state.ticket_queue.get(&chain_id) {
            Some(t) => {
                // clone
                let tickets = t.clone();
                // remove the tickets for the chain id
                hub_state.ticket_queue.remove(&chain_id);
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
