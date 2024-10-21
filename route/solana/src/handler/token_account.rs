use std::str::FromStr;

use crate::constants::{IC_GATEWAY, RETRY_LIMIT_SIZE};
use crate::state::{AccountInfo, TxStatus, UpdateToken};

use ic_solana::types::{Pubkey, TransactionConfirmationStatus};

use crate::handler::solana_rpc::{self, create_mint_account, update_token_metadata};

use crate::state::{mutate_state, read_state};
use ic_canister_log::log;
use ic_solana::ic_log::{CRITICAL, DEBUG, ERROR};
use ic_solana::token::{SolanaClient, TokenInfo};
use ic_solana::eddsa::hash_with_sha256;

use super::solana_rpc::solana_client;


pub async fn create_token_mint() {
    let creating_token_mint = read_state(|s| {
        let mut creating_token_mint = vec![];
        for (token_id, token) in s.tokens.iter() {
            match s.token_mint_accounts.get(&token_id) {
                None => creating_token_mint.push(token.to_owned()),

                //filter account,not finallized and retry < RETRY_LIMIT_SIZE
                Some(account) => {
                    if !matches!(account.status, TxStatus::Finalized)
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
            uri: format!("https://{}.{}/token_uri?id={}",
            ic_cdk::api::id().to_text(),
            
            IC_GATEWAY,
            token.token_id.to_string()),
        };
        let mint_account = if let Some(account) =
            read_state(|s| s.token_mint_accounts.get(&token.token_id))
        {
            // Pubkey::from_str(&account.account).expect("Invalid to_account address")
            account

        } else {
            let derive_path = hash_with_sha256(token.token_id.as_str());
            let new_account_address = SolanaClient::derive_account(
                sol_client.chainkey_name.clone(),
                derive_path,
            )
            .await;
            log!(
                DEBUG,
                "[token_account::create_token_mint] token id({:}) mint account address derive from schonnor chainkey: {:?} ",
                token_info.token_id,new_account_address,
            );
            let new_account_info = AccountInfo {
                account: new_account_address.to_string(),
                retry: 0,
                signature: None,
                status: TxStatus::New,
            };
            //save inited account info
            mutate_state(|s| {
                s.token_mint_accounts
                    .insert(token.token_id.to_string(), new_account_info.clone())
            });

            // new_account
            new_account_info
        };

        log!(
            DEBUG,
            "[token_account::create_token_mint] token id({:}) mint_account_info from solana route: {:?} ",
            token_info.token_id,mint_account,

        );

        // check mint account exists on solana
        let mint_account_info = sol_client.get_account_info(mint_account.account.to_string()).await;
        log!(
            DEBUG,
            "[token_account::create_token_mint] token mint: {:?} account_info from solana: {:?} ",
            mint_account.account.to_string(),mint_account_info,
        );
        if let Ok(account_info) = mint_account_info {
            if matches!(account_info,Some(..)){
                let mint = AccountInfo {
                    account: mint_account.account.to_string(),
                    retry: mint_account.retry,
                    signature: mint_account.signature,
                    status: TxStatus::Finalized,
                };
                //update mint account info
                mutate_state(|s| {
                    s.token_mint_accounts
                        .insert(token.token_id.to_string(), mint)
                });
                //skip this mint account
                continue;

            }
            
        }

        match &mint_account.status {
            TxStatus::New => {
                match &mint_account.signature {
                    // not exists,need to create it
                    None => {
                        handle_creating_mint_account(
                            mint_account.account.to_string(),
                            token_info,
                        )
                        .await
                    }
                    // already created,but not finallized
                    Some(sig) => {
                        log!(
                            DEBUG,
                            "[token_account::create_token_mint] the token mint ({:?}) already submited and waiting for the tx({:}) to be finallized ... ",
                            mint_account,sig
                            
                        );

                        // update status
                        update_mint_account_status(sig.to_string(), token_info.token_id).await;
                    }
                }
            }
            TxStatus::Pending => {
                match &mint_account.signature {
                    // be creating
                    None => {
                        log!(
                            DEBUG,
                            "[token_account::create_token_mint] the token mint ({:?}) is creating, please waite ... ",
                            mint_account
                            
                        );
                    }
                    // already created,but not finallized
                    Some(sig) => {
                        log!(
                            DEBUG,
                            "[token_account::create_token_mint] the token mint ({:?}) already submited and waiting for the tx({:}) to be finallized ... ",
                            mint_account,sig
                            
                        );

                        // update status
                        update_mint_account_status(sig.to_string(), token_info.token_id).await;
                    }
                }
            
            
            }
            TxStatus::Finalized => {
                log!(
                    DEBUG,
                    "[token_account::create_token_mint] token id: {:} -> token mint account: {:?} Already finalized !",
                    token.token_id,mint_account,
                );
            }
            TxStatus::TxFailed { e } => {
                log!(
                    ERROR,
                    "[token_account::create_token_mint] failed to create mint token for {:},error:{:}, retry ..",
                    token.token_id,e.to_string()
                );
                // handle_creating_mint_account(mint_account.account.to_string(), token_info)
                //     .await;
                match &mint_account.signature {
                    // not exists,need to create it
                    None => {
                        handle_creating_mint_account(
                            mint_account.account.to_string(),
                            token_info,
                        )
                        .await
                    }
                    // already created,but not finallized
                    Some(sig) => {
                        log!(
                            DEBUG,
                            "[token_account::create_token_mint] the token mint ({:?}) was already submited and waiting for the tx({:}) to be finallized ... ",
                            mint_account,sig
                            
                        );

                        // update status
                        update_mint_account_status(sig.to_string(), token_info.token_id).await;
                    }
                }
            }
        }
    }
}

pub async fn handle_creating_mint_account(account_address: String, token_info: TokenInfo) {
    let mint_account = Pubkey::from_str(&account_address).expect("Invalid to_account address");
    match create_mint_account(mint_account, token_info.clone()).await {
        Ok(sig) => {
            log!(
                DEBUG,
                "[token_account::handle_creating_mint_account] create_mint_account signature: {:?} for {:}",
                sig.to_string(),mint_account.to_string()
            );
            // update account.signature and account.retry ,but not finalized
            mutate_state(|s| {
                if let Some(account) = s.token_mint_accounts
                    .get(&token_info.token_id).as_mut() {
                        //only this place ,update signature
                        account.signature = Some(sig.to_string());
                        account.retry += 1;
                        s.token_mint_accounts.insert(token_info.token_id.to_string(),account.to_owned());
                    }
                    
            });
            // update status
            // update_mint_account_status(sig.to_string(), token_info.token_id).await;
        }
        Err(e) => {
            log!(
                CRITICAL,
                "[token_account::handle_creating_mint_account] create token mint for {:}, error: {:?}  ",
                token_info.token_id,e
            );
            // update retry
            mutate_state(|s| {
        
                if let Some(account) = s.token_mint_accounts
                    .get(&token_info.token_id).as_mut() {
                        account.status = TxStatus::TxFailed { e: e.to_string() };
                        account.retry += 1;
                        //TODO: reset signature
                        account.signature = None;
                        s.token_mint_accounts.insert(token_info.token_id.to_string(),account.to_owned());
                    }
            });
        }
    }
}

pub async fn update_mint_account_status(sig: String, token_id: String) {
    // query signature status
    let tx_status_ret = solana_rpc::get_signature_status(vec![sig.to_string()]).await;
    match tx_status_ret {
        Err(e) => {
            log!(
                CRITICAL,
                "[token_account::update_mint_account_status] get_signature_status for {} ,err: {:?}",
                sig.to_string(),
                e
            );
            //TOOD: update account info and retry?
            mutate_state(|s| {
                if let Some(account) = s.token_mint_accounts
                    .get(&token_id).as_mut() {
                        account.status = TxStatus::TxFailed { e: e.to_string() };
                        account.retry += 1;
                        //reset signature
                        account.signature = None;
                        s.token_mint_accounts.insert(token_id.to_string(),account.to_owned());
                    }
            });
        }
        Ok(status_vec) => {
            status_vec.first().map(|tx_status| {
                log!(
                    DEBUG,
                    "[token_account::update_mint_account_status] signature {} status : {:?} ",
                    sig.to_string(),
                    tx_status,
                );
                if let Some(status) = &tx_status.confirmation_status {
                    if matches!(status, TransactionConfirmationStatus::Finalized) {
                        // update account status to Finalized
                        mutate_state(|s| {
                            if let Some(account) = s.token_mint_accounts
                            .get(&token_id).as_mut() {
                                account.status = TxStatus::Finalized;
                                s.token_mint_accounts.insert(token_id.to_string(),account.to_owned());
                            }
                        });
                    }
                }
            });
        }
    }
}

pub async fn update_token() {
    // log!(DEBUG,  "[token_account::update_token] timer to execute update_token ");
       
    let update_tokens = read_state(|s| {
        s.update_token_queue
            .iter()
            .take(5)
            .map(|(token_id, update_token)| {
                (token_id.to_owned(), update_token)
            })
            .collect::<Vec<_>>()
    });

    for (token_id, update_token) in update_tokens.into_iter() {
        // limit retry to RETRY_LIMIT_SIZE
        if update_token.retry >= RETRY_LIMIT_SIZE {
            continue;
        }
        let account_info = read_state(|s| s.token_mint_accounts.get(&token_id));
        if let Some(account_info) = account_info {
            log!(DEBUG,"[token_account::update_token] token mint info : {:?} ",account_info);
            //query token metadata from solana chain and comparison metadata with new metadata
            // if not eq, execute update token metadata
            let token_update_info = TokenInfo {
                token_id: token_id.to_string(),
                name: update_token.token.name.to_owned(),
                symbol: update_token.token.symbol.to_owned(),
                decimals: update_token.token.decimals,
                uri: update_token.token.icon.to_owned().unwrap_or_default(),
            };
            log!(DEBUG,"[token_account::update_token] token_update_info: {:?} ",token_update_info);
            match update_token_metadata(account_info.account, token_update_info).await {
                Ok(signature) => {
                    log!(DEBUG,"[token_account::update_token]  update token metadata for {:?} already submited to solana and waiting for the tx({:}) to be finallized ...",
                    update_token.token.token_id.to_string(),
                    signature
                );
                    //TODO: check signature status
                    mutate_state(|s| {
                        // update the token info in route
                        s.add_token(update_token.token.to_owned());
                        // remove the updated token from queue
                        s.update_token_queue.remove(&token_id)
                    });
                }
                Err(e) => {
                    log!(
                        CRITICAL,
                        "[token_account::update_token] update token metadata error: {:?}  ",
                        e
                    );
                    mutate_state(|s| {
                        // update the retry num
                        let retry = update_token.retry + 1;
                        let update_token = UpdateToken::new(update_token.token.to_owned(), retry);
                        // update the token info in route
                        // remove the updated token from queue
                        s.update_token_queue.insert(token_id, update_token)
                    });
                }
            }
        } else {
            log!(
                ERROR,
                "[token_account::update_token] not found token mint for token id : {:?}",
                update_token.token.token_id
            );
        }
    }
}
