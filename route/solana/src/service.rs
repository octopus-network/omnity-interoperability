use crate::auth::{auth_update, is_controller};
use crate::call_error::{CallError, Reason};
use crate::constants::RETRY_4_BUILDING;
use crate::eddsa::KeyType;
use crate::guard::TaskType;
use crate::handler::associated_account;
use crate::solana_client::solana_client;
use crate::solana_client::solana_rpc::{SolanaClient, TokenInfo};

use candid::Principal;
use ic_cdk::{init, post_upgrade, pre_upgrade, query, update};
use ic_solana::rpc_client::{RpcApi, RpcServices};
use ic_solana::types::tagged::{UiAccount, UiTransaction};
use ic_solana::types::TransactionStatus;
use ic_spl::compute_budget::compute_budget::Priority;

use crate::handler::gen_ticket::{
    self, send_ticket, GenerateTicketError, GenerateTicketOk, GenerateTicketReq,
};
use crate::handler::mint_token::{self, update_tx_hash};

use crate::handler::{scheduler, token_account};
use crate::lifecycle::{self, RouteArg, UpgradeArgs};

use crate::state::{
    AccountInfo, AtaKey, RpcProvider, SnorKeyType, TokenMeta, TokenResp, KEY_TYPE_NAME,
};
use crate::types::{TicketId, TicketWithMemo, Token, TokenId};

use crate::service::mint_token::MintTokenRequest;
use crate::state::MintAccount;
use crate::state::Owner;
use crate::state::{mutate_state, read_state, TxStatus};
use crate::types::ChainState;
use crate::types::{Chain, ChainId, Ticket};
use ic_canister_log::log;
use ic_spl::token::associated_account::get_associated_token_address_with_program_id;

use ic_solana::types::Pubkey;
use std::str::FromStr;

use crate::logs::{http_log, DEBUG, ERROR};
use crate::state::Seqs;
use crate::state::TokenUri;
use crate::types::Factor;
use ic_canisters_http_types::{HttpRequest, HttpResponse, HttpResponseBuilder};
use ic_spl::token::constants::token_program_id;
use std::time::Duration;

async fn get_random_seed() -> [u8; 64] {
    match ic_cdk::api::management_canister::main::raw_rand().await {
        Ok(rand) => {
            let mut rand = rand.0;
            rand.extend(rand.clone());
            let rand: [u8; 64] = rand.try_into().expect("Expected a Vec of length 64");
            rand
        }
        Err(err) => {
            ic_cdk::trap(format!("Error getting random seed: {:?}", err).as_str());
        }
    }
}

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
    // init seeds
    ic_cdk_timers::set_timer(Duration::ZERO, || {
        ic_cdk::spawn(async move {
            let seed = get_random_seed().await;
            mutate_state(|s| s.seeds.insert(KEY_TYPE_NAME.to_string(), seed));
        });
    });
}

#[pre_upgrade]
fn pre_upgrade() {
    log!(DEBUG, "begin to execute pre_upgrade ...");
    scheduler::stop_schedule(None);
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
    scheduler::start_schedule(None);
    log!(DEBUG, "upgrade successfully!");
}

// devops method
#[update(guard = "is_controller", hidden = true)]
pub fn start_schedule(tasks: Option<Vec<TaskType>>) {
    log!(DEBUG, "start schedule task: {:?} ... ", tasks);
    scheduler::start_schedule(tasks);
}

// devops method
#[update(guard = "is_controller", hidden = true)]
pub fn stop_schedule(tasks: Option<Vec<TaskType>>) {
    log!(DEBUG, "stop schedule task: {:?} ...", tasks);
    scheduler::stop_schedule(tasks);
}

// devops method
#[query(guard = "is_controller", hidden = true)]
pub async fn active_tasks() -> Vec<TaskType> {
    read_state(|s| s.active_tasks.iter().map(|t| t.to_owned()).collect())
}

// devops method
#[update(guard = "is_controller", hidden = true)]
pub async fn update_schnorr_key(key_name: String) {
    mutate_state(|s| {
        s.schnorr_key_name = key_name;
    })
}

// devops method
#[query(guard = "is_controller", hidden = true)]
pub async fn proxy() -> String {
    read_state(|s| s.proxy.to_owned())
}

// devops method
#[update(guard = "is_controller", hidden = true)]
pub async fn update_proxy(proxy: String) {
    mutate_state(|s| s.proxy = proxy)
}

// devops method
#[update(guard = "is_controller", hidden = true)]
pub async fn update_providers(providers: Vec<RpcProvider>) {
    mutate_state(|s| s.providers = providers)
}

// devops method
#[query(guard = "is_controller", hidden = true)]
pub async fn provider() -> Vec<RpcProvider> {
    read_state(|s| s.providers.to_owned())
}

// devops method
#[update(guard = "is_controller", hidden = true)]
pub async fn priority() -> Option<Priority> {
    read_state(|s| s.priority.to_owned())
}

// devops method
#[update(guard = "is_controller", hidden = true)]
pub async fn update_priority(priority: Priority) {
    mutate_state(|s| s.priority = Some(priority))
}

