use crate::custom_to_dogecoin::SendTicketResult;
use crate::doge::transaction::Txid;
use crate::dogeoin_to_custom::query_and_save_utxo_for_payment_address;
use crate::errors::CustomsError;
use crate::generate_ticket::{GenerateTicketArgs, GenerateTicketWithTxidArgs};
use crate::state::{mutate_state, read_state, replace_state, DogeState, StateProfile};
use crate::tasks::start_tasks;
use crate::types::{
    Destination, LockTicketRequest, MultiRpcConfig, ReleaseTokenStatus, RpcConfig, TokenResp
};
use candid::{CandidType, Deserialize, Principal};
use ic_canister_log::log;
use ic_canisters_http_types::{HttpRequest, HttpResponse};
use ic_cdk::api::management_canister::http_request;
use ic_cdk::api::management_canister::http_request::TransformArgs;
use ic_cdk_macros::{init, post_upgrade, pre_upgrade, query, update};
use omnity_types::ic_log::{ERROR, INFO};
use omnity_types::{ChainId, Seq};
use std::str::FromStr;

#[init]
fn init(args: InitArgs) {
    replace_state(DogeState::init(args).expect("params error"));
    start_tasks();
}

#[pre_upgrade]
fn pre_upgrade() {
    read_state(|s| s.pre_upgrade());
}

#[post_upgrade]
fn post_upgrade() {
    DogeState::post_upgrade();
    start_tasks();
}

#[query]
pub fn get_finalized_lock_ticket_txids() -> Vec<String> {
    read_state(|s| {
        s.finalized_lock_ticket_requests_map
            .iter()
            .map(|e| e.1.txid.to_string())
            .collect()
    })
}

#[query]
pub fn get_finalized_unlock_ticket_results() -> Vec<SendTicketResult> {
    read_state(|s| {
        s.finalized_unlock_ticket_results_map
            .iter()
            .map(|e| e.1.clone())
            .collect()
    })
}

#[query(hidden = true)]
fn http_request(req: HttpRequest) -> HttpResponse {
    if ic_cdk::api::data_certificate().is_none() {
        ic_cdk::trap("update call rejected");
    }
    omnity_types::ic_log::http_request(req)
}

#[update]
pub async fn generate_ticket_by_txid(req: GenerateTicketWithTxidArgs)-> Result<(), CustomsError> {
    match crate::generate_ticket::generate_ticket(req.clone()).await {
        Ok(_) => {
            log!(INFO, "success to generate_ticket_by_txid, req: {:?}", req);
            Ok(())
        },
        Err(e) => {
            log!(ERROR, "failed to generate_ticket_by_txid error: {:?}", e);
            Err(CustomsError::from(e))
        }
    }
}

#[update]
pub async fn generate_ticket(req: GenerateTicketArgs) -> Result<Vec<String>, CustomsError> {
    let txids = crate::generate_ticket::get_ungenerated_txids(req.clone()).await?;
    log!(INFO, "find txids for generate_ticket: {:?}", txids);
    let mut success_txids = vec![];
    for txid in txids {
        let args = GenerateTicketWithTxidArgs {
            txid: txid.to_string(),
            target_chain_id: req.target_chain_id.clone(),
            token_id: req.token_id.clone(),
            receiver: req.receiver.clone(),
        };
        match crate::generate_ticket::generate_ticket(args).await {
            Ok(_) => {
                log!(INFO, "success to generate_ticket, txid: {:?}", txid);
                success_txids.push(txid.to_string());
            },
            Err(e) => {
                log!(ERROR, "generate_ticket error: {:?}", e);
            },
        }

        }

    Ok(success_txids)
}

#[query]
fn get_platform_fee(target_chain: ChainId) -> (Option<u128>, Option<String>) {
    read_state(|s| s.get_transfer_fee_info(&target_chain))
}

#[query]
pub fn get_deposit_address(
    target_chain_id: String,
    receiver: String,
) -> Result<String, CustomsError> {
    let dest = Destination::new(target_chain_id, receiver, None);
    read_state(|s| s.get_address(dest)).map(|a| a.0.to_string())
}

#[query(guard = "is_admin")]
pub fn query_state() -> StateProfile {
    read_state(|s| StateProfile::from(s))
}

#[update(guard = "is_admin")]
pub fn set_fee_collector(addr: String) {
    mutate_state(|s| s.fee_collector = addr);
}

