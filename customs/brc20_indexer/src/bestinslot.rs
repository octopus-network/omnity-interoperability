use std::ops::Div;
use std::str::FromStr;
use candid::{CandidType, Nat};
use ic_canister_log::log;
use ic_cdk::api::management_canister::http_request::{CanisterHttpRequestArgument, http_request, HttpHeader, HttpMethod, TransformContext, TransformFunc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use omnity_types::ic_log::{ERROR, INFO};
use crate::service::{Brc20TransferEvent, QueryBrc20TransferArgs};
use crate::state::{api_key, BitcoinNetwork, proxy_url, read_state};

const TESTNET_BASE_URL: &str = "https://testnet.api.bestinslot.xyz";
const MAINNET_BASE_URL: &str = "https://api.bestinslot.xyz";
const RPC_NAME: &str = "BESTINSLOT";

#[derive(Serialize, Clone, Deserialize)]
struct EventContent {
    pub tick: String,
    pub amount: String,
    pub using_tx_id: String,
    pub spent_wallet: String,
    pub original_tick: String,
    pub source_wallet: String,
    #[serde(rename = "spent_pkScript")]
    pub spent_pk_script: String,
    #[serde(rename = "source_pkScript")]
    pub source_pk_script: String,
}

#[derive(Serialize, Clone, Deserialize)]
struct Brc20Event {
    pub inscription_id: String,
    pub event_type: String,
    pub event: EventContent,
}

#[derive(Serialize, Deserialize)]
struct BestInSlotBrc20Respsonse {
    pub data: Vec<Brc20Event>,
    pub block_height: i64,
}

impl BestInSlotBrc20Respsonse {
    pub fn is_ok(&self) -> bool {
        self.data.len() == 1
    }

    pub fn check(&self, query_brc20transfer_args: &QueryBrc20TransferArgs) ->bool {
        self.is_ok() && self.data.first().cloned().unwrap().event.check(query_brc20transfer_args)
    }
}

impl EventContent {
    pub fn check(&self, query_transfer_args: &QueryBrc20TransferArgs) -> bool {
            let amt: u128 = self.amount.parse().unwrap_or(0);
            self.spent_wallet == query_transfer_args.to_addr &&
            self.tick.to_lowercase() == query_transfer_args.ticker.to_lowercase() &&
            amt == query_transfer_args.get_amt_satoshi()
    }
}


impl Into<Brc20TransferEvent> for BestInSlotBrc20Respsonse {
    fn into(self) -> Brc20TransferEvent {
        let event = self.data.first().cloned().unwrap();
        Brc20TransferEvent {
            amout: event.event.amount,
            from: event.event.source_wallet.clone(),
            to: event.event.spent_wallet.clone(),
            valid: true,
        }
    }
}


pub async fn bestinsolt_query_transfer_event(query_transfer_args: QueryBrc20TransferArgs) -> Option<Brc20TransferEvent> {
    let r = query(&query_transfer_args).await;
    match r {
        Ok(c) => {
            if c.is_ok() {

                if c.check(&query_transfer_args) {
                    let mut evt: Brc20TransferEvent = c.into();
                    let amt = Decimal::from_str(&evt.amout).unwrap()
                        .div(Decimal::from(10u128.pow(query_transfer_args.decimals as u32))).normalize().to_string();
                    evt.amout = amt;
                    return Some(evt);
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

async fn query(query_transfer_args: &QueryBrc20TransferArgs) -> Result<BestInSlotBrc20Respsonse, BestInSlotError> {
    let real_rpc_url = match read_state(|s|s.network) {
        BitcoinNetwork::Mainnet => {MAINNET_BASE_URL}
        BitcoinNetwork::Testnet => {TESTNET_BASE_URL}
    }.to_string();
    let api_key = api_key(RPC_NAME);
    log!(INFO, "bestinslot api key: {}", api_key.clone());
    let proxy_url = proxy_url();
    let uri = format!("/v3/brc20/event_from_txid?txid={}",query_transfer_args.tx_id);
    let url = format!("{proxy_url}{}",uri.clone());
    const MAX_CYCLES: u128 = 25_000_000_000;
    let idempotency_key = format!("bestinslot-{}",ic_cdk::api::time());
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
          name: "x-api-key".to_string(),
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
                    BestInSlotError::Rpc(
                        "Transformed response is not UTF-8 encoded".to_string(),
                    )
                })?;
                let tx: BestInSlotBrc20Respsonse = serde_json::from_str(&body).map_err(|_| {
                    BestInSlotError::Rpc(
                        "failed to decode transaction from json".to_string(),
                    )
                })?;
                Ok(tx)
            }else {
                Err(BestInSlotError::Rpc("http response not 200".to_string()))
            }
        }
        Err((_, m)) => Err(BestInSlotError::Rpc(m)),
    }
}

#[derive(CandidType, Clone, Debug, Deserialize, PartialEq, Eq)]
enum BestInSlotError {
    Rpc(String)
}