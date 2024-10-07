use candid::{CandidType, Deserialize, Nat};
use ic_canister_log::log;
use ic_cdk::api::management_canister::http_request::{CanisterHttpRequestArgument, http_request, HttpHeader, HttpMethod, TransformContext, TransformFunc};
use serde::Serialize;
use omnity_types::brc20::{Brc20TransferEvent, QueryBrc20TransferArgs};
use omnity_types::ic_log::ERROR;
use crate::state::{api_key, proxy_url};

pub const BASE_URL: &str = "https://www.oklink.com";
pub const RPC_NAME: &str = "OKX";


pub async fn okx_query_transfer_event(query_transfer_args: QueryBrc20TransferArgs) -> Option<omnity_types::brc20::Brc20TransferEvent> {
    let r = query(&query_transfer_args).await;
    match r {
        Ok(c) => {
            if c.is_ok() {
                if c.data.len() != 1 {
                    return None;
                }
                let data = c.data.first().cloned().unwrap();
                if data.total_page!= "1" || data.inscriptions_list.len() != 1 {
                    return None;
                }
                let resp = data.inscriptions_list.first().cloned().unwrap();

                if resp.check(&query_transfer_args) {
                    return Some(resp.into());
                }
                None
            }else {
                log!(ERROR, "unisat query result error: {:?}", serde_json::to_string(&c));
                None
            }
        }
        Err(e) => {
            log!(ERROR, "unisat query event rpc error: {:?}", e);
            None
        }
    }
}


async fn query(query_transfer_args: &QueryBrc20TransferArgs) -> Result<CommonResponse<Vec<PageInfo<OkxBrc20TransferEvent>>>, OkxError> {
    let real_rpc_url = BASE_URL.to_string();
    let api_key = api_key(RPC_NAME);
    let proxy_url = proxy_url();
    let uri = format!("/api/v5/explorer/btc/transaction-list?txId={}",query_transfer_args.tx_id);
    let url = format!("{proxy_url}{}",uri.clone());
    const MAX_CYCLES: u128 = 25_000_000_000;

    let request = CanisterHttpRequestArgument {
        url: url,
        method: HttpMethod::GET,
        body: None,
        max_response_bytes: None,
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
          value: uri,
        }],
    };

    match http_request(request, MAX_CYCLES).await {
        Ok((response,)) => {
            let status = response.status;
            if status == Nat::from(200_u32) {
                let body = String::from_utf8(response.body).map_err(|_| {
                    OkxError::Rpc(
                        "Transformed response is not UTF-8 encoded".to_string(),
                    )
                })?;
                let tx: CommonResponse<Vec<PageInfo<OkxBrc20TransferEvent>>> = serde_json::from_str(&body).map_err(|_| {
                    OkxError::Rpc(
                        "failed to decode transaction from json".to_string(),
                    )
                })?;
                Ok(tx)
            }else {
                Err(OkxError::Rpc("http response not 200".to_string()))
            }
        }
        Err((_, m)) => Err(OkxError::Rpc(m)),
    }
}

impl OkxBrc20TransferEvent {
    pub fn check(&self, query_transfer_args: &QueryBrc20TransferArgs) -> bool {
        self.tx_id == query_transfer_args.tx_id &&
            self.state == "success" &&
            self.to_address == query_transfer_args.to_addr &&
            self.token_type == query_transfer_args.ticker &&
            self.amount == query_transfer_args.amt.to_string()
    }
}

impl Into<Brc20TransferEvent> for OkxBrc20TransferEvent {
    fn into(self) -> Brc20TransferEvent {
        Brc20TransferEvent {
            amout: self.amount,
            from: self.from_address,
            to: self.to_address,
            valid: true,
        }
    }
}


#[derive(Clone, Debug, Eq, PartialEq, Default, Serialize, Deserialize)]
struct CommonResponse<T> {
    pub code: String,
    pub msg: String,
    pub data: T
}


impl<T> CommonResponse<T> {
    pub fn is_ok(&self) -> bool {
        return self.code == "0" && self.msg == "";
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
    pub inscription_id:String,
    #[serde(rename = "inscriptionNumber")]
    pub inscription_number: String,
    pub index: String,
    pub location: String,
    pub msg: String,
    pub time: String,
}



#[derive(CandidType, Clone, Debug, Deserialize, PartialEq, Eq)]
enum OkxError {
    Rpc(String)
}