use crate::handler::burn_token::BurnTx;
use crate::ic_sui::ck_eddsa::KeyType;
use crate::ic_sui::rpc_client::{self, RpcClient, RpcResult};

use crate::config::read_config;
use crate::ic_sui::sui_json_rpc_types::sui_object::{SuiObjectData, SuiObjectDataOptions};
use crate::ic_sui::sui_json_rpc_types::SuiEvent;
use crate::ic_sui::sui_types::base_types::{MoveObjectType_, ObjectID, ObjectType};
use crate::ic_sui::sui_types::digests::TransactionDigest;
use crate::ic_sui::sui_types::object::Owner;
use crate::ic_sui::sui_types::TypeTag;
use crate::types::{ChainState, Error, TicketType, TxAction};
use crate::types::{Memo, Ticket};
use candid::{CandidType, Principal};

use crate::ic_log::{DEBUG, WARNING};
// use crate::ic_sui::sui_types::sui_serde::BigInt;
use crate::{
    call_error::{CallError, Reason},
    state::{mutate_state, read_state},
};
use ic_stable_structures::storable::Bound;
use ic_stable_structures::Storable;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use serde_with::DisplayFromStr;
use std::borrow::Cow;
use std::str::FromStr;

use ic_canister_log::log;

// use serde_json::from_value;
// use serde_json::Value;

#[derive(CandidType, Clone, Debug, Deserialize, PartialEq, Eq)]
pub enum GenerateTicketError {
    TemporarilyUnavailable(String),
    UnsupportedToken(String),
    UnsupportedChainId(String),
    /// The redeem account does not hold the requested token amount.
    InsufficientFunds {
        balance: u64,
    },
    /// The caller didn't approve enough funds for spending.
    InsufficientAllowance {
        allowance: u64,
    },
    SendTicketErr(String),
    InsufficientRedeemFee {
        required: u64,
        provided: u64,
    },
    RedeemFeeNotSet,
    TransferFailure(String),
    UnsupportedAction(String),
}

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct GenerateTicketReq {
    pub digest: String,
    pub target_chain_id: String,
    pub sender: String,
    pub receiver: String,
    pub token_id: String,
    pub amount: u64,
    pub action: TxAction,
    pub memo: Option<String>,
}

impl Storable for GenerateTicketReq {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        let bytes = bincode::serialize(&self).expect("failed to serialize GenerateTicketReq");
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        bincode::deserialize(bytes.as_ref()).expect("failed to deserialize GenerateTicketReq")
    }

    const BOUND: Bound = Bound::Unbounded;
}

#[derive(CandidType, Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct GenerateTicketOk {
    pub ticket_id: String,
}

