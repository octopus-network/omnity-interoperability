use ic_cdk::query;

use log::info;
use omnity_types::{
    Chain, ChainId, ChainState, ChainType, Error, Fee, Ticket, TicketId, Token, TokenId,
    TokenOnChain,
};

use crate::with_state;

#[query]
pub async fn get_chains(
    chain_type: Option<ChainType>,
    chain_state: Option<ChainState>,
    from: usize,
    offset: usize,
) -> Result<Vec<Chain>, Error> {
    let condition = (chain_type, chain_state);
    info!(
        "get_chains condition: {:?}, from: {}, offset: {}",
        condition, from, offset
    );
    match condition {
        (None, None) => Ok(with_state(|hub_state| {
            hub_state
                .chains
                .iter()
                .skip(from)
                .take(offset)
                .map(|(_, chain)| chain.clone().into())
                .collect()
        })),

        (None, Some(dst_chain_state)) => Ok(with_state(|hub_state| {
            hub_state
                .chains
                .iter()
                .filter(|(_, chain)| chain.chain_state == dst_chain_state)
                .skip(from)
                .take(offset)
                .map(|(_, chain)| chain.clone().into())
                .collect()
        })),

        (Some(dst_chain_type), None) => Ok(with_state(|hub_state| {
            hub_state
                .chains
                .iter()
                .filter(|(_, chain)| chain.chain_type == dst_chain_type)
                .skip(from)
                .take(offset)
                .map(|(_, chain)| chain.clone().into())
                .collect()
        })),

        (Some(dst_chain_type), Some(dst_chain_state)) => Ok(with_state(|hub_state| {
            hub_state
                .chains
                .iter()
                .filter(|(_, chain)| {
                    chain.chain_type == dst_chain_type && chain.chain_state == dst_chain_state
                })
                .skip(from)
                .take(offset)
                .map(|(_, chain)| chain.clone().into())
                .collect()
        })),
    }
}

#[query]
pub async fn get_chain(chain_id: String) -> Result<Chain, Error> {
    info!("get_chain chain_id: {:?} ", chain_id);
    with_state(|hub_state| {
        if let Some(chain) = hub_state.chains.get(&chain_id) {
            Ok(chain.clone().into())
        } else {
            Err(Error::NotFoundChain(chain_id))
        }
    })
}

#[query]
pub async fn get_tokens(
    chain_id: Option<ChainId>,
    token_id: Option<TokenId>,
    from: usize,
    offset: usize,
) -> Result<Vec<Token>, Error> {
    let condition = (chain_id, token_id);
    info!(
        "get_tokens condition: {:?}, from: {}, offset: {}",
        condition, from, offset
    );
    match condition {
        (None, None) => Ok(with_state(|hub_state| {
            hub_state
                .tokens
                .iter()
                .skip(from)
                .take(offset)
                .map(|(_, token)| token.clone().into())
                .collect()
        })),
        (None, Some(dst_token_id)) => Ok(with_state(|hub_state| {
            hub_state
                .tokens
                .iter()
                .filter(|(_, token)| token.token_id.eq(&dst_token_id))
                .skip(from)
                .take(offset)
                .map(|(_, token)| token.clone().into())
                .collect()
        })),
        (Some(dst_chain_id), None) => Ok(with_state(|hub_state| {
            hub_state
                .tokens
                .iter()
                .filter(|(_, token)| token.issue_chain.eq(&dst_chain_id))
                .skip(from)
                .take(offset)
                .map(|(_, token)| token.clone().into())
                .collect()
        })),
        (Some(dst_chain_id), Some(dst_token_id)) => Ok(with_state(|hub_state| {
            hub_state
                .tokens
                .iter()
                .filter(|(_, token)| {
                    token.issue_chain.eq(&dst_chain_id) && token.token_id.eq(&dst_token_id)
                })
                .skip(from)
                .take(offset)
                .map(|(_, token)| token.clone().into())
                .collect()
        })),
    }
}

