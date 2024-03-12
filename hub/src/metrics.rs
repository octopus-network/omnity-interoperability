use std::collections::HashSet;

use ic_cdk::query;

use log::info;
use omnity_types::{
    Account, Chain, ChainId, ChainState, ChainType, Error, Ticket, TicketId, Token, TokenId,
    TokenOnChain, TxCondition,
};

use crate::with_state;

#[query]
pub async fn get_chains(
    chain_type: Option<ChainType>,
    chain_state: Option<ChainState>,
    from: usize,
    offset: usize,
) -> Result<Vec<Chain>, Error> {
    if matches!(chain_type, None) && matches!(chain_state, None) {
        let chains: Vec<Chain> = with_state(|hub_state| {
            hub_state
                .chains
                .iter()
                .skip(from)
                .take(offset)
                .map(|(_, chain)| chain.clone().into())
                .collect()
        });

        return Ok(chains);
    }
    // filter chains
    with_state(|hub_state| {
        // use hashset to keep unique
        let mut chain_set = HashSet::new();
        if let Some(dst_chain_type) = chain_type {
            chain_set.extend(
                hub_state
                    .chains
                    .iter()
                    .filter(|(_, chain)| chain.chain_type == dst_chain_type)
                    .map(|(_, chain)| chain.clone().into()),
            );
        } else if let Some(dst_chain_state) = chain_state {
            chain_set.extend(
                hub_state
                    .chains
                    .iter()
                    .filter(|(_, chain)| chain.chain_state == dst_chain_state)
                    .map(|(_, chain)| chain.clone().into()),
            );
        }
        // take value from to end (from + num )
        let chains: Vec<Chain> = chain_set.into_iter().skip(from).take(offset).collect();
        Ok(chains)
    })
}

