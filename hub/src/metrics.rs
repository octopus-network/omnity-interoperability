use crate::memory::{self, Memory};
use crate::{
    state::with_state,
    types::{ChainMeta, TokenKey, TokenMeta},
};

use ic_stable_structures::StableBTreeMap;
use log::{error, info};
use omnity_types::{
    Account, Chain, ChainId, ChainState, ChainType, Directive, Error, Ticket, TicketId, Token,
    TokenId, TokenOnChain,
};
use serde::Serialize;
use std::cell::RefCell;

const LEDGER_SEQ_KEY: &[u8] = b"ledger_seq";

thread_local! {
    static METRICS: RefCell<Metrics> = RefCell::new(Metrics::default());
}

#[derive(Serialize)]
pub struct Metrics {
    #[serde(skip, default = "memory::init_ledger_metric")]
    pub tickets_metric: StableBTreeMap<u64, Ticket, Memory>,
    #[serde(skip, default = "memory::init_metric_seqs")]
    pub metric_seqs: StableBTreeMap<Vec<u8>, u64, Memory>,
}

impl Default for Metrics {
    fn default() -> Self {
        Self {
            tickets_metric: StableBTreeMap::init(memory::get_ticket_metric()),
            metric_seqs: StableBTreeMap::init(memory::get_metric_seqs()),
        }
    }
}

pub fn with_metrics<R>(f: impl FnOnce(&Metrics) -> R) -> R {
    METRICS.with(|cell| f(&cell.borrow()))
}

pub fn with_metrics_mut<R>(f: impl FnOnce(&mut Metrics) -> R) -> R {
    METRICS.with(|cell| f(&mut cell.borrow_mut()))
}

pub fn set_metrics(metrics: Metrics) {
    METRICS.with(|cell| *cell.borrow_mut() = metrics);
}

impl Metrics {
    pub fn update_ticket_metric(&mut self, ticket: Ticket) {
        let latest_ticket_seq = self
            .metric_seqs
            .get(&LEDGER_SEQ_KEY.to_vec())
            .unwrap_or_default();
        self.tickets_metric
            .insert(latest_ticket_seq, ticket.clone());
        let latest_ticket_seq = latest_ticket_seq + 1;
        self.metric_seqs
            .insert(LEDGER_SEQ_KEY.to_vec(), latest_ticket_seq);
    }
    pub fn sync_ticket_size(&self) -> Result<u64, Error> {
        let total_num = self.tickets_metric.len();
        Ok(total_num)
    }

    pub fn sync_tickets(&self, from_seq: usize, limit: usize) -> Result<Vec<(u64, Ticket)>, Error> {
        info!("get_tickets  from: {}, limit: {}", from_seq, limit);
        let from_seq = from_seq as u64;
        let tickets = self
            .tickets_metric
            .iter()
            .filter(|(seq, _)| *seq >= from_seq)
            .take(limit)
            .map(|(seq, ticket)| (seq, ticket))
            .collect::<Vec<_>>();

        Ok(tickets)
    }
}

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
            .map(|(_, chain)| chain.into())
            .collect::<Vec<_>>()
    });

    Ok(chains)
}

pub async fn get_chain_metas(offset: usize, limit: usize) -> Result<Vec<ChainMeta>, Error> {
    info!("get_chains from {}, limit: {}", offset, limit);

    let chains = with_state(|hub_state| {
        hub_state
            .chains
            .iter()
            .skip(offset)
            .take(limit)
            .map(|(_, chain)| chain)
            .collect::<Vec<_>>()
    });

    Ok(chains)
}

pub async fn get_chain(chain_id: String) -> Result<Chain, Error> {
    info!("get_chain chain_id: {:?} ", chain_id);
    with_state(|hub_state| {
        if let Some(chain) = hub_state.chains.get(&chain_id) {
            Ok(chain.into())
        } else {
            error!("not found chain: (`{}`)", chain_id.to_string());
            Err(Error::NotFoundChain(chain_id))
        }
    })
}

pub async fn get_chain_size() -> Result<u64, Error> {
    with_state(|hub_state| {
        let total_num = hub_state.chains.len();
        Ok(total_num)
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
            .map(|(_, token_meta)| token_meta.into())
            .collect::<Vec<_>>()
    });

    Ok(tokens)
}

pub async fn get_token_metas(offset: usize, limit: usize) -> Result<Vec<TokenMeta>, Error> {
    info!("get_token_metas  from: {}, limit: {}", offset, limit);

    let tokens = with_state(|hub_state| {
        hub_state
            .tokens
            .iter()
            .skip(offset)
            .take(limit)
            .map(|(_, token_meta)| token_meta)
            .collect::<Vec<_>>()
    });

    Ok(tokens)
}

pub async fn get_token_size() -> Result<u64, Error> {
    with_state(|hub_state| {
        let total_num = hub_state.tokens.len();
        Ok(total_num)
    })
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
                    .get(&tf.target_chain_id)
                    .map(|chain_factor| {
                        (
                            tf.target_chain_id.to_string(),
                            tf.fee_token.to_string(),
                            chain_factor * tf.fee_token_factor,
                        )
                    })
            })
            .collect::<Vec<_>>()
    });

    Ok(fees)
}

pub async fn get_directive_size() -> Result<u64, Error> {
    with_state(|hub_state| {
        let total_num = hub_state.directives.len();
        Ok(total_num)
    })
}
pub async fn get_directives(offset: usize, limit: usize) -> Result<Vec<Directive>, Error> {
    info!("get_directives  from: {}, limit: {}", offset, limit);

    let dires = with_state(|hub_state| {
        hub_state
            .directives
            .iter()
            .skip(offset)
            .take(limit)
            .map(|(_, dire)| dire)
            .collect::<Vec<_>>()
    });

    Ok(dires)
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
            .map(|(_, ticket)| ticket)
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
            .map(|(_, ticket)| ticket)
            .collect::<Vec<_>>()
    });

    Ok(filtered_tickets)
}

pub async fn get_txs(offset: usize, limit: usize) -> Result<Vec<Ticket>, Error> {
    info!("get_txs offset: {}, limit: {}", offset, limit);

    let filtered_tickets = with_state(|hub_state| {
        hub_state
            .cross_ledger
            .iter()
            .skip(offset)
            .take(limit)
            .map(|(_, ticket)| ticket)
            .collect::<Vec<_>>()
    });

    Ok(filtered_tickets)
}

pub async fn get_tx(ticket_id: TicketId) -> Result<Ticket, Error> {
    info!("get_tx ticket_id: {:?} ", ticket_id);
    with_state(|hub_state| {
        if let Some(ticket) = hub_state.cross_ledger.get(&ticket_id) {
            Ok(ticket)
        } else {
            error!("Not found this ticket: {}", ticket_id);
            Err(Error::CustomError(format!(
                "Not found this ticket: {}",
                ticket_id
            )))
        }
    })
}

pub async fn get_total_tx() -> Result<u64, Error> {
    with_state(|hub_state| {
        let total_num = hub_state.cross_ledger.len();
        Ok(total_num)
    })
}

pub async fn get_chain_type(chain_id: ChainId) -> Result<ChainType, Error> {
    with_state(|hub_state| {
        if let Some(chain) = hub_state.chains.get(&chain_id) {
            Ok(chain.chain_type)
        } else {
            error!("Not found this chain: {}", chain_id);
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
                error!("not found chain id for caller:{:?}", caller);
                Err(Error::CustomError(format!(
                    "not found chain id for caller:{:?}",
                    caller
                )))
            }
        })?;
        Ok(chain_id)
    }
}