// devops method
#[update(guard = "is_controller", hidden = true)]
pub async fn key_type() -> SnorKeyType {
    read_state(|s| s.key_type.to_owned().into())
}

// devops method
#[update(guard = "is_controller", hidden = true)]
pub async fn update_key_type(key_type: SnorKeyType) {
    let key_type = match key_type {
        SnorKeyType::ChainKey => KeyType::ChainKey,
        SnorKeyType::Native => {
            let seed = get_random_seed().await;
            KeyType::Native(seed.to_vec())
        }
    };
    mutate_state(|s| s.key_type = key_type)
}

// devops method
#[update(guard = "is_controller", hidden = true)]
pub async fn minimum_response_count() -> u32 {
    read_state(|s| s.minimum_response_count)
}

// devops method
#[update(guard = "is_controller", hidden = true)]
pub async fn update_minimum_response_count(count: u32) {
    mutate_state(|s| s.minimum_response_count = count)
}

// devops method
#[update(guard = "is_controller", hidden = true)]
pub async fn signer(key_type: SnorKeyType) -> Result<String, String> {
    let key_type = match key_type {
        SnorKeyType::ChainKey => KeyType::ChainKey,
        SnorKeyType::Native => {
            let seed = read_state(|s| {
                s.seeds
                    .get(&KEY_TYPE_NAME.to_string())
                    .unwrap_or_else(|| panic!("No key with name {:?}", &KEY_TYPE_NAME.to_string()))
            });
            KeyType::Native(seed.to_vec())
        }
    };
    let pk = crate::solana_client::eddsa_public_key(key_type).await?;
    Ok(pk.to_string())
}

// devops method
#[update(guard = "is_controller", hidden = true)]
pub async fn sign(msg: String, key_type: SnorKeyType) -> Result<Vec<u8>, String> {
    let key_type = match key_type {
        SnorKeyType::ChainKey => KeyType::ChainKey,
        SnorKeyType::Native => {
            let seed = read_state(|s| {
                s.seeds
                    .get(&KEY_TYPE_NAME.to_string())
                    .unwrap_or_else(|| panic!("No key with name {:?}", &KEY_TYPE_NAME.to_string()))
            });
            KeyType::Native(seed.to_vec())
        }
    };
    let signature = crate::solana_client::sign(msg, key_type).await?;
    Ok(signature)
}

