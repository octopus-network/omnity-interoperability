use std::str::FromStr;

use ic_solana::token::associated_account::get_associated_token_address_with_program_id;
use ic_solana::token::constants::token22_program_id;
use ic_solana::types::Pubkey;

use super::solana_rpc::{self, create_ata};
use crate::handler::solana_rpc::solana_client;
use crate::state::{AccountInfo, AtaKey};
use crate::state::TxStatus;
use crate::state::{mutate_state, read_state};

use ic_solana::types::TransactionConfirmationStatus;

use crate::constants::{COUNTER_SIZE, RETRY_LIMIT_SIZE};
use ic_canister_log::log;
use ic_solana::ic_log::{CRITICAL, DEBUG, ERROR};

pub async fn create_associated_account() {
 
    let mut creating_atas = vec![];
    read_state(|s| {
        for (_seq, ticket) in s.tickets_queue.iter() {

            if let Some(token_mint) = s.token_mint_accounts.get(&ticket.token) {
                //the token mint account must be Finalized
                if matches!(token_mint.status,TxStatus::Finalized){
                    match s
                    .associated_accounts
                    .get(&
                    AtaKey{owner:ticket.receiver.to_string(), token_mint:token_mint.account.to_string()}
                    )
                {
                    None => creating_atas.push((ticket.receiver.to_owned(), token_mint.to_owned())),
                    Some(ata) => {
                        //filter account,unconformed and retry < RETRY_LIMIT_SIZE
                        if !matches!(ata.status, TxStatus::Finalized) && ata.retry < RETRY_LIMIT_SIZE {
                            creating_atas.push((ticket.receiver.to_owned(), token_mint.to_owned()))
                        }
                    }
                }
                }
            }
        }
    });

    let mut count = 0u64;
    // let sol_client = solana_client().await;
    for (owner, token_mint) in creating_atas.into_iter() {
        let to_account_pk = Pubkey::from_str(owner.as_str()).expect("Invalid to_account address");
        let token_mint_pk =
            Pubkey::from_str(token_mint.account.as_str()).expect("Invalid token_mint address");

        let associated_account = if let Some(account) = read_state(|s| {
            s.associated_accounts
                .get(&AtaKey{owner:owner.to_string(), token_mint:token_mint.account.to_string()}
            )
                
        }) {
            // Pubkey::from_str(&account.account).expect("Invalid to_account address")

            account
        } else {
            let associated_account = get_associated_token_address_with_program_id(
                &to_account_pk,
                &token_mint_pk,
                &token22_program_id(),
            );
            log!(
                DEBUG,  
                "[associated_account::create_associated_account] native associated_account: {:?} for {:?} and {:?}",
                associated_account,owner,token_mint.account
            );
            let new_account_info = AccountInfo {
                account: associated_account.to_string(),
                retry: 0,
                signature: None,
                status: TxStatus::New,
            };
            //save inited account info
            mutate_state(|s| {
                s.associated_accounts.insert(
                    AtaKey{owner:owner.to_string(), token_mint:token_mint.account.to_string()},
                    new_account_info.clone(),
                )
            });
            // associated_account
            new_account_info
        };

     
        log!(
            DEBUG,
            "[associated_account::create_associated_account] ata_account_info from solana route : {:?} ",
            associated_account);
       
         // check ATA exists on solana
         let sol_client = solana_client().await;
           let ata_account_info = sol_client
            .get_account_info(associated_account.account.to_string())
            .await;
         log!(
             DEBUG,
             "[associated_account::create_associated_account] ata_account_info: {:?} account info from solana: {:?} ",
             associated_account,ata_account_info,
         );
         if let Ok(account_info) = ata_account_info {
             if matches!(account_info,Some(..)){
                 let ata = AccountInfo {
                     account: associated_account.account.to_string(),
                     retry: associated_account.retry,
                     signature: associated_account.signature,
                     status: TxStatus::Finalized,
                 };
                 //update ata info
                 mutate_state(|s| {
                    s.associated_accounts.insert(
                        
                        AtaKey{owner:owner.to_string(), token_mint:token_mint.account.to_string()},
                        ata,
                    )
                });
                 //skip this ata
                 continue;
 
             }
             
         }
        
        match &associated_account.status {
            TxStatus::New => {
                match &associated_account.signature {
                     // not exists,create it
                    None => {
                       handle_creating_ata(owner.to_owned(), token_mint.account.to_string()).await;
                    }
                    Some(sig) => {
                        log!(
                            DEBUG,
                            "[associated_account::create_associated_account] The ata ({:?}) already submited and waiting for the tx({:?}) to be finallized! ",
                            associated_account.account.to_string(),
                            sig
                        );
                        // update ata status
                        update_ata_status(sig.to_string(),owner.to_string(),token_mint.account.to_string()).await;
                    }
                }
            }
            TxStatus::Pending => {
                match &associated_account.signature {
                    // nothing to do ,just waiting
                   None => {
                    //   handle_creating_ata(owner.to_owned(), token_mint.account.to_string()).await;
                    log!(
                        DEBUG,
                        "[associated_account::create_associated_account] the associated account ({:?}) is creating,pls wait ...",
                        associated_account
                    );
                   }
                   Some(sig) => {
                       log!(
                           DEBUG,
                           "[associated_account::create_associated_account] the ata {:?} already submited and waiting for the tx({:?}) to be finallized! ",
                           associated_account.account.to_string(),
                           sig
                       );
                       // update ata status
                       update_ata_status(sig.to_string(),owner.to_string(),token_mint.account.to_string()).await;
                   }
               }
            }
            TxStatus::Finalized => {
                log!(
                    DEBUG,
                    "[associated_account::create_associated_account] {:?} already finalized !",
                    associated_account,
                );
            }
            TxStatus::TxFailed { e } => {
                log!(
                    ERROR,
                   "[associated_account::create_associated_account] failed to create_associated_account for owner: {} and token mint: {}, error: {:}",
                   owner,token_mint.account,e
                );
                // handle_creating_ata(owner.to_string(), token_mint.account.to_string()).await;
                match &associated_account.signature {
                    // not exists,create it
                   None => {
                      handle_creating_ata(owner.to_owned(), token_mint.account.to_string()).await;
                   }
                   Some(sig) => {
                       log!(
                           DEBUG,
                           "[associated_account::create_associated_account] the ata {:?} already submited and waiting for the tx({:?}) to be finallized! ",
                           associated_account.account.to_string(),
                           sig
                       );
                       // update ata status
                       update_ata_status(sig.to_string(),owner.to_string(),token_mint.account.to_string()).await;
                   }
               }
            }
        }

        // Control foreach size, if >= COUNTER_SIZE, then break
        count += 1;
        if count >= COUNTER_SIZE {
            break;
        }

    }
}

