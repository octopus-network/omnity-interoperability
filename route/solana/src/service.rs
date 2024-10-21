use crate::auth::{is_admin, set_perms, Permission};
use crate::call_error::{CallError, Reason};
use crate::guard::TaskType;
use crate::handler::associated_account::update_ata_status;
use candid::Principal;
use ic_cdk::{init, post_upgrade, pre_upgrade, query, update};
use ic_solana::types::TransactionStatus;

use crate::handler::mint_token::{
    self, update_tx_hash, update_mint_token_status,
};
use crate::handler::gen_ticket::{self,send_ticket,query_tx_from_multi_rpc,GenerateTicketError,
    GenerateTicketOk, GenerateTicketReq};
    
use crate::handler::{ scheduler, solana_rpc};
use crate::lifecycle::{self, RouteArg, UpgradeArgs};
use crate::service::solana_rpc::solana_client;
use crate::state::{AccountInfo, AtaKey, MultiRpcConfig, TokenMeta, TokenResp};
use crate::types::{Token, TokenId};
use ic_solana::token::SolanaClient;
use ic_solana::token::TokenInfo;

use crate::service::mint_token::MintTokenRequest;
use crate::state::MintAccount;
use crate::state::Owner;
use crate::state::{mutate_state, read_state, TxStatus};
use crate::types::ChainState;
use crate::types::{Chain, ChainId, Ticket};
use ic_canister_log::log;
use ic_solana::token::associated_account::get_associated_token_address_with_program_id;
use ic_solana::token::constants::token22_program_id;
use ic_solana::types::Pubkey;
use std::str::FromStr;
use crate::handler::token_account::update_mint_account_status;
use ic_canisters_http_types::{HttpRequest, HttpResponse, HttpResponseBuilder};
use ic_solana::ic_log::{self, DEBUG, ERROR};
use crate::state::Seqs;
use crate::state::TokenUri;

#[init]
fn init(args: RouteArg) {
    match args {
        RouteArg::Init(args) => {
            lifecycle::init(args);
        }
        RouteArg::Upgrade(_) => {
            panic!("expected InitArgs got UpgradeArgs");
        }
    }
}

#[pre_upgrade]
fn pre_upgrade() {
    log!(DEBUG, "begin to execute pre_upgrade ...");
    scheduler::cancel_schedule();
    lifecycle::pre_upgrade();
    log!(DEBUG, "pre_upgrade end!");
}

#[post_upgrade]
fn post_upgrade(args: Option<RouteArg>) {
    log!(DEBUG, "begin to execute post_upgrade with :{:?}", args);
    let mut upgrade_arg: Option<UpgradeArgs> = None;
    if let Some(route_arg) = args {
        upgrade_arg = match route_arg {
            RouteArg::Upgrade(upgrade_args) => upgrade_args,
            RouteArg::Init(_) => panic!("expected Option<UpgradeArgs> got InitArgs."),
        };
    }

    lifecycle::post_upgrade(upgrade_arg);
    scheduler::start_schedule();
    log!(DEBUG, "upgrade successfully!");
}

// devops method
#[update(guard = "is_admin",hidden = true)]
pub fn start_schedule() {
    log!(DEBUG, "start schedule task ...");
    scheduler::start_schedule();
}

// devops method
#[update(guard = "is_admin",hidden = true)]
pub fn cancel_schedule() {
    log!(DEBUG, "cancel schedule task ...");
    scheduler::cancel_schedule();
}

// devops method
#[query(guard = "is_admin",hidden = true)]
pub async fn active_tasks()-> Vec<TaskType> {
    read_state(|s|
    s.active_tasks.iter().map(|t| t.to_owned()).collect())
}

// devops method
#[update(guard = "is_admin",hidden = true)]
pub async fn update_schnorr_key(key_name: String) {
    mutate_state(|s| {
        s.schnorr_key_name = key_name;
    })
}

// devops method
#[update(guard = "is_admin",hidden = true)]
pub async fn update_forward(forward: Option<String>) {
    mutate_state(|s| {
        s.forward = forward
    })
}

// devops method
#[query(guard = "is_admin",hidden = true)]
pub async fn forward()-> Option<String> {
    read_state(|s| {
        s.forward.to_owned()
    })
}

// devops method
#[update(guard = "is_admin",hidden = true)]
pub async fn update_multi_rpc(multi_prc_cofig: MultiRpcConfig) {
    mutate_state(|s| {
        s.multi_rpc_config = multi_prc_cofig;
    })
}

// devops method
#[query(guard = "is_admin",hidden = true)]
pub async fn multi_rpc_config() -> MultiRpcConfig {
    read_state(|s| s.multi_rpc_config.to_owned())
}

