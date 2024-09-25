
use crate::handler::solana_rpc;
use crate::types::{ Error,TicketId};
use candid::{CandidType, Principal};

use ic_stable_structures::Storable;
use ic_stable_structures::storable::Bound;
use std::borrow::Cow;
use serde::{Deserialize, Serialize};

use crate::state::{AtaKey};
use crate::state::TxStatus;
use crate::{
    call_error::{CallError, Reason},
    state::{mutate_state, read_state},
};
use ic_solana::types::{ TransactionConfirmationStatus};

use crate::constants::{ RETRY_LIMIT_SIZE};
use ic_canister_log::log;
use ic_solana::ic_log::{ERROR, DEBUG,CRITICAL};


#[derive(CandidType, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum MintTokenError {
    NotFoundToken(String),
    UnsupportedToken(String),
    AlreadyProcessed(TicketId),
    TemporarilyUnavailable(String),
}
#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct MintTokenRequest {
    pub ticket_id: TicketId,
    pub associated_account: String,
    pub amount: u64,
    pub token_mint: String,
    pub status: TxStatus,
    pub signature: Option<String>,
    pub retry:u64,
}

impl Storable for MintTokenRequest {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        let mut bytes = vec![];
        let _ = ciborium::ser::into_writer(self, &mut bytes);
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        let cm = ciborium::de::from_reader(bytes.as_ref()).expect("failed to decode ChainMeta");
        cm
    }

    const BOUND: Bound = Bound::Unbounded;
}

pub async fn mint_token() {
    // take 5 tickets to mint,very time
    let tickets = read_state(|s| {
        s.tickets_queue
            .iter()
            .take(5)
            .map(|(seq, ticket)| (seq, ticket))
            .collect::<Vec<_>>()
    });
    
    // TODO: check mint_token_requests 
    // if ticket id already exits in mint_token_requests,skip it
    
    for (seq, ticket) in tickets.into_iter() {
        let token_mint = read_state(|s| s.token_mint_accounts.get(&ticket.token));
        let token_mint = match token_mint {
            Some(token_mint) => {
                if !matches!(token_mint.status,TxStatus::Finalized) {
                    log!(DEBUG,
                        "[mint_token::mint_token] token_mint ({:?}) not finalized, waiting for finalized ...",
                        token_mint
                    );
                    continue;
                }
                token_mint
            }
            None => {
                log!(DEBUG,
                 "[mint_token::mint_token] the token({:}) mint account is not exists, waiting for create token mint ...",
                 ticket.token
             );
                continue;
            }
        };

        // check ata
        let associated_account = read_state(|s| {
            s.associated_accounts
                .get(&
                AtaKey{owner:ticket.receiver.to_owned(), token_mint: token_mint.account.to_owned()}
            )
               
        });
        let associated_account = match associated_account {
            Some(associated_account) => {
                if !matches!(associated_account.status,TxStatus::Finalized) {
                    log!(DEBUG,
                        "[mint_token::mint_token] associated_account ({:?}) not finalized, waiting for finalized ...",
                        associated_account
                    );
                    continue;
                }
                associated_account
            }
            None => {
                log!(DEBUG,
                 "[mint_token::mint_token] the associated_account for owner: ({}) and token_mint: ({}) is not exists, waiting for create associated account ...",
                 ticket.receiver.to_string(),token_mint.account.to_string()
             );
                continue;
            }
        };

        let mint_req = if let Some(req) =
            read_state(|s| s.mint_token_requests.get(&ticket.ticket_id))
        {
            req
        } else {
           let mint_req= MintTokenRequest {
                ticket_id: ticket.ticket_id.to_owned(),
                associated_account: associated_account.account.to_owned(),
                amount: ticket.amount.parse::<u64>().unwrap(),
                token_mint: token_mint.account,
                status: TxStatus::New,
                signature: None,
                retry:0
            };
            // save new token req
            mutate_state(|s| s.update_mint_token_req(mint_req.ticket_id.to_string(), mint_req.clone()));
            mint_req
        };
        log!(DEBUG, "[mint_token::mint_token] mint token request: {:?} ", mint_req);

        // retry < RETRY_LIMIT_SIZE,or skip
        if mint_req.retry >= RETRY_LIMIT_SIZE {
            continue;
        }
    
        match &mint_req.status {

            TxStatus::New => {
                match mint_req.signature.to_owned() {
                    //new mint req,mint_token
                    None => {
                        handle_mint_token(mint_req).await;
                    },
                    Some(sig) => {
                        log!(
                            DEBUG,
                            "[mint_token::mint_token] mint req tx({:?}) already submited and waiting for the tx({:?}) to be finallized! ",
                          mint_req,sig
                        );
                        update_mint_token_req(mint_req.to_owned(), sig).await;
                    }
                }
              
            },
            TxStatus::Pending => {
                match mint_req.signature.to_owned() {
                    
                    None => {
                        // handle_mint_token(mint_req).await;
                        log!(
                            DEBUG,
                            "[mint_token::mint_token] the mint token request ({:?}) is handling, pls wait ...",
                            mint_req
                        );
                    },
                    Some(sig) => {
                        log!(
                            DEBUG,
                            "[mint_token::mint_token] mint req tx({:?}) already submited and waiting for the tx({:?}) to be finallized! ",
                          mint_req,sig
                        );
                        update_mint_token_req(mint_req.to_owned(), sig).await;
                    }
                }
            }
            TxStatus::Finalized  => {
                // update txhash to hub
               let hub_principal = read_state(|s| s.hub_principal);
               let sig = mint_req.signature.unwrap();
               if let Err(err) =
                   update_tx_hash(hub_principal, ticket.ticket_id.to_string(), sig).await
               {
                   log!(
                       ERROR,
                       "[tickets::mint_token] failed to update tx hash after mint token:{}",
                       err
                   );
               }
                //only finalized mint_req, remove the handled ticket from queue
                mutate_state(|s| s.tickets_queue.remove(&seq));
                                      
           }
            TxStatus::TxFailed { e } => {
                log!(
                    ERROR,
                   "[mint_token::mint_token] failed to mint token for ticket id: {}, error: {:} and retry mint ..",
                    ticket.ticket_id,e 
                );
                 //retry mint_to ?
                //  handle_mint_token(mint_req).await;
 
            },
            
        }
         
    }
}

