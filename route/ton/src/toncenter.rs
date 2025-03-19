use crate::chainkey::minter_addr;
use crate::state::{bridge_fee, read_state};
use crate::GenerateTicketArgs;
use anyhow::anyhow;
use candid::CandidType;
use ic_canister_log::log;
use ic_cdk::api::management_canister::http_request::{
    http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod, TransformContext,
    TransformFunc,
};
use omnity_types::ic_log::INFO;
use omnity_types::TxAction::Redeem;
use omnity_types::{ChainId, Ticket, TicketType, TxAction, Memo};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tonlib_core::cell::BagOfCells;
use tonlib_core::TonAddress;
use uuid::Uuid;

const MAX_CYCLES: u128 = 3_100_000_000;
pub const TONCENTER_BASE_URL: &str = "toncenter.com";
pub const PROXY_URL: &str = "https://ton-idempotent-proxy-219952077564.us-central1.run.app";
pub const IDEMPOTENCY_KEY: &str = "idempotency-key";
pub const FORWARD_RPC: &str = "x-forwarded-host";

pub async fn send_boc(boc: String) -> anyhow::Result<String> {
    let url = format!("{TONCENTER_BASE_URL}/api/v3/message");
    log!(INFO, "boc = {}", boc);
    let json = json!({
        "boc": boc
    });
    let body = serde_json::to_string(&json).unwrap();
    let mut request = CanisterHttpRequestArgument {
        url,
        method: HttpMethod::POST,
        body: Some(body.as_bytes().to_vec()),
        max_response_bytes: Some(2000),
        transform: Some(TransformContext {
            function: TransformFunc(candid::Func {
                principal: ic_cdk::api::id(),
                method: "transform".to_string(),
            }),
            context: vec![],
        }),
        headers: vec![HttpHeader {
            name: "Content-Type".to_string(),
            value: "application/json".to_string(),
        }],
    };
    proxy_request(&mut request);
    let resp_body = do_http_request(request).await?;
    let resp: SendBocResponse = serde_json::from_str(&resp_body)
        .map_err(|_| anyhow!("failed to decode transaction from json".to_string()))?;
    if resp.message_hash.is_some() {
        Ok(resp.message_hash.unwrap())
    } else {
        let r = format!("{},{}", resp.error.unwrap_or_default(), resp_body);
        Err(anyhow!(r))
    }
}

pub async fn get_account_seqno(addr: &str) -> anyhow::Result<i32> {
    let addr = urlencoding::encode(addr).to_string();
    let url = format!("https://{TONCENTER_BASE_URL}/api/v3/walletInformation?address={addr}&use_v2=false");
    let mut request = CanisterHttpRequestArgument {
        url,
        method: HttpMethod::GET,
        body: None,
        max_response_bytes: Some(2000),
        transform: Some(TransformContext {
            function: TransformFunc(candid::Func {
                principal: ic_cdk::api::id(),
                method: "transform".to_string(),
            }),
            context: vec![],
        }),
        headers: vec![HttpHeader {
            name: "Content-Type".to_string(),
            value: "application/json".to_string(),
        }],
    };
    proxy_request(&mut request);
    let resp_body = do_http_request(request).await?;
    let resp: WalletInformation = serde_json::from_str(&resp_body)
        .map_err(|_| anyhow!("failed to decode transaction from json".to_string()))?;
    if resp.seqno.is_some() {
        Ok(resp.seqno.unwrap())
    } else {
        Err(anyhow!("response not contains seqno"))
    }
}

pub async fn query_burn_events(
    addr: &str,
    jetton_master: &str,
) -> anyhow::Result<QueryJettonBurnResponse> {
    log!(INFO, "query burn: {} {}", addr, jetton_master);
    let addr = urlencoding::encode(addr).to_string();
    let jetton_master = urlencoding::encode(jetton_master).to_string();
    let url = format!("https://{TONCENTER_BASE_URL}/api/v3/jetton/burns?address={addr}&jetton_master={jetton_master}&limit=5&offset=0&sort=desc");
    log!(INFO, "query burn url::{}", &url);
    let mut request = CanisterHttpRequestArgument {
        url,
        method: HttpMethod::GET,
        body: None,
        max_response_bytes: Some(10000),
        transform: Some(TransformContext {
            function: TransformFunc(candid::Func {
                principal: ic_cdk::api::id(),
                method: "transform".to_string(),
            }),
            context: vec![],
        }),
        headers: vec![HttpHeader {
            name: "Content-Type".to_string(),
            value: "application/json".to_string(),
        }],
    };
    proxy_request(&mut request);
    let resp_body = do_http_request(request).await?;
    serde_json::from_str(&resp_body)
        .map_err(|_| anyhow!("failed to decode transaction from json".to_string()))
}

