use crate::handler::clear_ticket::ClearTx;
use crate::ic_sui::rpc_client::{RpcClient, RpcError};
use crate::ic_sui::sui_json_rpc_types::sui_transaction::{SuiExecutionStatus, SuiTransactionBlockEffectsAPI, SuiTransactionBlockResponse};
use crate::ic_sui::sui_types::base_types::SuiAddress;
use crate::types::{ Error,TicketId};
use candid::{ CandidType, Principal};

use ic_stable_structures::Storable;
use ic_stable_structures::storable::Bound;

use std::borrow::Cow;
use std::str::FromStr;
use serde::{Deserialize, Serialize};

use crate::state::TxStatus;
use crate::{
    call_error::{CallError, Reason},
    state::{mutate_state, read_state},
};
use crate::config::read_config;

use crate::constants::{ RETRY_NUM, TAKE_SIZE};
use ic_canister_log::log;
use crate::ic_log::{ WARNING, DEBUG, ERROR};


#[derive(CandidType, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum MintTokenError {
    NotFoundToken(String),
    UnsupportedToken(String),
    AlreadyProcessed(TicketId),
    TemporarilyUnavailable(String),
    TxError(String),
}

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct MintTokenRequest {
    pub ticket_id: TicketId,
    pub token_id: String,
    pub recipient: String,
    pub amount: u64,
    pub status: TxStatus,
    pub digest: Option<String>,
    pub object: Option<String>,
    pub retry:u64,
}

impl Storable for MintTokenRequest {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {

        let bytes = bincode::serialize(&self).expect("failed to serialize MintTokenRequest");
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
      
        bincode::deserialize(bytes.as_ref()).expect("failed to deserialize MintTokenRequest")
    }

    const BOUND: Bound = Bound::Unbounded;
}

pub async fn mint_token() {
    // take tickets to mint,very time
    let tickets = read_state(|s| {
        s.tickets_queue
            .iter()
            .take(TAKE_SIZE.try_into().unwrap())
            .map(|(seq, ticket)| (seq, ticket))
            .collect::<Vec<_>>()
    });
    
    for (seq, ticket) in tickets.into_iter() {
      
        let mint_req = match read_state(|s| s.mint_token_requests.get(&ticket.ticket_id)){
            None => {
                // new req
                let mint_req= MintTokenRequest {
                    ticket_id: ticket.ticket_id.to_owned(),
                    token_id: ticket.token.to_owned(),
                    recipient: ticket.receiver.to_owned(),
                    amount: ticket.amount.parse::<u64>().unwrap(),
                    status: TxStatus::New,
                    digest: None,
                    object:None,
                    retry:0
                };
                // save new token req
                mutate_state(|s| s.mint_token_requests.insert(mint_req.ticket_id.to_string(), mint_req.to_owned()));
                mint_req
            }
            Some(mint_req) => {
                if mint_req.retry >= RETRY_NUM {
                  
                    log!(
                        WARNING,
                       "[mint_token::mint_token] failed to mint token for ticket id: {}, and reach to max retry,pls contact your administrator",
                        ticket.ticket_id
                    );
                    continue;
                }    
                mint_req
            }
            
        };

        log!(DEBUG, "[mint_token::mint_token] mint token request: {:?} ", mint_req);

        match &mint_req.status {
            TxStatus::New => {
                handle_mint_token(mint_req).await;
                
              
            },
            TxStatus::Pending => {
                log!(
                    DEBUG,
                    "[mint_token::mint_token] the mint token request ({:?}) is handling, pls wait ...",
                    mint_req
                );
               
            }
            TxStatus::Finalized  => {
                log!(
                    DEBUG,
                    "[mint_token::mint_token] the mint token request ({:?}) is finalized !",
                    mint_req
                );
               
                //only finalized mint_req, remove the handled ticket from queue  
                mutate_state(|s|{ 
                    s.tickets_queue.remove(&seq);
                    //add clear ticket
                    s.clr_ticket_queue.insert(mint_req.ticket_id.to_owned(), ClearTx::new())
                });
                // update tx digest to hub
                let hub_principal = read_config(|s| s.get().hub_principal);
                let digest = mint_req.digest.unwrap();
                       
                match update_tx_to_hub(hub_principal, ticket.ticket_id.to_string(), digest.to_owned()).await {
                   Ok(()) =>{
                       log!(
                           DEBUG,
                           "[mint_token::mint_token] mint req tx({:?}) already finallized and update tx digest to hub! ",
                           digest
                       );
                   }
                   Err(err) =>  {
                       log!(
                        ERROR,
                           "[mint_token::mint_token] failed to update tx hash to hub:{}",
                           err
                       );
                   }
               }   
                                      
           }
            TxStatus::TxFailed { e } => {
              
                if mint_req.retry < RETRY_NUM {
                    log!(
                        WARNING,
                       "[mint_token::mint_token] failed to mint token for ticket id: {}, error: {:} , and retry ... ",
                        ticket.ticket_id,e 
                    );
                    handle_mint_token(mint_req).await;
                } 
            },
            
        }
         
    }
}

