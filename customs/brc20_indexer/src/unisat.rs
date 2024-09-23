use bitcoin::Amount;
use candid::{CandidType, Nat};
use ic_btc_interface::Network;
use ic_canister_log::log;
use ic_cdk::api::management_canister::http_request::{CanisterHttpRequestArgument, http_request, HttpHeader, HttpMethod, TransformContext, TransformFunc};
use serde::{Deserialize, Serialize};
use omnity_types::ic_log::ERROR;
use crate::service::{Brc20TransferEvent, QueryBrc20TransferArgs};
use crate::state::{api_key, BitcoinNetwork, read_state};

const TESTNET_BASE_URL: &str = "https://open-api-testnet.unisat.io";
const MAINNET_BASE_URL: &str = "https://open-api.unisat.io";
#[derive(Clone, Debug, Eq, PartialEq, Default, Serialize, Deserialize)]
pub struct CommonResponse<T> {
    pub code: i32,
    pub msg: String,
    pub data: Option<T>
}

impl<T> CommonResponse<T> {
    pub fn is_ok(&self) -> bool {
        return self.code == 0 && self.msg == "ok" && self.data.is_some();
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Default, Serialize, Deserialize)]
pub struct QueryBrc20EventResponse {
    pub height: u32,
    pub total: u32,
    pub start: u32,
    pub detail: Vec<Brc20Event>
}

#[derive(Clone, Debug, Eq, PartialEq, Default, Serialize, Deserialize)]
pub struct Brc20Event {
    pub ticker: String,
    #[serde(rename = "type")]
    pub typec:  String,
    pub valid: bool,
    pub txid: String,
    pub idx: u32,
    pub vout: u32,
    pub offset: u32,
    #[serde(rename = "inscriptionNumber")]
    pub inscription_number: u64,
    #[serde(rename = "inscriptionId")]
    pub inscription_id: String,
    pub from: String,
    pub to: String,
    pub satoshi: u128,
    pub fee: u128,
    pub amount: String,
    #[serde(rename = "overallBalance")]
    pub overalll_balance: String,
    #[serde(rename = "transferBalance")]
    pub transfer_balance: String,
    #[serde(rename = "availableBalance")]
    pub available_balance: String,
    pub height: u32,
    pub txidx: u32,
    pub blockhash: String,
    pub blocktime: u64,
}

impl Brc20Event {
    pub fn check(&self, query_transfer_args: &QueryBrc20TransferArgs) -> bool {
        self.txid == query_transfer_args.tx_id &&
            self.valid == true &&
            self.to == query_transfer_args.to_addr &&
            self.ticker == query_transfer_args.ticker &&
            self.amount == query_transfer_args.amt.to_string()
    }
}

impl Into<Brc20TransferEvent> for Brc20Event {
    fn into(self) -> Brc20TransferEvent {
        Brc20TransferEvent {
            amout: self.amount.parse().unwrap(),
            from: self.from,
            to: self.to,
            valid: true,
        }
    }
}

pub async fn query_transfer_event(query_transfer_args: QueryBrc20TransferArgs) -> Option<Brc20TransferEvent> {
    let r = query(&query_transfer_args).await;
    match r {
        Ok(c) => {
            if c.is_ok() {
                let data = c.data.unwrap();
                let resp = data.detail;
                for event in resp {
                    if event.check(&query_transfer_args)  && data.height - event.height >= 4 {
                        return Some(event.into());
                    }
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

pub async fn query(query_transfer_args: &QueryBrc20TransferArgs) -> Result<CommonResponse<QueryBrc20EventResponse>, UnisatError> {
    let base_url = match read_state(|s|s.network) {
        BitcoinNetwork::Mainnet => {MAINNET_BASE_URL}
        BitcoinNetwork::Testnet => {TESTNET_BASE_URL}
    };
    let api_key = api_key();
    let url = format!("{base_url}/v1/indexer/address/{}/brc20/{}/history?type=receive&start=0&limit=50", query_transfer_args.to_addr, query_transfer_args.ticker);
    const MAX_CYCLES: u128 = 1_000_000_000;

    let request = CanisterHttpRequestArgument {
        url: url,
        method: HttpMethod::GET,
        body: None,
        max_response_bytes: None,
        transform: None /* //TODO Some(TransformContext {
            function: TransformFunc(candid::Func {
                principal: ic_cdk::api::id(),
                method: "transform".to_string(),
            }),
            context: vec![],
        })*/,
        headers: vec![HttpHeader {
            name: "Content-Type".to_string(),
            value: "application/json".to_string(),
        },
        HttpHeader {
            name: "Authorization".to_string(),
            value: api_key,
        }],
    };

     match http_request(request, MAX_CYCLES).await {
        Ok((response,)) => {
            let status = response.status;
            if status == Nat::from(200_u32) {
                let body = String::from_utf8(response.body).map_err(|_| {
                    UnisatError::Rpc(
                        "Transformed response is not UTF-8 encoded".to_string(),
                    )
                })?;
                let tx: CommonResponse<QueryBrc20EventResponse> = serde_json::from_str(&body).map_err(|_| {
                    UnisatError::Rpc(
                        "failed to decode transaction from json".to_string(),
                    )
                })?;
                Ok(tx)
            }else {
                Err(UnisatError::Rpc("http response not 200".to_string()))
            }
        }
        Err((_, m)) => Err(UnisatError::Rpc(m)),
    }
}

#[derive(CandidType, Clone, Debug, Deserialize, PartialEq, Eq)]
pub enum UnisatError {
    Rpc(String)
}