// devops method
#[update(guard = "is_admin",hidden = true)]
async fn valid_tx_from_multi_rpc(signature: String) -> Result<String, CallError> {
    use crate::service::solana_rpc::solana_client;
    let client = solana_client().await;
    let multi_rpc_config = read_state(|s| s.multi_rpc_config.to_owned());
    let tx_response =
        query_tx_from_multi_rpc(&client, signature, multi_rpc_config.rpc_list.to_owned()).await;
    let json_response = multi_rpc_config
        .valid_and_get_result(&tx_response)
        .map_err(|err| CallError {
            method: "valid_and_get_result".to_string(),
            reason: Reason::CanisterError(err.to_string()),
        })?;
    let ret = serde_json::to_string(&json_response).map_err(|err| CallError {
        method: "serde_json::to_string".to_string(),
        reason: Reason::CanisterError(err.to_string()),
    })?;
    Ok(ret)
}

// devops method
#[update(guard = "is_admin",hidden = true)]
pub async fn signer() -> Result<String, String> {
    let pk = solana_rpc::eddsa_public_key().await?;
    Ok(pk.to_string())
}

// devops method
#[update(guard = "is_admin",hidden = true)]
pub async fn sign(msg: String) -> Result<Vec<u8>, String> {
    let signature = solana_rpc::sign(msg).await?;
    Ok(signature)
}

// query supported chain list 
#[query]
fn get_chain_list() -> Vec<Chain> {
    read_state(|s| {
        s.counterparties
            .iter()
            .filter(|(_, chain)| matches!(chain.chain_state, ChainState::Active))
            .map(|(_, chain)| chain.clone())
            .collect()
    })
}

// query supported chain list 
#[query]
fn get_token_list() -> Vec<TokenResp> {
    read_state(|s| {
        s.tokens
            .iter()
            .filter(|(token_id, _token)| {
                s.token_mint_accounts.contains_key(&token_id.to_string())
                    && matches!(
                        s.token_mint_accounts
                            .get(&token_id.to_string())
                            .unwrap()
                            .status,
                        TxStatus::Finalized
                    )
            })
            .map(|(_, token)| token.to_owned().into())
            .collect()
    })
}

// devops method
#[update(guard = "is_admin",hidden = true)]
async fn get_latest_blockhash() -> Result<String, CallError> {
    use crate::service::solana_rpc::solana_client;
    let client = solana_client().await;

    let block_hash = client
        .get_latest_blockhash()
        .await
        .map_err(|err| CallError {
            method: "get_latest_blockhash".to_string(),
            reason: Reason::CanisterError(err.to_string()),
        })?;
    Ok(block_hash.to_string())
}

// devops method
#[update(guard = "is_admin",hidden = true)]
async fn get_transaction(signature: String, forward: Option<String>) -> Result<String, CallError> {
    use crate::service::solana_rpc::solana_client;
    let client = solana_client().await;
    client
        .query_transaction(signature, forward)
        .await
        .map_err(|err| CallError {
            method: "get_transaction".to_string(),
            reason: Reason::CanisterError(err.to_string()),
        })
}

// devops method
#[update(guard = "is_admin",hidden = true)]
async fn get_signature_status(
    signatures: Vec<String>,
) -> Result<Vec<TransactionStatus>, CallError> {
    solana_rpc::get_signature_status(signatures).await
}

// devops method
#[update(guard = "is_admin",hidden = true)]
pub async fn transfer_to(to_account: String, amount: u64) -> Result<String, CallError> {
    solana_rpc::transfer_to(to_account, amount).await
}

// devops method
#[update(guard = "is_admin",hidden = true)]
pub async fn derive_mint_account(req: TokenInfo) -> Result<String, CallError> {
    let sol_client = solana_client().await;

    let mint_account =
        SolanaClient::derive_account(sol_client.chainkey_name.clone(), req.token_id.to_string())
            .await;

    Ok(mint_account.to_string())
}

// devops method
#[update(guard = "is_admin",hidden = true)]
pub async fn get_account_info(account: String) -> Result<Option<String>, CallError> {
    let sol_client = solana_client().await;

    // query account info from solana
    let account_info = sol_client
        .get_account_info(account.to_string())
        .await
        .map_err(|e| CallError {
            method: "[service::get_account_info] get_account_info".to_string(),
            reason: Reason::CanisterError(e.to_string()),
        })?;
    log!(
        DEBUG,
        "[service::get_account_info] {} account_info from solana : {:?} ",
        account.to_string(),
        account_info,
    );
    Ok(account_info)
}

// devops method
#[query(hidden = true)]
pub async fn query_mint_account(token_id: TokenId) -> Option<AccountInfo> {
    read_state(|s| s.token_mint_accounts.get(&token_id))
}

