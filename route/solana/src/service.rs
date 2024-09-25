use crate::auth::{is_admin, set_perms, Permission};
use crate::call_error::{CallError, Reason};
use crate::handler::associated_account::update_ata_status;
use candid::Principal;
use ic_cdk::{init, post_upgrade, pre_upgrade, query, update};
use ic_solana::types::TransactionStatus;

use crate::handler::mint_token::{
    self, update_tx_hash, 
};
use crate::handler::gen_ticket::{self,send_ticket,query_tx_from_multi_rpc,GenerateTicketError,
    GenerateTicketOk, GenerateTicketReq};
    
use crate::handler::{ scheduler, solana_rpc};
use crate::lifecycle::{self, RouteArg, UpgradeArgs};
use crate::service::solana_rpc::solana_client;
use crate::state::{AccountInfo, AtaKey, MultiRpcConfig, TokenResp};
use crate::types::TokenId;
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
use ic_solana::types::TransactionConfirmationStatus;
use std::str::FromStr;
use crate::handler::token_account::update_mint_account_status;
use ic_canisters_http_types::{HttpRequest, HttpResponse};
use ic_solana::ic_log::{self, DEBUG, ERROR};

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

#[update(guard = "is_admin")]
pub fn start_schedule() {
    log!(DEBUG, "start schedule task ...");
    scheduler::start_schedule();
}

#[update(guard = "is_admin")]
pub fn cancel_schedule() {
    log!(DEBUG, "cancel schedule task ...");
    scheduler::cancel_schedule();
}

#[update(guard = "is_admin")]
pub async fn update_schnorr_key(key_name: String) {
    mutate_state(|s| {
        s.schnorr_key_name = key_name;
    })
}

#[update(guard = "is_admin")]
pub async fn update_forward(forward: Option<String>) {
    mutate_state(|s| {
        s.forward = forward;
    })
}

#[update(guard = "is_admin")]
pub async fn update_multi_rpc(multi_prc_cofig: MultiRpcConfig) {
    mutate_state(|s| {
        s.multi_rpc_config = multi_prc_cofig;
    })
}
#[query]
pub async fn multi_rpc_config() -> MultiRpcConfig {
    read_state(|s| s.multi_rpc_config.to_owned())
}

#[update]
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

#[update]
pub async fn signer() -> Result<String, String> {
    let pk = solana_rpc::eddsa_public_key().await?;
    Ok(pk.to_string())
}

#[update]
pub async fn sign(msg: String) -> Result<Vec<u8>, String> {
    let signature = solana_rpc::sign(msg).await?;
    Ok(signature)
}

#[update]
pub async fn get_fee_account() -> String {
    read_state(|s| s.fee_account.to_string())
}

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

#[update]
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

#[update]
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

#[update]
async fn get_signature_status(
    signatures: Vec<String>,
) -> Result<Vec<TransactionStatus>, CallError> {
    solana_rpc::get_signature_status(signatures).await
}

#[update(guard = "is_admin")]
pub async fn derive_mint_account(req: TokenInfo) -> Result<String, CallError> {
    let sol_client = solana_client().await;

    let mint_account =
        SolanaClient::derive_account(sol_client.chainkey_name.clone(), req.token_id.to_string())
            .await;

    Ok(mint_account.to_string())
}

#[update(guard = "is_admin")]
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

#[query]
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

#[update(guard = "is_admin")]
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
                        "[directive::create_token_mint] {:?} already created and waiting for {:} finallized ... ",
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

#[update(guard = "is_admin")]
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

#[update(guard = "is_admin")]
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

#[query]
pub async fn query_aossicated_account(
    owner: Owner,
    token_mint: MintAccount,
) -> Option<AccountInfo> {
    read_state(|s| s.associated_accounts.get(&AtaKey { owner, token_mint }))
}
#[query]
pub async fn query_aossicated_account_address(
    owner: Owner,
    token_mint: MintAccount,
) -> Option<String> {
    read_state(
        |s| match s.associated_accounts.get(&AtaKey { owner, token_mint }) {
            None => None,
            Some(ata) => {
                if matches!(ata.status, TxStatus::Finalized { .. }) {
                    Some(ata.account.to_string())
                } else {
                    None
                }
            }
        },
    )
}

#[update(guard = "is_admin")]
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



#[update(guard = "is_admin")]
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


#[query]
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


#[update(guard = "is_admin")]
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


