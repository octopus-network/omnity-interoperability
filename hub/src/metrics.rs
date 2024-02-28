use std::collections::{HashMap, HashSet};

use ic_cdk::query;

use omnity_types::{
    ChainCondition, ChainId, ChainInfo, ChainState, ChainType, DireQueue, Directive, Error, Fee,
    LockedToken, Proposal, Seq, StateAction, Ticket, TicketId, TicketQueue, TokenCondition,
    TokenId, TokenMetaData, Topic, TxAction, TxCondition,
};

use crate::{with_state, ChainInfoWithSeq, HubState};

#[query]
pub async fn get_chain_list(
    condition: ChainCondition,
    from: usize,
    num: usize,
) -> Result<Vec<ChainInfo>, Error> {
    // use hashset to keep unique
    let mut chain_set = HashSet::new();
    with_state(|hub_state| {
        if let Some(chain_type) = condition.chain_type {
            for (_, chain) in hub_state
                .chains
                .iter()
                .filter(|(_, chain)| chain.chain_type == chain_type)
            {
                let chain_info = ChainInfo {
                    chain_name: chain.chain_name.clone(),
                    chain_type: chain.chain_type.clone(),
                    chain_state: chain.chain_state.clone(),
                };
                chain_set.insert(chain_info);
            }
        } else if let Some(chain_state) = condition.chain_state {
            for (_, chain) in hub_state
                .chains
                .iter()
                .filter(|(_, chain)| chain_state == chain.chain_state)
            {
                let chain_info = ChainInfo {
                    chain_name: chain.chain_name.clone(),
                    chain_type: chain.chain_type.clone(),
                    chain_state: chain.chain_state.clone(),
                };
                chain_set.insert(chain_info);
            }
        } else {
            for (_, chain) in hub_state.chains.iter() {
                let chain_info = ChainInfo {
                    chain_name: chain.chain_name.clone(),
                    chain_type: chain.chain_type.clone(),
                    chain_state: chain.chain_state.clone(),
                };
                chain_set.insert(chain_info);
            }
        }
    });
    // take value from to end (from + num )
    let chains = chain_set.into_iter().skip(from).take(num).collect();
    Ok(chains)
}

#[query]
pub async fn get_chain(chain_id: String) -> Result<Option<ChainInfo>, Error> {
    with_state(|hub_state| {
        if let Some(chain) = hub_state.chains.get(&chain_id) {
            let chain_info = ChainInfo {
                chain_name: chain.chain_name.clone(),
                chain_type: chain.chain_type.clone(),
                chain_state: chain.chain_state.clone(),
            };
            Ok(Some(chain_info))
        } else {
            Ok(None)
        }
    })
}
#[query]
pub async fn get_chain_by_type(
    chain_type: ChainType,
    from: usize,
    num: usize,
) -> Result<Vec<ChainInfo>, Error> {
    let mut chains = Vec::new();
    with_state(|hub_state| {
        for (_, chain) in hub_state
            .chains
            .iter()
            .filter(|(_, chain)| chain_type == chain.chain_type)
            .skip(from)
            .take(num)
        {
            let chain_info = ChainInfo {
                chain_name: chain.chain_name.clone(),
                chain_type: chain.chain_type.clone(),
                chain_state: chain.chain_state.clone(),
            };
            chains.push(chain_info)
        }
    });

    Ok(chains)
}

#[query]
pub async fn get_chain_by_state(
    chain_state: ChainState,
    from: usize,
    num: usize,
) -> Result<Vec<ChainInfo>, Error> {
    let mut chains = Vec::new();
    with_state(|hub_state| {
        for (_, chain) in hub_state
            .chains
            .iter()
            .filter(|(_, chain)| chain_state == chain.chain_state)
            .skip(from)
            .take(num)
        {
            let chain_info = ChainInfo {
                chain_name: chain.chain_name.clone(),
                chain_type: chain.chain_type.clone(),
                chain_state: chain.chain_state.clone(),
            };
            chains.push(chain_info)
        }
    });

    Ok(chains)
}

