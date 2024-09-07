use business::mint_token::MintTokenRequest;
use cosmwasm::port::PortContractExecutor;
use memory::{get_periodic_job_manager, insert_periodic_job_manager, set_route_state, take_state};

use crate::*;
use omnity_types::ChainState;

pub fn process_ticket_task() {
    ic_cdk::spawn(async {
        let job_name = const_args::PROCESS_TICKET_JOB_NAME;
        match get_periodic_job_manager(job_name) {
            Some(mut periodic_job_manager) => {
                if !periodic_job_manager.should_execute() {
                    return;
                }
                periodic_job_manager.is_running = true;
                insert_periodic_job_manager(job_name.to_string(), periodic_job_manager.clone());
                match process_tickets().await {
                    Ok(_) => {
                        periodic_job_manager.job_execute_success();
                    }
                    Err(e) => {
                        periodic_job_manager.job_execute_failed();
                        log::error!("failed to process directives, err: {:?}", e);
                    }
                }
                insert_periodic_job_manager(job_name.to_string(), periodic_job_manager.clone());
            }
            None => {
                log::error!(
                    "periodic job({}) manager is none",
                    job_name
                );
                return;
            }
        }
    });
}

async fn process_tickets() -> Result<()> {
    let mut state = take_state();
    if state.chain_state == ChainState::Deactive {
        return Ok(());
    }

    // fetch tickets from hub if processing_tickets is empty
    if state.processing_tickets.is_empty() {
        let tickets = hub::query_tickets(
            state.hub_principal,
            state.next_ticket_seq,
            const_args::BATCH_QUERY_LIMIT,
        )
        .await?;
        state.processing_tickets = tickets.clone();
    }
    let port_contract_executor = PortContractExecutor::from_state()?;

    // Descending order
    state
        .processing_tickets
        .sort_by(|(seq1, _), (seq2, _)| seq2.cmp(seq1));

    while !state.processing_tickets.is_empty() {
        let (seq, ticket) = state.processing_tickets.pop().unwrap();
        match port_contract_executor
            .mint_token(MintTokenRequest {
                ticket_id: ticket.ticket_id.clone(),
                token_id: ticket.token.clone(),
                receiver: ticket.receiver.clone(),
                amount: ticket.amount.parse().unwrap(),
            })
            .await
        {
            Ok(_) => {
                state.next_ticket_seq = seq + 1;
                set_route_state(state.clone());
                log::info!(
                    "[process tickets] success to mint token, seq: {}, ticket: {:?}",
                    seq,
                    ticket
                );
            }
            Err(err) => {
                log::error!(
                    "[process tickets] failed to mint token, seq: {}, ticket: {:?}, err: {:?}",
                    seq,
                    ticket,
                    err
                );
                state.processing_tickets.push((seq, ticket));
                set_route_state(state.clone());
                break;
            }
        }
    }

    Ok(())
}
