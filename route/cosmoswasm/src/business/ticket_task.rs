use business::mint_token::MintTokenRequest;
use cosmoswasm::port::PortContractExecutor;
use memory::{mutate_state, read_state};

use crate::*;
use omnity_types::ChainState;

pub const BATCH_QUERY_LIMIT: u64 = 20;

pub fn process_ticket_msg_task() {
    ic_cdk::spawn(async {
        process_tickets().await;
    });
}

async fn process_tickets() {
    if read_state(|s| s.chain_state == ChainState::Deactive) {
        return;
    }

    let (hub_principal, offset) = read_state(|s| (s.hub_principal, s.next_ticket_seq));
    let port_contract_executor = PortContractExecutor::from_state();
    match hub::query_tickets(hub_principal, offset, BATCH_QUERY_LIMIT).await {
        Ok(tickets) => {
            for (seq, ticket) in &tickets {
                match port_contract_executor
                    .mint_token(MintTokenRequest {
                        ticket_id: ticket.ticket_id.clone(),
                        token_id: ticket.token.clone(),
                        receiver: ticket.receiver.clone(),
                        amount: ticket.amount.parse().unwrap(),
                    })
                    .await
                {
                    Ok(_) => mutate_state(|state| {
                        state.next_ticket_seq = seq + 1;
                    }),
                    Err(err) => {
                        log::error!(
                            "[process tickets] failed to mint token, seq: {}, ticket: {:?}, err: {:?}",
                            seq,
                            ticket,
                            err
                        );
                    }
                }
            }
        }

        Err(err) => {
            log::error!("[process tickets] failed to query tickets, err: {:?}", err);
        }
    }
}