#[update(guard = "is_admin")]
pub async fn mint_token(req: MintTokenRequest) -> Result<TxStatus, CallError> {
    let mut req = if let Some(req) = read_state(|s| s.mint_token_requests.get(&req.ticket_id)) {
        req
    } else {
        log!(
            DEBUG,
            "[service::mint_token] not found mint token req for ticket: {:?} ",
            req.ticket_id
        );
        req
    };
    log!(DEBUG, "[service::mint_token] mint token request: {:?} ", req);

    match &req.status {
        TxStatus::New | TxStatus::TxFailed { .. } => {
            match req.signature.clone() {
                None => {
                    // new mint req
                    let sig = solana_rpc::mint_to(
                      req.to_owned(),
                    )
                    .await?;

                    // update signature
                    req.signature = Some(sig.to_string());
                    mutate_state(|s| {
                        s.update_mint_token_req(req.ticket_id.to_owned(), req.clone())
                    });

                    // update req status
                    update_mint_req(sig, req.ticket_id.to_string()).await?;
                }
                Some(sig) => update_mint_req(sig, req.ticket_id.to_string()).await?,
            }
        }
        TxStatus::Pending => {
            match req.signature.to_owned() {
                None => {
                    log!(DEBUG, "[service::mint_token] the  mint token request ({:?}) is creating,pls wait ...", req);
                }
                Some(sig) => update_mint_req(sig, req.ticket_id.to_string()).await?,
            }
        }
        TxStatus::Finalized => {
            log!(
                DEBUG,
                "[service::mint_token] {:?} already finalized !",
                req.ticket_id.to_string()
            );
            // update txhash to hub
            let hub_principal = read_state(|s| s.hub_principal);
            if let Err(err) = update_tx_hash(
                hub_principal,
                req.ticket_id.to_string(),
                req.signature.unwrap(),
            )
            .await
            {
                log!(
                    ERROR,
                    "[service::mint_token] failed to update tx hash after mint token:{}",
                    err
                );
            }
            // remove the handled ticket from queue
            remove_ticket_from_quene(req.ticket_id.to_string()).await;
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

async fn update_mint_req(sig: String, ticket_id: String) -> Result<(), CallError> {
    let tx_status_vec = solana_rpc::get_signature_status(vec![sig.to_string()]).await?;

    if let Some(tx_status) = tx_status_vec.first() {
        log!(
            DEBUG,
            "[service::mint_to] signature: {} status: {:?} ",
            sig.to_string(),
            tx_status,
        );
        if let Some(status) = &tx_status.confirmation_status {
            if matches!(status, TransactionConfirmationStatus::Finalized) {
                // update req status

                mutate_state(|s| {
                    // s.mint_token_requests
                    //     .get_mut(&ticket_id)
                    //     .map(|req| req.status = TxStatus::Finalized)
                    if let Some(req) = s.mint_token_requests.get(&ticket_id).as_mut() {
                        req.status = TxStatus::Finalized;
                        s.mint_token_requests
                            .insert(ticket_id.to_string(), req.to_owned());
                    }
                });
                // update txhash to hub
                let hub_principal = read_state(|s| s.hub_principal);
                if let Err(err) =
                    update_tx_hash(hub_principal, ticket_id.to_string(), sig.to_string()).await
                {
                    log!(
                        ERROR,
                        "[tickets::mint_token] failed to update tx hash after mint token:{}",
                        err
                    );
                }
                // remove the handled ticket from queue
                remove_ticket_from_quene(ticket_id).await;
            }
        }
    }
    Ok(())
}

#[query]
fn get_ticket_from_queue(ticket_id: String) -> Option<(u64, Ticket)> {
    read_state(|s| {
        s.tickets_queue
            .iter()
            .find(|(_seq, ticket)| ticket.ticket_id.eq(&ticket_id))
    })
}

#[query]
fn get_tickets_from_queue() -> Vec<(u64, Ticket)> {
    read_state(|s| {
        s.tickets_queue
            .iter()
            .map(|(seq, ticket)| (seq, ticket))
            .collect()
    })
}

#[update(guard = "is_admin")]
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

#[update(guard = "is_admin")]
pub async fn transfer_to(to_account: String, amount: u64) -> Result<String, CallError> {
    solana_rpc::transfer_to(to_account, amount).await
}

#[query]
pub fn get_redeem_fee(chain_id: ChainId) -> Option<u128> {
    read_state(|s| s.get_fee(chain_id))
}

#[update]
async fn generate_ticket(args: GenerateTicketReq) -> Result<GenerateTicketOk, GenerateTicketError> {
   gen_ticket::generate_ticket(args).await
}

#[query]
pub fn get_tickets_failed_to_hub() -> Vec<Ticket> {
    read_state(|s| {
        s.tickets_failed_to_hub
            .iter()
            .map(|(_, ticket)| ticket)
            .collect()
    })
}

#[update(guard = "is_admin")]
pub async fn resend_tickets() -> Result<(), GenerateTicketError> {
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
            log!(ERROR, "resend ticket: {}", ticket.ticket_id);
            return Err(err);
        }
    }
    log!(DEBUG, "successfully resend {} tickets", tickets_size);
    Ok(())
}

#[update(guard = "is_admin")]
pub async fn set_permissions(caller: Principal, perm: Permission) {
    set_perms(caller.to_string(), perm)
}

#[query(hidden = true)]
fn http_request(req: HttpRequest) -> HttpResponse {
    if ic_cdk::api::data_certificate().is_none() {
        ic_cdk::trap("update call rejected");
    }
    ic_log::http_request(req)
}

// Enable Candid export
ic_cdk::export_candid!();