#[query]
pub async fn query_mint_address(token_id: TokenId) -> Option<String> {
    read_state(|s| match s.token_mint_accounts.get(&token_id) {
        None => None,
        Some(mint_account) => {
            if matches!(mint_account.status, TxStatus::Finalized { .. }) {
                Some(mint_account.account.to_string())
            } else {
                None
            }
        }
    })
}

// devops method
// add token manually 
#[update(guard = "is_admin",hidden = true)]
pub async fn add_token(token: Token) -> Option<Token> {
      mutate_state(|s| {
        s.tokens
            .insert(token.token_id.to_string(), token.to_owned())
    })

}

// devops method
#[update(guard = "is_admin",hidden = true)]
fn update_token(token: Token) -> Result<Option<Token>, CallError> {
    mutate_state(|s| {
        match s.tokens.get(&token.token_id) {
            None => Err(CallError {
                method: "[service::update_token] update_token".to_string(),
                reason: Reason::CanisterError(format!(
                    "Not found token id {} ",
                    token.token_id.to_string()
                )),
            }),
            Some(_) => Ok(s.tokens.insert(token.token_id.to_string(), token.to_owned()))
        }
        
    })
    // Ok(())
}

// devops method
#[update(guard = "is_admin",hidden = true)]
fn remove_token_and_account(token_id: TokenId) {
    mutate_state(|s| {
        s.tokens.remove(&token_id);
        s.token_mint_accounts.remove(&token_id);
    })
}

// devops method
#[update(guard = "is_admin",hidden = true)]
pub async fn create_mint_account(req: TokenInfo) -> Result<AccountInfo, CallError> {
    let sol_client = solana_client().await;

    let mint_account =
        SolanaClient::derive_account(sol_client.chainkey_name.clone(), req.token_id.to_string())
            .await;
    log!(
        DEBUG,
        "[service::create_mint_account] mint_account from schonnor chainkey: {:?} ",
        mint_account.to_string(),
    );

    let mut mint_account_info =
        if let Some(account_info) = read_state(|s| s.token_mint_accounts.get(&req.token_id)) {
            account_info
        } else {
            let new_account_info = AccountInfo {
                account: mint_account.to_string(),
                retry: 0,
                signature: None,
                status: TxStatus::New,
            };

            new_account_info
        };

    log!(
        DEBUG,
        "[service::create_mint_account] mint_account_info from solana route: {:?} ",
        mint_account_info,
    );

    // check mint account exists on solana
    let account_info = sol_client.get_account_info(mint_account.to_string()).await;
    log!(
        DEBUG,
        "[service::create_mint_account] token mint: {:?}  account_info from solana: {:?} ",
        mint_account.to_string(),
        account_info,
    );
    if let Ok(account_info) = account_info {
        if matches!(account_info, Some(..)) {
            let mint = AccountInfo {
                account: mint_account.to_string(),
                retry: mint_account_info.retry,
                signature: mint_account_info.signature,
                status: TxStatus::Finalized,
            };
            //update mint account info
            mutate_state(|s| {
                s.token_mint_accounts
                    .insert(req.token_id.to_string(), mint.to_owned())
            });

            return Ok(mint);
        }
    }

    match mint_account_info.status {
        TxStatus::New | TxStatus::TxFailed { .. } => {
            match &mint_account_info.signature {
                None => {
                    let sig = solana_rpc::create_mint_account(mint_account, req.clone()).await?;
                    log!(
                        DEBUG,
                        "[service::create_mint_account] create_mint_account signature: {:?} ",
                        sig.to_string(),
                    );

                    // update signature
                    mint_account_info.signature = Some(sig.to_string());
                    mint_account_info.retry += 1;
                    mutate_state(|s| {
                        s.token_mint_accounts
                            .insert(req.token_id.to_string(), mint_account_info.clone())
                    });

                    // update mint account status
                    update_mint_account_status(sig.to_string(), req.token_id.to_string()).await;
                }
                Some(sig) => {
                    // update mint account status
                    update_mint_account_status(sig.to_string(), req.token_id.to_string()).await;
                }
            }
        }
        TxStatus::Pending => {
            match &mint_account_info.signature {
                // be creating
                None => {
                    log!(
                        DEBUG,
                        "[directive::create_token_mint] the token mint ({:?}) is creating, please waite ... ",
                        mint_account_info
                        
                    );
                }
                // already created,but not finallized
                Some(sig) => {
                    log!(
                        DEBUG,
                        "[directive::create_token_mint]the token mint ({:?}) was already submited and waiting for the tx({:}) to be finallized ... ",
                        mint_account,sig
                        
                    );

                    // update status
                    update_mint_account_status(sig.to_string(), req.token_id.to_string()).await;
                }
            }
        }
        TxStatus::Finalized => return Ok(mint_account_info),
    }

    match read_state(|s| s.token_mint_accounts.get(&req.token_id)) {
        None => Err(CallError {
            method: "[service::create_mint_account] create_mint_account".to_string(),
            reason: Reason::CanisterError(format!(
                "Not found account for {} ",
                req.token_id.to_string()
            )),
        }),
        Some(account) => Ok(account),
    }
}