pub async fn handle_mint_token(mint_req: MintTokenRequest){
    match mint_to_with_req(mint_req.to_owned()).await {
        Ok(tx_resp) => {
            log!(
                DEBUG,
                "[mint_token::handle_mint_token] mint token req was submited for ticket id: {} and tx_resp: {:?} ",
                mint_req.ticket_id.to_string(),tx_resp);
            //check tx status
            match tx_resp.effects {
                None => {
                    log!(
                        ERROR,
                        "[mint_token::handle_mint_token] Not Found tx effects and retry ... ",
                    );
                
                    mutate_state(|s| {
                        if let Some(req)=s.mint_token_requests.get(&mint_req.ticket_id).as_mut() {
                            req.status =TxStatus::TxFailed { e: " Not Found effects in tx response".to_string() };
                            req.retry +=1;
                            // req.digest = None;
                            s.mint_token_requests.insert(mint_req.ticket_id.to_string(),req.to_owned());
                        }
                    });
                }
                Some(effects) => match effects.status() {
                    SuiExecutionStatus::Success => {
                        log!(
                            DEBUG,
                            "[mint_token::handle_mint_token] mint token req for ticket id: {} successfully!",
                            mint_req.ticket_id.to_string()
                        );
                        mutate_state(|s| {
                            if let Some(req)=s.mint_token_requests.get(&mint_req.ticket_id).as_mut() {
                                req.status= TxStatus::Finalized ;
                                req.digest = Some(tx_resp.digest.to_string());
                                //TODO: update object id
                                s.mint_token_requests.insert(mint_req.ticket_id.to_string(),req.to_owned());
                            }
                        });

                    }
                    SuiExecutionStatus::Failure { error } => {
                        log!(
                            ERROR,
                            "[mint_token::handle_mint_token] sui tx execute failured: {} ",error
                        );
                        mutate_state(|s| {
                            if let Some(req)=s.mint_token_requests.get(&mint_req.ticket_id).as_mut() {
                                req.status =TxStatus::TxFailed { e: error.to_owned() };
                                req.retry +=1;
                                s.mint_token_requests.insert(mint_req.ticket_id.to_string(),req.to_owned());
                            }
                        });
                    }
                }
            }
          
        }
        Err(e) => { 
            let error = format!( "[mint_token::mint_token] failed to mint token for ticket id: {}, rpc error: {:?}",
            mint_req.ticket_id,e);
            log!(ERROR,"{}", error.to_string());

            // if err, update req status 
            mutate_state(|s| {
                    if let Some(req)=s.mint_token_requests.get(&mint_req.ticket_id).as_mut() {
                        req.status =TxStatus::TxFailed { e: error };
                        req.retry +=1;
                        s.mint_token_requests.insert(mint_req.ticket_id.to_string(),req.to_owned());
                    }
                });

        }

    }
    
}

