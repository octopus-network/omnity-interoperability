use cosmwasm::port::PortContractExecutor;
use memory::{get_periodic_job_manager, insert_periodic_job_manager, set_route_state, take_state};

use crate::*;
use omnity_types::ChainState;

pub fn process_directive_task() {
    ic_cdk::spawn(async {
        let job_name = const_args::PROCESS_DIRECTIVE_JOB_NAME;
        match get_periodic_job_manager(job_name) {
            Some(mut periodic_job_manager) => {
                if !periodic_job_manager.should_execute() {
                    return;
                }
                periodic_job_manager.is_running = true;
                insert_periodic_job_manager(job_name.to_string(), periodic_job_manager.clone());
                match process_directives().await {
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

async fn process_directives() -> Result<()> {
    let mut state = take_state();
    if state.chain_state == ChainState::Deactive {
        return Ok(());
    }

    if state.processing_directive.is_empty() {
        let directives = hub::query_directives(
            state.hub_principal,
            state.next_directive_seq,
            const_args::BATCH_QUERY_LIMIT,
        )
        .await?;
        state.processing_directive = directives.clone();
    }

    let port_contract_executor = PortContractExecutor::from_state()?;
    state
        .processing_directive
        .sort_by(|(seq1, _), (seq2, _)| seq2.cmp(seq1));

    while !state.processing_directive.is_empty() {
        let (seq, directive) = state.processing_directive.pop().unwrap();
        match port_contract_executor
            .execute_directive(seq, directive.clone().into())
            .await
        {
            Ok(_) => {
                state.next_directive_seq = seq + 1;
                set_route_state(state.clone());
                log::info!(
                    "[process directives] success to execute directive, seq: {}, directive: {:?}",
                    seq,
                    directive
                );
            }
            Err(err) => {
                log::error!("[process directives] failed to execute directive, seq: {}, directive: {:?}, err: {:?}", seq, directive, err);
                state.processing_directive.push((seq, directive));
                set_route_state(state.clone());
                break;
            }
        }
    }

    Ok(())
}
