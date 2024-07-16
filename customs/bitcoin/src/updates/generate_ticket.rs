use crate::destination::Destination;
use crate::guard::{generate_ticket_guard, GuardError};
use crate::hub;
use crate::state::{
    audit, mutate_state, read_state, GenTicketRequestV2, GenTicketStatus, RUNES_TOKEN,
};
use crate::updates::get_btc_address::{
    destination_to_p2wpkh_address_from_state, init_ecdsa_public_key,
};
use crate::updates::rpc_types;
use candid::{CandidType, Deserialize, Nat};
use ic_btc_interface::{OutPoint, Txid, Utxo};
use ic_cdk::api::management_canister::http_request::{
    http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod, TransformContext,
    TransformFunc,
};
use omnity_types::rune_id::RuneId;
use omnity_types::{ChainState, Ticket, TicketType, TxAction};
use serde::Serialize;
use std::str::FromStr;

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct GenerateTicketArgs {
    pub target_chain_id: String,
    pub receiver: String,
    pub rune_id: String,
    pub amount: u128,
    pub txid: String,
}

#[derive(CandidType, Clone, Debug, Deserialize, PartialEq, Eq)]
pub enum GenerateTicketError {
    TemporarilyUnavailable(String),
    AlreadySubmitted,
    AlreadyProcessed,
    NoNewUtxos,
    TxNotFoundInMemPool,
    InvalidRuneId(String),
    InvalidTxId,
    UnsupportedChainId(String),
    UnsupportedToken(String),
    SendTicketErr(String),
    RpcError(String),
}

impl From<GuardError> for GenerateTicketError {
    fn from(e: GuardError) -> Self {
        match e {
            GuardError::TooManyConcurrentRequests => {
                Self::TemporarilyUnavailable("too many concurrent requests".to_string())
            }
        }
    }
}

pub async fn generate_ticket(args: GenerateTicketArgs) -> Result<(), GenerateTicketError> {
    if read_state(|s| s.chain_state == ChainState::Deactive) {
        return Err(GenerateTicketError::TemporarilyUnavailable(
            "chain state is deactive!".into(),
        ));
    }

    init_ecdsa_public_key().await;
    let _guard = generate_ticket_guard()?;

    let rune_id = RuneId::from_str(&args.rune_id)
        .map_err(|e| GenerateTicketError::InvalidRuneId(e.to_string()))?;

    let txid = Txid::from_str(&args.txid).map_err(|_| GenerateTicketError::InvalidTxId)?;

    if !read_state(|s| {
        s.counterparties
            .get(&args.target_chain_id)
            .is_some_and(|c| c.chain_state == ChainState::Active)
    }) {
        return Err(GenerateTicketError::UnsupportedChainId(
            args.target_chain_id.clone(),
        ));
    }

    let token_id = read_state(|s| {
        if let Some((token_id, _)) = s.tokens.iter().find(|(_, (r, _))| rune_id.eq(r)) {
            Ok(token_id.clone())
        } else {
            Err(GenerateTicketError::UnsupportedToken(args.rune_id))
        }
    })?;

    read_state(|s| match s.generate_ticket_status(txid) {
        GenTicketStatus::Pending(_) => Err(GenerateTicketError::AlreadySubmitted),
        GenTicketStatus::Finalized => Err(GenerateTicketError::AlreadyProcessed),
        GenTicketStatus::Unknown => Ok(()),
    })?;

    let (chain_id, hub_principal) = read_state(|s| (s.chain_id.clone(), s.hub_principal));

    let destination = Destination {
        target_chain_id: args.target_chain_id.clone(),
        receiver: args.receiver.clone(),
        token: Some(RUNES_TOKEN.into()),
    };

    let address = read_state(|s| destination_to_p2wpkh_address_from_state(s, &destination));

    // In order to prevent the memory from being exhausted,
    // ensure that the user has transferred token to this address.
    let new_utxos = fetch_new_utxos(txid, &address).await?;
    if new_utxos.len() == 0 {
        return Err(GenerateTicketError::NoNewUtxos);
    }

    hub::send_ticket(
        hub_principal,
        Ticket {
            ticket_id: args.txid.clone(),
            ticket_type: TicketType::Normal,
            ticket_time: ic_cdk::api::time(),
            src_chain: chain_id,
            dst_chain: args.target_chain_id.clone(),
            action: TxAction::Transfer,
            token: token_id.clone(),
            amount: args.amount.to_string(),
            sender: None,
            receiver: args.receiver.clone(),
            memo: None,
        },
    )
    .await
    .map_err(|err| GenerateTicketError::SendTicketErr(format!("{}", err)))?;

    let request = GenTicketRequestV2 {
        address,
        target_chain_id: args.target_chain_id,
        receiver: args.receiver,
        token_id,
        rune_id,
        amount: args.amount,
        txid,
        new_utxos: new_utxos.clone(),
        received_at: ic_cdk::api::time(),
    };

    mutate_state(|s| {
        audit::accept_generate_ticket_request(s, request);
    });
    Ok(())
}

async fn fetch_new_utxos(txid: Txid, address: &String) -> Result<Vec<Utxo>, GenerateTicketError> {
    const MAX_CYCLES: u128 = 1_000_000_000_000;
    const DERAULT_RPC_URL: &str = "https://mempool.space/api/tx";

    let url = format!(
        "{}/{}",
        read_state(|s| s.rpc_url.clone().unwrap_or(DERAULT_RPC_URL.to_string())),
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
                let tx: rpc_types::Transaction = serde_json::from_str(&body).map_err(|_| {
                    GenerateTicketError::RpcError(
                        "failed to decode transaction from json".to_string(),
                    )
                })?;
                let utxos = tx
                    .vout
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
                        height: tx.status.block_height,
                    })
                    .collect();
                Ok(utxos)
            } else if status == Nat::from(404_u32) {
                Err(GenerateTicketError::TxNotFoundInMemPool)
            } else {
                Err(GenerateTicketError::RpcError(format!(
                    "status code:{}",
                    status.to_string()
                )))
            }
        }
        Err((_, m)) => Err(GenerateTicketError::RpcError(m)),
    }
}
