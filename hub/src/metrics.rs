use crate::types::ChainMeta;
use crate::{state::with_state, types::TokenKey};
use log::info;
use omnity_types::{
    Account, Chain, ChainId, ChainState, ChainType, Error, Ticket, TicketId, Token, TokenId,
    TokenOnChain,
};

pub async fn get_chains(
    chain_type: Option<ChainType>,
    chain_state: Option<ChainState>,
    offset: usize,
    limit: usize,
) -> Result<Vec<Chain>, Error> {
    let condition = (chain_type, chain_state);
    info!(
        "get_chains condition: {:?}, from: {}, offset: {}",
        condition, offset, offset
    );

    let chains = with_state(|hub_state| {
        hub_state
            .chains
            .iter()
            .filter(|(_, chain)| match &condition {
                (None, None) => true,
                (None, Some(dst_chain_state)) => chain.chain_state == *dst_chain_state,
                (Some(dst_chain_type), None) => chain.chain_type == *dst_chain_type,
                (Some(dst_chain_type), Some(dst_chain_state)) => {
                    chain.chain_type == *dst_chain_type && chain.chain_state == *dst_chain_state
                }
            })
            .skip(offset)
            .take(limit)
            .map(|(_, chain)| <ChainMeta as Into<Chain>>::into(chain.clone()))
            .collect::<Vec<_>>()
    });

    Ok(chains)
}

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

pub async fn get_tokens(
    chain_id: Option<ChainId>,
    token_id: Option<TokenId>,
    offset: usize,
    limit: usize,
) -> Result<Vec<Token>, Error> {
    let condition = (chain_id, token_id);
    info!(
        "get_tokens condition: {:?}, from: {}, offset: {}",
        condition, offset, limit
    );

    let tokens = with_state(|hub_state| {
        hub_state
            .tokens
            .iter()
            .filter(|(_, token_meta)| match &condition {
                (None, None) => true,
                (None, Some(dst_token_id)) => token_meta.token_id.eq(dst_token_id),
                (Some(dst_chain_id), None) => token_meta.issue_chain.eq(dst_chain_id),
                (Some(dst_chain_id), Some(dst_token_id)) => {
                    token_meta.issue_chain.eq(dst_chain_id) && token_meta.token_id.eq(dst_token_id)
                }
            })
            .skip(offset)
            .take(limit)
            .map(|(_, token_meta)| token_meta.clone().into())
            .collect::<Vec<_>>()
    });

    Ok(tokens)
}

/// get fees
pub async fn get_fees(
    chain_id: Option<ChainId>,
    token_id: Option<TokenId>,
    offset: usize,
    limit: usize,
) -> Result<Vec<(ChainId, TokenId, u128)>, Error> {
    let condition = (chain_id, token_id);
    info!(
        "get_fees condition: {:?}, from: {}, offset: {}",
        condition, offset, limit
    );

    let fees = with_state(|hub_state| {
        hub_state
            .fee_token_factors
            .iter()
            .filter(|(token_key, _)| filter_chain_token(token_key, &condition))
            .skip(offset)
            .take(limit)
            .filter_map(|(_, tf)| {
                hub_state
                    .target_chain_factors
                    .get(&tf.dst_chain_id)
                    .map(|chain_factor| {
                        (
                            tf.dst_chain_id.to_string(),
                            tf.fee_token.to_string(),
                            chain_factor * tf.fee_token_factor as u128,
                        )
                    })
            })
            .collect::<Vec<_>>()
    });

    Ok(fees)
}

fn filter_chain_token(
    token_key: &TokenKey,
    condition: &(Option<ChainId>, Option<TokenId>),
) -> bool {
    match condition {
        (None, None) => true,
        (None, Some(dst_token_id)) => token_key.token_id.eq(dst_token_id),
        (Some(dst_chain_id), None) => token_key.chain_id.eq(dst_chain_id),
        (Some(dst_chain_id), Some(dst_token_id)) => {
            token_key.chain_id.eq(dst_chain_id) && token_key.token_id.eq(dst_token_id)
        }
    }
}
/// get tokens on dst chain

pub async fn get_chain_tokens(
    chain_id: Option<ChainId>,
    token_id: Option<TokenId>,
    offset: usize,
    limit: usize,
) -> Result<Vec<TokenOnChain>, Error> {
    let condition = (chain_id, token_id);
    info!(
        "get_chain_tokens condition: {:?}, from: {}, offset: {}",
        condition, offset, limit
    );

    let tokens_on_chain = with_state(|hub_state| {
        hub_state
            .token_position
            .iter()
            .filter(|(token_key, _)| filter_chain_token(token_key, &condition))
            .skip(offset)
            .take(limit)
            .map(|(token_key, total_amount)| TokenOnChain {
                chain_id: token_key.chain_id.to_string(),
                token_id: token_key.token_id.to_string(),
                amount: total_amount,
            })
            .collect::<Vec<_>>()
    });

    Ok(tokens_on_chain)
}