/// get fees
#[query]
pub async fn get_fees(
    chain_id: Option<ChainId>,
    token_id: Option<TokenId>,
    from: usize,
    offset: usize,
) -> Result<Vec<Fee>, Error> {
    let condition = (chain_id, token_id);
    info!(
        "get_fees condition: {:?}, from: {}, offset: {}",
        condition, from, offset
    );
    match condition {
        (None, None) => Ok(with_state(|hub_state| {
            hub_state
                .fees
                .iter()
                .skip(from)
                .take(offset)
                .map(|((_, _), fee)| fee.clone())
                .collect()
        })),
        (None, Some(dst_token_id)) => Ok(with_state(|hub_state| {
            hub_state
                .fees
                .iter()
                .filter(|((_, token_id), _)| token_id.eq(&dst_token_id))
                .skip(from)
                .take(offset)
                .map(|(_, fee)| fee.clone())
                .collect()
        })),
        (Some(dst_chain_id), None) => Ok(with_state(|hub_state| {
            hub_state
                .fees
                .iter()
                .filter(|((chain_id, _), _)| chain_id.eq(&dst_chain_id))
                .skip(from)
                .take(offset)
                .map(|((_, _), fee)| fee.clone())
                .collect()
        })),
        (Some(dst_chain_id), Some(dst_token_id)) => Ok(with_state(|hub_state| {
            hub_state
                .fees
                .iter()
                .filter(|((chain_id, token_id), _)| {
                    chain_id.eq(&dst_chain_id) && token_id.eq(&dst_token_id)
                })
                .skip(from)
                .take(offset)
                .map(|((_, _), fee)| fee.clone())
                .collect()
        })),
    }
}

/// get tokens on dst chain
#[query]
pub async fn get_chain_tokens(
    chain_id: Option<ChainId>,
    token_id: Option<TokenId>,
    from: usize,
    offset: usize,
) -> Result<Vec<TokenOnChain>, Error> {
    let condition = (chain_id, token_id);
    info!(
        "get_chain_tokens condition: {:?}, from: {}, offset: {}",
        condition, from, offset
    );
    match condition {
        (None, None) => Ok(with_state(|hub_state| {
            hub_state
                .token_position
                .iter()
                .skip(from)
                .take(offset)
                .map(|((chain_id, token_id), total_amount)| TokenOnChain {
                    chain_id: chain_id.to_string(),
                    token_id: token_id.to_string(),
                    amount: *total_amount,
                })
                .collect()
        })),
        (None, Some(dst_token_id)) => Ok(with_state(|hub_state| {
            hub_state
                .token_position
                .iter()
                .filter(|((_chain_id, token_id), _total_amount)| token_id.eq(&dst_token_id))
                .skip(from)
                .take(offset)
                .map(|((chain_id, token_id), total_amount)| TokenOnChain {
                    chain_id: chain_id.to_string(),
                    token_id: token_id.to_string(),
                    amount: *total_amount,
                })
                .collect()
        })),
        (Some(dst_chain_id), None) => Ok(with_state(|hub_state| {
            hub_state
                .token_position
                .iter()
                .filter(|((chain_id, _token_id), _total_amount)| chain_id.eq(&dst_chain_id))
                .skip(from)
                .take(offset)
                .map(|((chain_id, token_id), total_amount)| TokenOnChain {
                    chain_id: chain_id.to_string(),
                    token_id: token_id.to_string(),
                    amount: *total_amount,
                })
                .collect()
        })),
        (Some(dst_chain_id), Some(dst_token_id)) => Ok(with_state(|hub_state| {
            hub_state
                .token_position
                .iter()
                .filter(|((chain_id, token_id), _total_amount)| {
                    chain_id.eq(&dst_chain_id) && token_id.eq(&dst_token_id)
                })
                .skip(from)
                .take(offset)
                .map(|((chain_id, token_id), total_amount)| TokenOnChain {
                    chain_id: chain_id.to_string(),
                    token_id: token_id.to_string(),
                    amount: *total_amount,
                })
                .collect()
        })),
    }
}