// devops method
#[update(guard = "is_admin",hidden = true)]
pub async fn update_mint_account(token_id:TokenId,mint_account: AccountInfo) -> Option<AccountInfo>{
           //update mint account info
           mutate_state(|s| {
            s.token_mint_accounts
                .insert(token_id, mint_account)
        })
   
}

// devops method
#[update(guard = "is_admin",hidden = true)]
pub async fn update_token_metadata(req: TokenInfo) -> Result<String, CallError> {
    // token_mint must be exists
    match read_state(|s| s.token_mint_accounts.get(&req.token_id)) {
        None => {
            return Err(CallError {
                method: "[service::update_token_metadata] update_token_metadata".to_string(),
                reason: Reason::CanisterError(format!(
                    "{} token mint account not exists!",
                    req.token_id
                )),
            });
        }
        Some(account_info) => {
            let signature =
                solana_rpc::update_token_metadata(account_info.account, req.clone()).await?;
            log!(
                DEBUG,
                "[service::update_token_metadata] update_token_metadata signature: {:?} ",
                signature.to_string(),
            );
            //TODO: check signature status
            // update update_token_metadata result to state
            mutate_state(|s| {
                // update the token info
                if let Some(token) = s.tokens.get(&req.token_id).as_mut() {
                    token.name = req.name;
                    token.symbol = req.symbol;
                    token.decimals = req.decimals;
                    token.icon = Some(req.uri);
                    s.tokens.insert(req.token_id.to_string(), token.to_owned());
                }
            });

            // remove update_token req from queue
            if let Some(..) = read_state(|s| s.update_token_queue.get(&req.token_id)) {
                // update update_token_metadata result to state
                mutate_state(|s| {
                    // remove the updated token from queue
                    s.update_token_queue.remove(&req.token_id)
                });
            }
            Ok(signature)
        }
    }
}

// devops method
#[update(guard = "is_admin",hidden = true)]
pub async fn derive_aossicated_account(
    owner: String,
    token_mint: String,
) -> Result<String, CallError> {
    let to_account_pk = Pubkey::from_str(&owner).expect("Invalid to_account address");
    let token_mint_pk = Pubkey::from_str(&token_mint).expect("Invalid token mint address");
    let associated_account = get_associated_token_address_with_program_id(
        &to_account_pk,
        &token_mint_pk,
        &token22_program_id(),
    );
    log!(
        DEBUG,
        "[service::derive_aossicated_account] get_associated_token_address_with_program_id : {:?}",
        associated_account
    );

    Ok(associated_account.to_string())
}

// devops method
#[query(hidden = true)]
pub async fn query_aossicated_account(
    owner: Owner,
    token_mint: MintAccount,
) -> Option<AccountInfo> {
    read_state(|s| s.associated_accounts.get(&AtaKey { owner, token_mint }))
}

// devops method
#[query(hidden = true)]
pub async fn query_aossicated_account_address(
    owner: Owner,
    token_mint: MintAccount,
) -> Option<String> {
    read_state(
        |s| match s.associated_accounts.get(&AtaKey { owner, token_mint }) {
            None => None,
            Some(ata) => {
                if matches!(ata.status, TxStatus::Finalized) {
                    Some(ata.account.to_string())
                } else {
                    None
                }
            }
        },
    )
}

