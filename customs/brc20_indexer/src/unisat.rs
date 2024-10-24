use crate::service::{Brc20TransferEvent, QueryBrc20TransferArgs};
use crate::state::{api_key, proxy_url, read_state, BitcoinNetwork};
use candid::CandidType;
use ic_canister_log::log;
use ic_cdk::api::management_canister::http_request::{
    http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod, TransformContext,
    TransformFunc,
};
use omnity_types::ic_log::{ERROR, INFO};
use serde::{Deserialize, Serialize};

const TESTNET_BASE_URL: &str = "https://open-api-testnet.unisat.io";
const MAINNET_BASE_URL: &str = "https://open-api.unisat.io";
const RPC_NAME: &str = "UNISAT";

#[derive(Clone, Debug, Eq, PartialEq, Default, Serialize, Deserialize)]
struct CommonResponse<T> {
    pub code: i32,
    pub msg: String,
    pub data: Option<T>,
}

impl<T> CommonResponse<T> {
    pub fn is_ok(&self) -> bool {
        self.code == 0 && self.msg == "ok" && self.data.is_some()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Default, Serialize, Deserialize)]
pub struct QueryBrc20EventResponse {
    pub height: u32,
    pub total: u32,
    pub start: u32,
    detail: Vec<Brc20Event>,
}

#[derive(Clone, Debug, Eq, PartialEq, Default, Serialize, Deserialize)]
struct Brc20Event {
    pub ticker: String,
    #[serde(rename = "type")]
    pub typec: String,
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
        if self.txid != query_transfer_args.tx_id {
            log!(
                ERROR,
                "tixid ne, {} {}",
                self.txid,
                query_transfer_args.tx_id
            );
        }
        if !self.valid {
            log!(ERROR, "valid false ");
        }
        if self.to != query_transfer_args.to_addr {
            log!(
                ERROR,
                "to addr ne {} {}",
                self.to,
                query_transfer_args.to_addr
            );
        }
        if self.ticker != query_transfer_args.ticker {
            log!(
                ERROR,
                "ticker, {} {}",
                self.ticker,
                query_transfer_args.ticker
            );
        }
        if self.amount != query_transfer_args.amt {
            log!(
                ERROR,
                "amount ne {} {}",
                self.amount,
                query_transfer_args.amt
            );
        }
        self.txid == query_transfer_args.tx_id
            && self.valid
            && self.to == query_transfer_args.to_addr
            && self
                .ticker
                .eq_ignore_ascii_case(&query_transfer_args.ticker)
            && self.amount == query_transfer_args.amt
    }
}

impl From<Brc20Event> for Brc20TransferEvent {
    fn from(value: Brc20Event) -> Self {
        Brc20TransferEvent {
            amout: value.amount,
            from: value.from,
            to: value.to,
            valid: true,
            height: value.height as u64,
        }
    }
}

pub async fn unisat_query_transfer_event(
    query_transfer_args: &QueryBrc20TransferArgs,
) -> Option<Brc20TransferEvent> {
    let r = query(query_transfer_args).await;
    match r {
        Ok(c) => {
            if c.is_ok() {
                let data = c.data.unwrap();
                let resp = data.detail;
                log!(INFO, "{}", serde_json::to_string(&resp).unwrap());
                for event in resp {
                    if event.check(query_transfer_args) {
                        return Some(event.into());
                    }
                }
                log!(INFO, "a");
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
) -> Result<CommonResponse<QueryBrc20EventResponse>, UnisatError> {
    let real_rpc_url = match read_state(|s| s.network) {
        BitcoinNetwork::Mainnet => MAINNET_BASE_URL,
        BitcoinNetwork::Testnet => TESTNET_BASE_URL,
    }
    .to_string();
    let api_key = api_key(RPC_NAME);
    let proxy_url = proxy_url();
    let uri = format!(
        "/v1/indexer/brc20/{}/tx/{}/history?type=transfer&start=0&limit=16",
        query_transfer_args.ticker, query_transfer_args.tx_id
    );
    let url = format!("{proxy_url}{}", uri.clone());
    const MAX_CYCLES: u128 = 200_000_000;

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
                name: "Authorization".to_string(),
                value: api_key,
            },
            HttpHeader {
                name: crate::constant_args::FORWARD_SOLANA_RPC.to_string(),
                value: real_rpc_url,
            },
            HttpHeader {
                name: crate::constant_args::IDEMPOTENCY_KEY.to_string(),
                value: uri,
            },
        ],
    };

    match http_request(request, MAX_CYCLES).await {
        Ok((response,)) => {
            let status = response.status;
            if status == 200_u32 {
                let body = String::from_utf8(response.body).map_err(|_| {
                    UnisatError::Rpc("Transformed response is not UTF-8 encoded".to_string())
                })?;
                log!(INFO, "{}", body.clone());
                let tx: CommonResponse<QueryBrc20EventResponse> = serde_json::from_str(&body)
                    .map_err(|_| {
                        UnisatError::Rpc("failed to decode transaction from json".to_string())
                    })?;
                Ok(tx)
            } else {
                Err(UnisatError::Rpc("http response not 200".to_string()))
            }
        }
        Err((_, m)) => Err(UnisatError::Rpc(m)),
    }
}

#[derive(CandidType, Clone, Debug, Deserialize, PartialEq, Eq)]
enum UnisatError {
    Rpc(String),
}