pub async fn generate_ticket(
    req: GenerateTicketReq,
) -> Result<GenerateTicketOk, GenerateTicketError> {
    log!(DEBUG, "[generate_ticket] generate_ticket req: {:#?}", req);

    mutate_state(|s| {
        s.gen_ticket_reqs
            .insert(req.digest.to_owned(), req.to_owned())
    });

    if read_config(|s| s.get().chain_state == ChainState::Deactive) {
        return Err(GenerateTicketError::TemporarilyUnavailable(
            "chain state is deactive!".into(),
        ));
    }

    if !read_state(|s| {
        s.counterparties
            .get(&req.target_chain_id)
            .is_some_and(|c| c.chain_state == ChainState::Active)
    }) {
        return Err(GenerateTicketError::UnsupportedChainId(
            req.target_chain_id.to_owned(),
        ));
    }

    if !read_state(|s| s.tokens.contains_key(&req.token_id.to_string())) {
        return Err(GenerateTicketError::UnsupportedToken(
            req.token_id.to_owned(),
        ));
    }

    if !matches!(req.action, TxAction::Redeem) {
        return Err(GenerateTicketError::UnsupportedAction(
            "[generate_ticket] Transfer action is not supported".into(),
        ));
    }

    let (hub_principal, chain_id) =
        read_config(|s| (s.get().hub_principal, s.get().chain_id.to_owned()));

    if !verify_tx(req.to_owned()).await? {
        return Err(GenerateTicketError::TemporarilyUnavailable(format!(
            "[generate_ticket] Unable to verify the tx ({}) ",
            req.digest,
        )));
    }
    let fee = read_config(|s| s.get().get_fee(req.target_chain_id.to_owned())).unwrap_or_default();
    let memo = Memo {
        memo: req.memo,
        bridge_fee: fee,
    };
    // let memo = bridge_fee.add_to_memo(req.memo).unwrap_or_default();
    let memo_json = serde_json::to_string_pretty(&memo).map_err(|e| {
        GenerateTicketError::TemporarilyUnavailable(format!(
            "[generate_ticket] memo convert error: {}",
            e.to_string()
        ))
    })?;
    log!(DEBUG, "[generate_ticket] memo with fee: {:?}", memo_json);

    let ticket = Ticket {
        ticket_id: req.digest.to_string(),
        ticket_type: TicketType::Normal,
        ticket_time: ic_cdk::api::time(),
        src_chain: chain_id,
        dst_chain: req.target_chain_id.to_owned(),
        action: req.action.to_owned(),
        token: req.token_id.to_owned(),
        amount: req.amount.to_string(),
        sender: Some(req.sender.to_owned()),
        receiver: req.receiver.to_string(),
        memo: Some(memo_json.to_bytes().to_vec()),
    };

    match send_ticket(hub_principal, ticket.to_owned()).await {
        Err(err) => {
            mutate_state(|s| {
                s.tickets_failed_to_hub
                    .insert(ticket.ticket_id.to_string(), ticket.to_owned());
            });
            log!(
                WARNING,
                "[generate_ticket] failed to send ticket: {}",
                req.digest.to_string()
            );
            Err(GenerateTicketError::SendTicketErr(format!("{}", err)))
        }
        Ok(()) => {
            log!(
                DEBUG,
                "[generate_ticket] successful to send ticket: {:?}",
                ticket
            );

            mutate_state(|s| s.gen_ticket_reqs.remove(&req.digest.to_owned()));
            Ok(GenerateTicketOk {
                ticket_id: req.digest.to_string(),
            })
        }
    }
}

