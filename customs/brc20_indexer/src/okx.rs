use candid::{CandidType, Deserialize, Nat};
use ic_canister_log::log;
use ic_cdk::api::management_canister::http_request::{CanisterHttpRequestArgument, http_request, HttpHeader, HttpMethod, TransformContext, TransformFunc};
use log::__private_api::log;
use serde::Serialize;
use omnity_types::brc20::{Brc20TransferEvent, QueryBrc20TransferArgs};
use omnity_types::ic_log::{ERROR, INFO};
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
    log!(INFO, "okx api key: {}", api_key.clone());
    let proxy_url = proxy_url();
    let uri = format!("/api/v5/explorer/btc/transaction-list?txId={}",query_transfer_args.tx_id);
    let url = format!("{proxy_url}{}",uri.clone());
    const MAX_CYCLES: u128 = 25_000_000_000;
    let idempotency_key = format!("okx-{}",ic_cdk::api::time());
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
          value: idempotency_key,
        }],
    };

    match http_request(request, MAX_CYCLES).await {
        Ok((response,)) => {
            log!(INFO, "okx result: {}",serde_json::to_string(&response).unwrap());
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
            self.token.to_lowercase() == query_transfer_args.ticker.to_lowercase() &&
            self.amount == query_transfer_args.amt
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

#[test]
pub fn test() {
    let v: &[u8] = [123,34,99,111,100,101,34,58,34,48,34,44,34,109,115,103,34,58,34,34,44,34,100,97,116,97,34,58,91,123,34,112,97,103,101,34,58,34,49,34,44,34,108,105,109,105,116,34,58,34,50,48,34,44,34,116,111,116,97,108,80,97,103,101,34,58,34,49,34,44,34,116,111,116,97,108,84,114,97,110,115,97,99,116,105,111,110,34,58,34,49,34,44,34,105,110,115,99,114,105,112,116,105,111,110,115,76,105,115,116,34,58,91,123,34,116,120,73,100,34,58,34,53,50,48,49,100,55,101,56,56,53,49,99,101,102,52,97,54,54,98,99,51,53,53,52,57,99,56,52,52,51,49,99,99,102,52,56,53,52,97,101,102,51,54,53,57,53,99,49,54,51,54,55,97,100,55,55,49,98,100,51,99,56,98,51,34,44,34,98,108,111,99,107,72,101,105,103,104,116,34,58,34,56,54,50,54,54,56,34,44,34,115,116,97,116,101,34,58,34,115,117,99,99,101,115,115,34,44,34,116,111,107,101,110,84,121,112,101,34,58,34,66,82,67,50,48,34,44,34,97,99,116,105,111,110,84,121,112,101,34,58,34,116,114,97,110,115,102,101,114,34,44,34,102,114,111,109,65,100,100,114,101,115,115,34,58,34,98,99,49,113,116,116,120,57,100,56,101,107,117,53,101,50,119,99,100,109,122,54,118,51,114,122,120,110,121,120,121,108,107,115,122,113,53,56,121,113,117,48,34,44,34,116,111,65,100,100,114,101,115,115,34,58,34,98,99,49,113,121,101,108,103,107,120,112,102,104,102,106,114,103,54,104,103,56,104,108,114,57,116,52,100,122,110,55,110,56,56,101,97,106,120,102,121,53,99,34,44,34,97,109,111,117,110,116,34,58,34,49,48,48,34,44,34,116,111,107,101,110,34,58,34,89,67,66,83,34,44,34,105,110,115,99,114,105,112,116,105,111,110,73,100,34,58,34,100,56,102,98,100,56,52,48,48,55,57,54,48,100,57,101,57,51,97,50,102,99,97,102,49,55,55,102,57,50,52,57,50,98,56,50,49,100,100,57,100,98,48,55,100,50,55,53,51,52,57,56,101,97,57,97,101,49,50,102,102,97,100,48,105,48,34,44,34,105,110,115,99,114,105,112,116,105,111,110,78,117,109,98,101,114,34,58,34,55,54,48,57,51,49,56,57,34,44,34,105,110,100,101,120,34,58,34,48,34,44,34,108,111,99,97,116,105,111,110,34,58,34,53,50,48,49,100,55,101,56,56,53,49,99,101,102,52,97,54,54,98,99,51,53,53,52,57,99,56,52,52,51,49,99,99,102,52,56,53,52,97,101,102,51,54,53,57,53,99,49,54,51,54,55,97,100,55,55,49,98,100,51,99,56,98,51,58,48,58,48,34,44,34,109,115,103,34,58,34,34,44,34,116,105,109,101,34,58,34,49,55,50,55,49,56,49,57,50,56,48,48,48,34,125,93,125,93,125].as_slice();
    let s = String::from_utf8(v.to_vec()).unwrap();
    println!("{s}");

}