pub async fn get_txs_with_chain(
    src_chain: Option<ChainId>,
    dst_chain: Option<ChainId>,
    token_id: Option<TokenId>,
    time_range: Option<(u64, u64)>,
    offset: usize,
    limit: usize,
) -> Result<Vec<Ticket>, Error> {
    info!(
        "get_txs_with_chain condition: src chain:{:?},  dst chain:{:?},  token id:{:?}, time range:{:?}, offset: {}, limit: {}",
        src_chain, dst_chain, token_id, time_range, offset, limit
    );

    let filtered_tickets = with_state(|hub_state| {
        hub_state
            .cross_ledger
            .iter()
            .filter(|(_, ticket)| {
                let src_chain_match = src_chain
                    .as_ref()
                    .map_or(true, |chain| ticket.src_chain.eq(chain));
                let dst_chain_match = dst_chain
                    .as_ref()
                    .map_or(true, |chain| ticket.dst_chain.eq(chain));
                let token_id_match = token_id
                    .as_ref()
                    .map_or(true, |token_id| ticket.token.eq(token_id));

                let time_range_match = match time_range {
                    Some((start, end)) => ticket.ticket_time >= start && ticket.ticket_time <= end,
                    None => true,
                };
                src_chain_match && dst_chain_match && token_id_match && time_range_match
            })
            .skip(offset)
            .take(limit)
            .map(|(_, ticket)| ticket.clone())
            .collect::<Vec<_>>()
    });

    Ok(filtered_tickets)
}

pub async fn get_txs_with_account(
    sender: Option<Account>,
    receiver: Option<Account>,
    token_id: Option<TokenId>,
    time_range: Option<(u64, u64)>,
    offset: usize,
    limit: usize,
) -> Result<Vec<Ticket>, Error> {
    info!(
        "get_txs_with_account condition: sender:{:?}, receiver:{:?},  token id:{:?}, time range:{:?}, offset: {}, limit: {}",
        sender, receiver, token_id, time_range, offset, limit
    );

    let filtered_tickets = with_state(|hub_state| {
        hub_state
            .cross_ledger
            .iter()
            .filter(|(_, ticket)| {
                let sender_match = sender
                    .as_ref()
                    .map_or(true, |req_sender| matches!(&ticket.sender, Some(ticket_sender) if ticket_sender.eq(req_sender)));
                let receiver_match = receiver
                    .as_ref()
                    .map_or(true, |receiver| ticket.receiver.eq(receiver));
                let token_id_match = token_id
                    .as_ref()
                    .map_or(true, |token_id| ticket.token.eq(token_id));

                let time_range_match = match time_range {
                    Some((start, end)) => ticket.ticket_time >= start && ticket.ticket_time <= end,
                    None => true,
                };
                sender_match && receiver_match && token_id_match && time_range_match
            })
            .skip(offset)
            .take(limit)
            .map(|(_, ticket)| ticket.clone())
            .collect::<Vec<_>>()
    });

    Ok(filtered_tickets)
}

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

pub async fn get_total_tx() -> Result<u64, Error> {
    with_state(|hub_state| {
        let total_num = hub_state.cross_ledger.len() as u64;
        Ok(total_num)
    })
}

pub async fn get_chain_type(chain_id: ChainId) -> Result<ChainType, Error> {
    with_state(|hub_state| {
        if let Some(chain) = hub_state.chains.get(&chain_id) {
            Ok(chain.chain_type.clone())
        } else {
            Err(Error::NotFoundChain(chain_id))
        }
    })
}

// get chain id from canister
pub fn get_chain_id(chain_id: Option<ChainId>) -> Result<ChainId, Error> {
    if let Some(chain_id) = chain_id {
        Ok(chain_id)
    } else {
        let chain_id = with_state(|hs| {
            let caller = ic_cdk::api::caller().to_string();
            if let Some(chain_id) = hs.authorized_caller.get(&caller) {
                Ok(chain_id.to_string())
            } else {
                Err(Error::CustomError(format!(
                    "not found chain id for caller:{:?}",
                    caller
                )))
            }
        })?;
        Ok(chain_id)
    }
}