pub async fn verify_tx(req: GenerateTicketReq) -> Result<bool, GenerateTicketError> {
    let (provider, nodes, forward) = read_config(|s| {
        (
            s.get().rpc_provider.to_owned(),
            s.get().nodes_in_subnet,
            s.get().forward.to_owned(),
        )
    });
    let client = RpcClient::new(provider, Some(nodes));

    let multi_rpc_config = read_config(|s| s.get().multi_rpc_config.to_owned());
    multi_rpc_config
        .check_config_valid()
        .map_err(|e| GenerateTicketError::TemporarilyUnavailable(e.to_string()))?;
    let events = query_tx_from_multi_rpc(
        &client,
        req.digest.to_owned(),
        multi_rpc_config.rpc_list.to_owned(),
    )
    .await;

    let events = multi_rpc_config
        .valid_and_get_result(&events)
        .map_err(|e| GenerateTicketError::TemporarilyUnavailable(e.to_string()))?;

    let mut collect_fee_ok = false;
    let mut burn_token_ok = false;
    let mut redeem_ok = false;

    if events.len() < 3 {
        return Err(GenerateTicketError::TemporarilyUnavailable(
            "events size should be >= 3".to_string(),
        ));
    }
    let sui_port_action = read_config(|s| s.get().sui_port_action.to_owned());
    for event in &events {
        if !event.package_id.to_string().eq(&sui_port_action.package) {
            return Err(GenerateTicketError::TemporarilyUnavailable(
                "event is not from sui port action".to_string(),
            ));
        }
        if let Ok(collect_fee_event) =
            serde_json::from_value::<CollectFeeEvent>(event.parsed_json.to_owned())
        {
            log!(
                DEBUG,
                "[verify_tx] collect_fee_event: {:?}",
                collect_fee_event
            );
            let fee = read_config(|s| s.get().get_fee(req.target_chain_id.to_owned())).ok_or(
                GenerateTicketError::TemporarilyUnavailable(format!(
                    "[verify_tx] No found fee for {}",
                    req.target_chain_id
                )),
            )?;
            log!(DEBUG, "[verify_tx] fee from route: {}", fee);

            let collect_amount = collect_fee_event.fee_amount as u128;

            let fee_account = read_config(|s| s.get().fee_account.to_string());
            log!(DEBUG, "[verify_tx] fee_account from route: {}", fee_account);

            if !(collect_fee_event.sender.to_string().eq(&req.sender)
                && collect_fee_event.recipient.to_string().eq(&fee_account)
                && collect_amount == fee)
            {
                return Err(GenerateTicketError::TemporarilyUnavailable(format!(
                    "[verify_tx] Unable to verify the collect fee info",
                )));
            }
            // check the fee coin is gas coin
            collect_fee_ok = check_coin_type(
                &client,
                forward.to_owned(),
                collect_fee_event.fee_coin_id,
                MoveObjectType_::GasCoin,
            )
            .await
            .map_err(|e| GenerateTicketError::TemporarilyUnavailable(e.to_string()))?;
        } else if let Ok(burn_event) =
            serde_json::from_value::<BurnEvent>(event.parsed_json.to_owned())
        {
            log!(DEBUG, "[verify_tx] burn_event: {:?}", burn_event);
            let route_address = rpc_client::sui_route_address(KeyType::ChainKey)
                .await
                .map_err(|e| {
                    GenerateTicketError::TemporarilyUnavailable(format!(
                        "[verify_tx] {}",
                        e.to_string()
                    ))
                })?;

            if burn_event.sender.to_string().eq(&req.sender)
                && burn_event
                    .recipient
                    .to_string()
                    .eq(&route_address.to_string())
                && burn_event.burned_amount == req.amount
            {
                let burn_obj_data = get_coin_obj_data(
                    &client,
                    burn_event.burned_coin_id.to_owned(),
                    forward.to_owned(),
                )
                .await
                .map_err(|e| {
                    GenerateTicketError::TemporarilyUnavailable(format!(
                        "[verify_tx] {}",
                        e.to_string()
                    ))
                })?;
                log!(
                    DEBUG,
                    "[verify_tx] get_coin_obj_data ret: {:#?} ",
                    burn_obj_data
                );

                // check owner is sui route
                let owner = match burn_obj_data.owner {
                    None => false,
                    Some(obj_owner) => {
                        if matches!(obj_owner,Owner::AddressOwner(innter_owner) if innter_owner.eq(&route_address))
                        {
                            true
                        } else {
                            false
                        }
                    }
                };
                //check the burned coin is token_id type
                let sui_token =
                    read_state(|s| s.sui_tokens.get(&req.token_id)).expect("Not found sui token");

                let type_tag = TypeTag::from_str(&sui_token.type_tag).map_err(|e| {
                    GenerateTicketError::TemporarilyUnavailable(format!(
                        "[verify_tx] {}",
                        e.to_string()
                    ))
                })?;

                let coin_type = match burn_obj_data.type_.as_ref() {
                    Some(ObjectType::Struct(ty)) if ty.is_coin_t(&type_tag) => true,
                    Some(_) => false,
                    None => false,
                };
                if owner && coin_type {
                    burn_token_ok = true;
                    // save burned token to stable storage and burn it later
                    mutate_state(|s| {
                        s.burn_tokens.insert(
                            burn_event.burned_coin_id.to_string(),
                            BurnTx::new(req.token_id.to_owned()),
                        )
                    });
                }
            }
        } else if let Ok(redeem_event) =
            serde_json::from_value::<RedeemEvent>(event.parsed_json.to_owned())
        {
            log!(DEBUG, "[verify_tx] redeem_event: {:?}", redeem_event);

            if redeem_event.sender.to_string().eq(&req.sender)
                && redeem_event.receiver.eq(&req.receiver)
                && redeem_event.target_chain_id.eq(&req.target_chain_id)
                && redeem_event.token_id.eq(&req.token_id)
                && redeem_event.amount == req.amount
            {
                redeem_ok = true;
            }
        } else {
            log!(
                DEBUG,
                "[verify_tx] Unknown Parsed Value: {:#?}",
                event.parsed_json
            );
        }
    }
    log!(
        DEBUG,
        "[verify_tx] verify tx ,collect_fee :{},burn_token:{}, redeem :{}",
        collect_fee_ok,
        burn_token_ok,
        redeem_ok
    );
    Ok(collect_fee_ok && burn_token_ok && redeem_ok)
}

