#![allow(unused)]
use std::str::FromStr;

use crate::config::{mutate_config, read_config};
use crate::constants::TICKET_LIMIT_SIZE;
use crate::ic_log::ERROR;
use crate::ic_sui::sui_types::base_types::SuiAddress;
use crate::types::{ChainId, ChainState, Error, Seq, Ticket};
use candid::Principal;

use crate::{
    call_error::{CallError, Reason},
    state::mutate_state,
};

use ic_canister_log::log;

/// handler tickets from customs to sui
pub async fn query_tickets() {
    if read_config(|s| s.get().chain_state == ChainState::Deactive) {
        return;
    }

    let (hub_principal, offset) =
        read_config(|s| (s.get().hub_principal, s.get().seqs.next_ticket_seq));
    match inner_query_tickets(hub_principal, offset, TICKET_LIMIT_SIZE).await {
        Ok(tickets) => {
            let mut next_seq = offset;
            for (seq, ticket) in &tickets {
                if let Err(e) = SuiAddress::from_str(&ticket.receiver) {
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

                mutate_state(|s| s.tickets_queue.insert(*seq, ticket.to_owned()));
                next_seq = seq + 1;
            }
            mutate_config(|s| {
                let mut config = s.get().to_owned();
                config.seqs.next_ticket_seq = next_seq;
                s.set(config);
            })
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
