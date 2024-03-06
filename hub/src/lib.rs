mod auth;
mod errors;
mod memory;
mod metrics;
mod util;

use candid::types::principal::Principal;
use candid::CandidType;

use auth::auth;
use ic_cdk::{init, post_upgrade, pre_upgrade, query, update};
use ic_log::writer::Logs;
use ic_stable_structures::writer::Writer;
use ic_stable_structures::Memory;

use log::info;
use omnity_types::{
    Account, ChainCondition, ChainId, ChainInfo, ChainState, ChainType, DireQueue, Directive,
    Error, Fee, Proposal, Seq, StateAction, Ticket, TicketId, TicketQueue, TokenCondition, TokenId,
    TokenMeta, TokenOnChain, Topic, TxAction, TxCondition,
};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::num::ParseIntError;
use util::init_log;
pub type Amount = u128;

thread_local! {
    static STATE: RefCell<HubState> = RefCell::new(HubState::default());
}

#[derive(CandidType, Deserialize, Serialize, Default, Clone, Debug)]
pub struct ChainInfoWithSeq {
    pub chain_id: ChainId,
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
struct HubState {
    pub chains: HashMap<ChainId, ChainInfoWithSeq>,
    pub tokens: HashMap<(ChainId, TokenId), TokenMeta>,
    pub fees: HashMap<(ChainId, TokenId), Fee>,
    pub cross_ledger: HashMap<TicketId, Ticket>,
    pub accounts: HashMap<Account, HashMap<(ChainId, TokenId), Amount>>,
    pub token_position: HashMap<(ChainId, TokenId), Amount>,
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
    init_log();
    let caller = ic_cdk::api::caller();
    with_state_mut(|hs| hs.owner = Some(caller))
}

#[pre_upgrade]
fn pre_upgrade() {
    info!("begin to handle pre_update state ...");

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
    init_log();
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
            if chain.chain_id.is_empty() {
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
            if with_state(|hub_state| hub_state.chains.contains_key(&chain.chain_id)) {
                return Err(Error::ProposalError(format!(
                    "The chain({}) already exists",
                    chain.chain_id
                )));
            }
            let result = format!("Tne AddChain proposal is: {}", chain);
            info!("validate_proposal result:{} ", result);
            Ok(result)
        }
        Proposal::AddToken(token) => {
            if token.token_id.is_empty() || token.symbol.is_empty() || token.issue_chain.is_empty()
            {
                return Err(Error::ProposalError(
                    "Token id, token symbol or issue chain can not be empty".to_string(),
                ));
            }
            // check token repetitive
            if with_state(|hub_state| {
                hub_state
                    .tokens
                    .contains_key(&(token.issue_chain.clone(), token.token_id.clone()))
            }) {
                return Err(Error::ProposalError(format!(
                    "The token({}) already exists",
                    token.token_id
                )));
            }
            //check the issue chain must exsiting and not deactive!
            with_state(|hub_state| match hub_state.chains.get(&token.issue_chain) {
                Some(chain) => {
                    if matches!(chain.chain_state, ChainState::Deactive) {
                        Err(Error::ProposalError(format!(
                            "The chain({}) is deactive",
                            token.issue_chain
                        )))
                    } else {
                        let result = format!("The AddToken proposal is: {}", token);
                        info!("validate_proposal result:{} ", result);
                        Ok(result)
                    }
                }
                None => Err(Error::ProposalError(format!(
                    "The chain({}) is not exists",
                    token.issue_chain
                ))),
            })
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

            with_state(|hub_state| {
                match hub_state.chains.get(&toggle_state.chain_id) {
                    Some(chain) => {
                        //If the state and action are consistent, no need to switch
                        if (matches!(chain.chain_state, ChainState::Active)
                            && matches!(toggle_state.action, StateAction::Activate))
                            || (matches!(chain.chain_state, ChainState::Deactive)
                                && matches!(toggle_state.action, StateAction::Deactivate))
                        {
                            Err(Error::ProposalError(format!(
                                "The chain({}) don`nt need to switch",
                                toggle_state.chain_id
                            )))
                        } else {
                            let result =
                                format!("The ToggleChainStatus proposal is: {}", toggle_state);
                            info!("validate_proposal result:{} ", result);
                            Ok(result)
                        }
                    }
                    None => Err(Error::ProposalError(format!(
                        "The chain({}) is not exists",
                        toggle_state.chain_id
                    ))),
                }
            })
        }
        Proposal::UpdateFee(fee) => {
            if fee.fee_token.is_empty() {
                return Err(Error::ProposalError(
                    "The fee token can not be empty".to_string(),
                ));
            };
            //check the issue chain must exsiting and not deactive!
            with_state(|hub_state| match hub_state.chains.get(&fee.dst_chain_id) {
                Some(chain) => {
                    if matches!(chain.chain_state, ChainState::Deactive) {
                        Err(Error::ProposalError("The chain is deactive".to_string()))
                    } else {
                        let result = format!("The UpdateFee proposal is: {}", fee);
                        info!("validate_proposal result:{} ", result);
                        Ok(result)
                    }
                }
                None => Err(Error::ProposalError("The chain is not exists".to_string())),
            })
        }
    }
}

