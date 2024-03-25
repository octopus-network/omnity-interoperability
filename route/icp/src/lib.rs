use candid::Principal;
use state::{mutate_state, read_state};
use updates::mint_token::MintTokenArgs;

use crate::tasks::schedule_after;
use std::{str::FromStr, time::Duration};

pub mod call_error;
pub mod hub;
pub mod lifecycle;
pub mod state;
pub mod tasks;
pub mod updates;

/// Time constants
const SEC_NANOS: u64 = 1_000_000_000;
pub const BATCH_QUERY_LIMIT: u64 = 20;

async fn process_tickets() {
    let (hub_principal, offset) = read_state(|s| (s.hub_principal, s.next_ticket_seq));
    match hub::query_tickets(hub_principal, offset, BATCH_QUERY_LIMIT).await {
        Ok(tickets) => {
            let mut next_seq = offset;
            for (seq, ticket) in tickets {
                let receiver = if let Ok(receiver) = Principal::from_str(&ticket.receiver) {
                    receiver
                } else {
                    next_seq = seq + 1;
                    // TODO record err logs
                    continue;
                };
                let amount: u128 = if let Ok(amount) = ticket.amount.parse() {
                    amount
                } else {
                    next_seq = seq + 1;
                    continue;
                };
                match updates::mint_token(MintTokenArgs {
                    token_id: ticket.token,
                    receiver,
                    amount,
                })
                .await
                {
                    Ok(_) => {}
                    Err(_) => {}
                }
                next_seq = seq + 1;
            }
            mutate_state(|s| s.next_ticket_seq = next_seq)
        }
        Err(_) => {
            // TODO record logs
        }
    }
}

pub fn timer() {
    use tasks::{pop_if_ready, TaskType};

    const INTERVAL_PROCESSING: Duration = Duration::from_secs(5);

    let task = match pop_if_ready() {
        Some(task) => task,
        None => return,
    };

    match task.task_type {
        TaskType::ProcessHubMessages => {
            ic_cdk::spawn(async {
                process_tickets().await;
                // TODO process directive
                schedule_after(INTERVAL_PROCESSING, TaskType::ProcessHubMessages);
            });
        }
    }
}