// query supported chain list
#[query]
fn get_chain_list() -> Vec<Chain> {
    read_state(|s| {
        s.counterparties
            .iter()
            .filter(|(_, chain)| matches!(chain.chain_state, ChainState::Active))
            .map(|(_, chain)| chain.to_owned())
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
#[query(guard = "auth_update", hidden = true)]
fn get_token(token_id: TokenId) -> Option<Token> {
    read_state(|s| s.tokens.get(&token_id))
}

// devops method
#[update(guard = "auth_update")]
async fn get_latest_blockhash() -> Result<String, CallError> {
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
#[update(guard = "auth_update")]
async fn get_transaction(
    signature: String,
    // forward: Option<String>,
) -> Result<UiTransaction, CallError> {
    let client = solana_client().await;
    client
        .query_transaction(signature)
        .await
        .map_err(|err| CallError {
            method: "get_transaction".to_string(),
            reason: Reason::CanisterError(err.to_string()),
        })
}

// devops method
#[update(guard = "auth_update")]
async fn get_raw_transaction(
    signature: String,
    // forward: Option<String>,
) -> Result<String, CallError> {
    let client = solana_client().await;
    let providers = read_state(|s| (s.providers.to_owned()));

    let rpc_apis: Vec<_> = providers
        .iter()
        .map(|p| RpcApi {
            network: p.rpc_url(),
            headers: p.headers.to_owned(),
        })
        .collect();
    let source = RpcServices::Custom(rpc_apis[0..1].to_vec());
    let resp = client
        .query_raw_transaction(source, signature)
        .await
        .map_err(|err| CallError {
            method: "get_raw_transaction".to_string(),
            reason: Reason::CanisterError(err.to_string()),
        })?;
    String::from_utf8(resp).map_err(|err| CallError {
        method: "String::from_utf8".to_string(),
        reason: Reason::CanisterError(err.to_string()),
    })
}

// devops method
#[update(guard = "auth_update", hidden = true)]
async fn get_tx_instructions(
    signature: String,
    // forward: Option<String>,
) -> Result<String, CallError> {
    let client = solana_client().await;
    let providers = read_state(|s| (s.providers.to_owned()));

    let rpc_apis: Vec<_> = providers
        .iter()
        .map(|p| RpcApi {
            network: p.rpc_url(),
            headers: p.headers.to_owned(),
        })
        .collect();
    let source = RpcServices::Custom(rpc_apis[0..1].to_vec());
    let resp = client
        .query_raw_transaction(source, signature)
        .await
        .map_err(|err| CallError {
            method: "query_parsed_transaction".to_string(),
            reason: Reason::CanisterError(err.to_string()),
        })?;
    let instructions = gen_ticket::get_instruction(&resp).map_err(|err| CallError {
        method: "get_instruction".to_string(),
        reason: Reason::CanisterError(err.to_string()),
    })?;
    serde_json::to_string(&instructions).map_err(|err| CallError {
        method: "serde_json::to_string".to_string(),
        reason: Reason::CanisterError(err.to_string()),
    })
}

// devops method
#[update(guard = "auth_update")]
async fn get_signature_status(
    signatures: Vec<String>,
) -> Result<Vec<Option<TransactionStatus>>, CallError> {
    crate::solana_client::get_signature_status(signatures).await
}

// devops method
#[update(guard = "is_controller", hidden = true)]
pub async fn transfer_to(to_account: String, amount: u64) -> Result<String, CallError> {
    crate::solana_client::transfer_to(to_account, amount).await
}

// devops method
#[update(guard = "is_controller", hidden = true)]
pub async fn derive_mint_account(
    req: TokenInfo,
    key_type: SnorKeyType,
) -> Result<String, CallError> {
    let sol_client = solana_client().await;
    let key_type = match key_type {
        SnorKeyType::ChainKey => KeyType::ChainKey,
        SnorKeyType::Native => {
            let seed = read_state(|s| {
                s.seeds
                    .get(&KEY_TYPE_NAME.to_string())
                    .unwrap_or_else(|| panic!("No key with name {:?}", &KEY_TYPE_NAME.to_string()))
            });
            KeyType::Native(seed.to_vec())
        }
    };
    let mint_account = SolanaClient::derive_account(
        key_type,
        sol_client.chainkey_name.to_owned(),
        req.token_id.to_string(),
    )
    .await;

    Ok(mint_account.to_string())
}

// devops method
#[update(guard = "auth_update")]
pub async fn get_account_info(account: String) -> Result<Option<UiAccount>, CallError> {
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
#[update]
pub async fn get_balance(pubkey: String) -> Result<u64, String> {
    let sol_client = solana_client().await;

    // query account info from solana
    let balance = sol_client
        .get_balance(pubkey.to_string())
        .await
        .map_err(|e| e.to_string())?;
    log!(
        DEBUG,
        "[service::get_balance] account: {} current balance: {:?} ",
        pubkey.to_string(),
        balance,
    );
    Ok(balance)
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
#[query(hidden = false)]
pub async fn query_mint_account(token_id: TokenId) -> Option<AccountInfo> {
    read_state(|s| s.token_mint_accounts.get(&token_id))
}

// devops method
#[query(hidden = true)]
pub async fn failed_mint_accounts() -> Vec<(TokenId, AccountInfo)> {
    read_state(|s| {
        s.token_mint_accounts
            .iter()
            .filter(|(_, v)| {
                v.retry_4_building >= RETRY_4_BUILDING && !matches!(v.status, TxStatus::Finalized)
            })
            .map(|(k, v)| (k, v))
            .collect()
    })
}

// devops method
#[query(hidden = true)]
pub async fn failed_ata() -> Vec<(AtaKey, AccountInfo)> {
    read_state(|s| {
        s.associated_accounts
            .iter()
            .filter(|(_, v)| {
                v.retry_4_building >= RETRY_4_BUILDING && !matches!(v.status, TxStatus::Finalized)
            })
            .map(|(k, v)| (k, v))
            .take(3)
            .collect()
    })
}

// devops method
// add token manually
#[update(guard = "is_controller", hidden = true)]
pub async fn add_token(token: Token) -> Option<Token> {
    mutate_state(|s| {
        s.tokens
            .insert(token.token_id.to_string(), token.to_owned())
    })
}

// devops method
#[update(guard = "is_controller", hidden = true)]
fn update_token(token: Token) -> Result<Option<Token>, CallError> {
    mutate_state(|s| match s.tokens.get(&token.token_id) {
        None => Err(CallError {
            method: "[service::update_token] update_token".to_string(),
            reason: Reason::CanisterError(format!(
                "Not found token id {} ",
                token.token_id.to_string()
            )),
        }),
        Some(_) => Ok(s
            .tokens
            .insert(token.token_id.to_string(), token.to_owned())),
    })
    // Ok(())
}

// devops method
#[update(guard = "is_controller", hidden = true)]
fn remove_token_and_account(token_id: TokenId) {
    mutate_state(|s| {
        s.tokens.remove(&token_id);
        s.token_mint_accounts.remove(&token_id);
    })
}

// devops method
#[update(guard = "is_controller", hidden = true)]
pub async fn create_mint_account(
    req: TokenInfo,
    key_type: SnorKeyType,
) -> Result<AccountInfo, CallError> {
    let sol_client = solana_client().await;
    let key_type = match key_type {
        SnorKeyType::ChainKey => KeyType::ChainKey,
        SnorKeyType::Native => {
            let seed = read_state(|s| {
                s.seeds
                    .get(&KEY_TYPE_NAME.to_string())
                    .unwrap_or_else(|| panic!("No key with name {:?}", &KEY_TYPE_NAME.to_string()))
            });
            KeyType::Native(seed.to_vec())
        }
    };
    let mint_account = SolanaClient::derive_account(
        key_type,
        sol_client.chainkey_name.to_owned(),
        req.token_id.to_string(),
    )
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
                signature: None,
                status: TxStatus::New,
                retry_4_building: 0,
                retry_4_status: 0,
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
                retry_4_building: mint_account_info.retry_4_building,
                retry_4_status: mint_account_info.retry_4_status,
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
                    let sig =
                        crate::solana_client::create_mint_account(mint_account, req.to_owned())
                            .await?;
                    log!(
                        DEBUG,
                        "[service::create_mint_account] create_mint_account signature: {:?} ",
                        sig.to_string(),
                    );

                    // update signature
                    mint_account_info.signature = Some(sig.to_string());
                    mint_account_info.retry_4_building += 1;
                    mutate_state(|s| {
                        s.token_mint_accounts
                            .insert(req.token_id.to_string(), mint_account_info.to_owned())
                    });

                    // update mint account status
                    token_account::update_mint_account_status(
                        sig.to_string(),
                        req.token_id.to_string(),
                    )
                    .await;
                }
                Some(sig) => {
                    // update mint account status
                    token_account::update_mint_account_status(
                        sig.to_string(),
                        req.token_id.to_string(),
                    )
                    .await;
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
                    token_account::update_mint_account_status(
                        sig.to_string(),
                        req.token_id.to_string(),
                    )
                    .await;
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

#[update(guard = "is_controller", hidden = true)]
pub async fn rebuild_mint_account(token_id: String) -> Result<String, CallError> {
    let token = read_state(|s| s.tokens.get(&token_id)).unwrap();
    let token_info = TokenInfo {
        token_id: token.token_id.to_string(),
        name: token.name.to_string(),
        symbol: token.symbol.to_string(),
        decimals: token.decimals,
        uri: token.icon.unwrap_or_default(),
    };
    let mint_account_info =
        if let Some(account_info) = read_state(|s| s.token_mint_accounts.get(&token_id)) {
            account_info
        } else {
            return Err(CallError {
                method: "rebuild_mint_account".to_string(),
                reason: Reason::CanisterError("not found token mint account".to_string()),
            });
        };

    log!(
        DEBUG,
        "[service::rebuild_mint_account] mint_account_info from solana route: {:?} ",
        mint_account_info,
    );
    let mint_account = Pubkey::from_str(&mint_account_info.account).unwrap();

    let ret: Result<String, CallError> =
        crate::solana_client::create_mint_account(mint_account, token_info).await;
    log!(
        DEBUG,
        "[service::rebuild_mint_account] rebuild_mint_account ret: {:?} ",
        ret,
    );
    match &ret {
        Ok(sig) => {
            // update signature
            let mint = AccountInfo {
                account: mint_account_info.account.to_string(),
                retry_4_building: mint_account_info.retry_4_building + 1,
                retry_4_status: 0,
                signature: Some(sig.to_string()),
                status: TxStatus::Pending,
            };
            // update status and signature
            mutate_state(|s| s.token_mint_accounts.insert(token_id.to_owned(), mint));
        }
        Err(e) => {
            let tx_error = match &e.reason {
                Reason::QueueIsFull
                | Reason::OutOfCycles
                | Reason::CanisterError(_)
                | Reason::Rejected(_) => todo!(),
                Reason::TxError(tx_error) => tx_error,
            };
            // update status and error
            let mint = AccountInfo {
                account: mint_account_info.account.to_string(),
                retry_4_building: mint_account_info.retry_4_building + 1,
                retry_4_status: 0,
                signature: None,
                status: TxStatus::TxFailed {
                    e: tx_error.to_owned(),
                },
            };
            mutate_state(|s| s.token_mint_accounts.insert(token_id.to_owned(), mint));
        }
    }
    ret
}

// devops method
#[update(guard = "is_controller", hidden = true)]
pub async fn update_mint_account_status(
    sig: String,
    token_id: String,
) -> Result<AccountInfo, CallError> {
    let mint_account =
        if let Some(mint_account) = read_state(|s| s.token_mint_accounts.get(&token_id)) {
            mint_account
        } else {
            return Err(CallError {
                method: "update_mint_account_status".to_string(),
                reason: Reason::CanisterError("not found mint account".to_string()),
            });
        };
    log!(
        DEBUG,
        "[service::update_mint_account_status] mint account: {:?} ",
        mint_account
    );

    token_account::update_mint_account_status(sig, token_id.to_owned()).await;

    let latest_account = read_state(|s| s.token_mint_accounts.get(&token_id)).unwrap();

    Ok(latest_account)
}
// devops method
#[update(guard = "is_controller", hidden = true)]
pub async fn update_mint_account(
    token_id: TokenId,
    mint_account: AccountInfo,
) -> Option<AccountInfo> {
    //update mint account info
    mutate_state(|s| s.token_mint_accounts.insert(token_id, mint_account))
}

// devops method
#[update(guard = "is_controller", hidden = true)]
pub async fn update_token_metaplex(req: TokenInfo) -> Result<String, CallError> {
    log!(
        DEBUG,
        "[service::update_token_metaplex] token_info: {:?} ",
        req,
    );
    // token_mint must be exists
    match read_state(|s| s.token_mint_accounts.get(&req.token_id)) {
        None => {
            return Err(CallError {
                method: "[service::update_token_metaplex] update_token_metaplex".to_string(),
                reason: Reason::CanisterError(format!(
                    "{} token mint account not exists!",
                    req.token_id
                )),
            });
        }
        Some(account_info) => {
            let signature =
                crate::solana_client::update_with_metaplex(account_info.account, req.to_owned())
                    .await?;
            log!(
                DEBUG,
                "[service::update_token_metaplex] update_token_metaplex signature: {:?} ",
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
                    // token.icon = Some(req.uri);
                    token.metadata.insert("uri".to_string(), req.uri);
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
#[update(guard = "is_controller", hidden = true)]
pub async fn derive_aossicated_account(
    owner: String,
    token_mint: String,
) -> Result<String, CallError> {
    let to_account_pk = Pubkey::from_str(&owner).expect("Invalid to_account address");
    let token_mint_pk = Pubkey::from_str(&token_mint).expect("Invalid token mint address");
    let associated_account = get_associated_token_address_with_program_id(
        &to_account_pk,
        &token_mint_pk,
        &token_program_id(),
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
#[update(guard = "is_controller", hidden = true)]
pub async fn create_aossicated_account(
    owner: String,
    token_mint: String,
) -> Result<AccountInfo, CallError> {
    let to_account_pk = Pubkey::from_str(&owner).expect("Invalid to_account address");
    let token_mint_pk = Pubkey::from_str(&token_mint).expect("Invalid token mint address");
    let associated_account = get_associated_token_address_with_program_id(
        &to_account_pk,
        &token_mint_pk,
        &token_program_id(),
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
            retry_4_building: 0,
            retry_4_status: 0,
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
                retry_4_building: ata_account.retry_4_building,
                retry_4_status: ata_account.retry_4_status,
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
            match ata_account.signature.to_owned() {
                None => {
                    let sig =
                        crate::solana_client::create_ata(owner.to_string(), token_mint.to_string())
                            .await?;
                    log!(
                        DEBUG,
                        "[service::create_aossicated_account] create_aossicated_account signature: {:?} ",
                        sig.to_string(),
                    );
                    // update signature
                    ata_account.signature = Some(sig.to_string());
                    ata_account.retry_4_building += 1;
                    mutate_state(|s| {
                        s.associated_accounts.insert(
                            AtaKey {
                                owner: owner.to_string(),
                                token_mint: token_mint.to_string(),
                            },
                            ata_account.to_owned(),
                        )
                    });
                    // update ata status
                    associated_account::update_ata_status(
                        sig.to_string(),
                        owner.to_string(),
                        token_mint.to_string(),
                    )
                    .await;
                }
                Some(sig) => {
                    associated_account::update_ata_status(
                        sig.to_string(),
                        owner.to_string(),
                        token_mint.to_string(),
                    )
                    .await;
                }
            }
        }
        TxStatus::Pending => match ata_account.signature.to_owned() {
            None => {
                log!(
                        DEBUG,
                        "[service::create_associated_account] the associated account ({:?}) is creating,pls wait ...",
                        ata_account
                    );
            }
            Some(sig) => {
                associated_account::update_ata_status(
                    sig.to_string(),
                    owner.to_string(),
                    token_mint.to_string(),
                )
                .await;
            }
        },
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
#[update(guard = "is_controller", hidden = true)]
pub async fn rebuild_aossicated_account(
    owner: String,
    token_mint: String,
) -> Result<String, CallError> {
    let to_account_pk = Pubkey::from_str(&owner).expect("Invalid to_account address");
    let token_mint_pk = Pubkey::from_str(&token_mint).expect("Invalid token mint address");
    let associated_account = get_associated_token_address_with_program_id(
        &to_account_pk,
        &token_mint_pk,
        &token_program_id(),
    );
    log!(
        DEBUG,
        "[service::rebuild_aossicated_account] get_associated_token_address_with_program_id : {:?}",
        associated_account
    );
    let ata_key = AtaKey {
        owner: owner.to_string(),
        token_mint: token_mint.to_string(),
    };
    let ata = if let Some(account_info) = read_state(|s| s.associated_accounts.get(&ata_key)) {
        account_info
    } else {
        return Err(CallError {
            method: "rebuild_aossicated_account".to_string(),
            reason: Reason::CanisterError("not found associated_accounts info ".to_string()),
        });
    };

    log!(
        DEBUG,
        "[service::rebuild_aossicated_account] ata_info from solana route : {:?}",
        ata
    );

    let ret = crate::solana_client::create_ata(owner.to_string(), token_mint.to_string()).await;
    log!(
        DEBUG,
        "[service::create_aossicated_account] create_aossicated_account signature: {:?} ",
        ret,
    );

    match &ret {
        Ok(sig) => {
            // update signature and status
            let ata = AccountInfo {
                account: ata.account.to_string(),
                retry_4_building: ata.retry_4_building + 1,
                retry_4_status: 0,
                signature: Some(sig.to_owned()),
                status: TxStatus::Pending,
            };
            // update status and signature
            mutate_state(|s| s.associated_accounts.insert(ata_key, ata));
        }
        Err(e) => {
            let tx_error = match &e.reason {
                Reason::QueueIsFull
                | Reason::OutOfCycles
                | Reason::CanisterError(_)
                | Reason::Rejected(_) => todo!(),
                Reason::TxError(tx_error) => tx_error,
            };
            // update status and error
            let ata = AccountInfo {
                account: ata.account.to_string(),
                retry_4_building: ata.retry_4_building + 1,
                retry_4_status: 0,
                signature: None,
                status: TxStatus::TxFailed {
                    e: tx_error.to_owned(),
                },
            };

            mutate_state(|s| s.associated_accounts.insert(ata_key, ata));
        }
    }
    ret
}

// devops method
#[update(guard = "is_controller", hidden = true)]
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

#[update(guard = "is_controller", hidden = true)]
pub async fn update_ata_status(sig: String, ata_key: AtaKey) -> Result<AccountInfo, CallError> {
    let ata = if let Some(ata) = read_state(|s| s.associated_accounts.get(&ata_key)) {
        ata
    } else {
        return Err(CallError {
            method: "update_ata_status".to_string(),
            reason: Reason::CanisterError("not associated account".to_string()),
        });
    };
    log!(DEBUG, "[service::update_ata_status] ata: {:?} ", ata);
    associated_account::update_ata_status(
        sig,
        ata_key.owner.to_owned(),
        ata_key.token_mint.to_owned(),
    )
    .await;

    let latest_account = read_state(|s| s.associated_accounts.get(&ata_key)).unwrap();

    Ok(latest_account)
}

// devops method
#[update(guard = "is_controller", hidden = true)]
pub async fn remove_associated_account(owner: String, token_mint: String) -> Result<(), CallError> {
    log!(
        DEBUG,
        "[service::remove_associated_account] owner: {} and token_mint: {} ",
        owner,
        token_mint
    );

    mutate_state(|s| {
        s.associated_accounts
            .remove(&AtaKey::new(owner.to_string(), token_mint.to_string()))
    });

    Ok(())
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
#[update(guard = "is_controller", hidden = true)]
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

// devops method
#[query(hidden = false)]
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
#[update(guard = "auth_update", hidden = true)]
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
#[update(guard = "is_controller", hidden = true)]
pub async fn mint_to(ata: String, token_mint: String, amount: u64) -> Result<String, CallError> {
    let sol_client = solana_client().await;
    let associated_account = Pubkey::from_str(&ata).expect("Invalid ata address");
    let token_mint = Pubkey::from_str(&token_mint).expect("Invalid token_mint address");

    let signature = sol_client
        .mint_to(associated_account, amount, token_mint, token_program_id())
        .await
        .map_err(|e| CallError {
            method: "mint_to".to_string(),
            reason: Reason::CanisterError(e.to_string()),
        })?;

    Ok(signature)
}

// devops method
#[query(hidden = true)]
pub async fn failed_mint_reqs() -> Vec<(TicketId, MintTokenRequest)> {
    read_state(|s| {
        s.mint_token_requests
            .iter()
            .filter(|(_, v)| matches!(v.status, TxStatus::TxFailed { .. }))
            .map(|(k, v)| (k, v))
            .take(3)
            .collect()
    })
}

// devops method
#[update(guard = "auth_update", hidden = true)]
pub async fn mint_token_with_req(n_req: MintTokenRequest) -> Result<TxStatus, CallError> {
    let mut req = match read_state(|s| s.mint_token_requests.get(&n_req.ticket_id)) {
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
            } else {
                n_req
            }
        }
    };

    log!(
        DEBUG,
        "[service::mint_token] mint token request: {:?} ",
        req
    );

    match &req.status {
        TxStatus::New | TxStatus::TxFailed { .. } => {
            match req.signature.to_owned() {
                None => {
                    // new mint req
                    let sig = crate::solana_client::mint_to_with_req(req.to_owned()).await?;

                    // update signature
                    req.signature = Some(sig.to_string());
                    mutate_state(|s| {
                        s.mint_token_requests
                            .insert(req.ticket_id.to_owned(), req.to_owned())
                    });

                    // update req status
                    mint_token::update_mint_token_status(req.to_owned(), sig.to_owned()).await
                }
                Some(sig) => {
                    mint_token::update_mint_token_status(req.to_owned(), sig.to_owned()).await
                }
            }
        }
        TxStatus::Pending => {
            match req.signature.to_owned() {
                None => {
                    log!(DEBUG, "[service::mint_token] the mint token request ({:?}) is handling,pls wait ...", req);
                }
                Some(sig) => {
                    mint_token::update_mint_token_status(req.to_owned(), sig.to_owned()).await
                }
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
#[update(guard = "is_controller", hidden = true)]
pub async fn retry_mint_token(ticket_id: String) -> Result<String, CallError> {
    log!(
        DEBUG,
        "[service::retry_mint_token] retry mint token ticket_id: {:?} ",
        ticket_id
    );

    let mint_req = if let Some(mint_req) = read_state(|s| s.mint_token_requests.get(&ticket_id)) {
        mint_req
    } else {
        return Err(CallError {
            method: "retry_mint_token".to_string(),
            reason: Reason::CanisterError("not found ticket id ".to_string()),
        });
    };

    // retry mint token
    let ret = crate::solana_client::mint_to_with_req(mint_req.to_owned()).await;

    match &ret {
        Ok(sig) => {
            let new_req = MintTokenRequest {
                ticket_id: mint_req.ticket_id.to_owned(),
                associated_account: mint_req.associated_account,
                amount: mint_req.amount,
                token_mint: mint_req.token_mint,
                status: TxStatus::Pending,
                signature: Some(sig.to_string()),
                retry_4_building: mint_req.retry_4_building + 1,
                retry_4_status: 0,
            };

            // update status and signature
            mutate_state(|s| {
                s.mint_token_requests
                    .insert(mint_req.ticket_id.to_owned(), new_req)
            });
        }
        Err(e) => {
            let tx_error = match &e.reason {
                Reason::QueueIsFull
                | Reason::OutOfCycles
                | Reason::CanisterError(_)
                | Reason::Rejected(_) => todo!(),
                Reason::TxError(tx_error) => tx_error,
            };
            let new_req = MintTokenRequest {
                ticket_id: mint_req.ticket_id.to_owned(),
                associated_account: mint_req.associated_account,
                amount: mint_req.amount,
                token_mint: mint_req.token_mint,
                status: TxStatus::TxFailed {
                    e: tx_error.to_owned(),
                },
                signature: None,
                retry_4_building: mint_req.retry_4_building + 1,
                retry_4_status: 0,
            };
            // update status and error
            mutate_state(|s| {
                s.mint_token_requests
                    .insert(mint_req.ticket_id.to_owned(), new_req)
            });
        }
    }
    ret
}

#[update(guard = "auth_update", hidden = true)]
pub async fn update_mint_token_status(
    ticket_id: String,
    sig: String,
) -> Result<MintTokenRequest, CallError> {
    let mint_req = if let Some(mint_req) = read_state(|s| s.mint_token_requests.get(&ticket_id)) {
        mint_req
    } else {
        return Err(CallError {
            method: "update_mint_token_status".to_string(),
            reason: Reason::CanisterError("not found ticket id account".to_string()),
        });
    };
    log!(
        DEBUG,
        "[service::update_mint_token_status] mint token request: {:?} ",
        mint_req
    );
    mint_token::update_mint_token_status(mint_req, sig).await;
    let latest_req = read_state(|s| s.mint_token_requests.get(&ticket_id)).unwrap();

    Ok(latest_req)
}
// devops method
#[update(guard = "auth_update", hidden = true)]
pub async fn update_tx_hash_to_hub(sig: String, ticket_id: String) -> Result<(), CallError> {
    let hub_principal = read_state(|s| s.hub_principal);

    match update_tx_hash(hub_principal, ticket_id.to_string(), sig.to_owned()).await {
        Ok(()) => {
            log!(
                DEBUG,
                "[service::update_tx_hash_to_hub] successfully update tx hash ({})) to hub! ",
                sig
            );
            //only finalized mint_req, remove the handled ticket from queue
            // remove_ticket_from_quene(ticket_id.to_string()).await;
        }
        Err(err) => {
            log!(
                ERROR,
                "[service::update_tx_hash_to_hub] failed to update tx hash ({})) to hub : {}",
                sig,
                err
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
#[update(guard = "is_controller", hidden = true)]
pub async fn update_fee_account(fee_account: String) {
    mutate_state(|s| s.fee_account = fee_account)
}

// query fee account for the dst chain
#[query]
pub fn get_redeem_fee(chain_id: ChainId) -> Option<u128> {
    read_state(|s| s.get_fee(chain_id))
}

#[update(guard = "is_controller", hidden = true)]
pub async fn update_redeem_fee(fee: Factor) {
    mutate_state(|s| s.update_fee(fee))
}

// generate ticket ,called by front end or other sys
#[update]
async fn generate_ticket(args: GenerateTicketReq) -> Result<GenerateTicketOk, GenerateTicketError> {
    gen_ticket::generate_ticket(args).await
}

// devops method
#[query(hidden = true)]
pub fn gen_tickets_req(signature: String) -> Option<GenerateTicketReq> {
    read_state(|s| s.gen_ticket_reqs.get(&signature))
}

// devops method
#[update(guard = "is_controller", hidden = true)]
pub async fn remove_gen_tickets_req(signature: String) -> Option<GenerateTicketReq> {
    mutate_state(|state| state.gen_ticket_reqs.remove(&signature))
}

// devops method
#[query(hidden = true)]
pub fn get_failed_tickets_to_hub() -> Vec<Ticket> {
    read_state(|s| {
        s.tickets_failed_to_hub
            .iter()
            .map(|(_, ticket)| ticket)
            .collect()
    })
}

// devops method
#[query(hidden = true)]
pub fn get_failed_ticket_to_hub(ticket_id: String) -> Option<Ticket> {
    read_state(|s| s.tickets_failed_to_hub.get(&ticket_id))
}

// devops method
// when gen ticket and send it to hub failed ,call this method
#[update(guard = "auth_update", hidden = true)]
pub async fn send_failed_ticket_to_hub(ticket: TicketWithMemo) -> Result<(), GenerateTicketError> {
    let ticket: Ticket = ticket.into();
    let hub_principal = read_state(|s| (s.hub_principal));
    match send_ticket(hub_principal, ticket.to_owned()).await {
        Ok(()) => {
            mutate_state(|state| state.tickets_failed_to_hub.remove(&ticket.ticket_id));
            return Ok(());
        }
        Err(err) => {
            return Err(GenerateTicketError::SendTicketErr(format!("{}", err)));
        }
    }
}

// devops method
#[update(guard = "auth_update", hidden = true)]
pub async fn remove_failed_tickets_to_hub(ticket_id: String) -> Option<Ticket> {
    mutate_state(|state| state.tickets_failed_to_hub.remove(&ticket_id))
}

// devops method
#[query(guard = "is_controller", hidden = true)]
pub async fn seqs() -> Seqs {
    read_state(|s| s.seqs.to_owned())
}

// devops method
#[update(guard = "is_controller", hidden = true)]
pub async fn update_seqs(seqs: Seqs) {
    mutate_state(|s| {
        s.seqs = seqs;
    })
}

// devops method
#[query(guard = "is_controller", hidden = true)]
pub async fn sol_canister() -> Principal {
    read_state(|s| s.sol_canister)
}

// devops method
#[update(guard = "is_controller", hidden = true)]
pub async fn update_sol_canister(sol_canister: Principal) {
    mutate_state(|s| {
        s.sol_canister = sol_canister;
    })
}

// devops method
#[update(guard = "is_controller", hidden = true)]
pub fn debug(enable: bool) {
    mutate_state(|s| s.enable_debug = enable);
}

#[query(hidden = true)]
fn http_request(req: HttpRequest) -> HttpResponse {
    if ic_cdk::api::data_certificate().is_none() {
        ic_cdk::trap("update call rejected");
    }

    match req.path() {
        "/logs" => {
            let endable_debug = read_state(|s| s.enable_debug);
            http_log(req, endable_debug)
        }
        "/token_uri" => match req.raw_query_param("id") {
            None => HttpResponseBuilder::bad_request()
                .with_body_and_content_length("pls provide token id")
                .build(),
            Some(id) => {
                use urlencoding::decode;
                let id: String = decode(id).unwrap().into_owned();
                let token = read_state(|s| s.tokens.get(&id).to_owned());

                match token {
                    None => HttpResponseBuilder::bad_request()
                        .with_body_and_content_length(format!("not found the {} token uri ", id))
                        .build(),
                    Some(t) => {
                        let token_uri: TokenUri = t.into();
                        HttpResponseBuilder::ok()
                            .header("Content-Type", "application/json; charset=utf-8")
                            .with_body_and_content_length(
                                serde_json::to_string(&token_uri).unwrap_or_default(),
                            )
                            .build()
                    }
                }
            }
        },
        "/token_meta" => match req.raw_query_param("id") {
            None => HttpResponseBuilder::bad_request()
                .with_body_and_content_length("pls provide token id")
                .build(),
            Some(id) => {
                use urlencoding::decode;
                let id: String = decode(id).unwrap().into_owned();
                let token = read_state(|s| s.tokens.get(&id).to_owned());

                match token {
                    None => HttpResponseBuilder::bad_request()
                        .with_body_and_content_length(format!("not found the {} token meta", id))
                        .build(),
                    Some(t) => {
                        let token_meta: TokenMeta = t.into();
                        HttpResponseBuilder::ok()
                            .header("Content-Type", "application/json; charset=utf-8")
                            .with_body_and_content_length(
                                serde_json::to_string(&token_meta).unwrap_or_default(),
                            )
                            .build()
                    }
                }
            }
        },

        _ => HttpResponseBuilder::not_found().build(),
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
        println!("Decoded: {}", decoded); // Bitcoin-runes-HOPE•YOU•GET•NICE202410141209
        let decoded_string: String = decoded.into_owned();
        println!("decoded_string: {}", decoded_string);
        let encoded = "Bitcoin-runes-HOPE•YOU•GET•NICE202410141209";
        let decoded = urlencoding::decode(encoded).unwrap();
        println!("Decoded: {}", decoded); // Bitcoin-runes-HOPE•YOU•GET•NICE202410141209
        let decoded_string: String = decoded.into_owned();
        println!("decoded_string: {}", decoded_string);
    }
}
