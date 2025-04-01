use crate::constants::TICKET_LIMIT_SIZE;
use crate::types::{ChainId, ChainState, Error, Seq, Ticket};
use candid::Principal;
use ic_solana::types::Pubkey;

use crate::{
    call_error::{CallError, Reason},
    state::{mutate_state, read_state},
};

use crate::handler::mint_token::mint_token;
use ic_canister_log::log;
use ic_cdk::spawn;
use ic_solana::ic_log::{ERROR, INFO};

/// handler tickets from customs to solana
pub async fn query_tickets() {
    if read_state(|s| s.chain_state == ChainState::Deactive) {
        return;
    }

    let (hub_principal, offset) = read_state(|s| (s.hub_principal, s.seqs.next_ticket_seq));
    match inner_query_tickets(hub_principal, offset, TICKET_LIMIT_SIZE).await {
        Ok(tickets) => {
            let mut next_seq = offset;
            for (seq, ticket) in tickets {
                if let Err(e) = Pubkey::try_from(ticket.receiver.as_str()) {
                    log!(
                        ERROR,
                        "[fetch_ticket::query_tickets] failed to parse ticket receiver: {}, error:{}",
                        ticket.receiver,
                        e.to_string()
                    );
                    next_seq = seq + 1;
                    continue;
                };
                if let Err(e) = ticket.amount.parse::<u64>() {
                    log!(
                        ERROR,
                        "[fetch_ticket::query_tickets] failed to parse ticket amount: {}, Error:{}",
                        ticket.amount,
                        e.to_string()
                    );
                    next_seq = seq + 1;
                    continue;
                };

                log!(
                    INFO,
                    "[Consolidation]fetch_ticket::query_tickets ticket id: {:?}",
                    ticket.ticket_id
                );
                mutate_state(|s| s.tickets_queue.insert(seq, ticket));
                next_seq = seq + 1;
            }

            spawn(mint_token());
            mutate_state(|s| s.seqs.next_ticket_seq = next_seq)
        }
        Err(e) => {
            log!(
                ERROR,
                "[fetch_ticket::query_tickets] failed to query tickets, err: {}",
                e.to_string()
            );
        }
    }
}

/// query ticket from hub
pub async fn inner_query_tickets(
    hub_principal: Principal,
    offset: u64,
    limit: u64,
) -> Result<Vec<(Seq, Ticket)>, CallError> {
    let resp: (Result<Vec<(Seq, Ticket)>, Error>,) = ic_cdk::api::call::call(
        hub_principal,
        "query_tickets",
        (None::<Option<ChainId>>, offset, limit),
    )
    .await
    .map_err(|(code, message)| CallError {
        method: "query_tickets".to_string(),
        reason: Reason::from_reject(code, message),
    })?;
    let data = resp.0.map_err(|err| CallError {
        method: "query_tickets".to_string(),
        reason: Reason::CanisterError(err.to_string()),
    })?;
    Ok(data)
}