pub async fn get_coin_obj_data(
    client: &RpcClient,
    coin_id: ObjectID,
    forward: Option<String>,
) -> Result<SuiObjectData, String> {
    let obj_option = SuiObjectDataOptions {
        show_type: true,
        show_owner: true,
        show_previous_transaction: false,
        show_display: false,
        show_content: true,
        show_bcs: false,
        show_storage_rebate: false,
    };
    let resp = client
        .get_object(coin_id, Some(obj_option), forward)
        .await
        .map_err(|e| e.to_string())?;
    log!(
        DEBUG,
        "[gen_ticket::check_coin_type] get_object ret: {:?} ",
        resp
    );
    resp.into_object().map_err(|e| e.to_string())
}

pub async fn check_coin_type(
    client: &RpcClient,
    forward: Option<String>,
    coin_id: ObjectID,
    coin_type: MoveObjectType_,
) -> Result<bool, String> {
    let obj_option = SuiObjectDataOptions {
        show_type: true,
        show_owner: true,
        show_previous_transaction: false,
        show_display: false,
        show_content: true,
        show_bcs: false,
        show_storage_rebate: false,
    };
    let resp = client
        .get_object(coin_id, Some(obj_option), forward)
        .await
        .map_err(|e| e.to_string())?;
    log!(
        DEBUG,
        "[gen_ticket::check_coin_type] get_object ret: {:?} ",
        resp
    );
    let obj_data = resp.into_object().map_err(|e| e.to_string())?;
    match coin_type {
        MoveObjectType_::Other(..) => Ok(false),
        MoveObjectType_::GasCoin => Ok(obj_data.is_gas_coin()),
        MoveObjectType_::StakedSui => Ok(false),
        MoveObjectType_::Coin(type_tag) => match obj_data.type_.as_ref() {
            Some(ObjectType::Struct(ty)) if ty.is_coin_t(&type_tag) => Ok(true),
            Some(_) => Ok(false),
            None => Ok(false),
        },
    }
}

/// send ticket to hub
pub async fn send_ticket(hub_principal: Principal, ticket: Ticket) -> Result<(), CallError> {
    let resp: (Result<(), Error>,) =
        ic_cdk::api::call::call(hub_principal, "send_ticket", (ticket,))
            .await
            .map_err(|(code, message)| CallError {
                method: "send_ticket".to_string(),
                reason: Reason::from_reject(code, message),
            })?;
    let data = resp.0.map_err(|err| CallError {
        method: "send_ticket".to_string(),
        reason: Reason::CanisterError(err.to_string()),
    })?;
    Ok(data)
}

