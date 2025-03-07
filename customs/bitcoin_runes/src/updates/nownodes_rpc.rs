use ic_btc_interface::{OutPoint, Txid, Utxo};
use ic_cdk::api::management_canister::http_request::{CanisterHttpRequestArgument, http_request, HttpHeader, HttpMethod, TransformContext, TransformFunc};
use serde::{Deserialize, Serialize};
use crate::state::read_state;
use crate::updates::generate_ticket::GenerateTicketError;
use crate::updates::rpc_types::{Transaction, TxOut};

#[derive(Serialize, Deserialize)]
struct NownodesVout {
    pub value: String,
    pub n: i64,
    pub hex: String,
    pub addresses: Vec<String>,
    #[serde(rename = "isAddress")]
    pub is_address: bool,
}

#[derive(Serialize, Deserialize)]
struct NownodesTransaction {
    pub vout: Vec<NownodesVout>
}

pub async fn fetch_new_utxos_from_nownodes(
    txid: Txid,
    address: &String,
) -> Result<(Vec<Utxo>, Transaction), GenerateTicketError> {
    const MAX_CYCLES: u128 = 6_000_000_000_000;
    let url = format!(
        "https://btcbook.nownodes.io/{}/api/v2/tx/{}",
        read_state(|s| s.nownodes_apikey.clone()),
        txid
    );

    let request = CanisterHttpRequestArgument {
        url: url.to_string(),
        method: HttpMethod::GET,
        body: None,
        max_response_bytes: Some(300000),
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

    match http_request(request, MAX_CYCLES).await {
        Ok((response,)) => {
            let status = response.status;
            if status == 200_u32 {
                let body = String::from_utf8(response.body).map_err(|_| {
                    GenerateTicketError::RpcError(
                        "Transformed response is not UTF-8 encoded".to_string(),
                    )
                })?;
                let tx: NownodesTransaction = serde_json::from_str(&body).map_err(|_| {
                    GenerateTicketError::RpcError(
                        "failed to decode transaction from json".to_string(),
                    )
                })?;
                let vouts:Vec<TxOut> = tx.vout.iter().enumerate().map(|out| {
                    let addr = if out.1.is_address {
                        out.1.addresses.first().cloned()
                    }else { None };
                    let value: u64 = out.1.value.parse().unwrap_or_default();
                    TxOut {
                        scriptpubkey_address: addr,
                        value,
                    }
                }).collect();
                let transfer_utxos = vouts
                    .iter()
                    .enumerate()
                    .filter(|(_, out)| {
                        out.scriptpubkey_address
                            .clone()
                            .is_some_and(|addr| addr.eq(address))
                    })
                    .map(|(i, out)| Utxo {
                        outpoint: OutPoint {
                            txid,
                            vout: i as u32,
                        },
                        value: out.value,
                        // The height is not known at this time
                        // as the transaction may not be confirmed yet.
                        // We will update the height when the transaction is confirmed.
                        height: 0,
                    })
                    .collect();
                let tx = Transaction {
                    vout: vouts,
                };
                Ok((transfer_utxos, tx))
            } else if status == 404_u32 {
                Err(GenerateTicketError::TxNotFoundInMemPool)
            } else {
                Err(GenerateTicketError::RpcError(format!(
                    "status code:{}",
                    status
                )))
            }
        }
        Err((_, m)) => Err(GenerateTicketError::RpcError(m)),
    }
}