// devops method
#[update(guard = "is_admin",hidden = true)]
pub async fn create_aossicated_account(
    owner: String,
    token_mint: String,
) -> Result<AccountInfo, CallError> {
    let to_account_pk = Pubkey::from_str(&owner).expect("Invalid to_account address");
    let token_mint_pk = Pubkey::from_str(&token_mint).expect("Invalid token mint address");
    let associated_account = get_associated_token_address_with_program_id(
        &to_account_pk,
        &token_mint_pk,
        &token22_program_id(),
    );
    log!(
        DEBUG,
        "[service::create_associated_account] get_associated_token_address_with_program_id : {:?}",
        associated_account
    );
    let mut ata_account = if let Some(account_info) = read_state(|s| {
        s.associated_accounts.get(&AtaKey {
            owner: owner.to_string(),
            token_mint: token_mint.to_string(),
        })
    }) {
        account_info
    } else {
        let new_account_info = AccountInfo {
            account: associated_account.to_string(),
            retry: 0,
            signature: None,
            status: TxStatus::New,
        };

        new_account_info
    };

    log!(
        DEBUG,
        "[service::create_associated_account] ata_info from solana route : {:?}",
        ata_account
    );

    // check ATA exists on solana
    let sol_client = solana_client().await;
    let ata_account_info = sol_client
        .get_account_info(associated_account.to_string())
        .await;
    log!(
        DEBUG,
        "[service::create_associated_account] ATA: {:?} account info from solana: {:?} ",
        associated_account,
        ata_account_info,
    );
    if let Ok(account_info) = ata_account_info {
        if matches!(account_info, Some(..)) {
            let ata = AccountInfo {
                account: ata_account.account.to_string(),
                retry: ata_account.retry,
                signature: ata_account.signature,
                status: TxStatus::Finalized,
            };
            //update ata info
            mutate_state(|s| {
                s.associated_accounts.insert(
                    AtaKey {
                        owner: owner.to_string(),
                        token_mint: token_mint.to_string(),
                    },
                    ata.to_owned(),
                )
            });

            return Ok(ata);
        }
    }

    match ata_account.status {
        TxStatus::New | TxStatus::TxFailed { .. } => {
            match ata_account.signature.clone() {
                None => {
                    let sig =
                        solana_rpc::create_ata(owner.to_string(), token_mint.to_string()).await?;
                    log!(
                        DEBUG,
                        "[service::create_aossicated_account] create_aossicated_account signature: {:?} ",
                        sig.to_string(),
                    );
                    // update signature
                    ata_account.signature = Some(sig.to_string());
                    ata_account.retry += 1;
                    mutate_state(|s| {
                        s.associated_accounts.insert(
                            AtaKey {
                                owner: owner.to_string(),
                                token_mint: token_mint.to_string(),
                            },
                            ata_account.clone(),
                        )
                    });
                    // update ata status
                    update_ata_status(sig.to_string(), owner.to_string(), token_mint.to_string())
                        .await;
                }
                Some(sig) => {
                    update_ata_status(sig.to_string(), owner.to_string(), token_mint.to_string())
                        .await;
                }
            }
        }
        TxStatus::Pending => {
            match ata_account.signature.clone() {
                None => {
                    log!(
                        DEBUG,
                        "[service::create_associated_account] the associated account ({:?}) is creating,pls wait ...",
                        ata_account
                    );
                  
                }
                Some(sig) => {
                    update_ata_status(sig.to_string(), owner.to_string(), token_mint.to_string())
                        .await;
                }
            }
        }
        TxStatus::Finalized => return Ok(ata_account),
    }
    match read_state(|s| {
        s.associated_accounts.get(&AtaKey {
            owner: owner.to_string(),
            token_mint: token_mint.to_string(),
        })
    }) {
        None => Err(CallError {
            method: "[service::create_aossicated_account] create_aossicated_account".to_string(),
            reason: Reason::CanisterError(format!(
                "Not found account for {} and {}",
                owner.to_string(),
                token_mint.to_string()
            )),
        }),
        Some(account) => Ok(account),
    }
}


// devops method
#[update(guard = "is_admin",hidden = true)]
pub async fn update_associated_account(
    owner: String,
    token_mint: String,
    associated_account: AccountInfo,
) -> Result<AccountInfo, CallError> {

    log!(
        DEBUG,
        "[service::update_associated_account] owner: {} and token_mint: {}  associated_account: {:?}",
        owner,token_mint,associated_account
    );

    mutate_state(|s| {
        s.associated_accounts.insert(
            AtaKey::new(owner.to_string(), token_mint.to_string()),
            associated_account,
        )
    });

    match read_state(|s| {
        s.associated_accounts
            .get(&AtaKey::new(owner.to_string(), token_mint.to_string()))
            
    }) {
        None => Err(CallError {
            method: "[service::update_associated_account] update_associated_account".to_string(),
            reason: Reason::CanisterError(format!(
                "Not found account for {} and {}",
                owner.to_string(),
                token_mint.to_string()
            )),
        }),
        Some(account) => Ok(account),
    }
}



// devops method
#[query(hidden = true)]
fn get_ticket_from_queue(ticket_id: String) -> Option<(u64, Ticket)> {
    read_state(|s| {
        s.tickets_queue
            .iter()
            .find(|(_seq, ticket)| ticket.ticket_id.eq(&ticket_id))
    })
}

// devops method
#[query(hidden = true)]
fn get_tickets_from_queue() -> Vec<(u64, Ticket)> {
    read_state(|s| {
        s.tickets_queue
            .iter()
            .map(|(seq, ticket)| (seq, ticket))
            .collect()
    })
}

