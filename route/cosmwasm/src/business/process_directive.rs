use cosmwasm::port::PortContractExecutor;
use memory::{set_route_state, take_state};

use crate::*;
use omnity_types::ChainState;

pub fn process_directive_task() {
    ic_cdk::spawn(async {
        let _guard = match crate::guard::TimerLogicGuard::new(
            const_args::FETCH_HUB_DIRECTIVE_NAME.to_string(),
        ) {
            Some(guard) => guard,
            None => return,
        };
        match process_directives().await {
            Ok(_) => {}
            Err(e) => {
                log::error!("failed to process directives, err: {:?}", e);
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