pub async fn handle_mint_token(mint_req: MintTokenRequest){
    match mint_to(mint_req.to_owned()).await {
        Ok(signature) => {
            log!(
                DEBUG,
                "[mint_token::mint_token] mint token req was submited for ticket id: {} and signature is :{}",
                mint_req.ticket_id.to_string(),
                signature
            );
            // update req signature,but not comfirmed
            mutate_state(|s| {
   
                if let Some(req)=s.mint_token_requests.get(&mint_req.ticket_id).as_mut() {
                    req.signature = Some(signature.to_string());
                    req.retry +=1;
                    s.mint_token_requests.insert(mint_req.ticket_id.to_string(),req.to_owned());
                }
            });
            // remove the handled ticket from queue
            // mutate_state(|s| s.tickets_queue.remove(&seq));

        }
        Err(e) => {
            let err_info = format!( "[mint_token::mint_token] failed to mint token for ticket id: {}, err: {:?}",
            mint_req.ticket_id,e);
            log!(CRITICAL,"{}", err_info.to_string());
            // if err, update req status 
            mutate_state(|s| {
                    if let Some(req)=s.mint_token_requests.get(&mint_req.ticket_id).as_mut() {
                        req.status =TxStatus::TxFailed { e: err_info };
                        req.retry +=1;
                        //reset signature
                        req.signature = None;
                        s.mint_token_requests.insert(mint_req.ticket_id.to_string(),req.to_owned());
                    }
                });
            
            // remove the handled ticket from queue,don`t retry 
            // mutate_state(|s| s.tickets_queue.remove(&seq));
         
        }
    }
}

pub async fn update_mint_token_req(mut mint_req:MintTokenRequest,sig:String) {
    // query signature status
    let tx_status_ret = solana_rpc::get_signature_status(vec![sig.to_string()]).await;
    match tx_status_ret {
        Err(e) => {
            log!(
                ERROR,
                "[mint_token::update_mint_token_req] get_signature_status for {} ,err: {:?}",
                sig.to_string(),
                e
            );
            //TOOD: retry?
            mutate_state(|s| {
               if let Some(req)=s.mint_token_requests.get(&mint_req.ticket_id).as_mut() {
                   req.status =TxStatus::TxFailed { e: e.to_string() };
                   req.retry +=1;
                   //reset signature
                   // req.signature = None;
                   s.mint_token_requests.insert(mint_req.ticket_id.to_string(),req.to_owned());
               }
           });
           
        }
        Ok(status_vec) => {
            status_vec.first().map(|tx_status| {
                log!(
                    DEBUG,
                    "[mint_token::update_mint_token_req] signature {}  status : {:?} ",
                    sig.to_string(),
                    tx_status,
                );
                if let Some(status) = &tx_status.confirmation_status {
                    if matches!(status, TransactionConfirmationStatus::Finalized) {
                        // update mint token req status
                        mint_req.status = TxStatus::Finalized;
                        mutate_state(|s| {
                            s.update_mint_token_req(mint_req.ticket_id.to_owned(), mint_req)
                        });
    
                    }
                }
            });
        
        }
    }

}

/// send tx to solana for mint token
pub async fn mint_to( req: MintTokenRequest) -> Result<String, MintTokenError> {
    // if read_state(|s| s.mint_token_requests.contains_key(&req.ticket_id)) {
    //     return Err(MintTokenError::AlreadyProcessed(req.ticket_id.to_string()));
    // }
    let signature = solana_rpc::mint_to(
       req.to_owned(),
    )
    .await
    .map_err(|e| {
        MintTokenError::TemporarilyUnavailable(e.to_string())
    })?;

    Ok(signature)
}

pub async fn update_tx_hash(
    hub_principal: Principal,
    ticket_id: TicketId,
    mint_tx_hash: String,
) -> Result<(), CallError> {
    let resp: (Result<(), Error>,) =
        ic_cdk::api::call::call(hub_principal, "update_tx_hash", (ticket_id, mint_tx_hash))
            .await
            .map_err(|(code, message)| CallError {
                method: "update_tx_hash".to_string(),
                reason: Reason::from_reject(code, message),
            })?;
    resp.0.map_err(|err| CallError {
        method: "update_tx_hash".to_string(),
        reason: Reason::CanisterError(err.to_string()),
    })?;
    Ok(())
}