/// send tx to sui for mint token
pub async fn mint_to_with_req(req: MintTokenRequest) -> Result<SuiTransactionBlockResponse,RpcError> {
   
    // update status to pending
    mutate_state(|s| {
        let new_req = MintTokenRequest {
            status: TxStatus::Pending,
            ..req.to_owned()
          
        };
        s.mint_token_requests
            .insert(req.ticket_id.to_owned(), new_req);
    });

    let sui_token = read_state(|s| s.sui_tokens.get(&req.token_id)).expect("sui token should exists");
    let recipient = SuiAddress::from_str(&req.recipient).map_err(|e| RpcError::Text(e.to_string()))?;
    let (action,provider, nodes, gas_budget,forward) = read_config(|s| {
        (   
            s.get().sui_port_action.to_owned(),
            s.get().rpc_provider.to_owned(),
            s.get().nodes_in_subnet,
            s.get().gas_budget,
            s.get().forward.to_owned(),
        )
    });
    let client = RpcClient::new(provider, Some(nodes));
    let tx_resp = client.mint_with_ticket(action,req.ticket_id,
        sui_token,
        recipient,
        req.amount,
        Some(gas_budget),
        forward,
        )
        .await?;

    Ok(tx_resp)
}


/// send tx to sui for mint token
pub async fn remove_ticket_from_port(ticket_id: String) {
   let (action,provider, nodes, gas_budget,forward) = read_config(|s| {
        (   
            s.get().sui_port_action.to_owned(),
            s.get().rpc_provider.to_owned(),
            s.get().nodes_in_subnet,
            s.get().gas_budget,
            s.get().forward.to_owned(),
        )
    });
    let client = RpcClient::new(provider, Some(nodes));
    let tx_resp = client.remove_ticket(action,ticket_id.to_owned(),
        Some(gas_budget),
        forward,
        )
        .await;
    log!(
        DEBUG,
        "[mint_token::remove_ticket_from_port] remove ticket id: {} from port result: {:?} ",ticket_id,tx_resp
    );  
  
}

pub async fn update_tx_to_hub(
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

#[cfg(test)]
mod test {

    #[test]
    fn test_match_tx_error() {
            let log_message = r#"
            TxFailed { e: \"management call '[solana_rpc::create_mint_account] create_mint_with_metaplex' failed: canister error: TxError: block_hash=B9p4ZCrQuWqbWFdhTx3ZseunFiV1sNQ5ZyjEZvuKNjbJ, signature=5o1BYJ76Yx65U3brvkuFwkJ4LkZVev28337mq8u4eg2Vi8S2DBjvSn9LuNuuNp5Gqi1D3BDexmRRHjYM6NdhWAVW, error=[solana_client::send_raw_transaction] rpc error: RpcResponseError { code: -32002, message: \\\"Transactionsimulationfailed: Blockhashnotfound\\\", data: None }\" } 
            "#;
            if log_message.contains("Transactionsimulationfailed: Blockhashnotfound") {
                println!("{}", log_message);
            } else {
                println!("not found");
            }

            if log_message.contains("Transactionsimulationfailed") {
                println!("{}", log_message);
            } else {
                println!("not found");
            }
    }

    #[test]
    fn test_match_status_error() {
            let log_message = r#"
          TxFailed { e: \"management call 'sol_getSignatureStatuses' failed: canister error: parse error: expected invalid type: null, expected struct TransactionStatus at line 1 column 91\" }
            "#;
            if log_message.contains("expected invalid type: null") {
                println!("{}", log_message);
            } else {
                println!("not found");
            }

            if log_message.contains("expected struct TransactionStatus") {
                println!("{}", log_message);
            } else {
                println!("not found");
            }
    }
}
