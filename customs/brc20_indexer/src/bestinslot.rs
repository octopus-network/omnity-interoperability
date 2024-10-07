use std::ops::Div;
use bitcoin::Amount;
use candid::{CandidType, Nat};
use ic_btc_interface::Network;
use ic_canister_log::log;
use ic_cdk::api::management_canister::http_request::{CanisterHttpRequestArgument, http_request, HttpHeader, HttpMethod, TransformContext, TransformFunc};
use serde::{Deserialize, Serialize};
use omnity_types::ic_log::ERROR;
use crate::service::{Brc20TransferEvent, QueryBrc20TransferArgs};
use crate::state::{api_key, BitcoinNetwork, proxy_url, read_state};


/*
{
  "data": [
    {
      "inscription_id": "6d4a1438ab43f941db9671fe8c4e5566984e14f0545f97143fa4df397295b755i0",
      "event_type": "transfer-transfer",
      "event": {
        "tick": "𝛑",
        "amount": "31415926535000000000000000000",
        "using_tx_id": "60998014",
        "spent_wallet": "bc1q7jyhzrgmaw26sggejpc5fe0ghecc53lu06u28c",
        "original_tick": "𝛑",
        "source_wallet": "bc1qgzdxs7vtzj3xywa9g50kdjkhsxqlu8ce6h6c63",
        "spent_pkScript": "0014f489710d1beb95a82119907144e5e8be718a47fc",
        "source_pkScript": "0014409a68798b14a2623ba5451f66cad78181fe1f19"
      }
    }
  ],
  "block_height": 864566
}
*/#[derive(Serialize, Deserialize)]
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

#[derive(Serialize, Deserialize)]
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


const TESTNET_BASE_URL: &str = "https://testnet.api.bestinslot.xyz";
const MAINNET_BASE_URL: &str = "https://api.bestinslot.xyz";
const RPC_NAME: &str = "BESTINSLOT";

pub async fn bestinsolt_query_transfer_event(query_transfer_args: QueryBrc20TransferArgs) -> Option<Brc20TransferEvent> {
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

async fn query(query_transfer_args: &QueryBrc20TransferArgs) -> Result<BestInSlotBrc20Respsonse, UnisatError> {
    let real_rpc_url = match read_state(|s|s.network) {
        BitcoinNetwork::Mainnet => {MAINNET_BASE_URL}
        BitcoinNetwork::Testnet => {TESTNET_BASE_URL}
    }.to_string();
    let api_key = api_key(RPC_NAME);
    let proxy_url = proxy_url();
    let uri = format!("/v3/brc20/event_from_txid?txid={}",query_transfer_args.tx_id);
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
                          name: "x-api-key".to_string(),
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
                    UnisatError::Rpc(
                        "Transformed response is not UTF-8 encoded".to_string(),
                    )
                })?;
                let tx: BestInSlotBrc20Respsonse = serde_json::from_str(&body).map_err(|_| {
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
enum UnisatError {
    Rpc(String)
}