use business::add_new_token::add_new_token;
use state::read_state;

use crate::*;
use omnity_types::Directive;

pub fn process_directive_msg_task() {
    ic_cdk::spawn(async {
        // Considering that the directive is queried once a minute, guard protection is not needed.
        process_directives().await;
    });
}

async fn process_directives() {
    let (hub_principal, offset) = read_state(|s| (s.hub_principal, s.next_directive_seq));
    match hub::query_directives(hub_principal, offset, BATCH_QUERY_LIMIT).await {
        Ok(directives) => {
            for (_, directive) in &directives {
                match directive {
                    Directive::AddChain(chain) | Directive::UpdateChain(chain) => {
                        mutate_state(|s| add_chain( chain.clone()));
                    }
                    Directive::AddToken(token) | Directive::UpdateToken(token) => {
                        match add_new_token(token.clone()).await {
                            Ok(_) => {
                                log::info!(
                                    "[process directives] add token successful, token id: {}",
                                    token.token_id
                                );
                            }
                            Err(err) => {
                                log::error!(
                                    "[process directives] failed to add token: token id: {}, err: {:?}",
                                    token.token_id,
                                    err
                                );
                            }
                        }
                    }
                    Directive::ToggleChainState(toggle) => {
                        mutate_state(|s| toggle_chain_state(toggle.clone()));
                    }
                    Directive::UpdateFee(fee) => {
                        mutate_state(|s| update_fee(s, fee.clone()));
                        log::info!("[process_directives] success to update fee, fee: {}", fee);
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