pub async fn query_mint_message() -> anyhow::Result<QueryMessageResponse> {
    let source_addr = urlencoding::encode(&minter_addr()).to_string();
    let url = format!("{TONCENTER_BASE_URL}/api/v3/messages?source={source_addr}&opcode=15&limit=20&offset=0&sort=desc");
    let mut request = CanisterHttpRequestArgument {
        url,
        method: HttpMethod::GET,
        body: None,
        max_response_bytes: Some(20000),
        transform: Some(TransformContext {
            function: TransformFunc(candid::Func {
                principal: ic_cdk::api::id(),
                method: "transform".to_string(),
            }),
            context: vec![],
        }),
        headers: vec![HttpHeader {
            name: "Content-Type".to_string(),
            value: "application/json".to_string(),
        }],
    };
    proxy_request(&mut request);
    let resp_body = do_http_request(request).await?;
    serde_json::from_str(resp_body.as_str())
        .map_err(|_| anyhow!("failed to decode message resp from json".to_string()))
}

async fn do_http_request(request: CanisterHttpRequestArgument) -> anyhow::Result<String> {
    match http_request(request, MAX_CYCLES).await {
        Ok((response,)) => {
            let r = serde_json::to_string(&response).unwrap_or_default();
            let status = response.status;
            if status == 200_u32 {
                String::from_utf8(response.body).map_err(|_| {
                    anyhow!(format!(
                        "Transformed response is not UTF-8 encoded, resp {}",
                        r
                    ))
                })
            } else {
                Err(anyhow!(format!("http response not 200, resp {}", r)))
            }
        }
        Err((_, m)) => Err(anyhow!(m)),
    }
}

pub fn proxy_request(request: &mut CanisterHttpRequestArgument) {
    request.url = request.url.replace(TONCENTER_BASE_URL, PROXY_URL);
    let idempotency_key = format!("ton_route-{}{}", ic_cdk::api::time(), uuid::Uuid::new_v4().to_string());
    request.headers.push(HttpHeader {
        name: IDEMPOTENCY_KEY.to_string(),
        value: idempotency_key,
    });
    request.headers.push(HttpHeader {
        name: FORWARD_RPC.to_string(),
        value: TONCENTER_BASE_URL.to_string().replace("https://",""),
    });
}

pub async fn create_ticket_by_generate_ticket(
    params: &GenerateTicketArgs,
) -> anyhow::Result<Ticket> {
    let token_jetton_master =
        read_state(|s| s.token_jetton_master_map.get(&params.token_id).cloned())
            .ok_or(anyhow!("token not found".to_string()))?;
    let r = query_burn_events(&params.sender, &token_jetton_master)
        .await
        .map_err(|e| anyhow!(e.to_string()))?;
    log!(
        INFO,
        " burn events: {}",
        serde_json::to_string_pretty(&r).unwrap()
    );
    for jbe in r.jetton_burns {
        if jbe.trace_id != params.tx_hash {
            continue;
        }
        if jbe.transaction_aborted || jbe.custom_payload.is_none() {
            return Err(anyhow!("transaction is aborted or custom payload is null"));
        }
        let r = BagOfCells::parse_base64(jbe.custom_payload.unwrap().as_str())
            .map_err(|e| anyhow!(e.to_string()))?
            .roots
            .pop()
            .ok_or(anyhow!("parse custom paload error"))?
            .data()
            .to_vec();
        let s = String::from_utf8(r).map_err(|e| anyhow!(e.to_string()))?;
        let payload: BurnCustomPayload =
            serde_json::from_str(s.as_str()).map_err(|e| anyhow!(e.to_string()))?;
        log!(INFO, "burn event: {:?}", &payload);
        let amt = jbe
            .amount
            .parse::<u128>()
            .map_err(|e| anyhow!(e.to_string()))?;
        if amt == params.amount
            && payload.target_chain == params.target_chain_id
            && payload.receiver == params.receiver
        {
            let tx_action = if params.token_id.starts_with(&payload.target_chain) {
                Redeem
            } else {
                TxAction::Transfer
            };
            
            let fee = bridge_fee(&params.target_chain_id);
            let memo_json = Memo {
                memo: None,
                bridge_fee: fee.unwrap_or_default() as u128,
            }.convert_to_memo_json().unwrap_or_default();

            return Ok(Ticket {
                ticket_id: jbe.trace_id.clone(),
                ticket_type: TicketType::Normal,
                ticket_time: ic_cdk::api::time(),
                src_chain: crate::state::TON_CHAIN_ID.to_string(),
                dst_chain: payload.target_chain.clone(),
                action: tx_action,
                token: params.token_id.clone(),
                amount: params.amount.to_string(),
                sender: Some(params.sender.clone()),
                receiver: params.receiver.clone(),
                memo: Some(memo_json.as_bytes().to_vec()),
            });
        } else {
            return Err(anyhow!("params invalid".to_string()));
        }
    }
    Err(anyhow!("burn event not found".to_string()))
}