pub async fn query_tx_from_multi_rpc(
    client: &RpcClient,
    digest: String,
    rpc_url_vec: Vec<String>,
) -> Vec<RpcResult<Vec<SuiEvent>>> {
    let tx_digest = TransactionDigest::from_str(digest.as_ref()).expect("invalid digest");
    let mut fut = Vec::with_capacity(rpc_url_vec.len());
    for rpc_url in rpc_url_vec {
        fut.push(async { client.get_events(tx_digest.to_owned(), Some(rpc_url)).await });
    }
    futures::future::join_all(fut).await
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
// #[serde(rename_all = "camelCase")]
pub struct MintEvent {
    pub sender: String,
    pub recipient: ObjectID,
    #[serde_as(as = "DisplayFromStr")]
    pub mint_amount: u64,
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
// #[serde(rename_all = "camelCase")]
pub struct MintTicketEvent {
    pub ticket_id: String,
    pub sender: String,
    pub recipient: ObjectID,
    #[serde_as(as = "DisplayFromStr")]
    pub mint_amount: u64,
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
// #[serde(rename_all = "camelCase")]
pub struct CollectFeeEvent {
    pub sender: ObjectID,
    pub recipient: ObjectID,
    pub fee_coin_id: ObjectID,
    #[serde_as(as = "DisplayFromStr")]
    pub fee_amount: u64,
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct BurnEvent {
    pub sender: ObjectID,
    pub recipient: ObjectID,
    pub burned_coin_id: ObjectID,
    #[serde_as(as = "DisplayFromStr")]
    pub burned_amount: u64,
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
// #[serde(rename_all = "camelCase")]
pub struct RedeemEvent {
    pub target_chain_id: String,
    pub token_id: String,
    pub sender: ObjectID,
    pub receiver: String,
    #[serde_as(as = "DisplayFromStr")]
    pub amount: u64,
    pub action: String,
    pub memo: Option<String>,
}

#[cfg(test)]
mod test {
    use candid::Principal;

    use crate::{
        handler::gen_ticket::{BurnEvent, CollectFeeEvent, MintTicketEvent, RedeemEvent},
        ic_sui::{rpc_client::JsonRpcResponse, sui_json_rpc_types::SuiEvent},
    };
    #[test]
    fn test_management_canister() {
        let principal = Principal::management_canister();
        println!("The management principal value is: {}", principal)
    }
    #[test]
    fn parse_mint_event() {
        let json_str = r#" 
            {
                "jsonrpc": "2.0",
                "result": [
                    {
                    "id": {
                        "txDigest": "E4KDwN9MuLDa2aqFSUCcPAQ24ADWTr9Pn8d2QDXzFoVf",
                        "eventSeq": "0"
                    },
                    "packageId": "0x65d19d249dbac222c96d741fe695307c524316377482df8558bdc304f38a5917",
                    "transactionModule": "apple_pie",
                    "sender": "0x69201f957c4342fa5ba959bdcf4d6cfa887500c6c824a8213ca6fd0558e726c5",
                    "type": "0x65d19d249dbac222c96d741fe695307c524316377482df8558bdc304f38a5917::apple_pie::MintToEvent",
                    "parsedJson": {
                        "mint_amount": "1000000000",
                        "recipient": "ed5e7d559d66db01f7a9db05e7c3b803170d7d2d645a553e0c148513da5ad770",
                        "ticket_id": "apple_pie_ticket_id_4"
                    },
                    "bcsEncoding": "base64",
                    "bcs": "FWFwcGxlX3BpZV90aWNrZXRfaWRfNEBlZDVlN2Q1NTlkNjZkYjAxZjdhOWRiMDVlN2MzYjgwMzE3MGQ3ZDJkNjQ1YTU1M2UwYzE0ODUxM2RhNWFkNzcwAMqaOwAAAAA="
                    }
                ],
                "id": 1
            }
            "#;
        let json_response = serde_json::from_str::<JsonRpcResponse<Vec<SuiEvent>>>(json_str);
        println!("json_response: {:#?}", json_response);
        let events = json_response.unwrap().result.unwrap();
        // println!("events: {:#?}", events);
        for event in &events {
            let parsed_json = serde_json::to_string(&event.parsed_json).unwrap();
            println!("parsed_json: {:?}", parsed_json);

            if let Ok(mint_event) =
                serde_json::from_value::<MintTicketEvent>(event.parsed_json.to_owned())
            {
                println!("mint_event: {:#?}", mint_event);
            } else {
                println!(" Unknown Parsed Value: {:?}", event.parsed_json);
            }
        }
    }

    #[test]
    fn parse_redeem_events() {
        let json_str = r#" 
          {
            "jsonrpc": "2.0",
            "result": [
                {
                    "id": {
                        "txDigest": "32LPck96ThoVAGUcKDs6d4As9oDfUJS4UeE3FevbfHgd",
                        "eventSeq": "0"
                    },
                    "packageId": "0x58cf166ca300288cd32bbe5d9e864955301cf3e41a3cd027c6dd4a5760bfc628",
                    "transactionModule": "action",
                    "sender": "0x021e364dfa89ce87cbfbbae322ebd730c0737ff10a41d4a3b295f1b386031c51",
                    "type": "0x58cf166ca300288cd32bbe5d9e864955301cf3e41a3cd027c6dd4a5760bfc628::action::CollectFeeEvent",
                    "parsedJson": {
                        "fee_amount": "20000000",
                        "fee_coin_id": "4091bc4cdfcbf107805aba3cc318395253be7578b881054e00224aaba2215840",
                        "recipient": "af9306cac62396be300b175046140c392eed876bd8ac0efac6301cea286fa272",
                        "sender": "021e364dfa89ce87cbfbbae322ebd730c0737ff10a41d4a3b295f1b386031c51"
                    },
                    "bcsEncoding": "base64",
                    "bcs": "QDAyMWUzNjRkZmE4OWNlODdjYmZiYmFlMzIyZWJkNzMwYzA3MzdmZjEwYTQxZDRhM2IyOTVmMWIzODYwMzFjNTFAYWY5MzA2Y2FjNjIzOTZiZTMwMGIxNzUwNDYxNDBjMzkyZWVkODc2YmQ4YWMwZWZhYzYzMDFjZWEyODZmYTI3MkA0MDkxYmM0Y2RmY2JmMTA3ODA1YWJhM2NjMzE4Mzk1MjUzYmU3NTc4Yjg4MTA1NGUwMDIyNGFhYmEyMjE1ODQwAC0xAQAAAAA="
                    },
                    {
                    "id": {
                        "txDigest": "32LPck96ThoVAGUcKDs6d4As9oDfUJS4UeE3FevbfHgd",
                        "eventSeq": "1"
                    },
                    "packageId": "0x58cf166ca300288cd32bbe5d9e864955301cf3e41a3cd027c6dd4a5760bfc628",
                    "transactionModule": "action",
                    "sender": "0x021e364dfa89ce87cbfbbae322ebd730c0737ff10a41d4a3b295f1b386031c51",
                    "type": "0x58cf166ca300288cd32bbe5d9e864955301cf3e41a3cd027c6dd4a5760bfc628::action::BurnEvent",
                    "parsedJson": {
                        "burned_amount": "700000000",
                        "burned_coin_id": "e8b9bb426c11dd4d2546191175a9816356a06208ba00d30074707c7000951737",
                        "recipient": "bdaec7bab097484feaf9719d85951c81532d584a82bd8334b96c8b484780f0e9",
                        "sender": "021e364dfa89ce87cbfbbae322ebd730c0737ff10a41d4a3b295f1b386031c51"
                    },
                    "bcsEncoding": "base64",
                    "bcs": "QDAyMWUzNjRkZmE4OWNlODdjYmZiYmFlMzIyZWJkNzMwYzA3MzdmZjEwYTQxZDRhM2IyOTVmMWIzODYwMzFjNTFAYmRhZWM3YmFiMDk3NDg0ZmVhZjk3MTlkODU5NTFjODE1MzJkNTg0YTgyYmQ4MzM0Yjk2YzhiNDg0NzgwZjBlOUBlOGI5YmI0MjZjMTFkZDRkMjU0NjE5MTE3NWE5ODE2MzU2YTA2MjA4YmEwMGQzMDA3NDcwN2M3MDAwOTUxNzM3ACe5KQAAAAA="
                    },
                    {
                    "id": {
                        "txDigest": "32LPck96ThoVAGUcKDs6d4As9oDfUJS4UeE3FevbfHgd",
                        "eventSeq": "2"
                    },
                    "packageId": "0x58cf166ca300288cd32bbe5d9e864955301cf3e41a3cd027c6dd4a5760bfc628",
                    "transactionModule": "action",
                    "sender": "0x021e364dfa89ce87cbfbbae322ebd730c0737ff10a41d4a3b295f1b386031c51",
                    "type": "0x58cf166ca300288cd32bbe5d9e864955301cf3e41a3cd027c6dd4a5760bfc628::action::RedeemEvent",
                    "parsedJson": {
                        "action": "Redeem",
                        "amount": "700000000",
                        "memo": "This ticket is redeemed from Sui to Bitcoin",
                        "receiver": "bc1qmh0chcr9f73a3ynt90k0w8qsqlydr4a6espnj6",
                        "sender": "021e364dfa89ce87cbfbbae322ebd730c0737ff10a41d4a3b295f1b386031c51",
                        "target_chain_id": "sICP",
                        "token_id": "sICP-native-ICP"
                    },
                    "bcsEncoding": "base64",
                    "bcs": "BHNJQ1APc0lDUC1uYXRpdmUtSUNQQDAyMWUzNjRkZmE4OWNlODdjYmZiYmFlMzIyZWJkNzMwYzA3MzdmZjEwYTQxZDRhM2IyOTVmMWIzODYwMzFjNTEqYmMxcW1oMGNoY3I5ZjczYTN5bnQ5MGswdzhxc3FseWRyNGE2ZXNwbmo2ACe5KQAAAAAGUmVkZWVtAStUaGlzIHRpY2tldCBpcyByZWRlZW1lZCBmcm9tIFN1aSB0byBCaXRjb2lu"
                    }
                ],
                "id": 1
            }
            "#;
        let json_response = serde_json::from_str::<JsonRpcResponse<Vec<SuiEvent>>>(json_str);
        println!("json_response: {:?}", json_response);
        let events = json_response.unwrap().result.unwrap();
        // println!("events: {:#?}", events);
        for event in &events {
            let parsed_json = serde_json::to_string(&event.parsed_json).unwrap();
            println!("parsed_json: {:#?}", parsed_json);

            if let Ok(collect_fee_event) =
                serde_json::from_value::<CollectFeeEvent>(event.parsed_json.to_owned())
            {
                println!("collect_fee_event: {:#?}", collect_fee_event);
            } else if let Ok(burn_event) =
                serde_json::from_value::<BurnEvent>(event.parsed_json.to_owned())
            {
                println!("burn_event: {:#?}", burn_event);
            } else if let Ok(redeem_event) =
                serde_json::from_value::<RedeemEvent>(event.parsed_json.to_owned())
            {
                println!("redeem_event: {:#?}", redeem_event);
            } else {
                println!(" Unknown Parsed Value: {:?}", event.parsed_json);
            }
        }
    }

    #[test]
    fn memo_with_fee() {
        use crate::types::Memo;
        // user memo is Some(...)
        let memo = Some("some memo".to_string());
        let fee = 20000000 as u128;
        let memo_with_fee = Memo {
            memo,
            bridge_fee: fee,
        };

        let memo = serde_json::to_string_pretty(&memo_with_fee).unwrap();
        println!("[generate_ticket] Memo is some and fee: {:?}", memo);

        // user memo is None
        let memo = None;
        let fee = 20000000 as u128;
        let memo_with_fee = Memo {
            memo,
            bridge_fee: fee,
        };
        let memo = serde_json::to_string_pretty(&memo_with_fee).unwrap();

        println!("[generate_ticket] Memo is None and fee: {:?}", memo);
    }
}
