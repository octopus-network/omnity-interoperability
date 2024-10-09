use crate::constants::{BATCH_QUERY_LIMIT, FETCH_HUB_DIRECTIVE_NAME, FETCH_HUB_TICKET_NAME};
use crate::state::{mutate_state, read_state};
use crate::{audit, hub};
use ic_canister_log::log;
use omnity_types::ic_log::ERROR;
use omnity_types::{ChainState, Directive, Seq, Ticket};

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
            log!(
                ERROR,
                "[process tickets] failed to query tickets, err: {}",
                err
            );
        }
    }
}

pub fn store_tickets(tickets: Vec<(Seq, Ticket)>, offset: u64) {
    let mut next_seq = offset;
    for (seq, ticket) in tickets {
        if ticket.amount.parse::<u128>().is_err() {
            log!(
                ERROR,
                "[process tickets] failed to parse ticket amount: {}",
                ticket.amount
            );
            next_seq = seq + 1;
            continue;
        };
        mutate_state(|s| {
            let ticketid = ticket.ticket_id.clone();
            s.ticket_id_seq_indexer.insert(ticketid, seq);
            s.tickets_queue.insert(seq, ticket);
        });
        next_seq = seq + 1;
    }
    mutate_state(|s| s.next_ticket_seq = next_seq)
}

async fn process_directives() {
    let (hub_principal, offset) = read_state(|s| (s.hub_principal, s.next_directive_seq));
    match hub::query_directives(hub_principal, offset, BATCH_QUERY_LIMIT).await {
        Ok(directives) => {
            for (seq, directive) in &directives {
                let final_directive = directive.clone();
                match directive.clone() {
                    Directive::AddChain(chain) | Directive::UpdateChain(chain) => {
                        mutate_state(|s| audit::add_chain(s, chain.clone()));
                    }
                    Directive::ToggleChainState(t) => {
                        mutate_state(|s| {
                            if let Some(chain) = s.counterparties.get_mut(&t.chain_id) {
                                chain.chain_state = t.action.clone().into();
                            }
                            if t.chain_id == s.chain_id {
                                s.chain_state = t.action.into();
                            }
                        });
                    }
                    Directive::UpdateToken(token) => {
                        mutate_state(|s| audit::add_token(s, token.clone()));
                    }
                    Directive::AddToken(token) => {
                        mutate_state(|s| audit::add_token(s, token));
                    }
                    Directive::UpdateFee(_) => {}
                }
                mutate_state(|s| s.directives_queue.insert(*seq, final_directive));
            }
            let next_seq = directives.last().map_or(offset, |(seq, _)| seq + 1);
            mutate_state(|s| {
                s.next_directive_seq = next_seq;
            });
        }
        Err(err) => {
            log!(
                ERROR,
                "[process directives] failed to query directives, err: {:?}",
                err
            );
        }
    };
}

pub fn fetch_hub_ticket_task() {
    ic_cdk::spawn(async {
        let _guard = match crate::guard::TimerLogicGuard::new(FETCH_HUB_TICKET_NAME.to_string()) {
            Some(guard) => guard,
            None => return,
        };
        process_tickets().await;
    });
}

pub fn fetch_hub_directive_task() {
    ic_cdk::spawn(async {
        let _guard = match crate::guard::TimerLogicGuard::new(FETCH_HUB_DIRECTIVE_NAME.to_string())
        {
            Some(guard) => guard,
            None => return,
        };
        process_directives().await;
    });
}