// devops method
#[update(guard = "is_admin",hidden = true)]
pub async fn remove_ticket_from_quene(ticket_id: String) -> Option<Ticket> {
    mutate_state(|s| {
        let ticket = s
            .tickets_queue
            .iter()
            .find(|(_seq, ticket)| ticket.ticket_id.eq(&ticket_id));

        match ticket {
            None => None,
            Some((seq, _ticket)) => s.tickets_queue.remove(&seq),
        }
    })
}


// query mint_token_statue for the given ticket id
#[query]
pub async fn mint_token_status(ticket_id: String) -> Result<TxStatus, CallError> {
    let req = read_state(|s| s.mint_token_requests.get(&ticket_id));
    match req {
        None => Err(CallError {
            method: "[service::mint_token_status] mint_token_status".to_string(),
            reason: Reason::CanisterError(format!(
                "Not found ticket({}) MintTokenStatus",
                ticket_id.to_string()
            )),
        }),

        Some(req) => Ok(req.status),
    }
}

// query mint token tx hash or signature for the given ticket id
#[query]
pub async fn mint_token_tx_hash(ticket_id: String) -> Result<Option<String>, CallError> {
    let req = read_state(|s| s.mint_token_requests.get(&ticket_id).to_owned());
    match req {
        None => Err(CallError {
            method: "[service::mint_token_tx_hash] mint_token_tx_hash".to_string(),
            reason: Reason::CanisterError(format!(
                "Not found ticket({}) mint token tx hash",
                ticket_id.to_string()
            )),
        }),

        Some(req) => Ok(req.signature),
    }
}

#[query(hidden = true)]
pub async fn mint_token_req(ticket_id: String) -> Result<MintTokenRequest, CallError> {
    let req = read_state(|s| s.mint_token_requests.get(&ticket_id));
    match req {
        None => Err(CallError {
            method: "[service::mint_token_req] mint_token_req".to_string(),
            reason: Reason::CanisterError(format!(
                "Not found ticket({}) mint token request",
                ticket_id.to_string()
            )),
        }),

        Some(req) => Ok(req),
    }
}

// devops method
#[update(guard = "is_admin",hidden = true)]
pub async fn update_mint_token_req(req: MintTokenRequest) -> Result<MintTokenRequest, CallError> {
    mutate_state(|s| {
        s.mint_token_requests
            .insert(req.ticket_id.to_string(), req.to_owned())
    });

    match read_state(|s| s.mint_token_requests.get(&req.ticket_id)) {
        None => Err(CallError {
            method: "[service::update_mint_token_req] update_mint_token_req".to_string(),
            reason: Reason::CanisterError(format!(
                "Not found ticket({}) mint token request",
                req.ticket_id.to_string()
            )),
        }),
        Some(req) => Ok(req),
    }
}

// devops method
#[update(guard = "is_admin",hidden = true)]
pub async fn mint_token(n_req: MintTokenRequest) -> Result<TxStatus, CallError> {

   let mut req =  match read_state(|s| s.mint_token_requests.get(&n_req.ticket_id)) {
        None => {
            log!(
                DEBUG,
                "[service::mint_token] not found mint token req for ticket: {:?} ",
                n_req.ticket_id
            );
            n_req
        }
        Some(o_req) => {
            if o_req.eq(&n_req) {
                o_req
            }else {
                n_req
            }
        }
    };

    log!(DEBUG, "[service::mint_token] mint token request: {:?} ", req);

    match &req.status {
        TxStatus::New | TxStatus::TxFailed { .. } => {
            match req.signature.to_owned() {
                None => {
                    // new mint req
                    let sig = solana_rpc::mint_to(
                      req.to_owned(),
                    )
                    .await?;

                    // update signature
                    req.signature = Some(sig.to_string());
                    mutate_state(|s| {
                        s.mint_token_requests.insert(req.ticket_id.to_owned(), req.to_owned())
                    });

                    // update req status
                    update_mint_token_status(req.to_owned(),sig.to_owned()).await
                }
                Some(sig) => update_mint_token_status(req.to_owned(),sig.to_owned()).await,
            }
        }
        TxStatus::Pending => {
            match req.signature.to_owned() {
                None => {
                    log!(DEBUG, "[service::mint_token] the mint token request ({:?}) is handling,pls wait ...", req);
                }
                Some(sig) => update_mint_token_status(req.to_owned(),sig.to_owned()).await,
            }
        }
        TxStatus::Finalized => {
            log!(
                DEBUG,
                "[service::mint_token] {:?} already finalized ,pls update tx hash to hub!",
                req.ticket_id.to_string()
            );
        }
    }

    let q = read_state(|s| s.mint_token_requests.get(&req.ticket_id));
    match q {
        None => Err(CallError {
            method: "[service::mint_token] mint_token".to_string(),
            reason: Reason::CanisterError(format!(
                "Not found ticket({}) mint token request",
                req.ticket_id.to_string()
            )),
        }),

        Some(q) => Ok(q.status),
    }
}

