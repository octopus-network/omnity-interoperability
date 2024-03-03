use std::collections::HashSet;

use ic_cdk::query;

use omnity_types::{
    ChainCondition, ChainId, ChainInfo, ChainType, Error, Ticket, TicketId, TokenCondition,
    TokenId, TokenMeta, TokenOnChain, TxCondition,
};

use crate::with_state;

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
                    chain_id: chain.chain_id.clone(),
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
                    chain_id: chain.chain_id.clone(),
                    chain_type: chain.chain_type.clone(),
                    chain_state: chain.chain_state.clone(),
                };
                chain_set.insert(chain_info);
            }
        } else {
            //TODO: may be take range at this
            for (_, chain) in hub_state.chains.iter() {
                let chain_info = ChainInfo {
                    chain_id: chain.chain_id.clone(),
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
pub async fn get_chain(chain_id: String) -> Result<ChainInfo, Error> {
    with_state(|hub_state| {
        if let Some(chain) = hub_state.chains.get(&chain_id) {
            let chain_info = ChainInfo {
                chain_id: chain.chain_id.clone(),
                chain_type: chain.chain_type.clone(),
                chain_state: chain.chain_state.clone(),
            };
            Ok(chain_info)
        } else {
            Err(Error::NotFoundChain(chain_id))
        }
    })
}

#[query]
pub async fn get_token_list(
    token_id: Option<TokenId>,
    chain_id: Option<ChainId>,
    from: usize,
    num: usize,
) -> Result<Vec<TokenMeta>, Error> {
    let mut token_set = HashSet::new();
    let _ = with_state(|hub_state| {
        if let Some(token_id) = token_id {
            for (_, token) in hub_state
                .tokens
                .iter()
                .filter(|(_, token)| token.token_id == token_id)
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
            //TODO: may be take range at this
            for (_, token) in hub_state.tokens.iter() {
                token_set.insert(token.clone());
            }
        }
    });
    // take value from to end (from + num )
    let tokens: Vec<TokenMeta> = token_set.into_iter().skip(from).take(num).collect();
    Ok(tokens)
}

#[query]
pub async fn get_tx(ticket_id: TicketId) -> Result<Ticket, Error> {
    with_state(|hub_state| {
        if let Some(ticket) = hub_state.cross_ledger.get(&ticket_id) {
            Ok(ticket.clone())
        } else {
            Err(Error::CustomError(format!(
                "Not found this ticket: {}",
                ticket_id
            )))
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
            for (_ticket_id, ticket) in hub_state
                .cross_ledger
                .iter()
                .filter(|(_ticket_id, ticket)| ticket.src_chain.eq(&src_chain))
            {
                ticket_set.insert(ticket.clone());
            }
        } else if let Some(dst_chain) = condition.dst_chain {
            for (_ticket_id, ticket) in hub_state
                .cross_ledger
                .iter()
                .filter(|(_ticket_id, ticket)| ticket.dst_chain.eq(&dst_chain))
            {
                ticket_set.insert(ticket.clone());
            }
        } else if let Some(token_id) = condition.token_id {
            for (_ticket_id, ticket) in hub_state
                .cross_ledger
                .iter()
                .filter(|(_ticket_id, ticket)| ticket.token.eq(&token_id))
            {
                ticket_set.insert(ticket.clone());
            }
        } else if let Some(time_range) = condition.time_range {
            //TODO: aseet end time >= start time
            for (_ticket_id, ticket) in
                hub_state
                    .cross_ledger
                    .iter()
                    .filter(|(_ticket_id, ticket)| {
                        ticket.ticket_time >= time_range.0 && ticket.ticket_time <= time_range.0
                    })
            {
                ticket_set.insert(ticket.clone());
            }
        } else {
            //TODO: may be take range at this
            for (_ticket_id, ticket) in hub_state.cross_ledger.iter() {
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
        let total_num = hub_state.cross_ledger.len() as u64;
        Ok(total_num)
    })
}

/// get tokens on execution chain
#[query]
pub async fn get_chain_tokens(
    condition: TokenCondition,
    from: usize,
    num: usize,
) -> Result<Vec<TokenOnChain>, Error> {
    let mut chain_token_set = HashSet::new();
    with_state(|hub_state| {
        if let Some(dst_token_id) = condition.token_id {
            for ((chain_id, token_id), total_amount) in hub_state
                .token_position
                .iter()
                .filter(|((_chain_id, token_id), _total_amount)| token_id.eq(&dst_token_id))
            {
                let chain_token = TokenOnChain {
                    token_id: token_id.to_string(),
                    amount: *total_amount,
                    chain_id: chain_id.to_string(),
                };
                chain_token_set.insert(chain_token);
            }
        } else if let Some(dst_chain_id) = condition.chain_id {
            for ((chain_id, token_id), total_amount) in hub_state
                .token_position
                .iter()
                .filter(|((chain_id, _token_id), _total_amount)| chain_id.eq(&dst_chain_id))
            {
                let chain_token = TokenOnChain {
                    token_id: token_id.to_string(),
                    amount: *total_amount,
                    chain_id: chain_id.to_string(),
                };
                chain_token_set.insert(chain_token);
            }
        } else {
            //TODO: take range here?
            for ((chain_id, token_id), total_amount) in hub_state.token_position.iter() {
                let chain_token = TokenOnChain {
                    token_id: token_id.to_string(),
                    amount: *total_amount,
                    chain_id: chain_id.to_string(),
                };
                chain_token_set.insert(chain_token);
            }
        }
    });
    // take value from to end (from + num )
    let chain_tokens = chain_token_set.into_iter().skip(from).take(num).collect();
    Ok(chain_tokens)
}

#[query]
pub async fn get_chain_type(chain_id: ChainId) -> Result<ChainType, Error> {
    with_state(|hub_state| {
        if let Some(chain) = hub_state.chains.get(&chain_id) {
            Ok(chain.chain_type.clone())
        } else {
            Err(Error::NotFoundChain(chain_id))
        }
    })
}
