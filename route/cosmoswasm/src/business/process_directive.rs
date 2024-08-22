use cosmoswasm::{
    port::{PortContractExecutor, DIRECTIVE_EXECUTED_EVENT_KIND},
    rpc::{response::TxCommitResponse, wrapper::Wrapper},
    TxHash,
};
use memory::{mutate_state, read_state};
use tendermint::abci::{Event, EventAttributeIndexExt};

use crate::*;
use omnity_types::Directive;

pub fn process_directive_msg_task() {
    ic_cdk::spawn(async {
        // Considering that the directive is queried once a minute, guard protection is not needed.
        process_directives().await;
    });
}

pub const BATCH_QUERY_LIMIT: u64 = 20;
async fn process_directives() {
    let (hub_principal, offset) = read_state(|s| (s.hub_principal, s.next_directive_seq));
    let port_contract_executor = PortContractExecutor::from_state();
    match hub::query_directives(hub_principal, offset, BATCH_QUERY_LIMIT).await {
        Ok(directives) => {
            for (seq, directive) in &directives {
                match port_contract_executor
                    .execute_directive(seq.clone(), directive.clone().into())
                    .await
                {
                    Ok(_) => {
                        mutate_state(|s| {
                            s.next_directive_seq = seq + 1;
                        });
                        log::info!("[process directives] success to execute directive, seq: {}, directive: {:?}", seq, directive);
                    }
                    Err(err) => {
                        log::error!("[process directives] failed to execute directive, seq: {}, directive: {:?}, err: {:?}", seq, directive, err);
                    }
                }
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

pub async fn execute_directive(seq: Seq, directive: Directive) -> Result<TxHash> {
    let msg = ExecuteMsg::ExecDirective {
        seq,
        directive: directive.into(),
    };

    let client = CosmosWasmClient::cosmos_wasm_port_client();

    let contract_id = get_contract_id();

    let public_key_response = query_cw_public_key().await?;

    let tendermint_public_key: tendermint::PublicKey =
        tendermint::public_key::PublicKey::from_raw_secp256k1(
            public_key_response.public_key.as_slice(),
        )
        .unwrap();

    let response = client
        .execute_msg(contract_id, msg, tendermint_public_key)
        .await?;

    let wrapper: Wrapper<TxCommitResponse> =
        serde_json::from_slice(response.body.as_slice()).unwrap();

    assert!(wrapper.error.is_none(), "Error: {:?}", wrapper.error);
    let result: TxCommitResponse = wrapper.into_result()?;

    let expect_event = Event::new(
        DIRECTIVE_EXECUTED_EVENT_KIND,
        [("sequence", seq.to_string()).no_index()],
    );
    result.assert_event_exist(&expect_event)?;

    Ok(result.hash.to_string())
}
