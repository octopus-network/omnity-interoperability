use crate::custom_to_dogecoin::SendTicketResult;
use crate::doge::transaction::Txid;
use crate::dogeoin_to_custom::query_and_save_utxo_for_payment_address;
use crate::errors::CustomsError;
use crate::generate_ticket::{GenerateTicketArgs, GenerateTicketWithTxidArgs};
use crate::state::{mutate_state, read_state, replace_state, DogeState, StateProfile};
use crate::tasks::start_tasks;
use crate::types::{
    Destination, LockTicketRequest, ReleaseTokenStatus, RpcConfig, TokenResp,
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

#[update(guard = "is_admin")]
pub fn tmp_fix() {
    let v = vec![
        (
            0,
            SendTicketResult {
                txid: crate::types::Txid::from_str("a2892cb2095e0446416a47c55ec7878aadf111875f6ec6b0baffc9c4dd618e21").unwrap(),
                success: true,
                time_at: 1_735_704_485_582_704_364,
            },
        ),
        (
            1,
            SendTicketResult {
                txid: crate::types::Txid::from_str("d741f87bc99cae542997377069db0b8d25a2205aaf6ff344a682d606923d4117").unwrap(),
                success: true,
                time_at: 1_736_152_340_017_781_767,
            },
        ),
        (
            2,
            SendTicketResult {
                txid: crate::types::Txid::from_str("62b68ffd3532d32225b9c3ce96a46af31ab9103918a3f3f80e5b86fa5fe62b85").unwrap(),
                success: true,
                time_at: 1_736_152_528_438_327_721,
            },
        ),
        (
            3,
            SendTicketResult {
                txid: crate::types::Txid::from_str("02d4d2f94d2ded95bcba86ad51c5529225c6d96e8f62c77f7d8b25683ed14a4a").unwrap(),
                success: true,
                time_at: 1_736_158_610_935_813_349,
            },
        ),
        (
            4,
            SendTicketResult {
                txid: crate::types::Txid::from_str("aebf2b3bb6e5c5773ce66af5645dba555bf40c35b73daa2edbf0ef4e5d2ab843").unwrap(),
                success: true,
                time_at: 1_736_216_381_913_371_532,
            },
        ),
        (
            5,
            SendTicketResult {
                txid: crate::types::Txid::from_str("ec75e54b76befd0e4d6418704c2827e0bef3ed37a2d1daad7af6597f02c0332a").unwrap(),
                success: true,
                time_at: 1_736_246_946_415_382_518,
            },
        ),
        (
            6,
            SendTicketResult {
                txid: crate::types::Txid::from_str("5fc3a7a2fc2f9fca6b53b191765d8219a63d9a561742ec508e233ee350cccabf").unwrap(),
                success: true,
                time_at: 1_736_251_053_091_159_392,
            },
        ),
    ];
    mutate_state(
        |s| {
            for e in v {
                s.finalized_unlock_ticket_results_map.insert(e.0, e.1);
            }
        }
    );
}

// #[update(guard = "is_admin")]
// pub fn save_tx_in_memory() {
//     let request: Vec<LockTicketRequest> = mutate_state(
//         |s| s.finalized_lock_ticket_requests.iter().map(|(txid, req)| {
//             req.clone()
//         }).collect()
//     );

//     let results: Vec<SendTicketResult> = mutate_state(
//         |s| s.finalized_unlock_ticket_results_map.iter().map(|(txid, req)| {
//             req.clone()
//         }).collect()
//     );

//     for req in request {
//         mutate_state(|s| {
//             s.finalized_lock_ticket_requests_map.insert(req.txid.clone(), req);
//         });
//     }

//     for res in results {
//         mutate_state(|s| {
//             s.finalized_unlock_ticket_results_map.insert(res.txid.clone(), res);
//         });
//     }
// }

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

// #[update(guard = "is_admin")]
// fn restore_utxo() {
//     let change_destination = Destination::change_address();
//     // let change_address = read_state(|s| s.get_address(Destination::fee_payment_address()).0);

//     // let fee_payment_destination = Destination::fee_payment_address();
//     // let fee_payment_address = read_state(|s| s.get_address(Destination::fee_payment_address()).0);

//     let mut change_utxo = vec![
//         (Utxo {
//             txid: crate::types::Txid::from_str("02d4d2f94d2ded95bcba86ad51c5529225c6d96e8f62c77f7d8b25683ed14a4a").unwrap(),
//             vout: 1,
//             value: 4 * DOGE_AMOUNT,
//         }, change_destination.clone()),
//         (Utxo {
//             txid: crate::types::Txid::from_str("472eca0781c37646802481733535ab35b0d30755aad1f849877104b69807172d").unwrap(),
//             vout: 1,
//             value: DOGE_AMOUNT / 10,
//         }, change_destination.clone()),
//         (Utxo {
//             txid: crate::types::Txid::from_str("5c7931d648bc3700bda7e24b6d39b59654f7778b6faf1596c71a563f423cd2d3").unwrap(),
//             vout: 1,
//             value: DOGE_AMOUNT,
//         }, change_destination.clone()),
//     ];

//     let osmosis_destination = Destination::new("osmosis-1".to_string(), "osmo1uqwp92j0a2xdntfxfjrs4a8gmpvh5elre07l3s".to_string(), None);
//     let mut osmosis_deposit = vec![
//         (Utxo {
//             txid: crate::types::Txid::from_str("14013504a0f52b898a434bb08992e14cd3a864dfb61acff758b09590c83a1a3d").unwrap(),
//             vout: 0,
//             value: DOGE_AMOUNT,
//         }, osmosis_destination.clone()),
//         (Utxo {
//             txid: crate::types::Txid::from_str("6794f9c15a9c1173fd615f7bd2aa28a95014eca71c43d3e0bcabdaaea46beee8").unwrap(),
//             vout: 0,
//             value: 2 * DOGE_AMOUNT,
//         }, osmosis_destination.clone()),
//     ];

//     let mut payment_utxo = vec![
//         Utxo {
//             txid: crate::types::Txid::from_str("f4d1f706e829d029045a2b3a41b1d31c0dc57e0b01376b2ab2207156a7e37380").unwrap(),
//             vout: 0,
//             value: 10 * DOGE_AMOUNT }
//     ];

//     mutate_state(|s| {
//         s.deposited_utxo.append(&mut change_utxo);
//         s.deposited_utxo.append(&mut osmosis_deposit);
//         s.fee_payment_utxo.append(&mut payment_utxo);
//     });
// }

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
}

fn is_admin() -> Result<(), String> {
    let c = ic_cdk::caller();
    match ic_cdk::api::is_controller(&c) || read_state(|s| s.admins.contains(&c)) {
        true => Ok(()),
        false => Err("permission deny".to_string()),
    }
}

ic_cdk::export_candid!();