pub async fn check_bridge_fee(hsh: &String, chain_id: &ChainId) -> anyhow::Result<()> {
    match bridge_fee(chain_id) {
        None => Ok(()),
        Some(fee) => {
            log!(INFO, "query events params: {}", &hsh);
            let r = query_events(hsh).await?.events;
            log!(
                INFO,
                "query events result: {:?}",
                serde_json::to_string_pretty(&r)
            );
            let minter = minter_addr();
            for e in r {
                for actions in e.actions {
                    if actions.rtype != "ton_transfer" {
                        continue;
                    }
                    if !actions.success {
                        continue;
                    }
                    let details = actions.details;
                    let details = serde_json::from_value::<TonTransferDetail>(details).unwrap();
                    let r = TonAddress::from_hex_str(details.destination.as_str())
                        .unwrap()
                        .to_base64_std_flags(true, false);
                    let v: u64 = details.value.parse().unwrap();
                    if r == minter && v >= fee {
                        return Ok(());
                    }
                }
            }
            Err(anyhow!("no fee transfer"))
        }
    }
}

pub async fn query_events(tx_hash: &str) -> anyhow::Result<QueryEventResponse> {
    let tx_hash = urlencoding::encode(tx_hash);
    let url =
        format!("{TONCENTER_BASE_URL}/api/v3/events?tx_hash={tx_hash}&limit=5&offset=0&sort=desc");
    let mut request = CanisterHttpRequestArgument {
        url,
        method: HttpMethod::GET,
        body: None,
        max_response_bytes: Some(100000),
        transform: Some(TransformContext {
            function: TransformFunc(candid::Func {
                principal: ic_cdk::api::id(),
                method: "transform".to_string(),
            }),
            context: vec![],
        }),
        headers: vec![HttpHeader {
            name: "Content-Type".to_string(),
            value: "application/json".to_string(),
        }],
    };
    proxy_request(&mut request);
    let resp_body = do_http_request(request).await?;
    serde_json::from_str(&resp_body)
        .map_err(|_| anyhow!("failed to decode transaction from json".to_string()))
}

#[derive(Serialize, Deserialize)]
pub struct QueryEventResponse {
    pub events: Vec<TonEvent>,
}

#[derive(Serialize, Deserialize)]
pub struct TonEvent {
    actions: Vec<TonEventAction>,
}

#[derive(Serialize, Deserialize)]
struct Details {
    pub opcode: String,
    pub destination: String,
}

#[derive(Serialize, Deserialize)]
struct TonEventAction {
    pub trace_id: String,
    pub action_id: String,
    pub start_lt: String,
    pub end_lt: String,
    pub start_utime: i64,
    pub end_utime: i64,
    pub success: bool,
    #[serde(rename = "type")]
    pub rtype: String,
    pub details: serde_json::Value,
}

#[derive(Serialize, Deserialize, Default)]
struct TonTransferDetail {
    pub source: String,
    pub destination: String,
    pub value: String,
    pub encrypted: bool,
}

#[derive(Serialize, Default, Deserialize, Clone, CandidType)]
pub struct QueryJettonBurnResponse {
    pub(self) jetton_burns: Vec<JettonBurnEvent>,
}
#[derive(Serialize, Deserialize, Clone, Default, CandidType)]
struct JettonBurnEvent {
    pub amount: String,
    pub custom_payload: Option<String>,
    pub jetton_master: String,
    pub jetton_wallet: String,
    pub owner: String,
    pub query_id: String,
    pub response_destination: String,
    pub trace_id: String,
    pub transaction_aborted: bool,
    pub transaction_hash: String,
    pub transaction_lt: String,
    pub transaction_now: u64,
}

#[derive(Serialize, Deserialize, Default)]
struct SendBocResponse {
    pub code: Option<i64>,
    pub error: Option<String>,
    pub message_hash: Option<String>,
}

#[derive(Serialize, Deserialize, Default)]
struct WalletInformation {
    pub balance: String,
    pub wallet_type: String,
    pub seqno: Option<i32>,
    pub wallet_id: i64,
    pub last_transaction_lt: String,
    pub last_transaction_hash: String,
    pub status: String,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct BurnCustomPayload {
    pub target_chain: String,
    pub receiver: String,
}

#[derive(Serialize, Default, Debug, Deserialize, Clone, CandidType)]
pub struct MessageContent {
    pub hash: String,
    pub body: String,
}

#[derive(Serialize, Debug, Default, Deserialize, Clone, CandidType)]
pub struct Message {
    pub hash: String,
    pub source: String,
    pub destination: String,
    pub value: String,
    pub fwd_fee: String,
    pub ihr_fee: String,
    pub created_lt: String,
    pub created_at: String,
    pub opcode: String,
    pub ihr_disabled: bool,
    pub bounce: bool,
    pub bounced: bool,
    pub message_content: MessageContent,
}

#[derive(Serialize, Default, Deserialize, Debug, Clone, CandidType)]
pub struct QueryMessageResponse {
    pub messages: Vec<Message>,
}
