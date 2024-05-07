use candid::{CandidType, Principal};
use log::{self};
use serde::{Deserialize, Serialize};
use crate::{hub, audit};
use crate::state::{ mutate_state, read_state};
use std::str::FromStr;
use crate::evm_address::EvmAddress;
use crate::updates::mint_token::{MintTokenError, MintTokenRequest};
use crate::types::{ChainState, Directive, Token, TokenId};


pub const PERIODIC_TASK_INTERVAL: u64 = 5;
pub const BATCH_QUERY_LIMIT: u64 = 20;
pub const ICP_TRANSFER_FEE: u64 = 10_000;

async fn process_tickets() {
    if read_state(|s| s.chain_state == ChainState::Deactive) {
        return;
    }

    let (hub_principal, offset) = read_state(|s| (s.hub_principal, s.next_ticket_seq));
    match hub::query_tickets(hub_principal, offset, BATCH_QUERY_LIMIT).await {
        Ok(tickets) => {
            let mut next_seq = offset;
            for (seq, ticket) in &tickets {
                //TODO EVM_Address
                let receiver = if let Ok(receiver) = EvmAddress::from_str(&ticket.receiver) {
                    receiver
                } else {
                    log::error!(
                        "[process tickets] failed to parse ticket receiver: {}",
                        ticket.receiver
                    );
                    next_seq = seq + 1;
                    continue;
                };
                let amount: u128 = if let Ok(amount) = ticket.amount.parse() {
                    amount
                } else {
                    log::error!(
                        "[process tickets] failed to parse ticket amount: {}",
                        ticket.amount
                    );
                    next_seq = seq + 1;
                    continue;
                };
                mutate_state(|s|s.tickets_queue.insert(*seq, ticket.clone()));
                next_seq = seq + 1;
            }
            mutate_state(|s| s.next_ticket_seq = next_seq)
        }
        Err(err) => {
            log::error!("[process tickets] failed to query tickets, err: {}", err);
        }
    }
}

async fn process_directives() {
    let (hub_principal, offset) = read_state(|s| (s.hub_principal, s.next_directive_seq));
    match hub::query_directives(hub_principal, offset, BATCH_QUERY_LIMIT).await {
        Ok(directives) => {
            for (seq, directive) in &directives {
                mutate_state(|s|s.directives_queue.insert(*seq, directive.clone()));
              /*  match directive {
                    Directive::AddChain(chain) => {
                        mutate_state(|s| audit::add_chain(s, chain.clone()));
                    }
                    Directive::AddToken(token) => {
                        match crate::updates::add_new_token(token.clone()).await {
                            Ok(_) => {
                                log::info!(
                                    "[process directives] add token successful, token id: {}",
                                    token.token_id
                                );
                            }
                            Err(err) => {
                                log::error!(
                                    "[process directives] failed to add token: token id: {}, err: {:?}",
                                    token.token_id,
                                    err
                                );
                            }
                        }
                    }
                    Directive::ToggleChainState(toggle) => {
                        mutate_state(|s| audit::toggle_chain_state(s, toggle.clone()));
                    }
                    Directive::UpdateFee(fee) => {
                        mutate_state(|s| audit::update_fee(s, fee.clone()));
                        log::info!("[process_directives] success to update fee, fee: {}", fee);
                    }
                }*/
            }
            let next_seq = directives.last().map_or(offset, |(seq, _)| seq + 1);
            mutate_state(|s| {
                s.next_directive_seq = next_seq;
            });
        }
        Err(err) => {
            log::error!(
                "[process directives] failed to query directives, err: {:?}",
                err
            );
        }
    };
}

pub fn periodic_task() {
    ic_cdk::spawn(async {
        let _guard = match crate::guard::TimerLogicGuard::new() {
            Some(guard) => guard,
            None => return,
        };

        process_directives().await;
        process_tickets().await;
    });
}

pub fn to_cdk_tickets_task() {
    ic_cdk::spawn(async {
        let _guard = match crate::guard::TimerLogicGuard::new() {
            Some(guard) => guard,
            None => return,
        };

    });
}


#[derive(CandidType, Clone, Debug, Serialize, Deserialize)]
pub struct TokenResp {
    pub token_id: TokenId,
    pub symbol: String,
    pub decimals: u8,
    pub icon: Option<String>,
    pub rune_id: Option<String>,
}

impl From<Token> for TokenResp {
    fn from(value: Token) -> Self {
        TokenResp {
            token_id: value.token_id,
            symbol: value.symbol,
            decimals: value.decimals,
            icon: value.icon,
            rune_id: value.metadata.get("rune_id").map(|rune_id| rune_id.clone()),
        }
    }
}