#[query]
pub fn get_fee_payment_address() -> Result<String, CustomsError> {
    mutate_state(|s| s.get_address(Destination::fee_payment_address())).map(|a| a.0.to_string())
}

#[update(guard = "is_admin")]
pub async fn save_utxo_for_payment_address(txid: String) -> Result<u64, CustomsError> {
    query_and_save_utxo_for_payment_address(txid).await
}

#[update(guard = "is_admin")]
pub fn set_min_deposit_amount(amount: u64) {
    mutate_state(|s| s.min_deposit_amount = amount);
}

#[query]
fn release_token_status(ticket_id: String) -> ReleaseTokenStatus {
    read_state(|s| s.unlock_tx_status(&ticket_id))
}

#[query(guard = "is_admin")]
pub fn pending_unlock_tickets(seq: Seq) -> String {
    let r = read_state(|s| s.flight_unlock_ticket_map.get(&seq).cloned().unwrap());
    serde_json::to_string(&r).unwrap()
}

#[update(guard = "is_admin")]
pub async fn init_ecdsa_public_key() -> Result<(), CustomsError> {
    crate::state::init_ecdsa_public_key().await.map(|_| ())
}

#[update(guard = "is_admin")]
pub async fn set_tatum_api_config(url: String, api_key: Option<String>) {
    mutate_state(|s| {
        s.tatum_api_config = RpcConfig { url, api_key };
    });
}

#[update(guard = "is_admin")]
pub async fn set_default_doge_rpc_config(url: String, api_key: Option<String>) {
    mutate_state(|s| {
        s.default_doge_rpc_config = RpcConfig { url, api_key };
    });
}

#[update(guard = "is_admin")]
pub async fn set_multi_rpc_config(
    multi_rpc_config: MultiRpcConfig
) {
    mutate_state(|s| {
        s.multi_rpc_config = multi_rpc_config;
    });
}

#[query(hidden = true)]
fn transform(raw: TransformArgs) -> http_request::HttpResponse {
    http_request::HttpResponse {
        status: raw.response.status.clone(),
        body: raw.response.body.clone(),
        headers: vec![],
    }
}

#[update(guard = "is_admin")]
pub async fn resend_unlock_ticket(seq: Seq, fee_rate: Option<u64>) -> Result<String, String> {
    match crate::custom_to_dogecoin::submit_unlock_ticket(seq, fee_rate).await {
        Ok(r) => {
            log!(
                INFO,
                "success to resend_unlock_ticket, seq: {:?}, txid: {:?}",
                seq,
                r.txid.to_string()
            );
            mutate_state(|s| s.flight_unlock_ticket_map.insert(seq, r.clone()));
            Ok(serde_json::to_string(&r).unwrap())
        }
        Err(e) => {
            log!(ERROR, "resend_unlock_ticket error: {:?}", e);
            return Err("resend_unlock_ticket error".to_string());
        }
    }
}

// #[update]
// pub async fn test_rpc(rpc_config: RpcConfig, txid: String) -> Result<String, CustomsError> {
//     let doge_rpc = crate::doge::rpc::DogeRpc::from(rpc_config);
//     doge_rpc.get_raw_transaction(txid.as_str()).await.map(|r| format!("{:?}", r))
// }

#[query]
fn get_token_list() -> Vec<TokenResp> {
    read_state(|s| s.tokens.values().map(|t| t.clone().into()).collect())
}

#[query(guard = "is_admin")]
fn query_finalized_lock_tickets(txid: String) -> Option<LockTicketRequest> {
    let txid = Txid::from_str(txid.as_str()).unwrap();
    read_state(|s| s.finalized_lock_ticket_requests_map.get(&txid.into()))
}

#[derive(CandidType, Deserialize)]
pub struct InitArgs {
    pub admins: Vec<Principal>,
    pub hub_principal: Principal,
    // pub network: Network,
    pub chain_id: String,
    // pub indexer_principal: Principal,
    pub fee_token: String,
    pub default_doge_rpc_config: RpcConfig,
}

fn is_admin() -> Result<(), String> {
    let c = ic_cdk::caller();
    match ic_cdk::api::is_controller(&c) || read_state(|s| s.admins.contains(&c)) {
        true => Ok(()),
        false => Err("permission deny".to_string()),
    }
}

ic_cdk::export_candid!();
