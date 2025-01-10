use crate::const_args::{BATCH_QUERY_LIMIT};
use crate::eth_common::EvmAddress;
use crate::state::{mutate_state, read_state};
use omnity_types::{Directive, Seq, Ticket};
use crate::{audit, hub};
use std::str::FromStr;
use ic_canister_log::log;
use omnity_types::ic_log::{CRITICAL, ERROR, INFO};

pub async fn process_tickets() {
    let (hub_principal, offset) = read_state(|s| (s.hub_principal, s.next_ticket_seq));
    match hub::query_tickets(hub_principal, offset, BATCH_QUERY_LIMIT).await {
        Ok(tickets) => {
            store_tickets(tickets, offset);
        }
        Err(err) => {
            log!(ERROR, "[process tickets] failed to query tickets, err: {}", err);
        }
    }
}

pub fn store_tickets(tickets: Vec<(Seq, Ticket)>, offset: u64) {
    let mut next_seq = offset;
    for (seq, ticket) in tickets.into_iter() {
        if EvmAddress::from_str(&ticket.receiver).is_err() {
            log!(CRITICAL,
                "[process tickets] failed to parse ticket receiver: {}",
                ticket.receiver
            );
            next_seq = seq + 1;
            continue;
        };
        if ticket.amount.parse::<u128>().is_err() {
            log!(CRITICAL,
                "[process tickets] failed to parse ticket amount: {}",
                ticket.amount
            );
            next_seq = seq + 1;
            continue;
        };
        mutate_state(|s| s.tickets_queue.insert(seq, ticket));
        next_seq = seq + 1;
    }
    mutate_state(|s| s.next_ticket_seq = next_seq)
}

pub async fn process_directives() {
    let (hub_principal, offset) = read_state(|s| (s.hub_principal, s.next_directive_seq));
    match hub::query_directives(hub_principal, offset, BATCH_QUERY_LIMIT).await {
        Ok(directives) => {
            let next_seq = directives.last().map_or(offset, |(seq, _)| seq + 1);
            for (seq, directive) in directives {
                let mut final_directive = directive.clone();
                match directive {
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
                        let is_old_token = read_state(|s| s.tokens.contains_key(&token.token_id));
                        if is_old_token {
                            mutate_state(|s| audit::add_token(s, token));
                        } else {
                            //special condition, when add current chain into token's dst chain,
                            // updateToken means addtoken for current chain.
                            final_directive = Directive::AddToken(token);
                        }
                    }
                    _ => {
                        //process after port contract executed, don't handle it now.
                    }
                }
                mutate_state(|s| s.directives_queue.insert(seq, final_directive));
            }
            mutate_state(|s| {
                s.next_directive_seq = next_seq;
            });
        }
        Err(err) => {
            log!(ERROR,
                "[process directives] failed to query directives, err: {:?}",
                err
            );
        }
    };
}