pub async fn handle_creating_ata(owner:String,mint_address:String) {

    match create_ata(owner.to_string(), mint_address.to_string()).await {
        Ok(sig) => {
            log!(
                DEBUG,
                "[associated_account::handle_creating_ata] create_ata signature : {:?}",
                sig
            );
            // update account created signature and retry ,but not confirmed
            mutate_state(|s| {
  
                let ata_key = AtaKey{owner:owner.to_string(), token_mint:mint_address.to_string()};
                if let Some(account) = s.associated_accounts
                .get(&ata_key              
                 ).as_mut() {
                    account.signature = Some(sig.to_string());
                    account.retry += 1;
                    s.associated_accounts.insert(ata_key, account.to_owned());

                }
                
            });
             // update ata status
            //  update_ata_status(sig.to_string(),owner.to_string(),mint_address.to_string()).await;
        }
        Err(e) => {
            log!(
                CRITICAL,
                "[associated_account::handle_creating_ata] create_ata for owner: {:} and token_mint: {:}, error: {:?}  ",
                owner.to_string(), mint_address.to_string(), e
            );
           // update account retry 
            mutate_state(|s| {
                let ata_key = AtaKey{owner:owner.to_string(), token_mint:mint_address.to_string()};
                if let Some(account)= s.associated_accounts
                    .get(& ata_key).as_mut() {
                        account.status =
                            TxStatus::TxFailed { e: e.to_string() };
                        account.retry += 1;
                        //reset signature
                        account.signature = None;
                         s.associated_accounts.insert(ata_key, account.to_owned());
                }
                   
            });
           
        }
    }

}

pub async fn update_ata_status(sig:String,owner:String,mint_address:String) {
    let tx_status_ret =
    solana_rpc::get_signature_status(vec![sig.to_string()]).await;
   match tx_status_ret {
    Err(e) => {
        log!(
            CRITICAL,
             "[associated_account::update_ata_status] get_signature_status for {} ,err: {:?}",
             sig.to_string(),
             e
         );
        
       //TOOD: update account and retry ?
       mutate_state(|s| {
        let ata_key = AtaKey{owner:owner.to_string(), token_mint:mint_address.to_string()};
        if let Some(account)= s.associated_accounts
            .get(& ata_key).as_mut() {
                account.status =
                    TxStatus::TxFailed { e: e.to_string() };
                account.retry += 1;
                //reset signature
                account.signature = None;
                s.associated_accounts.insert(ata_key, account.to_owned());
        }
           
        });
    }
    Ok(status_vec) => {
        status_vec.first().map(|tx_status| {
             log!(
                 DEBUG,
                 "[associated_account::update_ata_status] signature {}  status : {:?} ",
                 sig.to_string(),
                 tx_status,
             );
             if let Some(status) = &tx_status.confirmation_status {
                 if matches!(status, TransactionConfirmationStatus::Finalized) {
                    // update account status to Finalized
                    mutate_state(|s| {
                        let ata_key=AtaKey{owner:owner.to_string(), token_mint:mint_address.to_string()};
                       if let Some(account)  = s.associated_accounts
                            .get(&ata_key).as_mut() {
                                account.status = TxStatus::Finalized;
                                s.associated_accounts.insert(ata_key, account.to_owned());
                            }
                           
                    });
                 }
             }
         });
    }
}
}


