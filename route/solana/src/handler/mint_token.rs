use crate::handler::solana_rpc;
use crate::types::{Error, TicketId};
use candid::{CandidType, Principal};

use ic_solana::token::TxError;
use ic_stable_structures::storable::Bound;
use ic_stable_structures::Storable;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

use crate::state::AtaKey;
use crate::state::TxStatus;
use crate::{
    call_error::{CallError, Reason},
    state::{mutate_state, read_state},
};
use ic_solana::types::TransactionConfirmationStatus;

use crate::constants::{RETRY_4_STATUS, TAKE_SIZE};
use ic_canister_log::log;
use ic_solana::ic_log::{CRITICAL, DEBUG, WARNING};

#[derive(CandidType, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum MintTokenError {
    NotFoundToken(String),
    UnsupportedToken(String),
    AlreadyProcessed(TicketId),
    TemporarilyUnavailable(String),
    TxError(TxError),
}
#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct MintTokenRequest {
    pub ticket_id: TicketId,
    pub associated_account: String,
    pub amount: u64,
    pub token_mint: String,
    pub status: TxStatus,
    pub signature: Option<String>,
    pub retry_4_building: u64,
    pub retry_4_status: u64,
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
    // take 3 tickets to mint,very time
    let tickets = read_state(|s| {
        s.tickets_queue
            .iter()
            .take(TAKE_SIZE.try_into().unwrap())
            .map(|(seq, ticket)| (seq, ticket))
            .collect::<Vec<_>>()
    });

    for (seq, ticket) in tickets.into_iter() {
        log!(
            DEBUG,
            "[Consolidation]mint_token::mint_token start to mint token for ticket id: {}",
            ticket.ticket_id
        );

        let token_mint = read_state(|s| s.token_mint_accounts.get(&ticket.token));
        let token_mint = match token_mint {
            Some(token_mint) => {
                if !matches!(token_mint.status, TxStatus::Finalized) {
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
            s.associated_accounts.get(&AtaKey {
                owner: ticket.receiver.to_owned(),
                token_mint: token_mint.account.to_owned(),
            })
        });
        let associated_account = match associated_account {
            Some(associated_account) => {
                if !matches!(associated_account.status, TxStatus::Finalized) {
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

        let mint_req =
            if let Some(req) = read_state(|s| s.mint_token_requests.get(&ticket.ticket_id)) {
                // skip ticket casued by get_signature_status error
                if req.retry_4_status >= RETRY_4_STATUS {
                    continue;
                }
                req
            } else {
                let mint_req = MintTokenRequest {
                    ticket_id: ticket.ticket_id.to_owned(),
                    associated_account: associated_account.account.to_owned(),
                    amount: ticket.amount.parse::<u64>().unwrap(),
                    token_mint: token_mint.account,
                    status: TxStatus::New,
                    signature: None,
                    retry_4_building: 0,
                    retry_4_status: 0,
                };
                // save new token req
                mutate_state(|s| {
                    s.mint_token_requests
                        .insert(mint_req.ticket_id.to_string(), mint_req.to_owned())
                });
                mint_req
            };
        log!(
            DEBUG,
            "[mint_token::mint_token] mint token request: {:?} ",
            mint_req
        );

        match &mint_req.status {
            TxStatus::New => {
                match mint_req.signature.to_owned() {
                    //new mint req,mint_token
                    None => {
                        handle_mint_token(mint_req).await;
                    }
                    Some(sig) => {
                        log!(
                            DEBUG,
                            "[mint_token::mint_token] mint req tx({:?}) already submited and waiting for the tx({:?}) to be finallized! ",
                          mint_req,sig
                        );
                        update_mint_token_status(mint_req.to_owned(), sig).await;
                    }
                }
            }
            TxStatus::Pending => match mint_req.signature.to_owned() {
                None => {
                    log!(
                            DEBUG,
                            "[mint_token::mint_token] the mint token request ({:?}) is handling, pls wait ...",
                            mint_req
                        );
                }
                Some(sig) => {
                    log!(
                            DEBUG,
                            "[mint_token::mint_token] mint req tx({:?}) already submitted and waiting for the tx({:?}) to be finalized! ",
                          mint_req,sig
                        );
                    update_mint_token_status(mint_req.to_owned(), sig).await;
                }
            },
            TxStatus::Finalized => {
                // update txhash to hub
                let hub_principal = read_state(|s| s.hub_principal);
                let sig = mint_req.signature.unwrap();

                match update_tx_hash(hub_principal, ticket.ticket_id.to_string(), sig.to_owned())
                    .await
                {
                    Ok(()) => {
                        log!(
                        DEBUG,
                        "[mint_token::mint_token] mint req tx({:?}) already finalized and update tx hash to hub! ",
                        sig
                    );
                    }
                    Err(err) => {
                        log!(
                            CRITICAL,
                            "[tickets::mint_token] failed to update tx hash after mint token:{}",
                            err
                        );
                    }
                }
                //only finalized mint_req, remove the handled ticket from queue
                mutate_state(|s| s.tickets_queue.remove(&seq));
            }
            TxStatus::TxFailed { e } => {
                match mint_req.signature.to_owned() {
                    None => {
                        log!(
                            DEBUG,
                           "[mint_token::mint_token] failed to mint token for ticket id: {}, error: {:} ,pls check and retry  ",
                            ticket.ticket_id,e
                        );
                        //TODO: retry mint_to?
                    }
                    Some(sig) => {
                        log!(
                            DEBUG,
                            "[mint_token::mint_token] mint req tx({:?}) already submitted and waiting for the tx({:?}) to be finalized! ",
                          mint_req,sig
                        );
                        update_mint_token_status(mint_req.to_owned(), sig).await;
                    }
                }
            }
        }
    }
}

pub async fn handle_mint_token(mint_req: MintTokenRequest) {
    match mint_to_with_req(mint_req.to_owned()).await {
        Ok(signature) => {
            log!(
                DEBUG,
                "[mint_token::mint_token] mint token req was submited for ticket id: {} and signature is :{}",
                mint_req.ticket_id.to_string(),
                signature
            );
            // update req signature,but not comfirmed
            mutate_state(|s| {
                if let Some(req) = s.mint_token_requests.get(&mint_req.ticket_id).as_mut() {
                    req.signature = Some(signature.to_string());
                    // req.retry_4_building +=1;
                    s.mint_token_requests
                        .insert(mint_req.ticket_id.to_string(), req.to_owned());
                }
            });
        }
        Err(e) => {
            let err_info = format!(
                "[mint_token::mint_token] failed to mint token for ticket id: {}, err: {:?}",
                mint_req.ticket_id, e
            );
            log!(CRITICAL, "{}", err_info.to_string());

            //TODO: handle the tx error,maybe retry
            let tx_error = match e.reason {
                Reason::QueueIsFull
                | Reason::OutOfCycles
                | Reason::CanisterError(_)
                | Reason::Rejected(_) => todo!(),
                Reason::TxError(tx_error) => tx_error,
            };
            // if err, update req status
            mutate_state(|s| {
                if let Some(req) = s.mint_token_requests.get(&mint_req.ticket_id).as_mut() {
                    req.status = TxStatus::TxFailed { e: tx_error };
                    req.retry_4_building += 1;
                    //reset signature
                    req.signature = None;
                    s.mint_token_requests
                        .insert(mint_req.ticket_id.to_string(), req.to_owned());
                }
            });
        }
    }
}

pub async fn update_mint_token_status(mut mint_req: MintTokenRequest, sig: String) {
    // query signature status
    log!(
        DEBUG,
        "[mint_token::update_mint_token_status] start to get_signature_status for {}",
        mint_req.ticket_id
    );
    let tx_status_ret = solana_rpc::get_signature_status(vec![sig.to_string()]).await;
    match tx_status_ret {
        Err(e) => {
            log!(
                WARNING,
                "[mint_token::update_mint_token_status] get_signature_status for {} ,err: {:?}",
                sig.to_string(),
                e
            );
            let tx_error = match e.reason {
                Reason::QueueIsFull
                | Reason::OutOfCycles
                | Reason::TxError(_)
                | Reason::Rejected(_) => todo!(),
                Reason::CanisterError(tx_error) => tx_error,
            };

            mutate_state(|s| {
                if let Some(req) = s.mint_token_requests.get(&mint_req.ticket_id).as_mut() {
                    // if update statue is up to the RETRY_4_STATUS,the tx was droped and retry mint
                    if req.retry_4_status >= RETRY_4_STATUS {
                        log!(
                       WARNING,
                       "[mint_token::update_mint_token_status] retry for get_signature_status up to limit size :{} ,and need to rebuild the mint_to",
                       RETRY_4_STATUS,);
                        //TODO: retry to mint
                    } else {
                        req.status = TxStatus::TxFailed {
                            e: TxError {
                                block_hash: String::default(),
                                signature: sig.to_owned(),
                                error: tx_error.to_owned(),
                            },
                        };
                        req.retry_4_status += 1;
                        //reset signature
                        // req.signature = None;
                        s.mint_token_requests
                            .insert(mint_req.ticket_id.to_string(), req.to_owned());
                    }
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
                            s.mint_token_requests
                                .insert(mint_req.ticket_id.to_owned(), mint_req)
                        });
                    }
                }
            });
        }
    }
}