#[query]
pub async fn get_tx(ticket_id: TicketId) -> Result<Ticket, Error> {
    info!("get_tx ticket_id: {:?} ", ticket_id);
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
pub async fn get_txs(
    src_chain: Option<ChainId>,
    dst_chain: Option<ChainId>,
    token_id: Option<TokenId>,
    // time range: from .. end
    time_range: Option<(u64, u64)>,
    from: usize,
    offset: usize,
) -> Result<Vec<Ticket>, Error> {
    let condition = (src_chain, dst_chain, token_id, time_range);
    info!(
        "get_txs condition: {:?}, from: {}, offset: {}",
        condition, from, offset
    );
    match condition {
        (None, None, None, None) => Ok(with_state(|hub_state| {
            hub_state
                .cross_ledger
                .iter()
                .skip(from)
                .take(offset)
                .map(|(_, ticket)| ticket.clone())
                .collect()
        })),
        (None, None, None, Some(time_range)) => Ok(with_state(|hub_state| {
            hub_state
                .cross_ledger
                .iter()
                .filter(|(_ticket_id, ticket)| {
                    ticket.ticket_time >= time_range.0 && ticket.ticket_time <= time_range.0
                })
                .skip(from)
                .take(offset)
                .map(|(_, ticket)| ticket.clone())
                .collect()
        })),
        (None, None, Some(token_id), None) => Ok(with_state(|hub_state| {
            hub_state
                .cross_ledger
                .iter()
                .filter(|(_ticket_id, ticket)| ticket.token.eq(&token_id))
                .skip(from)
                .take(offset)
                .map(|(_, ticket)| ticket.clone())
                .collect()
        })),
        (None, None, Some(token_id), Some(time_range)) => Ok(with_state(|hub_state| {
            hub_state
                .cross_ledger
                .iter()
                .filter(|(_ticket_id, ticket)| {
                    ticket.token.eq(&token_id)
                        && (ticket.ticket_time >= time_range.0
                            && ticket.ticket_time <= time_range.0)
                })
                .skip(from)
                .take(offset)
                .map(|(_, ticket)| ticket.clone())
                .collect()
        })),
        (None, Some(dst_chain), None, None) => Ok(with_state(|hub_state| {
            hub_state
                .cross_ledger
                .iter()
                .filter(|(_ticket_id, ticket)| ticket.dst_chain.eq(&dst_chain))
                .skip(from)
                .take(offset)
                .map(|(_, ticket)| ticket.clone())
                .collect()
        })),
        (None, Some(dst_chain), None, Some(time_range)) => Ok(with_state(|hub_state| {
            hub_state
                .cross_ledger
                .iter()
                .filter(|(_ticket_id, ticket)| {
                    ticket.dst_chain.eq(&dst_chain)
                        && (ticket.ticket_time >= time_range.0
                            && ticket.ticket_time <= time_range.0)
                })
                .skip(from)
                .take(offset)
                .map(|(_, ticket)| ticket.clone())
                .collect()
        })),
        (None, Some(dst_chain), Some(token_id), None) => Ok(with_state(|hub_state| {
            hub_state
                .cross_ledger
                .iter()
                .filter(|(_ticket_id, ticket)| {
                    ticket.dst_chain.eq(&dst_chain) && ticket.token.eq(&token_id)
                })
                .skip(from)
                .take(offset)
                .map(|(_, ticket)| ticket.clone())
                .collect()
        })),
        (None, Some(dst_chain), Some(token_id), Some(time_range)) => Ok(with_state(|hub_state| {
            hub_state
                .cross_ledger
                .iter()
                .filter(|(_ticket_id, ticket)| {
                    ticket.dst_chain.eq(&dst_chain)
                        && ticket.token.eq(&token_id)
                        && (ticket.ticket_time >= time_range.0
                            && ticket.ticket_time <= time_range.0)
                })
                .skip(from)
                .take(offset)
                .map(|(_, ticket)| ticket.clone())
                .collect()
        })),
        (Some(src_chain), None, None, None) => Ok(with_state(|hub_state| {
            hub_state
                .cross_ledger
                .iter()
                .filter(|(_ticket_id, ticket)| ticket.src_chain.eq(&src_chain))
                .skip(from)
                .take(offset)
                .map(|(_, ticket)| ticket.clone())
                .collect()
        })),
        (Some(src_chain), None, None, Some(time_range)) => Ok(with_state(|hub_state| {
            hub_state
                .cross_ledger
                .iter()
                .filter(|(_ticket_id, ticket)| {
                    ticket.src_chain.eq(&src_chain)
                        && (ticket.ticket_time >= time_range.0
                            && ticket.ticket_time <= time_range.0)
                })
                .skip(from)
                .take(offset)
                .map(|(_, ticket)| ticket.clone())
                .collect()
        })),
        (Some(src_chain), None, Some(token_id), None) => Ok(with_state(|hub_state| {
            hub_state
                .cross_ledger
                .iter()
                .filter(|(_ticket_id, ticket)| {
                    ticket.src_chain.eq(&src_chain) && ticket.token.eq(&token_id)
                })
                .skip(from)
                .take(offset)
                .map(|(_, ticket)| ticket.clone())
                .collect()
        })),
        (Some(src_chain), None, Some(token_id), Some(time_range)) => Ok(with_state(|hub_state| {
            hub_state
                .cross_ledger
                .iter()
                .filter(|(_ticket_id, ticket)| {
                    ticket.src_chain.eq(&src_chain)
                        && ticket.token.eq(&token_id)
                        && (ticket.ticket_time >= time_range.0
                            && ticket.ticket_time <= time_range.0)
                })
                .skip(from)
                .take(offset)
                .map(|(_, ticket)| ticket.clone())
                .collect()
        })),
        (Some(src_chain), Some(dst_chain), None, None) => {
            Ok(with_state(|hub_state: &crate::HubState| {
                hub_state
                    .cross_ledger
                    .iter()
                    .filter(|(_ticket_id, ticket)| {
                        ticket.src_chain.eq(&src_chain) && ticket.dst_chain.eq(&dst_chain)
                    })
                    .skip(from)
                    .take(offset)
                    .map(|(_, ticket)| ticket.clone())
                    .collect()
            }))
        }
        (Some(src_chain), Some(dst_chain), None, Some(time_range)) => {
            Ok(with_state(|hub_state: &crate::HubState| {
                hub_state
                    .cross_ledger
                    .iter()
                    .filter(|(_ticket_id, ticket)| {
                        ticket.src_chain.eq(&src_chain)
                            && ticket.dst_chain.eq(&dst_chain)
                            && (ticket.ticket_time >= time_range.0
                                && ticket.ticket_time <= time_range.0)
                    })
                    .skip(from)
                    .take(offset)
                    .map(|(_, ticket)| ticket.clone())
                    .collect()
            }))
        }
        (Some(src_chain), Some(dst_chain), Some(token_id), None) => Ok(with_state(|hub_state| {
            hub_state
                .cross_ledger
                .iter()
                .filter(|(_ticket_id, ticket)| {
                    ticket.src_chain.eq(&src_chain)
                        && ticket.dst_chain.eq(&dst_chain)
                        && ticket.token.eq(&token_id)
                })
                .skip(from)
                .take(offset)
                .map(|(_, ticket)| ticket.clone())
                .collect()
        })),
        (Some(src_chain), Some(dst_chain), Some(token_id), Some(time_range)) => {
            Ok(with_state(|hub_state| {
                hub_state
                    .cross_ledger
                    .iter()
                    .filter(|(_ticket_id, ticket)| {
                        ticket.src_chain.eq(&src_chain)
                            && ticket.dst_chain.eq(&dst_chain)
                            && ticket.token.eq(&token_id)
                            && (ticket.ticket_time >= time_range.0
                                && ticket.ticket_time <= time_range.0)
                    })
                    .skip(from)
                    .take(offset)
                    .map(|(_, ticket)| ticket.clone())
                    .collect()
            }))
        }
    }
}

#[query]
pub async fn get_total_tx() -> Result<u64, Error> {
    with_state(|hub_state| {
        let total_num = hub_state.cross_ledger.len() as u64;
        Ok(total_num)
    })
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