#[query]
pub async fn get_token_list(
    token_id: Option<TokenId>,
    chain_id: Option<ChainId>,
    from: usize,
    num: usize,
) -> Result<Vec<TokenMetaData>, Error> {
    let mut token_set = HashSet::new();
    let _ = with_state(|hub_state| {
        if let Some(token_id) = token_id {
            for (_, token) in hub_state
                .tokens
                .iter()
                .filter(|(_, token)| token.name == token_id)
            {
                token_set.insert(token.clone());
            }
        } else if let Some(chain_id) = chain_id {
            for (_, token) in hub_state
                .tokens
                .iter()
                .filter(|(_, token)| token.issue_chain.eq(&chain_id))
            {
                token_set.insert(token.clone());
            }
        } else {
            for (_, token) in hub_state.tokens.iter() {
                token_set.insert(token.clone());
            }
        }
    });
    // take value from to end (from + num )
    let tokens: Vec<TokenMetaData> = token_set.into_iter().skip(from).take(num).collect();
    Ok(tokens)
}

#[query]
pub async fn get_locked_tokens(
    condition: TokenCondition,
    from: usize,
    num: usize,
) -> Result<Vec<LockedToken>, Error> {
    let mut locked_token_set = HashSet::new();
    with_state(|hub_state| {
        if let Some(token_id) = condition.token_id {
            if let Some(locked_token_map) = hub_state.locked_tokens.get(&token_id) {
                for (chain_id, locked_amount) in locked_token_map.iter() {
                    //TODO: handle the erorr result
                    let chain_type = get_chain_type(&chain_id).unwrap();
                    let locked_token = LockedToken {
                        token_id: token_id.to_string(),
                        total_locked_amount: *locked_amount,
                        chain_id: chain_id.to_string(),
                        chain_type,
                    };
                    locked_token_set.insert(locked_token);
                }
            }
        } else if let Some(chain_id) = condition.chain_id {
            for (token_id, locked_token_map) in hub_state
                .locked_tokens
                .iter()
                .filter(|(_, locked_token_map)| locked_token_map.contains_key(&chain_id))
            {
                for (chain_id, locked_amount) in locked_token_map.iter() {
                    //TODO: handle the erorr result
                    let chain_type = get_chain_type(&chain_id).unwrap();
                    let locked_token = LockedToken {
                        token_id: token_id.to_string(),
                        total_locked_amount: *locked_amount,
                        chain_id: chain_id.to_string(),
                        chain_type,
                    };
                    locked_token_set.insert(locked_token);
                }
            }
        } else if let Some(dst_chain_type) = condition.chain_type {
            for (token_id, locked_token_map) in hub_state.locked_tokens.iter() {
                for (chain_id, locked_amount) in locked_token_map.iter() {
                    //TODO: handle the erorr result
                    let chain_type = get_chain_type(chain_id).unwrap();
                    if chain_type == dst_chain_type {
                        let locked_token = LockedToken {
                            token_id: token_id.to_string(),
                            total_locked_amount: *locked_amount,
                            chain_id: chain_id.to_string(),
                            chain_type,
                        };
                        locked_token_set.insert(locked_token);
                    }
                }
            }
        } else {
            for (token_id, locked_token_map) in hub_state.locked_tokens.iter() {
                for (chain_id, locked_amount) in locked_token_map.iter() {
                    //TODO: handle the erorr result
                    let chain_type = get_chain_type(chain_id).unwrap();
                    let locked_token = LockedToken {
                        token_id: token_id.to_string(),
                        total_locked_amount: *locked_amount,
                        chain_id: chain_id.to_string(),
                        chain_type,
                    };
                    locked_token_set.insert(locked_token);
                }
            }
        }
    });
    // take value from to end (from + num )
    let lock_tokens = locked_token_set.into_iter().skip(from).take(num).collect();
    Ok(lock_tokens)
}