/// send tx to solana for mint token
pub async fn mint_to_with_req(req: MintTokenRequest) -> Result<String, CallError> {
    let signature = solana_rpc::mint_to_with_req(req.to_owned()).await?;

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

pub async fn mock_handle_mint_token_failed(mint_req: MintTokenRequest) {
    let err_info = r#"
            TxFailed { e: \"management call '[solana_rpc::create_mint_account] create_mint_with_metaplex' failed: canister error: TxError: block_hash=B9p4ZCrQuWqbWFdhTx3ZseunFiV1sNQ5ZyjEZvuKNjbJ, signature=5o1BYJ76Yx65U3brvkuFwkJ4LkZVev28337mq8u4eg2Vi8S2DBjvSn9LuNuuNp5Gqi1D3BDexmRRHjYM6NdhWAVW, error=[solana_client::send_raw_transaction] rpc error: RpcResponseError { code: -32002, message: \\\"Transactionsimulationfailed: Blockhashnotfound\\\", data: None }\" } 
            "#;
    log!(CRITICAL, "{}", err_info.to_string());
    // if err, update req status
    mutate_state(|s| {
        if let Some(req) = s.mint_token_requests.get(&mint_req.ticket_id).as_mut() {
            req.status = TxStatus::TxFailed {
                e: TxError {
                    block_hash: String::default(),
                    signature: String::default(),
                    error: err_info.to_owned(),
                },
            };
            req.retry_4_building += 1;
            //reset signature
            req.signature = None;
            s.mint_token_requests
                .insert(mint_req.ticket_id.to_string(), req.to_owned());
        }
    });
}

pub async fn mock_update_mint_token_status_failed(mint_req: MintTokenRequest, _sig: String) {
    let err_info = r#"
     TxFailed { e: \"management call 'sol_getSignatureStatuses' failed: canister error: parse error: expected invalid type: null, expected struct TransactionStatus at line 1 column 91\" }
    "#;
    log!(DEBUG, "{}", err_info.to_string());
    mutate_state(|s| {
        if let Some(req) = s.mint_token_requests.get(&mint_req.ticket_id).as_mut() {
            req.status = TxStatus::TxFailed {
                e: TxError {
                    block_hash: String::default(),
                    signature: String::default(),
                    error: err_info.to_owned(),
                },
            };
            req.retry_4_status += 1;
            if req.retry_4_status >= RETRY_4_STATUS {
                log!(
                   WARNING,
                   "[mint_token::update_mint_token_status] retry for get_signature_status up to limit size :{} ,and need to rebuild the mint_to",
                   RETRY_4_STATUS,);
            }
            s.mint_token_requests
                .insert(mint_req.ticket_id.to_string(), req.to_owned());
        }
    });
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