/// build directive based on proposal, this method will be called by sns
/// add chain / add token /change chain status / update fee
#[update(guard = "auth")]
pub async fn build_directive(proposal: Proposal) -> Result<(), Error> {
    match proposal {
        Proposal::AddChain(chain) => {
            info!("build directive for `AddChain` proposal :{:?}", chain);

            with_state_mut(|hub_state| {
                let mut new_chain = ChainInfoWithSeq {
                    chain_id: chain.chain_id.clone(),
                    chain_type: chain.chain_type.clone(),
                    chain_state: chain.chain_state.clone(),
                    latest_dire_seq: 0,
                    latest_ticket_seq: 0,
                };

                // build directives
                match chain.chain_type {
                    // nothing to do
                    ChainType::SettlementChain => {
                        info!("for settlement chain,  no need to build directive!");
                    }

                    ChainType::ExecutionChain => {
                        for (dst_chain_id, dst_chain) in hub_state.chains.iter_mut() {
                            //check: chain state != deactive
                            if matches!(dst_chain.chain_state, ChainState::Deactive) {
                                continue;
                            }
                            // build directive for exsiting chain
                            info!(" build directive for exsiting chain!");
                            hub_state
                                .dire_queue
                                .entry(dst_chain_id.to_string())
                                .and_modify(|dires| {
                                    // increases the new chain seq
                                    dst_chain.latest_dire_seq += 1;
                                    dires.insert(
                                        dst_chain.latest_dire_seq,
                                        Directive::AddChain(chain.clone()),
                                    );
                                })
                                .or_insert_with(|| {
                                    let mut dires = BTreeMap::new();
                                    dires.insert(0u64, Directive::AddChain(chain.clone()));
                                    dires
                                });

                            // build directive for new chain except new chain self
                            info!(" build directive for new chain, but except new chain self!");
                            if dst_chain_id.ne(&new_chain.chain_id) {
                                let new_dst_chain_info = ChainInfo {
                                    chain_id: dst_chain_id.to_string(),
                                    chain_type: dst_chain.chain_type.clone(),
                                    chain_state: dst_chain.chain_state.clone(),
                                };
                                hub_state
                                    .dire_queue
                                    .entry(new_chain.chain_id.clone())
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
                info!(" save new chain: {:?}", new_chain);
                hub_state
                    .chains
                    .insert(chain.chain_id.clone(), new_chain.clone());
            });
            //TODO: build `add token` directive for new chain ?
        }

        Proposal::AddToken(token) => {
            info!("build directive for `AddToken` proposal :{:?}", token);
            with_state_mut(|hub_state| {
                // save token info
                hub_state.tokens.insert(
                    (token.issue_chain.to_string(), token.token_id.to_string()),
                    token.clone(),
                );

                // build directive
                for (dst_chain_id, dst_chain) in hub_state.chains.iter_mut() {
                    //check: chain state !=Deactive
                    if matches!(dst_chain.chain_state, ChainState::Deactive) {
                        continue;
                    }
                    //TODO: except the token`s issue chain ?
                    hub_state
                        .dire_queue
                        .entry(dst_chain_id.to_string())
                        .and_modify(|dires| {
                            dst_chain.latest_dire_seq += 1;
                            dires.insert(
                                dst_chain.latest_dire_seq,
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
            info!(
                "build directive for `ToggleChainState` proposal :{:?}",
                toggle_status
            );
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
                            // TODO: for activate, need to build `add chain/add token/update fee` directive that during Deactive losted!
                        }
                    }
                }
            });
        }
        Proposal::UpdateFee(fee) => {
            info!("build directive for `UpdateFee` proposal :{:?}", fee);
            with_state_mut(|hub_state| {
                if let Some(dst_chain) = hub_state.chains.get_mut(&fee.dst_chain_id) {
                    // save fee info
                    hub_state
                        .fees
                        .entry((dst_chain.chain_id.to_string(), fee.fee_token.to_string()))
                        .and_modify(|f| *f = fee.clone())
                        .or_insert(fee.clone());

                    // build `update fee` directive for dst chain
                    hub_state
                        .dire_queue
                        .entry(dst_chain.chain_id.to_string())
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
    // validate proposal
    validate_proposal(Proposal::UpdateFee(fee.clone())).await?;
    //  build directive
    build_directive(Proposal::UpdateFee(fee)).await?;

    Ok(())
}

/// query directives for chain id filter by topic,this method will be called by route and custom
#[query(guard = "auth")]
pub async fn query_directives(
    chain_id: ChainId,
    topic: Option<Topic>,
    from: usize,
    num: usize,
) -> Result<Vec<(Seq, Directive)>, Error> {
    info!(
        "query directive for chain: {}, with topic: {:?} ",
        chain_id, topic
    );
    // let end = from + num;
    with_state(|hub_state| match hub_state.dire_queue.get(&chain_id) {
        Some(d) => {
            let mut directives: Vec<(u64, Directive)> = Vec::new();
            if let Some(topic) = topic {
                match topic {
                    Topic::AddChain(chain_type) => {
                        if let Some(dst_chain_type) = chain_type {
                            for (&seq, &ref dire) in d.iter() {
                                if let Directive::AddChain(chain_info) = dire {
                                    if dst_chain_type == chain_info.chain_type {
                                        directives.push((seq, dire.clone()));
                                    }
                                }
                            }
                        } else {
                            for (&seq, &ref dire) in d.iter() {
                                if matches!(dire, Directive::AddChain(_)) {
                                    directives.push((seq, dire.clone()));
                                }
                            }
                        }
                    }
                    Topic::AddToken(token_id) => {
                        if let Some(token) = token_id {
                            for (&seq, &ref dire) in d.iter() {
                                if let Directive::AddToken(token_meta) = dire {
                                    if token_meta.token_id.eq(&token) {
                                        directives.push((seq, dire.clone()));
                                    }
                                }
                            }
                        } else {
                            for (&seq, &ref dire) in d.iter() {
                                if matches!(dire, Directive::AddToken(_)) {
                                    directives.push((seq, dire.clone()));
                                }
                            }
                        }
                    }
                    Topic::UpdateFee(token_id) => {
                        if let Some(token) = token_id {
                            for (&seq, &ref dire) in d.iter() {
                                if let Directive::UpdateFee(fee) = dire {
                                    if fee.fee_token.eq(&token) {
                                        directives.push((seq, dire.clone()));
                                    }
                                }
                            }
                        } else {
                            for (&seq, &ref dire) in d.iter() {
                                if matches!(dire, Directive::UpdateFee(_)) {
                                    directives.push((seq, dire.clone()));
                                }
                            }
                        }
                    }
                    Topic::ActivateChain => {
                        for (&seq, &ref dire) in d.iter() {
                            if let Directive::ToggleChainState(toggle_state) = dire {
                                if toggle_state.action == StateAction::Activate {
                                    directives.push((seq, dire.clone()));
                                }
                            }
                        }
                    }
                    Topic::DeactivateChain => {
                        for (&seq, &ref dire) in d.iter() {
                            if let Directive::ToggleChainState(toggle_state) = dire {
                                if toggle_state.action == StateAction::Deactivate {
                                    directives.push((seq, dire.clone()));
                                }
                            }
                        }
                    }
                }
            } else {
                for (&seq, &ref dire) in d.iter() {
                    directives.push((seq, dire.clone()));
                }
            }

            let dires = directives.into_iter().skip(from).take(num).collect();
            info!("query directive result: {:?}", dires);
            Ok(dires)
        }
        None => {
            info!("not found directives for chain: {}", chain_id);
            Ok(Vec::new())
        }
    })
}

/// check the ticket availability
async fn check_and_update(ticket: &Ticket) -> Result<(), Error> {
    with_state_mut(|hub_state| {
        // check ticket id repetitive
        if hub_state.ticket_queue.contains_key(&ticket.ticket_id) {
            return Error::CustomError(
                format!("ticket id ({}) already exists!", ticket.ticket_id,),
            );
        }
        // check chain and state
        let _src_chain_type = match hub_state.chains.get(&ticket.src_chain) {
            Some(chain) => {
                if matches!(chain.chain_state, ChainState::Deactive) {
                    return Err(Error::CustomError(format!(
                        "The {} is deactive",
                        ticket.src_chain
                    )));
                }
                &chain.chain_type
            }
            None => return Err(Error::NotFoundChain(ticket.src_chain.to_string())),
        };

        let _dst_chain_type = match hub_state.chains.get(&ticket.dst_chain) {
            Some(chain) => {
                if matches!(chain.chain_state, ChainState::Deactive) {
                    return Err(Error::CustomError(format!(
                        "The {} is deactive",
                        ticket.dst_chain
                    )));
                }
                &chain.chain_type
            }
            None => return Err(Error::NotFoundChain(ticket.dst_chain.to_string())),
        };

        //parse ticket token amount to unsigned bigint
        let ticket_amount: u128 = ticket.amount.parse().map_err(|e: ParseIntError| {
            Error::CustomError(format!(
                "ticket amount({}) parse error: {}",
                ticket.amount,
                e.to_string()
            ))
        })?;

        // check account asset availability
        match ticket.action {
            TxAction::Transfer => {
                // ticket from issue chain
                if hub_state
                    .tokens
                    .contains_key(&(ticket.src_chain.to_string(), ticket.token.to_string()))
                {
                    info!(
                        "ticket token({}) from issue chain({}).",
                        ticket.token, ticket.src_chain,
                    );
                    // just add or increase receiver token amount
                    hub_state
                        .accounts
                        .entry(ticket.receiver.to_string())
                        .and_modify(|account_assets| {
                            account_assets
                                .entry((ticket.dst_chain.to_string(), ticket.token.to_string()))
                                .and_modify(|balance| *balance += ticket_amount)
                                .or_insert(ticket_amount);
                        })
                        .or_insert_with(|| {
                            let mut account_assets = HashMap::new();
                            account_assets.insert(
                                (ticket.dst_chain.to_string(), ticket.token.to_string()),
                                ticket_amount,
                            );
                            account_assets
                        });
                    // update token count on dst chain
                    hub_state
                        .token_position
                        .entry((ticket.dst_chain.to_string(), ticket.token.to_string()))
                        .and_modify(|total_amount| *total_amount += ticket_amount)
                        .or_insert(ticket_amount);

                    // not issue chain
                } else {
                    // reduce sender token amount
                    info!(
                        "ticket token({}) from a not issue chain({}).",
                        ticket.token, ticket.src_chain,
                    );
                    if let Some(account_assets) = hub_state.accounts.get_mut(&ticket.sender) {
                        if let Some(balance) = account_assets
                            .get_mut(&(ticket.src_chain.to_string(), ticket.token.to_string()))
                        {
                            // check account balance
                            if *balance < ticket_amount {
                                return Err(Error::CustomError(format!(
                                "Insufficient account({}) balance: sender token amount({}) <  transfer token amount({}) !)",
                                ticket.sender,balance, ticket_amount
                            )));
                            }
                            *balance -= ticket_amount;

                            // update token count on src chain
                            if let Some(total_amount) = hub_state
                                .token_position
                                .get_mut(&(ticket.src_chain.to_string(), ticket.token.to_string()))
                            {
                                *total_amount -= ticket_amount
                            } else {
                                return Err(Error::CustomError(format!(
                                    "Not found this token count info: chain({}) and token({})",
                                    ticket.src_chain, ticket.token
                                )));
                            }
                        } else {
                            return Err(Error::CustomError(format!(
                                "Not found this account({}) asset: token({}) on chain({}) ",
                                ticket.sender, ticket.token, ticket.src_chain
                            )));
                        }
                    } else {
                        return Err(Error::CustomError(format!(
                            "Not found this account: {}",
                            ticket.sender
                        )));
                    }
                    // add or increase the receiver token amount
                    hub_state
                        .accounts
                        .entry(ticket.receiver.to_string())
                        .and_modify(|account_assets| {
                            account_assets
                                .entry((ticket.dst_chain.to_string(), ticket.token.to_string()))
                                .and_modify(|balance| *balance += ticket_amount)
                                .or_insert(ticket_amount);
                        })
                        .or_insert_with(|| {
                            let mut account_assets = HashMap::new();
                            account_assets.insert(
                                (ticket.dst_chain.to_string(), ticket.token.to_string()),
                                ticket_amount,
                            );
                            account_assets
                        });
                    // update token count on dst chain
                    hub_state
                        .token_position
                        .entry((ticket.dst_chain.to_string(), ticket.token.to_string()))
                        .and_modify(|total_amount| *total_amount += ticket_amount)
                        .or_insert(ticket_amount);
                }
            }

            TxAction::Redeem => {
                // The sender account must exist and have sufficient assets
                if let Some(account_assets) = hub_state.accounts.get_mut(&ticket.sender) {
                    if let Some(balance) = account_assets
                        .get_mut(&(ticket.src_chain.to_string(), ticket.token.to_string()))
                    {
                        // check account balance
                        if *balance < ticket_amount {
                            return Err(Error::CustomError(format!(
                                "Insufficient account({}) balance: sender token amount({}) <  redeem token amount({}) !)",
                                ticket.sender,balance, ticket_amount
                            )));
                        }
                        *balance -= ticket_amount;

                        // update token count on src chain
                        if let Some(total_amount) = hub_state
                            .token_position
                            .get_mut(&(ticket.src_chain.to_string(), ticket.token.to_string()))
                        {
                            *total_amount -= ticket_amount
                        } else {
                            return Err(Error::CustomError(format!(
                                "Not found this token count info: chain({}) and token({})",
                                ticket.src_chain, ticket.token
                            )));
                        }
                    } else {
                        return Err(Error::CustomError(format!(
                            "Not found this account({}) asset: token({}) on chain({}) ",
                            ticket.sender, ticket.token, ticket.src_chain
                        )));
                    }
                } else {
                    return Err(Error::CustomError(format!(
                        "Not found this account: {}",
                        ticket.sender
                    )));
                }
                // if the dst chain is not issue chain,then update receiver asset and token count
                if !hub_state
                    .tokens
                    .contains_key(&(ticket.dst_chain.to_string(), ticket.token.to_string()))
                {
                    // the receiver must be existing
                    if let Some(account_assets) = hub_state.accounts.get_mut(&ticket.receiver) {
                        if let Some(balance) = account_assets
                            .get_mut(&(ticket.dst_chain.to_string(), ticket.token.to_string()))
                        {
                            *balance += ticket_amount;

                            // update token count on dst chain
                            if let Some(total_amount) = hub_state
                                .token_position
                                .get_mut(&(ticket.dst_chain.to_string(), ticket.token.to_string()))
                            {
                                *total_amount += ticket_amount
                            } else {
                                return Err(Error::CustomError(format!(
                                    "Not found this token count info: chain({}) and token({})",
                                    ticket.src_chain, ticket.token
                                )));
                            }
                        } else {
                            return Err(Error::CustomError(format!(
                                "Not found this account({}) asset: token({}) on chain({}) ",
                                ticket.receiver, ticket.token, ticket.dst_chain
                            )));
                        }
                    } else {
                        return Err(Error::CustomError(format!(
                            "Not found receiver account: {}",
                            ticket.receiver
                        )));
                    }
                }
            }
        }

        Ok(())
    })
}

/// check and push ticket into queue
#[update(guard = "auth")]
pub async fn send_ticket(ticket: Ticket) -> Result<(), Error> {
    info!("received ticket: {:?}", ticket);
    // checke ticket and update balance
    check_and_update(&ticket).await?;

    // build and push ticket into queue
    with_state_mut(|hub_state| {
        if let Some(chain) = hub_state.chains.get_mut(&ticket.dst_chain) {
            hub_state
                .ticket_queue
                .entry(ticket.dst_chain.to_string())
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
        //save ticket
        hub_state
            .cross_ledger
            .insert(ticket.ticket_id.to_string(), ticket.clone())
    });

    Ok(())
}

/// query tickets for chain id,this method will be called by route and custom
#[query(guard = "auth")]
pub async fn query_tickets(
    chain_id: ChainId,
    from: usize,
    num: usize,
) -> Result<Vec<(Seq, Ticket)>, Error> {
    // let end = from + num;
    with_state(|hub_state| match hub_state.ticket_queue.get(&chain_id) {
        Some(t) => {
            let mut tickets: Vec<(u64, Ticket)> = Vec::new();
            for (&seq, &ref ticket) in t.iter().skip(from).take(num) {
                tickets.push((seq, ticket.clone()));
            }

            Ok(tickets)
        }
        None => Err(Error::NotFoundChain(chain_id.to_string())),
    })
}

ic_cdk::export_candid!();

#[cfg(test)]
mod tests {

    use super::*;
    use omnity_types::{
        ChainInfo, ChainType, Fee, Proposal, StateAction, Ticket, ToggleState, TokenMeta, TxAction,
    };

    use std::time::{SystemTime, UNIX_EPOCH};
    use uuid::Uuid;

    fn get_timestamp() -> u64 {
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        since_the_epoch.as_millis() as u64
    }
    async fn build_chains() {
        let btc = ChainInfo {
            chain_id: "Bitcoin".to_string(),
            chain_type: ChainType::SettlementChain,
            chain_state: ChainState::Active,
        };

        // validate proposal
        let result = validate_proposal(Proposal::AddChain(btc.clone())).await;
        assert!(result.is_ok());
        println!("Proposal::AddChain(chain_info) result:{:?}", result);
        // build directive
        let result = build_directive(Proposal::AddChain(btc)).await;
        assert!(result.is_ok());

        let ethereum = ChainInfo {
            chain_id: "Ethereum".to_string(),
            chain_type: ChainType::SettlementChain,
            chain_state: ChainState::Active,
        };
        let result = validate_proposal(Proposal::AddChain(ethereum.clone())).await;
        assert!(result.is_ok());
        println!("Proposal::AddChain(chain_info) result:{:?}", result);
        let result = build_directive(Proposal::AddChain(ethereum)).await;
        assert!(result.is_ok());

        let icp = ChainInfo {
            chain_id: "ICP".to_string(),
            chain_type: ChainType::SettlementChain,
            chain_state: ChainState::Active,
        };
        let result = validate_proposal(Proposal::AddChain(icp.clone())).await;
        assert!(result.is_ok());
        println!("Proposal::AddChain(chain_info) result:{:?}", result);
        let result = build_directive(Proposal::AddChain(icp)).await;
        assert!(result.is_ok());

        let arbitrum = ChainInfo {
            chain_id: "Arbitrum".to_string(),
            chain_type: ChainType::ExecutionChain,
            chain_state: ChainState::Active,
        };
        let result = validate_proposal(Proposal::AddChain(arbitrum.clone())).await;
        assert!(result.is_ok());
        println!("Proposal::AddChain(chain_info) result:{:?}", result);
        let result = build_directive(Proposal::AddChain(arbitrum)).await;
        assert!(result.is_ok());

        let optimistic = ChainInfo {
            chain_id: "Optimistic".to_string(),
            chain_type: ChainType::ExecutionChain,
            chain_state: ChainState::Active,
        };

        let result = validate_proposal(Proposal::AddChain(optimistic.clone())).await;
        assert!(result.is_ok());
        println!("Proposal::AddChain(chain_info) result:{:?}", result);
        let result = build_directive(Proposal::AddChain(optimistic)).await;
        assert!(result.is_ok());

        let starknet = ChainInfo {
            chain_id: "Starknet".to_string(),
            chain_type: ChainType::ExecutionChain,
            chain_state: ChainState::Active,
        };
        let result = validate_proposal(Proposal::AddChain(starknet.clone())).await;
        assert!(result.is_ok());
        println!("Proposal::AddChain(chain_info) result:{:?}", result);
        let result = build_directive(Proposal::AddChain(starknet)).await;
        assert!(result.is_ok());
    }

    async fn build_tokens() {
        let btc = TokenMeta {
            token_id: "BTC".to_string(),
            symbol: "BTC".to_owned(),
            issue_chain: "Bitcion".to_string(),
            decimals: 18,
            icon: None,
        };
        // validate proposal
        let result = validate_proposal(Proposal::AddToken(btc.clone())).await;
        assert!(result.is_ok());
        println!("Proposal::AddToken(token) result:{:?}", result);
        // build directive
        let result = build_directive(Proposal::AddToken(btc)).await;
        assert!(result.is_ok());

        let eth = TokenMeta {
            token_id: "ETH".to_string(),
            symbol: "ETH".to_owned(),
            issue_chain: "Ethereum".to_string(),
            decimals: 18,
            icon: None,
        };
        let result = validate_proposal(Proposal::AddToken(eth.clone())).await;
        assert!(result.is_ok());
        println!("Proposal::AddToken(token) result:{:?}", result);
        let result = build_directive(Proposal::AddToken(eth)).await;
        assert!(result.is_ok());

        let icp = TokenMeta {
            token_id: "ICP".to_string(),
            symbol: "ICP".to_owned(),
            issue_chain: "ICP".to_string(),
            decimals: 18,
            icon: None,
        };
        let result = validate_proposal(Proposal::AddToken(icp.clone())).await;
        assert!(result.is_ok());
        println!("Proposal::AddToken(token) result:{:?}", result);
        let result = build_directive(Proposal::AddToken(icp)).await;
        assert!(result.is_ok());

        let arb = TokenMeta {
            token_id: "ARB".to_string(),
            symbol: "ARB".to_owned(),
            issue_chain: "Arbitrum".to_string(),
            decimals: 18,
            icon: None,
        };
        let result = validate_proposal(Proposal::AddToken(arb.clone())).await;
        assert!(result.is_ok());
        println!("Proposal::AddToken(token) result:{:?}", result);
        let result = build_directive(Proposal::AddToken(arb)).await;
        assert!(result.is_ok());

        let op = TokenMeta {
            token_id: "OP".to_string(),
            symbol: "OP".to_owned(),
            issue_chain: "Optimistic".to_string(),
            decimals: 18,
            icon: None,
        };
        let result = validate_proposal(Proposal::AddToken(op.clone())).await;
        assert!(result.is_ok());
        println!("Proposal::AddToken(token) result:{:?}", result);
        let result = build_directive(Proposal::AddToken(op)).await;
        assert!(result.is_ok());

        let starknet = TokenMeta {
            token_id: "StarkNet".to_string(),
            symbol: "StarkNet".to_owned(),
            issue_chain: "Starknet".to_string(),
            decimals: 18,
            icon: None,
        };
        let result = validate_proposal(Proposal::AddToken(starknet.clone())).await;
        assert!(result.is_ok());
        println!("Proposal::AddToken(token) result:{:?}", result);
        let result = build_directive(Proposal::AddToken(starknet)).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_add_chain() {
        // add chain
        build_chains().await;

        let chaid_ids = [
            "Bitcoin",
            "Ethereum",
            "ICP",
            "Arbitrum",
            "Optimistic",
            "Starknet",
        ];
        for chain_id in chaid_ids {
            let result =
                query_directives(chain_id.to_string(), Some(Topic::AddChain(None)), 0, 5).await;
            println!("query_directives for {:} dires: {:?}", chain_id, result);
            assert!(result.is_ok());
        }
    }

    #[tokio::test]
    async fn test_add_token() {
        // add chain
        build_chains().await;
        // add token
        build_tokens().await;

        // print all tokens
        with_state(|hs| {
            for (token_key, token) in hs.tokens.iter() {
                println!("token key: {:?}, : token meta data: {:?}", token_key, token);
            }
        });

        let chaid_ids = [
            "Bitcoin",
            "Ethereum",
            "ICP",
            "Arbitrum",
            "Optimistic",
            "Starknet",
        ];
        for chain_id in chaid_ids {
            let result =
                query_directives(chain_id.to_string(), Some(Topic::AddToken(None)), 0, 5).await;
            println!("query_directives for {:} dires: {:?}", chain_id, result);
            assert!(result.is_ok());
        }
    }

    #[tokio::test]
    async fn test_toggle_chain_state() {
        // add chain
        build_chains().await;
        // add token
        build_tokens().await;

        // change chain state
        let chain_state = ToggleState {
            chain_id: "Optimistic".to_string(),
            action: StateAction::Deactivate,
        };

        let toggle_state = Proposal::ToggleChainState(chain_state);
        let result = validate_proposal(toggle_state.clone()).await;
        assert!(result.is_ok());
        println!(
            "Proposal::ToggleChainState(chain_state) result:{:?}",
            result
        );
        let result = build_directive(toggle_state).await;
        assert!(result.is_ok());

        // query directives for chain id
        let chaid_ids = [
            "Bitcoin",
            "Ethereum",
            "ICP",
            "Arbitrum",
            "Optimistic",
            "Starknet",
        ];

        for chain_id in chaid_ids {
            let result =
                query_directives(chain_id.to_string(), Some(Topic::DeactivateChain), 0, 5).await;
            println!("query_directives for {:} dires: {:?}", chain_id, result);
            assert!(result.is_ok());
        }
    }

    #[tokio::test]
    async fn test_update_fee() {
        // add chain
        build_chains().await;
        // add token
        build_tokens().await;

        // change chain state
        let fee = Fee {
            dst_chain_id: "Arbitrum".to_string(),
            fee_token: "OP".to_string(),
            factor: 12,
        };

        // let update_fee = Proposal::UpdateFee(fee);
        // let _ = build_directive(update_fee).await;
        let result = update_fee(fee).await;
        assert!(result.is_ok());
        println!("Proposal::UpdateFee(fee) result:{:?}", result);

        // print fee info
        with_state(|hs| {
            for (fee_key, fee) in hs.fees.iter() {
                println!("fee key: {:?}, fee: {:?}", fee_key, fee);
            }
        });

        // query directives for chain id
        let chaid_ids = [
            "Bitcoin",
            "Ethereum",
            "ICP",
            "Arbitrum",
            "Optimistic",
            "Starknet",
        ];

        for chain_id in chaid_ids {
            let result =
                query_directives(chain_id.to_string(), Some(Topic::UpdateFee(None)), 0, 5).await;
            println!("query_directives for {:} dires: {:?}", chain_id, result);
            assert!(result.is_ok());
        }
    }

    #[tokio::test]
    async fn test_a_b_send_ticket() {
        // add chain
        build_chains().await;
        // add token
        build_tokens().await;
        //
        // A->B: `transfer` ticket
        let src_chain = "Bitcoin";
        let dst_chain = "Arbitrum";
        let sender = "address_on_Bitcoin";
        let receiver = "address_on_Arbitrum";

        let ticket = Ticket {
            ticket_id: Uuid::new_v4().to_string(),
            ticket_time: get_timestamp(),
            src_chain: src_chain.to_string(),
            dst_chain: dst_chain.to_string(),
            action: TxAction::Transfer,
            token: "BTC".to_string(),
            amount: 88888.to_string(),
            sender: sender.to_string(),
            receiver: receiver.to_string(),
            memo: None,
        };

        println!(" {} -> {} ticket:{:?}", src_chain, dst_chain, ticket);
        let result = send_ticket(ticket).await;
        assert!(result.is_ok());
        println!(
            "{} -> {} transfer result:{:?}",
            src_chain, dst_chain, result
        );
        // query tickets for chain id
        let result = query_tickets(dst_chain.to_string(), 0, 5).await;
        println!("query tickets for {:} tickets: {:?}", dst_chain, result);
        assert!(result.is_ok());

        // B->A: `redeem` ticket
        let src_chain = "Arbitrum";
        let dst_chain = "Bitcoin";
        let sender = "address_on_Arbitrum";
        let receiver = "address_on_Bitcoin";

        let ticket = Ticket {
            ticket_id: Uuid::new_v4().to_string(),
            ticket_time: get_timestamp(),
            src_chain: src_chain.to_string(),
            dst_chain: dst_chain.to_string(),
            action: TxAction::Redeem,
            token: "BTC".to_string(),
            amount: 88888.to_string(),
            sender: sender.to_string(),
            receiver: receiver.to_string(),
            memo: None,
        };

        println!(" {} -> {} ticket:{:?}", src_chain, dst_chain, ticket);
        let result = send_ticket(ticket).await;
        assert!(result.is_ok());
        println!("{} -> {} redeem result:{:?}", src_chain, dst_chain, result);

        // query tickets for chain id
        let result = query_tickets(dst_chain.to_string(), 0, 5).await;
        assert!(result.is_ok());
        println!("query tickets for {:} tickets: {:?}", dst_chain, result);
    }

    #[tokio::test]
    async fn test_a_b_c_send_ticket() {
        // add chain
        build_chains().await;
        // add token
        build_tokens().await;

        // transfer
        // A->B: `transfer` ticket
        let src_chain = "Ethereum";
        let dst_chain = "Optimistic";
        let sender = "address_on_Ethereum";
        let receiver = "address_on_Optimistic";

        let ticket = Ticket {
            ticket_id: Uuid::new_v4().to_string(),
            ticket_time: get_timestamp(),
            src_chain: src_chain.to_string(),
            dst_chain: dst_chain.to_string(),
            action: TxAction::Transfer,
            token: "ETH".to_string(),
            amount: 6666.to_string(),
            sender: sender.to_string(),
            receiver: receiver.to_string(),
            memo: None,
        };

        println!(" {} -> {} ticket:{:?}", src_chain, dst_chain, ticket);
        let result = send_ticket(ticket).await;
        assert!(result.is_ok());
        println!(
            "{} -> {} transfer result:{:?}",
            src_chain, dst_chain, result
        );
        // query tickets for chain id
        let result = query_tickets(dst_chain.to_string(), 0, 5).await;
        assert!(result.is_ok());
        println!("query tickets for {:} tickets: {:?}", dst_chain, result);

        // B->C: `transfer` ticket
        let sender = "address_on_Optimistic";
        let receiver = "address_on_Starknet";
        let src_chain = "Optimistic";
        let dst_chain = "Starknet";

        let ticket = Ticket {
            ticket_id: Uuid::new_v4().to_string(),
            ticket_time: get_timestamp(),
            src_chain: src_chain.to_string(),
            dst_chain: dst_chain.to_string(),
            action: TxAction::Transfer,
            token: "ETH".to_string(),
            amount: 6666.to_string(),
            sender: sender.to_string(),
            receiver: receiver.to_string(),
            memo: None,
        };

        println!(" {} -> {} ticket:{:?}", src_chain, dst_chain, ticket);
        assert!(result.is_ok());
        let result = send_ticket(ticket).await;
        println!(
            "{} -> {} transfer result:{:?}",
            src_chain, dst_chain, result
        );

        // query tickets for chain id
        let result = query_tickets(dst_chain.to_string(), 0, 5).await;
        assert!(result.is_ok());
        println!("query tickets for {:} tickets: {:?}", dst_chain, result);

        // redeem
        // C->B: `redeem` ticket
        let src_chain = "Starknet";
        let dst_chain = "Optimistic";
        let sender = "address_on_Starknet";
        let receiver = "address_on_Optimistic";

        let ticket = Ticket {
            ticket_id: Uuid::new_v4().to_string(),
            ticket_time: get_timestamp(),
            src_chain: src_chain.to_string(),
            dst_chain: dst_chain.to_string(),
            action: TxAction::Redeem,
            token: "ETH".to_string(),
            amount: 6666.to_string(),
            sender: sender.to_string(),
            receiver: receiver.to_string(),
            memo: None,
        };

        println!(" {} -> {} ticket:{:?}", src_chain, dst_chain, ticket);
        let result = send_ticket(ticket).await;
        assert!(result.is_ok());
        println!("{} -> {} redeem result:{:?}", src_chain, dst_chain, result);
        // query tickets for chain id
        let result = query_tickets(dst_chain.to_string(), 0, 5).await;
        assert!(result.is_ok());
        println!("query tickets for {:} tickets: {:?}", dst_chain, result);

        // B->A: `redeem` ticket
        let sender = "address_on_Optimistic";
        let receiver = "address_on_Ethereum";
        let src_chain = "Optimistic";
        let dst_chain = "Ethereum";

        let ticket = Ticket {
            ticket_id: Uuid::new_v4().to_string(),
            ticket_time: get_timestamp(),
            src_chain: src_chain.to_string(),
            dst_chain: dst_chain.to_string(),
            action: TxAction::Redeem,
            token: "ETH".to_string(),
            amount: 6666.to_string(),
            sender: sender.to_string(),
            receiver: receiver.to_string(),
            memo: None,
        };

        println!(" {} -> {} ticket:{:?}", src_chain, dst_chain, ticket);
        assert!(result.is_ok());
        let result = send_ticket(ticket).await;
        println!("{} -> {} redeem result:{:?}", src_chain, dst_chain, result);

        // query tickets for chain id
        let result = query_tickets(dst_chain.to_string(), 0, 5).await;
        assert!(result.is_ok());
        println!("query tickets for {:} tickets: {:?}", dst_chain, result);
    }
}
