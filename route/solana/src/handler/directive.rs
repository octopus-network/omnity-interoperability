use std::str::FromStr;

use crate::constants::RETRY_LIMIT_SIZE;
use crate::state::{AccountInfo, TxStatus};
use crate::types::{ChainId, Directive, Error, Seq, Topic};
use candid::Principal;
use ic_solana::types::{Pubkey, TransactionConfirmationStatus};

use crate::handler::sol_call::{self, create_mint_account, update_token_metadata};

use crate::{
    call_error::{CallError, Reason},
    state::{mutate_state, read_state},
};
use ic_canister_log::log;
use ic_solana::logs::{DEBUG, ERROR};
use ic_solana::token::{SolanaClient, TokenInfo};

use super::sol_call::solana_client;

pub const DIRECTIVE_LIMIT_SIZE: u64 = 20;

/// query directives from hub and save to route state
pub async fn query_directives() {
    let (hub_principal, offset) = read_state(|s| (s.hub_principal, s.next_directive_seq));
    match inner_query_directives(hub_principal, offset, DIRECTIVE_LIMIT_SIZE).await {
        Ok(directives) => {
            for (_, directive) in &directives {
                match directive {
                    Directive::AddChain(chain) | Directive::UpdateChain(chain) => {
                        mutate_state(|s| s.add_chain(chain.to_owned()));
                    }

                    Directive::AddToken(token) => {
                        mutate_state(|s| s.add_token(token.to_owned()));
                    }
                    Directive::UpdateToken(update_token) => {
                        let t = read_state(|s| s.tokens.get(&update_token.token_id).cloned());
                        match t {
                            None => mutate_state(|s| s.add_token(update_token.to_owned())),
                            //if update_token, need to update solana token metadata
                            Some(current_token) => {
                                log!(
                                    DEBUG,
                                    "[Directive::UpdateToken] need to update token metadata for :{:?} ",
                                    current_token,
                                );
                                mutate_state(|s| {
                                    s.update_token_queue.insert(
                                        update_token.token_id.to_string(),
                                        (update_token.to_owned(), 0),
                                    )
                                });
                            }
                        }
                    }
                    Directive::ToggleChainState(toggle) => {
                        mutate_state(|s| s.toggle_chain_state(toggle.to_owned()));
                    }
                    Directive::UpdateFee(fee) => {
                        mutate_state(|s| s.update_fee(fee.to_owned()));
                    }
                }
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

pub async fn inner_query_directives(
    hub_principal: Principal,
    offset: u64,
    limit: u64,
) -> Result<Vec<(Seq, Directive)>, CallError> {
    let resp: (Result<Vec<(Seq, Directive)>, Error>,) = ic_cdk::api::call::call(
        hub_principal,
        "query_directives",
        (
            None::<Option<ChainId>>,
            None::<Option<Topic>>,
            offset,
            limit,
        ),
    )
    .await
    .map_err(|(code, message)| CallError {
        method: "query_directives".to_string(),
        reason: Reason::from_reject(code, message),
    })?;
    let data = resp.0.map_err(|err| CallError {
        method: "query_directives".to_string(),
        reason: Reason::CanisterError(err.to_string()),
    })?;
    Ok(data)
}

pub async fn create_token_mint() {
    let creating_token_mint = read_state(|s| {
        let mut creating_token_mint = vec![];
        for (token_id, token) in s.tokens.iter() {
            match s.token_mint_accounts.get(token_id) {
                None => creating_token_mint.push(token.to_owned()),

                //filter account,not finallized and retry < RETRY_LIMIT_SIZE
                Some(account) => {
                    if !matches!(account.status, TxStatus::Finalized { .. })
                        && account.retry < RETRY_LIMIT_SIZE
                    {
                        creating_token_mint.push(token.to_owned())
                    }
                }
            }
        }
        creating_token_mint
    });
    let sol_client = solana_client().await;
    for token in creating_token_mint.into_iter() {
        let token_info = TokenInfo {
            token_id: token.token_id.to_string(),
            name: token.name,
            symbol: token.symbol,
            decimals: token.decimals,
            uri: token.icon.unwrap_or_default(),
        };
        let mint_account_account = if let Some(account) =
            read_state(|s| s.token_mint_accounts.get(&token.token_id).cloned())
        {
            // Pubkey::from_str(&account.account).expect("Invalid to_account address")
            account
        } else {
            let new_account_address = SolanaClient::derive_account(
                sol_client.schnorr_canister.clone(),
                sol_client.chainkey_name.clone(),
                token_info.token_id.to_string(),
            )
            .await;
            log!(
                DEBUG,
                "[directive::create_token_mint] token id({:}) mint account address derive from schonnor chainkey: {:?} ",
                token_info.token_id,new_account_address,
            );
            let new_account_info = AccountInfo {
                account: new_account_address.to_string(),
                retry: 0,
                signature: None,
                status: TxStatus::Unknown,
            };
            //save inited account info
            mutate_state(|s| {
                s.token_mint_accounts
                    .insert(token.token_id.to_string(), new_account_info.clone())
            });

            // new_account
            new_account_info
        };

        // query mint account from solana
        // let mint_account_info = sol_client.get_account_info(mint_account.to_string()).await;
        log!(
            DEBUG,
            "[directive::create_token_mint] token id({:}) mint_account_info from solana route: {:?} ",
            token_info.token_id,mint_account_account,
        );
        // retry < RETRY_LIMIT_SIZE,or skip
        // if mint_account_account.retry >= RETRY_LIMIT_SIZE {
        //     continue;
        // }
        match &mint_account_account.status {
            TxStatus::Unknown => {
                match &mint_account_account.signature {
                    // not exists,need to create it
                    None => {
                        handle_creating_mint_account(
                            mint_account_account.account.to_string(),
                            token_info,
                        )
                        .await
                    }
                    // already created,but not finallized
                    Some(sig) => {
                        log!(
                            DEBUG,
                            "[directive::create_token_mint] {:?} already created and waiting for {:} finallized ... ",
                            mint_account_account,sig
                            
                        );

                        // update status
                        update_mint_account_status(sig.to_string(), token_info.token_id).await;
                    }
                }
            }
            TxStatus::Finalized { .. } => {
                log!(
                    DEBUG,
                    "[directive::create_token_mint] token id: {:} -> token mint account: {:?} Already finalized !",
                    token.token_id,mint_account_account,
                );
            }
            TxStatus::TxFailed { .. } => {
                log!(
                    ERROR,
                    "[directive::create_token_mint] failed to create mint token for {:}, retry ..",
                    token.token_id
                );
                handle_creating_mint_account(mint_account_account.account.to_string(), token_info)
                    .await
            }
        }
    }
}

pub async fn handle_creating_mint_account(account_address: String, token_info: TokenInfo) {
    let mint_account = Pubkey::from_str(&account_address).expect("Invalid to_account address");
    match create_mint_account(mint_account, token_info.clone()).await {
        Ok(signature) => {
            log!(
                DEBUG,
                "[directive::handle_creating_mint_account] create_mint_account signature: {:?} for {:}",
                signature.to_string(),mint_account.to_string()
            );
            // update account.signature and account.retry ,but not finalized
            mutate_state(|s| {
                s.token_mint_accounts
                    .get_mut(&token_info.token_id)
                    .map(|account| {
                        account.signature = Some(signature);
                        account.retry += 1;
                    })
            });
        }
        Err(e) => {
            log!(
                ERROR,
                "[directive::handle_creating_mint_account] create token mint error: {:?}  ",
                e
            );
            // update retry
            mutate_state(|s| {
                s.token_mint_accounts
                    .get_mut(&token_info.token_id)
                    .map(|account| {
                        account.status = TxStatus::TxFailed { e: e.to_string() };
                        account.retry += 1;
                    })
            });
        }
    }
}

pub async fn update_mint_account_status(sig: String, token_id: String) {
    // query signature status
    let tx_status_ret = sol_call::get_signature_status(vec![sig.to_string()]).await;
    match tx_status_ret {
        Err(e) => {
            log!(
                ERROR,
                "[directive::update_mint_account_status] get_signature_status for {} ,err: {:?}",
                sig.to_string(),
                e
            );
        }
        Ok(status_vec) => {
            status_vec.first().map(|tx_status| {
                log!(
                    DEBUG,
                    "[directive::update_mint_account_status] signature {} status : {:?} ",
                    sig.to_string(),
                    tx_status,
                );
                if let Some(status) = &tx_status.confirmation_status {
                    if matches!(status, TransactionConfirmationStatus::Finalized) {
                        // update account status to Finalized
                        mutate_state(|s| {
                            s.token_mint_accounts.get_mut(&token_id).map(|account| {
                                account.status = TxStatus::Finalized {
                                    signature: sig.to_string(),
                                };
                            })
                        });
                    }
                }
            });
        }
    }
}

pub async fn update_token() {
    let update_tokens = read_state(|s| {
        s.update_token_queue
            .iter()
            .take(5)
            .map(|(token_id, (token, retry))| {
                (token_id.to_owned(), (token.to_owned(), retry.to_owned()))
            })
            .collect::<Vec<_>>()
    });

    for (token_id, (token, retry)) in update_tokens.into_iter() {
        // limit retry to RETRY_LIMIT_SIZE
        if retry >= RETRY_LIMIT_SIZE {
            continue;
        }
        let account_info = read_state(|s| s.token_mint_accounts.get(&token_id).cloned());
        if let Some(account_info) = account_info {
            //query token metadata from solana chain and comparison metadata with new metadata
            // if not eq, execute update token metadata
            let token_update_info = TokenInfo {
                token_id: token_id.to_string(),
                name: token.name.to_owned(),
                symbol: token.symbol.to_owned(),
                decimals: token.decimals,
                uri: token.icon.to_owned().unwrap_or_default(),
            };

            match update_token_metadata(account_info.account, token_update_info).await {
                Ok(signature) => {
                    log!(DEBUG,"[directive::update_token] {:?} update token metadata on solana sucessfully ! \n{:?} ",
                    token.token_id.to_string(),
                    signature
                );
                    //TODO: check signature status
                    mutate_state(|s| {
                        // update the token info
                        s.add_token(token.to_owned());
                        // remove the updated token from queue
                        s.update_token_queue.remove(&token_id)
                    });
                }
                Err(e) => {
                    log!(
                        ERROR,
                        "[directive::update_token] update token metadata error: {:?}  ",
                        e
                    );
                    mutate_state(|s| {
                        // update the retry num
                        let retry = retry + 1;
                        // remove the updated token from queue
                        s.update_token_queue.insert(token_id, (token, retry))
                    });
                }
            }
        } else {
            log!(
                ERROR,
                "[directive::update_token] not found token mint for token id : {:?}",
                token.token_id
            );
        }
    }
}