// devops method
#[update(guard = "is_admin",hidden = true)]
pub async fn update_tx_hash_to_hub(sig: String, ticket_id: String) -> Result<(), CallError> {
    let hub_principal = read_state(|s| s.hub_principal);
  
    match update_tx_hash(hub_principal, ticket_id.to_string(),  sig.to_owned()).await {
        Ok(()) =>{
            log!(
                DEBUG,
                "[service::update_tx_hash_to_hub] successfully update tx hash ({})) to hub! ",
                sig
            );
            //only finalized mint_req, remove the handled ticket from queue
            // remove_ticket_from_quene(ticket_id.to_string()).await;
        }
        Err(err) =>  {
            log!(
                ERROR,
                "[service::update_tx_hash_to_hub] failed to update tx hash ({})) to hub : {}",
                sig,err
            );
        }
       }
        
    Ok(())
}

// query collect fee account 
#[query]
pub async fn get_fee_account() -> String {
    read_state(|s| s.fee_account.to_string())
}

// update collect fee account 
#[update(guard = "is_admin",hidden = true)]
pub async fn update_fee_account(fee_account: String)  {
    mutate_state(|s| s.fee_account = fee_account)
}

// query fee account for the dst chain 
#[query]
pub fn get_redeem_fee(chain_id: ChainId) -> Option<u128> {
    read_state(|s| s.get_fee(chain_id))
}

// generate ticket ,called by front end or other sys
#[update]
async fn generate_ticket(args: GenerateTicketReq) -> Result<GenerateTicketOk, GenerateTicketError> {
   gen_ticket::generate_ticket(args).await
}

// devops method
#[query(guard = "is_admin",hidden = true)]
pub fn gen_tickets_req(signature:String) -> Option<GenerateTicketReq> {
    read_state(|s| {
        s.gen_ticket_reqs.get(&signature)
    })
}

// devops method
#[update(guard = "is_admin",hidden = true)]
pub async fn remove_gen_tickets_req(signature:String) -> Option<GenerateTicketReq> {
    mutate_state(|state| {
        state
            .gen_ticket_reqs
            .remove(&signature)
    })

}

// devops method
#[query(guard = "is_admin",hidden = true)]
pub fn get_failed_tickets_to_hub() -> Vec<Ticket> {
    read_state(|s| {
        s.tickets_failed_to_hub
            .iter()
            .map(|(_, ticket)| ticket)
            .collect()
    })
}

// devops method
#[query(guard = "is_admin",hidden = true)]
pub fn get_failed_ticket_to_hub(ticket_id:String) -> Option<Ticket> {
    read_state(|s| {
        s.tickets_failed_to_hub.get(&ticket_id)
    })
}

// devops method
// when gen ticket and send it to hub failed ,call this method
#[update(guard = "is_admin",hidden = true)]
pub async fn send_failed_tickets_to_hub() -> Result<(), GenerateTicketError> {
    let tickets_size = read_state(|s| s.tickets_failed_to_hub.len());
    while !read_state(|s| s.tickets_failed_to_hub.is_empty()) {
        let (ticket_id, ticket) = mutate_state(|rs| rs.tickets_failed_to_hub.pop_first()).unwrap();

        let hub_principal = read_state(|s| (s.hub_principal));
        if let Err(err) = send_ticket(hub_principal, ticket.to_owned())
            .await
            .map_err(|err| GenerateTicketError::SendTicketErr(format!("{}", err)))
        {
            mutate_state(|state| {
                state
                    .tickets_failed_to_hub
                    .insert(ticket_id, ticket.to_owned());
            });
            log!(ERROR, "failed to resend ticket: {}", ticket.ticket_id);
            return Err(err);
        }
    }
    log!(DEBUG, "successfully resend {} tickets", tickets_size);
    Ok(())
}

// devops method
// when gen ticket and send it to hub failed ,call this method
#[update(guard = "is_admin",hidden = true)]
pub async fn send_failed_ticket_to_hub(ticket_id:String) -> Result<(), GenerateTicketError> {
    if let Some(ticket) = read_state(|rs| rs.tickets_failed_to_hub.get(&ticket_id)) {
        let hub_principal = read_state(|s| (s.hub_principal));
        match send_ticket(hub_principal, ticket.to_owned())
            .await
           
        {
           Ok(()) => {
                mutate_state(|state| {
                state
                    .tickets_failed_to_hub
                    .remove(&ticket_id)});
                log!(DEBUG, "successfully resend ticket : {} ", ticket_id);
                return Ok(())
           },
           Err(err) =>{
                log!(ERROR, "failed to resend ticket: {}, error: {:?}", ticket_id,err);
                return Err(GenerateTicketError::SendTicketErr(format!("{}", err)))
           }
        }

    }

    Ok(())
   
   
}

