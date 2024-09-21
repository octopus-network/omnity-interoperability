use bitcoin::Transaction;
use candid::Nat;
use ic_btc_interface::Network;
use ic_cdk::api::management_canister::http_request::{CanisterHttpRequestArgument,
                                                     http_request, HttpHeader, HttpMethod, TransformContext, TransformFunc};
use crate::generate_ticket::{GenerateTicketArgs, GenerateTicketError};
use crate::generate_ticket::GenerateTicketError::InvalidArgs;
use crate::ord::inscription::brc20::{Brc20, Brc20Transfer};
use crate::ord::parser::OrdParser;
use crate::ord::mempool_rpc_types::TxInfo;
use crate::state::{deposit_addr, read_state};

pub async fn check_transaction(req: GenerateTicketArgs) -> Result<(), GenerateTicketError> {
    let token = read_state(|s|s.tokens.get(&req.token_id).cloned()).ok_or(InvalidArgs)?;
    let chain = read_state(|s|s.counterparties.get(&req.token_id).cloned()).ok_or(InvalidArgs)?;
    let transfer_transfer = query_transaction(&req.txid).await?;
    let receiver = transfer_transfer.vout.first().cloned().unwrap().scriptpubkey_address.unwrap();
    if receiver != deposit_addr().to_string() {
        return Err(GenerateTicketError::InvalidTxId)
    }
    let inscribe_txid = transfer_transfer.vin.first().cloned().unwrap().txid;
    let inscribe_transfer: Transaction = query_transaction(&inscribe_txid).await?
        .try_into().map_err(|e: anyhow::Error|GenerateTicketError::RpcError(e.to_string()))?;
    let (_inscription_id, parsed_inscription) = OrdParser::parse_one(&inscribe_transfer, 0)
                                        .map_err(|e| GenerateTicketError::OrdTxError(e.to_string()))?;
    let brc20 = Brc20::try_from(parsed_inscription).map_err(|e|GenerateTicketError::OrdTxError(e.to_string()))?;
    match brc20 {
        Brc20::Transfer(t) => {
            if t.amt as u128 != req.amount
                || t.tick != token.name
                || t.refx != req.receiver
                || t.chain != chain.chain_id {
                return Err(InvalidArgs);
            }else {
                return Ok(());
            }
        }
        _ => {
            return Err(GenerateTicketError::NotBridgeTx);
        }
    }
}


pub async fn query_transaction(txid: &String) -> Result<TxInfo, GenerateTicketError> {
    let nw = read_state(|s|s.btc_network);
    let network_str = match nw {
        Network::Mainnet => {"".to_string()}
        Network::Testnet => { "testnet".to_string()}
        Network::Regtest => {panic!("unsupported network")}
    };
    const MAX_CYCLES: u128 = 1_000_000_000;
    const DERAULT_RPC_URL: &str = "https://mempool.space/api/tx";
    let url = format!(
        "https://mempool.space/{}/api/tx/{}",
        network_str,
        txid.to_string()
    );

    let request = CanisterHttpRequestArgument {
        url: url.to_string(),
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
        }],
    };

    match http_request(request, MAX_CYCLES).await {
        Ok((response,)) => {
            let status = response.status;
            if status == Nat::from(200_u32) {
                let body = String::from_utf8(response.body).map_err(|_| {
                    GenerateTicketError::RpcError(
                        "Transformed response is not UTF-8 encoded".to_string(),
                    )
                })?;
                let tx: TxInfo = serde_json::from_str(&body).map_err(|_| {
                    GenerateTicketError::RpcError(
                        "failed to decode transaction from json".to_string(),
                    )
                })?;
                Ok(tx)
            }else {
                Err(GenerateTicketError::RpcError("http response not 200".to_string()))
            }
        }
        Err((_, m)) => Err(GenerateTicketError::RpcError(m)),
    }

}