#[query]
pub async fn get_chain(chain_id: String) -> Result<Chain, Error> {
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
    token_id: Option<TokenId>,
    chain_id: Option<ChainId>,
    from: usize,
    offset: usize,
) -> Result<Vec<Token>, Error> {
    if matches!(token_id, None) && matches!(chain_id, None) {
        let tokens: Vec<Token> = with_state(|hub_state| {
            hub_state
                .tokens
                .iter()
                .skip(from)
                .take(offset)
                .map(|(_, token)| token.clone().into())
                .collect()
        });

        return Ok(tokens);
    }
    // filter token
    with_state(|hub_state| {
        let mut token_set = HashSet::new();
        if let Some(token_id) = token_id {
            token_set.extend(
                hub_state
                    .tokens
                    .iter()
                    .filter(|(_, token)| token.token_id == token_id)
                    .map(|(_, token)| token.clone().into()),
            )
        } else if let Some(chain_id) = chain_id {
            token_set.extend(
                hub_state
                    .tokens
                    .iter()
                    .filter(|(_, token)| token.issue_chain.eq(&chain_id))
                    .map(|(_, token)| token.clone().into()),
            )
        }
        // take value from to end (from + num )
        let tokens: Vec<Token> = token_set.into_iter().skip(from).take(offset).collect();
        Ok(tokens)
    })
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
pub async fn get_txs(
    condition: TxCondition,
    from: usize,
    offset: usize,
) -> Result<Vec<Ticket>, Error> {
    if matches!(condition.src_chain, None)
        && matches!(condition.dst_chain, None)
        && matches!(condition.token_id, None)
        && matches!(condition.time_range, None)
    {
        let tokens: Vec<Ticket> = with_state(|hub_state| {
            hub_state
                .cross_ledger
                .iter()
                .skip(from)
                .take(offset)
                .map(|(_, ticket)| ticket.clone())
                .collect()
        });

        return Ok(tokens);
    }

    // filter tx
    with_state(|hub_state| {
        let mut ticket_set = HashSet::new();
        if let Some(src_chain) = condition.src_chain {
            ticket_set.extend(
                hub_state
                    .cross_ledger
                    .iter()
                    .filter(|(_ticket_id, ticket)| ticket.src_chain.eq(&src_chain))
                    .map(|(_, ticket)| ticket.clone()),
            );
        } else if let Some(dst_chain) = condition.dst_chain {
            ticket_set.extend(
                hub_state
                    .cross_ledger
                    .iter()
                    .filter(|(_ticket_id, ticket)| ticket.dst_chain.eq(&dst_chain))
                    .map(|(_, ticket)| ticket.clone()),
            );
        } else if let Some(token_id) = condition.token_id {
            ticket_set.extend(
                hub_state
                    .cross_ledger
                    .iter()
                    .filter(|(_ticket_id, ticket)| ticket.token.eq(&token_id))
                    .map(|(_, ticket)| ticket.clone()),
            );
        } else if let Some(time_range) = condition.time_range {
            ticket_set.extend(
                hub_state
                    .cross_ledger
                    .iter()
                    .filter(|(_ticket_id, ticket)| {
                        ticket.ticket_time >= time_range.0 && ticket.ticket_time <= time_range.0
                    })
                    .map(|(_, ticket)| ticket.clone()),
            );
        }
        // take value from to end (from + num )
        let tickets: Vec<Ticket> = ticket_set.into_iter().skip(from).take(offset).collect();
        Ok(tickets)
    })
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
    chain_id: Option<ChainId>,
    token_id: Option<TokenId>,
    from: usize,
    offset: usize,
) -> Result<Vec<TokenOnChain>, Error> {
    if matches!(chain_id, None) && matches!(token_id, None) {
        let chain_tokens: Vec<TokenOnChain> = with_state(|hub_state| {
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
        });

        return Ok(chain_tokens);
    }
    // filter
    with_state(|hub_state| {
        let mut chain_token_set = HashSet::new();
        if let Some(dst_token_id) = token_id {
            chain_token_set.extend(
                hub_state
                    .token_position
                    .iter()
                    .filter(|((_chain_id, token_id), _total_amount)| token_id.eq(&dst_token_id))
                    .map(|((chain_id, token_id), total_amount)| TokenOnChain {
                        chain_id: chain_id.to_string(),
                        token_id: token_id.to_string(),
                        amount: *total_amount,
                    }),
            )
        } else if let Some(dst_chain_id) = chain_id {
            chain_token_set.extend(
                hub_state
                    .token_position
                    .iter()
                    .filter(|((chain_id, _token_id), _total_amount)| chain_id.eq(&dst_chain_id))
                    .map(|((chain_id, token_id), total_amount)| TokenOnChain {
                        chain_id: chain_id.to_string(),
                        token_id: token_id.to_string(),
                        amount: *total_amount,
                    }),
            )
        }
        // take value from to end (from + num )
        let chain_tokens: Vec<TokenOnChain> = chain_token_set
            .into_iter()
            .skip(from)
            .take(offset)
            .collect();
        Ok(chain_tokens)
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

#[query]
pub async fn get_account_assets(
    account: Account,
    dst_chain: Option<ChainId>,
    dst_token: Option<TokenId>,
) -> Result<Vec<TokenOnChain>, Error> {
    with_state(|hub_state| {
        if let Some(account_assets) = hub_state.accounts.get(&account) {
            if matches!(dst_chain, None) && matches!(dst_token, None) {
                let chain_tokens: Vec<TokenOnChain> = account_assets
                    .iter()
                    .map(|((chain_id, token_id), total_amount)| TokenOnChain {
                        chain_id: chain_id.to_string(),
                        token_id: token_id.to_string(),
                        amount: *total_amount,
                    })
                    .collect();

                return Ok(chain_tokens);
            }

            // filter
            let mut assets = HashSet::new();
            if let Some(dst_chain) = dst_chain {
                assets.extend(
                    account_assets
                        .iter()
                        .filter(|((chain, _token), _balance)| chain.eq(&dst_chain))
                        .map(|((chain_id, token_id), total_amount)| TokenOnChain {
                            chain_id: chain_id.to_string(),
                            token_id: token_id.to_string(),
                            amount: *total_amount,
                        }),
                )
            } else if let Some(dst_token) = dst_token {
                assets.extend(
                    account_assets
                        .iter()
                        .filter(|((_chain, token), _balance)| token.eq(&dst_token))
                        .map(|((chain_id, token_id), total_amount)| TokenOnChain {
                            chain_id: chain_id.to_string(),
                            token_id: token_id.to_string(),
                            amount: *total_amount,
                        }),
                )
            }
            let ret = assets.into_iter().collect();
            info!("get_account_assets: {:?}", ret);
            Ok(ret)
        } else {
            return Err(Error::NotFoundAccount(account.to_string()));
        }
    })
}