// devops method
#[update(guard = "is_admin",hidden = true)]
pub async fn remove_failed_tickets_to_hub(ticket_id: String) -> Option<Ticket> {
    mutate_state(|state| {
        state
            .tickets_failed_to_hub
            .remove(&ticket_id)
    })

}

// devops method
#[query(guard = "is_admin",hidden = true)]
pub async fn seqs()-> Seqs {
    read_state(|s| {
        s.seqs.to_owned()
    })
}

// devops method
#[update(guard = "is_admin",hidden = true)]
pub async fn update_seqs(seqs: Seqs) {
    mutate_state(|s| {
        s.seqs = seqs;
    })
}

// devops method
#[update(guard = "is_admin",hidden = true)]
pub async fn set_permissions(caller: Principal, perm: Permission) {
    set_perms(caller.to_string(), perm)
}

// devops method
#[update(guard = "is_admin",hidden = true)]
pub fn debug(enable: bool) {
    mutate_state(|s| s.enable_debug = enable);
}

#[query(hidden = true)]
fn http_request(req: HttpRequest) -> HttpResponse {
    if ic_cdk::api::data_certificate().is_none() {
        ic_cdk::trap("update call rejected");
    }
   
    match  req.path() {
        "/logs" => {
            let endable_debug = read_state(|s|s.enable_debug);
            ic_log::http_log(req,endable_debug)
        },
        "/token_uri" => {
            match req.raw_query_param("id") {
                None => HttpResponseBuilder::bad_request()
                .with_body_and_content_length("pls provide token id")
                .build(),
                Some(id) => {
                    use urlencoding::decode;
                    let id:String = decode(id).unwrap().into_owned();
                    let token = read_state(|s|s.tokens.get(&id).to_owned());
                    
            
                    match token {
                        None => HttpResponseBuilder::bad_request()
                        .with_body_and_content_length(format!("not found the {} token uri ",id))
                        .build(),
                        Some(t) => {
                            let token_uri: TokenUri = t.into();
                            HttpResponseBuilder::ok()
                            .header("Content-Type", "application/json; charset=utf-8")
                            .with_body_and_content_length(serde_json::to_string(&token_uri).unwrap_or_default())
                            .build()
                        }

                    }

                   
                }
            }
        },
        "/token_meta" => {
            match req.raw_query_param("id") {
                None => HttpResponseBuilder::bad_request()
                .with_body_and_content_length("pls provide token id")
                .build(),
                Some(id) => {
                    use urlencoding::decode;
                    let id:String = decode(id).unwrap().into_owned();
                    let token = read_state(|s|s.tokens.get(&id).to_owned());
                    
            
                    match token {
                        None => HttpResponseBuilder::bad_request()
                        .with_body_and_content_length(format!("not found the {} token meta",id))
                        .build(),
                        Some(t) => {
                            let token_meta: TokenMeta = t.into();
                            HttpResponseBuilder::ok()
                            .header("Content-Type", "application/json; charset=utf-8")
                            .with_body_and_content_length(serde_json::to_string(&token_meta).unwrap_or_default())
                            .build()
                        }

                    }

                   
                }
            }
        }
       
        _ => HttpResponseBuilder::not_found().build()
    }
  
}

// Enable Candid export
ic_cdk::export_candid!();

mod test {
    
    // use urlencoding::decode;

    #[test]
    fn test_urlencode_decode() {
        let encoded = "Bitcoin-runes-HOPE%E2%80%A2YOU%E2%80%A2GET%E2%80%A2NICE202410141209";
        let decoded = urlencoding::decode(encoded).unwrap();
        println!("Decoded: {}", decoded);  // Bitcoin-runes-HOPE•YOU•GET•NICE202410141209
        let decoded_string: String = decoded.into_owned();
        println!("decoded_string: {}", decoded_string);  
        let encoded = "Bitcoin-runes-HOPE•YOU•GET•NICE202410141209";
        let decoded = urlencoding::decode(encoded).unwrap();
        println!("Decoded: {}", decoded);  // Bitcoin-runes-HOPE•YOU•GET•NICE202410141209
        let decoded_string: String = decoded.into_owned();
        println!("decoded_string: {}", decoded_string);  
    
    }
}