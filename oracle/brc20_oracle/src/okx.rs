use crate::state::{api_key, proxy_url};
use candid::{CandidType, Deserialize};
use ic_canister_log::log;
use ic_cdk::api::management_canister::http_request::{
    http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod, TransformContext,
    TransformFunc,
};
use omnity_types::brc20::{Brc20TransferEvent, QueryBrc20TransferArgs};
use omnity_types::ic_log::{ERROR, INFO};
use serde::Serialize;
use std::str::FromStr;

pub const BASE_URL: &str = "https://www.oklink.com";
pub const RPC_NAME: &str = "OKX";

pub async fn okx_query_transfer_event(
    query_transfer_args: &QueryBrc20TransferArgs,
) -> Option<omnity_types::brc20::Brc20TransferEvent> {
    let r = query(query_transfer_args).await;
    match r {
        Ok(c) => {
            if c.is_ok() {
                if c.data.len() != 1 {
                    return None;
                }
                let data = c.data.first().cloned().unwrap();
                if data.total_page != "1" || data.inscriptions_list.len() != 1 {
                    return None;
                }
                let resp = data.inscriptions_list.first().cloned().unwrap();

                if resp.check(query_transfer_args) {
                    return Some(resp.into());
                }
                None
            } else {
                log!(
                    ERROR,
                    "unisat query result error: {:?}",
                    serde_json::to_string(&c)
                );
                None
            }
        }
        Err(e) => {
            log!(ERROR, "unisat query event rpc error: {:?}", e);
            None
        }
    }
}

async fn query(
    query_transfer_args: &QueryBrc20TransferArgs,
) -> Result<CommonResponse<Vec<PageInfo<OkxBrc20TransferEvent>>>, OkxError> {
    let real_rpc_url = BASE_URL.to_string();
    let api_key = api_key(RPC_NAME);
    log!(INFO, "okx api key: {}", api_key.clone());
    let proxy_url = proxy_url();
    let uri = format!(
        "/api/v5/explorer/btc/transaction-list?txId={}",
        query_transfer_args.tx_id
    );
    let url = format!("{proxy_url}{}", uri.clone());
    const MAX_CYCLES: u128 = 200_000_000;
    let idempotency_key = format!("okx-{}", ic_cdk::api::time());
    let request = CanisterHttpRequestArgument {
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
        headers: vec![
            HttpHeader {
                name: "Content-Type".to_string(),
                value: "application/json".to_string(),
            },
            HttpHeader {
                name: "Ok-Access-Key".to_string(),
                value: api_key,
            },
            HttpHeader {
                name: crate::constant_args::FORWARD_SOLANA_RPC.to_string(),
                value: real_rpc_url,
            },
            HttpHeader {
                name: crate::constant_args::IDEMPOTENCY_KEY.to_string(),
                value: idempotency_key,
            },
        ],
    };

    match http_request(request, MAX_CYCLES).await {
        Ok((response,)) => {
            log!(
                INFO,
                "okx result: {}",
                serde_json::to_string(&response).unwrap()
            );
            let status = response.status;
            if status == 200_u32 {
                let body = String::from_utf8(response.body).map_err(|_| {
                    OkxError::Rpc("Transformed response is not UTF-8 encoded".to_string())
                })?;
                let tx: CommonResponse<Vec<PageInfo<OkxBrc20TransferEvent>>> =
                    serde_json::from_str(&body).map_err(|_| {
                        OkxError::Rpc("failed to decode transaction from json".to_string())
                    })?;
                Ok(tx)
            } else {
                Err(OkxError::Rpc("http response not 200".to_string()))
            }
        }
        Err((_, m)) => Err(OkxError::Rpc(m)),
    }
}

impl OkxBrc20TransferEvent {
    pub fn check(&self, query_transfer_args: &QueryBrc20TransferArgs) -> bool {
        self.tx_id == query_transfer_args.tx_id
            && self.state == "success"
            && self.to_address == query_transfer_args.to_addr
            && self.token.to_lowercase() == query_transfer_args.ticker.to_lowercase()
            && self.amount == query_transfer_args.amt
    }
}

impl From<OkxBrc20TransferEvent> for Brc20TransferEvent {
    fn from(value: OkxBrc20TransferEvent) -> Self {
        Brc20TransferEvent {
            amout: value.amount,
            from: value.from_address,
            to: value.to_address,
            valid: true,
            height: u64::from_str(value.block_height.as_str()).unwrap_or_default(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Default, Serialize, Deserialize)]
struct CommonResponse<T> {
    pub code: String,
    pub msg: String,
    pub data: T,
}

impl<T> CommonResponse<T> {
    pub fn is_ok(&self) -> bool {
        self.code == "0" && self.msg.is_empty()
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Default, Serialize, Deserialize)]
struct PageInfo<T> {
    pub page: String,
    pub limit: String,
    #[serde(rename = "totalPage")]
    pub total_page: String,
    #[serde(rename = "totalTransaction")]
    pub total_transaction: String,
    #[serde(rename = "inscriptionsList")]
    pub inscriptions_list: Vec<T>,
}

#[derive(Clone, Debug, Eq, PartialEq, Default, Serialize, Deserialize)]
struct OkxBrc20TransferEvent {
    #[serde(rename = "txId")]
    pub tx_id: String,
    #[serde(rename = "blockHeight")]
    pub block_height: String,
    pub state: String,
    #[serde(rename = "tokenType")]
    pub token_type: String,
    #[serde(rename = "actionType")]
    pub action_type: String,
    #[serde(rename = "fromAddress")]
    pub from_address: String,
    #[serde(rename = "toAddress")]
    pub to_address: String,
    pub amount: String,
    pub token: String,
    #[serde(rename = "inscriptionId")]
    pub inscription_id: String,
    #[serde(rename = "inscriptionNumber")]
    pub inscription_number: String,
    pub index: String,
    pub location: String,
    pub msg: String,
    pub time: String,
}

#[derive(CandidType, Clone, Debug, Deserialize, PartialEq, Eq)]
enum OkxError {
    Rpc(String),
}