#[query]
pub async fn get_tx(ticket_id: TicketId) -> Result<Option<Ticket>, Error> {
    with_state(|hub_state| {
        if let Some(ticket) = hub_state.cross_ledger.transfers.get(&ticket_id) {
            Ok(Some(ticket.clone()))
        } else if let Some(ticket) = hub_state.cross_ledger.redeems.get(&ticket_id) {
            Ok(Some(ticket.clone()))
        } else {
            Ok(None)
        }
    })
}

#[query]
pub async fn get_tx_list(
    condition: TxCondition,
    from: usize,
    num: usize,
) -> Result<Vec<Ticket>, Error> {
    let mut ticket_set = HashSet::new();
    with_state(|hub_state| {
        if let Some(src_chain) = condition.src_chain {
            for (ticket_id, ticket) in hub_state
                .cross_ledger
                .transfers
                .iter()
                .filter(|(ticket_id, ticket)| ticket.src_chain.eq(&src_chain))
            {
                ticket_set.insert(ticket.clone());
            }
            for (ticket_id, ticket) in hub_state
                .cross_ledger
                .redeems
                .iter()
                .filter(|(ticket_id, ticket)| ticket.src_chain.eq(&src_chain))
            {
                ticket_set.insert(ticket.clone());
            }
        } else if let Some(dst_chain) = condition.dst_chain {
            for (ticket_id, ticket) in hub_state
                .cross_ledger
                .transfers
                .iter()
                .filter(|(ticket_id, ticket)| ticket.dst_chain.eq(&dst_chain))
            {
                ticket_set.insert(ticket.clone());
            }
            for (ticket_id, ticket) in hub_state
                .cross_ledger
                .redeems
                .iter()
                .filter(|(ticket_id, ticket)| ticket.dst_chain.eq(&dst_chain))
            {
                ticket_set.insert(ticket.clone());
            }
        } else if let Some(token_id) = condition.token_id {
            for (ticket_id, ticket) in hub_state
                .cross_ledger
                .transfers
                .iter()
                .filter(|(ticket_id, ticket)| ticket.token.eq(&token_id))
            {
                ticket_set.insert(ticket.clone());
            }
            for (ticket_id, ticket) in hub_state
                .cross_ledger
                .redeems
                .iter()
                .filter(|(ticket_id, ticket)| ticket.token.eq(&token_id))
            {
                ticket_set.insert(ticket.clone());
            }
        } else if let Some(time_range) = condition.time_range {
            //TODO: aseet end time >= start time
            for (ticket_id, ticket) in
                hub_state
                    .cross_ledger
                    .transfers
                    .iter()
                    .filter(|(ticket_id, ticket)| {
                        ticket.created_time >= time_range.0 && ticket.created_time <= time_range.0
                    })
            {
                ticket_set.insert(ticket.clone());
            }
            for (ticket_id, ticket) in
                hub_state
                    .cross_ledger
                    .redeems
                    .iter()
                    .filter(|(ticket_id, ticket)| {
                        ticket.created_time >= time_range.0 && ticket.created_time <= time_range.0
                    })
            {
                ticket_set.insert(ticket.clone());
            }
        } else {
            for (ticket_id, ticket) in hub_state.cross_ledger.transfers.iter() {
                ticket_set.insert(ticket.clone());
            }
            for (ticket_id, ticket) in hub_state.cross_ledger.redeems.iter() {
                ticket_set.insert(ticket.clone());
            }
        }
    });
    // take value from to end (from + num )
    let tickets = ticket_set.into_iter().skip(from).take(num).collect();
    Ok(tickets)
}

#[query]
pub async fn get_total_tx() -> Result<u64, Error> {
    with_state(|hub_state| {
        let total_num = hub_state.cross_ledger.transfers.len() as u64
            + hub_state.cross_ledger.redeems.len() as u64;
        Ok(total_num)
    })
}

pub fn get_chain_type(chain_id: &ChainId) -> Result<ChainType, Error> {
    with_state(|hub_state| {
        if let Some(chain) = hub_state.chains.get(chain_id) {
            Ok(chain.chain_type.clone())
        } else {
            Err(Error::CustomError(format!(
                "The {} is not exists",
                chain_id
            )))
        }
    })
}
