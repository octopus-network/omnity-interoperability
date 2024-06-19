use crate::const_args::{BATCH_QUERY_LIMIT, FETCH_HUB_TASK_NAME};
use crate::eth_common::EvmAddress;
use crate::state::{mutate_state, read_state};
use crate::types::{ChainState, Directive, Seq, Ticket};
use crate::{audit, hub};
use log::{self};
use std::str::FromStr;

async fn process_tickets() {
    if read_state(|s| s.chain_state == ChainState::Deactive) {
        return;
    }
    let (hub_principal, offset) = read_state(|s| (s.hub_principal, s.next_ticket_seq));
    match hub::query_tickets(hub_principal, offset, BATCH_QUERY_LIMIT).await {
        Ok(tickets) => {
            store_tickets(tickets, offset);
        }
        Err(err) => {
            log::error!("[process tickets] failed to query tickets, err: {}", err);
        }
    }
}

pub fn store_tickets(tickets: Vec<(Seq, Ticket)>, offset: u64) {
    let mut next_seq = offset;
    for (seq, ticket) in &tickets {
        if let Err(_) = EvmAddress::from_str(&ticket.receiver) {
            log::error!(
                "[process tickets] failed to parse ticket receiver: {}",
                ticket.receiver
            );
            next_seq = seq + 1;
            continue;
        };
        if let Err(_) = ticket.amount.parse::<u128>() {
            log::error!(
                "[process tickets] failed to parse ticket amount: {}",
                ticket.amount
            );
            next_seq = seq + 1;
            continue;
        };
        mutate_state(|s| s.tickets_queue.insert(*seq, ticket.clone()));
        next_seq = seq + 1;
    }
    mutate_state(|s| s.next_ticket_seq = next_seq)
}

async fn process_directives() {
    let (hub_principal, offset) = read_state(|s| (s.hub_principal, s.next_directive_seq));
    match hub::query_directives(hub_principal, offset, BATCH_QUERY_LIMIT).await {
        Ok(directives) => {
            for (seq, directive) in &directives {
                let mut final_directive = directive.clone();
                match directive.clone() {
                    Directive::AddChain(chain) | Directive::UpdateChain(chain) => {
                        mutate_state(|s| audit::add_chain(s, chain.clone()));
                    }
                    Directive::ToggleChainState(t) => {
                        mutate_state(|s| {
                            if let Some(chain) = s.counterparties.get_mut(&t.chain_id) {
                                chain.chain_state = t.action.into();
                            }
                            // if toggle self chain, handle after port contract executed
                        });
                    }
                    Directive::UpdateToken(token) => {
                        let is_old_token = read_state(|s| s.tokens.get(&token.token_id).is_some());
                        if is_old_token {
                            mutate_state(|s| audit::add_token(s, token.clone()));
                        } else {
                            //special condition, when add current chain into token's dst chain,
                            // updateToken means addtoken for current chain.
                            final_directive = Directive::AddToken(token.clone());
                        }
                    }
                    _ => {
                        //process after port contract executed, don't handle it now.
                    }
                }
                mutate_state(|s| s.directives_queue.insert(*seq, final_directive));
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

pub fn fetch_hub_periodic_task() {
    ic_cdk::spawn(async {
        let _guard = match crate::guard::TimerLogicGuard::new(FETCH_HUB_TASK_NAME.to_string()) {
            Some(guard) => guard,
            None => return,
        };
        process_directives().await;
        process_tickets().await;
    });
}
