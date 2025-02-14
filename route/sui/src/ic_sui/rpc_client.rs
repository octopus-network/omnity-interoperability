#![allow(unused)]
use crate::config::{mutate_config, read_config, SuiPortAction};
use crate::constants::{
    BURN_FUNC, COIN_MODULE, COIN_PKG_ID, DEFAULT_GAS_BUDGET, MINT_FUNC, MINT_WITH_TICKET_FUNC,
    SUI_COIN, UPDATE_DESC_FUNC, UPDATE_ICON_FUNC, UPDATE_NAME_FUNC, UPDATE_SYMBOL_FUNC,
};
use crate::ic_log::{DEBUG, ERROR};

use crate::ic_sui::ck_eddsa::hash_with_sha256;
use crate::ic_sui::constants::{FORWARD_KEY, IDEMPOTENCY_KEY};
use crate::ic_sui::constants::{HEADER_SIZE_LIMIT, TRANSACTION_RESPONSE_SIZE_ESTIMATE};
use crate::ic_sui::move_core_types::identifier::Identifier;
use crate::ic_sui::request::RpcRequest;
use crate::ic_sui::shared_inent::intent::{Intent, IntentMessage};
use crate::ic_sui::sui_json_rpc_types::CoinPage;
use crate::ic_sui::sui_providers::Provider;
use crate::ic_sui::sui_types::crypto::{DefaultHash, SignatureScheme};
use crate::ic_sui::sui_types::sui_serde::BigInt;
use crate::ic_sui::sui_types::transaction::{
    Argument, Command, ObjectArg, ProgrammableTransaction,
};
use crate::ic_sui::sui_types::{gas, TypeTag};
use crate::ic_sui::utils::get_http_request_cost;

use crate::state::{mutate_state, read_state, SuiToken, UpdateType};
use candid::CandidType;
use futures::{stream, StreamExt};
use futures_core::Stream;
use ic_canister_log::log;
use ic_cdk::api;
use ic_cdk::api::management_canister::http_request::{
    http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod, TransformContext,
};

use serde::Deserialize;
use serde::Serialize;
use serde_json::{json, Value};
use std::cell::RefCell;
use std::future;
use std::str::FromStr;
use std::sync::Arc;

use crate::ic_sui::sui_json_rpc_types::sui_object::{
    ObjectsPage, SuiObjectData, SuiObjectDataOptions,
};

use crate::ic_sui::sui_types::ptb::ProgrammableTransactionBuilder;

use super::ck_eddsa::{self, KeyType};
use super::fastcrypto::hash::HashFunction;
use super::sui_json_rpc_types::sui_object::SuiObjectResponse;
use super::sui_json_rpc_types::sui_transaction::{
    SuiTransactionBlockResponse, SuiTransactionBlockResponseOptions,
};
use super::sui_json_rpc_types::{Balance, Coin, SuiEvent};
use super::sui_types::base_types::{ObjectID, SuiAddress};
use super::sui_types::crypto::{Ed25519SuiSignature, Signature};
use super::sui_types::digests::TransactionDigest;
use super::sui_types::object::Owner;
use super::sui_types::quorum_driver_types::ExecuteTransactionRequestType;
use super::sui_types::transaction::{Transaction, TransactionData};
use serde_bytes::ByteBuf;

thread_local! {
    static NEXT_ID: RefCell<u64> = RefCell::default();
}

//Note: The sui address must be: hash(signature schema + sender public key bytes)
pub async fn sui_route_address(key_type: KeyType) -> Result<SuiAddress, String> {
    let pk = public_key_ed25519(key_type).await?;

    let mut hasher = DefaultHash::default();
    let sig_schema = SignatureScheme::ED25519;
    hasher.update([sig_schema.flag()]);
    hasher.update(&pk);
    let g_arr = hasher.finalize();
    let address = SuiAddress(g_arr.digest);
    Ok(address)
}

//TODO: cache the sui route address to save the cycles
pub async fn public_key_ed25519(key_type: KeyType) -> Result<Vec<u8>, String> {
    let address = read_state(|s| s.sui_route_addresses.get(&key_type));
    log!(
        DEBUG,
        "[rpc_client::public_key_ed25519] key type: {:?} and value from state: {:?} ",
        key_type,
        address,
    );

    match address {
        Some(address) => Ok(address),
        // create new address
        None => {
            let (chain_id, schnorr_key_name) = read_config(|s| {
                (
                    s.get().chain_id.to_owned(),
                    s.get().schnorr_key_name.to_owned(),
                    // s.get().sui_route_address.get(&key_type).cloned(),
                )
            });
            let derived_path = vec![ByteBuf::from(chain_id.as_bytes())];
            let pk =
                ck_eddsa::public_key_ed25519(key_type.to_owned(), schnorr_key_name, derived_path)
                    .await;
            //save the new address
            mutate_state(|s| {
                s.sui_route_addresses.insert(key_type, pk.to_owned());
            });
            Ok(pk)
        }
    }
}

pub async fn sign(msg: Vec<u8>, key_type: KeyType) -> Result<Vec<u8>, String> {
    let (chain_id, schnorr_key_name) = read_config(|s| {
        (
            s.get().chain_id.to_owned(),
            s.get().schnorr_key_name.to_owned(),
        )
    });
    let derived_path = vec![ByteBuf::from(chain_id.as_bytes())];
    // let msg = msg.as_bytes().to_vec();
    let signature = ck_eddsa::sign_with_eddsa(&key_type, schnorr_key_name, derived_path, msg).await;
    // let sig = String::from_utf8_lossy(&signature).to_string();
    Ok(signature)
}

//Note: The sui tx signature protocol: signature schema + signature+ public key
pub async fn sui_sign(raw_tx: Vec<u8>, key_type: KeyType) -> Result<Signature, String> {
    //first, hash tx with Blake2b256
    let mut hasher = DefaultHash::default();
    hasher.update(raw_tx);
    let digest = hasher.finalize().digest;
    log!(
        DEBUG,
        "[rpc_client::sui_sign] hashed digest : {:?} ",
        digest
    );
    // sign the hash(tx) degist with chain key
    let ck_signature = sign(digest.to_vec(), key_type).await.unwrap();

    log!(
        DEBUG,
        "[rpc_client::sui_sign] chain key signature: {:?} ",
        ck_signature
    );
    let mut sui_signature_bytes: Vec<u8> = Vec::new();

    let pk = public_key_ed25519(KeyType::ChainKey).await.map_err(|e| e)?;

    let sig_schema = SignatureScheme::ED25519;
    sui_signature_bytes.extend_from_slice(&[sig_schema.flag()]);
    sui_signature_bytes.extend_from_slice(ck_signature.as_ref());
    sui_signature_bytes.extend_from_slice(pk.as_ref());
    log!(
        DEBUG,
        "[rpc_client::sui_sign] sui signature_bytes: {:?} ",
        sui_signature_bytes
    );
    let ed25519_signature = Signature::Ed25519SuiSignature(
        Ed25519SuiSignature::from_bytes(sui_signature_bytes.as_ref()).unwrap(),
    );
    log!(
        DEBUG,
        "[rpc_client::sui_sign] Ed25519SuiSignature: {:?} ",
        ed25519_signature
    );

    Ok(ed25519_signature)
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct JsonRpcError {
    pub code: i64,
    pub message: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct JsonRpcResponse<T> {
    pub jsonrpc: String,
    pub result: Option<T>,
    pub error: Option<JsonRpcError>,
    pub id: u64,
}

#[derive(Debug, thiserror::Error, Deserialize, CandidType)]
pub enum RpcError {
    #[error("RPC request error: {0}")]
    RpcRequestError(String),
    #[error("RPC response error {code}: {message} {data:?}")]
    RpcResponseError {
        code: i64,
        message: String,
        data: Option<String>,
    },
    #[error("parse error: expected {0}")]
    ParseError(String),
    #[error("{0}")]
    Text(String),
}

impl From<JsonRpcError> for RpcError {
    fn from(e: JsonRpcError) -> Self {
        Self::RpcResponseError {
            code: e.code,
            message: e.message,
            data: None,
        }
    }
}

impl From<serde_json::Error> for RpcError {
    fn from(e: serde_json::Error) -> Self {
        let error_string = e.to_string();
        Self::ParseError(error_string)
    }
}

pub type RpcResult<T> = Result<T, RpcError>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RpcClient {
    pub provider: Provider,
    pub nodes_in_subnet: Option<u32>,
}

impl RpcClient {
    pub fn new(provider: Provider, nodes_in_subnet: Option<u32>) -> Self {
        Self {
            provider,
            nodes_in_subnet,
        }
    }

    pub fn with_nodes_in_subnet(mut self, nodes_in_subnet: u32) -> Self {
        self.nodes_in_subnet = Some(nodes_in_subnet);
        self
    }

    /// Asynchronously sends an HTTP POST request to the specified URL with the given payload and
    /// maximum response bytes, and returns the response as a string.
    /// This function calculates the required cycles for the HTTP request and logs the request
    /// details and response status. It uses a transformation named "cleanup_response" for the
    /// response body.
    ///
    /// # Arguments
    ///
    /// * `payload` - A string slice that holds the JSON payload to be sent in the HTTP request.
    /// * `max_response_bytes` - A u64 value representing the maximum number of bytes for the response.
    ///
    /// # Returns
    ///
    /// * `RpcResult<String>` - A result type that contains the response body as a string if the request
    /// is successful, or an `RpcError` if the request fails.
    ///
    /// # Errors
    ///
    /// This function returns an `RpcError` in the following cases:
    /// * If the response body cannot be parsed as a UTF-8 string, a `ParseError` is returned.
    /// * If the HTTP request fails, an `RpcRequestError` is returned with the error details.
    ///
    pub async fn call(
        &self,
        forward: Option<String>,
        payload: &str,
        max_response_bytes: u64,
        transform: Option<TransformContext>,
    ) -> RpcResult<String> {
        let transform = transform.unwrap_or(TransformContext::from_name(
            "cleanup_response".to_owned(),
            vec![],
        ));

        let mut headers = vec![HttpHeader {
            name: "Content-Type".to_string(),
            value: "application/json".to_string(),
        }];
        // add idempotency_key
        let idempotency_key = hash_with_sha256(payload);

        headers.push(HttpHeader {
            name: IDEMPOTENCY_KEY.to_string(),
            value: idempotency_key,
        });

        //TODO: get api key from config
        headers.push(HttpHeader {
            name: "api-key".to_string(),
            value: "c358082d-9e68-43da-a0fb-6f7240d01136".to_string(),
        });

        // add forward address
        if let Some(forward) = forward {
            headers.push(HttpHeader {
                name: FORWARD_KEY.to_string(),
                value: forward,
            });
        }

        // headers.push(HttpHeader {
        //     name: CLIENT_SDK_TYPE_HEADER.to_string(),
        //     value: CLIENT_VERSION.to_string(),
        // });
        // headers.push(HttpHeader {
        //     name: CLIENT_TARGET_API_VERSION_HEADER.to_string(),
        //     value: CLIENT_VERSION.to_string(),
        // });
        // headers.push(HttpHeader {
        //     name: CLIENT_SDK_VERSION_HEADER.to_string(),
        //     value: CLIENT_VERSION.to_string(),
        // });

        // log!(
        //     DEBUG,
        //     "ic-sui::rpc_client::call: http header: {:?}",
        //     headers
        // );

        let request = CanisterHttpRequestArgument {
            url: self.provider.url().to_string(),
            max_response_bytes: Some(max_response_bytes + HEADER_SIZE_LIMIT),
            // max_response_bytes: None,
            method: HttpMethod::POST,
            headers: headers,
            body: Some(payload.as_bytes().to_vec()),
            transform: Some(transform),
        };

        let url = self.provider.url();

        let cycles = get_http_request_cost(
            request.body.as_ref().map_or(0, |b| b.len() as u64),
            request.max_response_bytes.unwrap_or(2 * 1024 * 1024), // default 2Mb
        );

        log!(
            DEBUG,
            "Calling url: {url} with payload: {payload}. Cycles: {cycles}"
        );
        let start = api::time();
        match http_request(request, cycles).await {
            Ok((response,)) => {
                let end = api::time();
                let elapsed = (end - start) / 1_000_000_000;

                log!(
                    DEBUG,
                    "Got response (with {} bytes): {} from url: {} with status: {} the time elapsed: {}",
                    response.body.len(),
                    String::from_utf8_lossy(&response.body),
                    url,
                    response.status,
                    elapsed
                );

                match String::from_utf8(response.body) {
                    Ok(body) => Ok(body),
                    Err(error) => Err(RpcError::ParseError(error.to_string())),
                }
            }
            Err((r, m)) => {
                let end = api::time();
                let elapsed = (end - start) / 1_000_000_000;
                log!(
                    ERROR,
                    "Got response  error : {:?},{} from url: {} ,the time elapsed: {}",
                    r,
                    m,
                    url,
                    elapsed
                );
                Err(RpcError::RpcRequestError(format!("({r:?}) {m:?}")))
            }
        }
    }

    pub fn next_request_id(&self) -> u64 {
        NEXT_ID.with(|next_id| {
            let mut next_id = next_id.borrow_mut();
            let id = *next_id;
            *next_id = next_id.wrapping_add(1);
            id
        })
    }

    pub async fn get_coins(
        &self,
        owner: SuiAddress,
        coin_type: Option<String>,
        cursor: Option<ObjectID>,
        limit: Option<usize>,
        forward: Option<String>,
    ) -> RpcResult<CoinPage> {
        let mut params = vec![json!(owner)];
        if let Some(coin_type) = coin_type {
            params.push(json!(coin_type));
        }
        if let Some(cursor) = cursor {
            params.push(json!(cursor));
        }
        if let Some(limit) = limit {
            params.push(json!(limit));
        }

        let payload = RpcRequest::GetCoins
            .build_request_json(self.next_request_id(), json!(params))
            .to_string();
        log!(DEBUG, "[rpc_client::get_coins] get_coins: {} ", payload);

        let max_response_bytes = 5_000u64;
        let response = self
            .call(forward, &payload, max_response_bytes, None)
            .await?;
        log!(DEBUG, "[rpc_client::get_coins] response: {} ", response);
        // Ok(response)
        let json_response = serde_json::from_str::<JsonRpcResponse<CoinPage>>(&response)?;

        if let Some(e) = json_response.error {
            Err(e.into())
        } else {
            json_response.result.ok_or(RpcError::Text(
                "[rpc_client::get_coins] json_response.result is null".to_string(),
            ))
        }
    }

    pub fn get_coins_stream(
        &self,
        owner: SuiAddress,
        coin_type: Option<String>,
        forward: Option<String>,
    ) -> impl Stream<Item = Coin> + '_ {
        let forward = Arc::new(forward);
        stream::unfold(
            (vec![], None, true, coin_type, forward.clone()),
            move |(mut data, cursor, has_next_page, coin_type, forward)| async move {
                if let Some(item) = data.pop() {
                    Some((item, (data, cursor, true, coin_type, forward)))
                } else if has_next_page {
                    let page = self
                        .get_coins(owner, coin_type.clone(), cursor, None, (*forward).clone())
                        .await
                        .ok()?;
                    let mut data = page.data;
                    data.reverse();
                    data.pop().map(|item| {
                        (
                            item,
                            (
                                data,
                                page.next_cursor,
                                page.has_next_page,
                                coin_type,
                                forward,
                            ),
                        )
                    })
                } else {
                    None
                }
            },
        )
    }

    pub async fn fetch_coin(
        &self,
        owner: SuiAddress,
        coin_type: Option<String>,
        threshold: u64,
        forward: Option<String>,
    ) -> RpcResult<Option<Coin>> {
        let coins_stream = self.get_coins_stream(owner, coin_type, forward);

        let mut coins = coins_stream
            .skip_while(|c| future::ready(c.balance < threshold))
            .boxed();
        let coin = coins.next().await;
        Ok(coin)
    }

    pub async fn get_owned_objects(
        &self,
        address: SuiAddress,
        query: Option<Value>,
        cursor: Option<ObjectID>,
        limit: Option<usize>,
        forward: Option<String>,
    ) -> RpcResult<ObjectsPage> {
        let mut params = vec![json!(address)];
        if let Some(query) = query {
            params.push(query);
        }
        if let Some(cursor) = cursor {
            params.push(json!(cursor));
        }
        if let Some(limit) = limit {
            params.push(json!(limit));
        }

        let payload = RpcRequest::GetOwnedObjects
            .build_request_json(self.next_request_id(), json!(params))
            .to_string();
        log!(
            DEBUG,
            "[rpc_client::get_owned_objects] payload: {} ",
            payload
        );

        let max_response_bytes = 6_500u64;
        let response = self
            .call(forward, &payload, max_response_bytes, None)
            .await?;
        // Ok(response)
        let json_response = serde_json::from_str::<JsonRpcResponse<ObjectsPage>>(&response)?;

        if let Some(e) = json_response.error {
            Err(e.into())
        } else {
            json_response.result.ok_or(RpcError::Text(
                "[rpc_client::get_owned_objects] json_response.result is null".to_string(),
            ))
        }
    }

    pub async fn get_object(
        &self,
        object_id: ObjectID,
        options: Option<SuiObjectDataOptions>,
        forward: Option<String>,
    ) -> RpcResult<SuiObjectResponse> {
        let mut params = vec![json!(object_id)];
        if let Some(options) = options {
            params.push(json!(options));
        }

        let payload = RpcRequest::GetObject
            .build_request_json(self.next_request_id(), json!(params))
            .to_string();
        log!(DEBUG, "[rpc_client::get_object] payload: {} ", payload);

        let max_response_bytes = 2000u64;
        let response = self
            .call(forward, &payload, max_response_bytes, None)
            .await?;
        // Ok(response)
        let json_response = serde_json::from_str::<JsonRpcResponse<SuiObjectResponse>>(&response)?;

        if let Some(e) = json_response.error {
            Err(e.into())
        } else {
            json_response.result.ok_or(RpcError::Text(
                "[rpc_client::get_object] json_response.result is null".to_string(),
            ))
        }
    }

    pub async fn check_object_exists(
        &self,
        owner: SuiAddress,
        object_id: ObjectID,
        forward: Option<String>,
    ) -> RpcResult<bool> {
        let obj_option = SuiObjectDataOptions {
            show_type: true,
            show_owner: true,
            show_previous_transaction: false,
            show_display: false,
            show_content: true,
            show_bcs: false,
            show_storage_rebate: false,
        };

        let ret = self
            .get_object(object_id.to_owned(), Some(obj_option), forward)
            .await?;

        let obj_data: SuiObjectData = ret
            .into_object()
            .map_err(|e| RpcError::Text(e.to_string()))?;
        log!(
            DEBUG,
            "[rpc_client::check_object_exists] obj_data: {:?} ",
            obj_data
        );

        match obj_data.owner {
            None => {
                return Ok(false);
            }
            Some(obj_owner) => {
                if matches!(obj_owner,Owner::AddressOwner(innter_owner) if innter_owner.eq(&owner))
                {
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
        }
    }

    pub async fn get_balance(
        &self,
        owner: SuiAddress,
        coin_type: Option<String>,
        // options: Option<String>,
        forward: Option<String>,
    ) -> RpcResult<Balance> {
        let mut params = vec![json!(owner)];
        if let Some(coin_type) = coin_type {
            params.push(json!(coin_type));
        }
        let payload = RpcRequest::GetBalance
            .build_request_json(self.next_request_id(), json!(params))
            .to_string();
        log!(DEBUG, "[rpc_client::get_balance] payload: {} ", payload);

        let max_response_bytes = 500u64;
        let response = self
            .call(forward, &payload, max_response_bytes, None)
            .await?;

        let json_response = serde_json::from_str::<JsonRpcResponse<Balance>>(&response)?;

        if let Some(e) = json_response.error {
            Err(e.into())
        } else {
            json_response.result.ok_or(RpcError::Text(
                "[rpc_client::get_balance] json_response.result is null".to_string(),
            ))
        }
    }

    pub async fn get_gas_price(&self, forward: Option<String>) -> RpcResult<u64> {
        let payload = RpcRequest::GetReferenceGasPrice
            .build_request_json(self.next_request_id(), json!([]))
            .to_string();
        log!(DEBUG, "[rpc_client::get_gas_price] payload: {} ", payload);

        let max_response_bytes = 5_00u64;
        let response = self
            .call(forward, &payload, max_response_bytes, None)
            .await?;
        log!(DEBUG, "[rpc_client::get_gas_price] response: {} ", response);
        let json_response = serde_json::from_str::<JsonRpcResponse<BigInt<u64>>>(&response)?;
        log!(
            DEBUG,
            "[rpc_client::get_gas_price] json_response: {:#?} ",
            json_response
        );
        if let Some(e) = json_response.error {
            Err(e.into())
        } else {
            let gas_price = json_response.result.ok_or(RpcError::Text(
                "[rpc_client::get_gas_price] json_response.result is null".to_string(),
            ))?;
            Ok(gas_price.into_inner())
        }
    }

    pub async fn get_events(
        &self,
        digest: TransactionDigest,
        forward: Option<String>,
    ) -> RpcResult<Vec<SuiEvent>> {
        let params = vec![json!(digest)];

        let payload = RpcRequest::GetEvents
            .build_request_json(self.next_request_id(), json!(params))
            .to_string();
        log!(DEBUG, "[rpc_client::get_events] payload: {} ", payload);

        let max_response_bytes = 5_000u64;
        let response = self
            .call(forward, &payload, max_response_bytes, None)
            .await?;
        log!(DEBUG, "[rpc_client::get_events] response: {} ", response);
        // Ok(response)
        let json_response = serde_json::from_str::<JsonRpcResponse<Vec<SuiEvent>>>(&response)?;

        if let Some(e) = json_response.error {
            Err(e.into())
        } else {
            json_response.result.ok_or(RpcError::Text(
                "[rpc_client::get_events] json_response.result is null".to_string(),
            ))
        }
    }

    pub async fn get_transaction_block(
        &self,
        digest: TransactionDigest,
        options: Option<SuiTransactionBlockResponseOptions>,
        forward: Option<String>,
    ) -> RpcResult<SuiTransactionBlockResponse> {
        let mut params = vec![json!(digest)];
        if let Some(options) = options {
            params.push(json!(options));
        }
        let payload = RpcRequest::GetTransactionBlock
            .build_request_json(self.next_request_id(), json!(params))
            .to_string();
        log!(
            DEBUG,
            "[rpc_client::get_transaction_block] payload: {} ",
            payload
        );

        let max_response_bytes = TRANSACTION_RESPONSE_SIZE_ESTIMATE;
        let response = self
            .call(forward, &payload, max_response_bytes, None)
            .await?;
        log!(
            DEBUG,
            "[rpc_client::get_transaction_block] response: {} ",
            response
        );
        let json_response =
            serde_json::from_str::<JsonRpcResponse<SuiTransactionBlockResponse>>(&response)?;
        log!(
            DEBUG,
            "[rpc_client::get_transaction_block] json_response: {:#?} ",
            json_response
        );
        if let Some(e) = json_response.error {
            Err(e.into())
        } else {
            json_response.result.ok_or(RpcError::Text(
                "[rpc_client::get_transaction_block] json_response.result is null".to_string(),
            ))
        }
    }

    pub async fn transfer_objects(
        &self,
        recipient: SuiAddress,
        object_ids: Vec<ObjectID>,
        gas_budget: Option<u64>,
        forward: Option<String>,
    ) -> RpcResult<SuiTransactionBlockResponse> {
        let mut ptb = ProgrammableTransactionBuilder::new();

        let mut args = vec![];
        for object_id in object_ids {
            let obj_ref = self
                .get_object(object_id, None, forward.to_owned())
                .await?
                .into_object()
                .map_err(|e| RpcError::Text(e.to_string()))?
                .object_ref();
            let obj_arg = ptb
                .obj(ObjectArg::ImmOrOwnedObject(obj_ref))
                .map_err(|e| RpcError::Text(e.to_string()))?;
            args.push(obj_arg);
        }

        log!(DEBUG, "[rpc_client::transfer_object] obj_args: {:?} ", args);
        ptb.transfer_args(recipient, args);

        let pt = ptb.finish();
        self.build_and_send_tx(pt, gas_budget, forward).await
    }

    pub async fn transfer_sui(
        &self,
        amount: u64,
        recipient: SuiAddress,
        gas_budget: Option<u64>,
        forward: Option<String>,
    ) -> RpcResult<SuiTransactionBlockResponse> {
        let mut ptb = ProgrammableTransactionBuilder::new();
        ptb.transfer_sui(recipient, Some(amount));
        let pt = ptb.finish();
        self.build_and_send_tx(pt, gas_budget, forward).await
    }

    pub async fn update_token_meta(
        &self,
        sui_token: SuiToken,
        update_type: UpdateType,
        gas_budget: Option<u64>,
        forward: Option<String>,
    ) -> RpcResult<SuiTransactionBlockResponse> {
        let package = COIN_PKG_ID
            .parse::<ObjectID>()
            .map_err(|e| RpcError::Text(e.to_string()))?;
        let module = Identifier::new(COIN_MODULE).map_err(|e| RpcError::Text(e.to_string()))?;

        let (function, arg) = match update_type {
            crate::state::UpdateType::Name(name) => (
                Identifier::new(UPDATE_NAME_FUNC).map_err(|e| RpcError::Text(e.to_string()))?,
                name,
            ),
            crate::state::UpdateType::Symbol(symbol) => (
                Identifier::new(UPDATE_SYMBOL_FUNC).map_err(|e| RpcError::Text(e.to_string()))?,
                symbol,
            ),
            crate::state::UpdateType::Icon(icon) => (
                Identifier::new(UPDATE_ICON_FUNC).map_err(|e| RpcError::Text(e.to_string()))?,
                icon,
            ),
            crate::state::UpdateType::Description(desc) => (
                Identifier::new(UPDATE_DESC_FUNC).map_err(|e| RpcError::Text(e.to_string()))?,
                desc,
            ),
        };

        let treasury_cap = sui_token
            .treasury_cap
            .parse::<ObjectID>()
            .map_err(|e| RpcError::Text(e.to_string()))?;
        let treasury_ref = self
            .get_object(treasury_cap, None, forward.to_owned())
            .await?
            .into_object()
            .map_err(|e| RpcError::Text(e.to_string()))?
            .object_ref();

        log!(
            DEBUG,
            "[rpc_client::update_token_meta] treasury_ref: {:?} ",
            treasury_ref
        );

        let coin_metadata = sui_token
            .metadata
            .parse::<ObjectID>()
            .map_err(|e| RpcError::Text(e.to_string()))?;
        let meta_data_ref = self
            .get_object(coin_metadata, None, forward.to_owned())
            .await?
            .into_object()
            .map_err(|e| RpcError::Text(e.to_string()))?
            .object_ref();
        log!(
            DEBUG,
            "[rpc_client::update_token_meta] meta_data_ref: {:?} ",
            meta_data_ref
        );

        // programmable transactions allows the user to bundle a number of actions into one transaction
        let mut ptb = ProgrammableTransactionBuilder::new();

        let treasury_input = ptb
            .obj(ObjectArg::ImmOrOwnedObject(treasury_ref))
            .map_err(|e| RpcError::Text(e.to_string()))?;
        let metadata_input = ptb
            .obj(ObjectArg::ImmOrOwnedObject(meta_data_ref))
            .map_err(|e| RpcError::Text(e.to_string()))?;
        let arg_input = ptb.pure(arg).map_err(|e| RpcError::Text(e.to_string()))?;

        let type_arg =
            TypeTag::from_str(&sui_token.type_tag).map_err(|e| RpcError::Text(e.to_string()))?;

        ptb.command(Command::move_call(
            package,
            module,
            function,
            vec![type_arg],
            vec![treasury_input, metadata_input, arg_input],
        ));
        let pt = ptb.finish();
        self.build_and_send_tx(pt, gas_budget, forward).await
    }

    pub async fn build_and_send_tx(
        &self,
        pt: ProgrammableTransaction,
        gas_budget: Option<u64>,
        forward: Option<String>,
    ) -> RpcResult<SuiTransactionBlockResponse> {
        let sender = sui_route_address(KeyType::ChainKey)
            .await
            .map_err(|e| RpcError::Text(e.to_string()))?;

        // prepare gas
        let gas_budget = gas_budget.unwrap_or(DEFAULT_GAS_BUDGET);
        let gas_price = self.get_gas_price(forward.to_owned()).await?;
        // testnet must >=1000
        // let gas_price = if gas_price < 1000 { 1000 } else { gas_price };
        let coin_type = Some(SUI_COIN.to_string());

        let coin = self
            .fetch_coin(sender, coin_type, gas_budget, forward.to_owned())
            .await?
            .ok_or(RpcError::Text(
                "[rpc_client::build_and_send_tx] Insufficient Funds".to_string(),
            ))?;

        log!(
            DEBUG,
            "[rpc_client::build_and_send_tx] gas coin : {:?} ",
            coin
        );
        let coin_obj_ref = coin.object_ref();

        // create the transaction data that will be sent to the network
        let tx_data = TransactionData::new_programmable(
            sender,
            vec![coin_obj_ref],
            pt,
            gas_budget,
            gas_price,
        );

        let intent = Intent::sui_transaction();
        let intent_msg = IntentMessage::new(intent, tx_data.to_owned());
        let raw_tx = bcs::to_bytes(&intent_msg).expect("Message serialization should not fail");

        // sign tx with sui signature protocol
        let ed25519_signature = sui_sign(raw_tx, KeyType::ChainKey)
            .await
            .map_err(|e| RpcError::Text(e.to_string()))?;

        // encapsulate tx
        let tx = Transaction::from_data(tx_data, vec![ed25519_signature]);
        let (tx_bytes, signatures) = tx.to_tx_bytes_and_signatures();

        let mut params = vec![json!(tx_bytes), json!(signatures)];
        let resp_options = SuiTransactionBlockResponseOptions::full_content();
        params.push(json!(resp_options));
        // let request_type = ExecuteTransactionRequestType::WaitForLocalExecution;
        let request_type = ExecuteTransactionRequestType::WaitForEffectsCert;
        params.push(json!(request_type));
        let payload = RpcRequest::ExecuteTransactionBlock
            .build_request_json(self.next_request_id(), json!(params))
            .to_string();

        log!(
            DEBUG,
            "[rpc_client::build_and_send_tx] payload: {} ",
            payload
        );

        // Submit the transaction
        let max_response_bytes = TRANSACTION_RESPONSE_SIZE_ESTIMATE;
        let response = self
            .call(forward, &payload, max_response_bytes, None)
            .await?;

        let json_response =
            serde_json::from_str::<JsonRpcResponse<SuiTransactionBlockResponse>>(&response)?;

        if let Some(e) = json_response.error {
            Err(e.into())
        } else {
            json_response.result.ok_or(RpcError::Text(
                "[rpc_client::build_and_send_tx] json_response.result is null".to_string(),
            ))
        }
    }

    pub async fn mint_with_ticket(
        &self,
        action: SuiPortAction,
        ticket_id: String,
        sui_token: SuiToken,
        recipient: SuiAddress,
        amount: u64,
        gas_budget: Option<u64>,
        forward: Option<String>,
    ) -> RpcResult<SuiTransactionBlockResponse> {
        let package = action
            .package
            .parse::<ObjectID>()
            .map_err(|e| RpcError::Text(e.to_string()))?;
        let module = Identifier::new(action.module).map_err(|e| RpcError::Text(e.to_string()))?;
        let mint_func =
            Identifier::new(MINT_WITH_TICKET_FUNC).map_err(|e| RpcError::Text(e.to_string()))?;
        let owner_cap = action
            .port_owner_cap
            .parse::<ObjectID>()
            .map_err(|e| RpcError::Text(e.to_string()))?;
        let owner_cap_ref = self
            .get_object(owner_cap, None, forward.to_owned())
            .await?
            .into_object()
            .map_err(|e| RpcError::Text(e.to_string()))?
            .object_ref();
        log!(
            DEBUG,
            "[rpc_client::mint_with_ticket] owner_cap_ref: {:?} ",
            owner_cap_ref
        );
        let treasury_cap = sui_token
            .treasury_cap
            .parse::<ObjectID>()
            .map_err(|e| RpcError::Text(e.to_string()))?;
        let treasury_ref = self
            .get_object(treasury_cap, None, forward.to_owned())
            .await?
            .into_object()
            .map_err(|e| RpcError::Text(e.to_string()))?
            .object_ref();
        let type_arg =
            TypeTag::from_str(&sui_token.type_tag).map_err(|e| RpcError::Text(e.to_string()))?;

        let ticket_table = action
            .ticket_table
            .parse::<ObjectID>()
            .map_err(|e| RpcError::Text(e.to_string()))?;
        let ticket_table_ref = self
            .get_object(ticket_table, None, forward.to_owned())
            .await?
            .into_object()
            .map_err(|e| RpcError::Text(e.to_string()))?
            .object_ref();

        log!(
            DEBUG,
            "[rpc_client::mint_with_ticket] ticket_table_ref: {:?} ",
            ticket_table_ref
        );
        // programmable transactions allows the user to bundle a number of actions into one transaction
        let mut ptb = ProgrammableTransactionBuilder::new();

        let owner_cap_input = ptb
            .obj(ObjectArg::ImmOrOwnedObject(owner_cap_ref))
            .map_err(|e| RpcError::Text(e.to_string()))?;

        let ticket_table_input = ptb
            .obj(ObjectArg::ImmOrOwnedObject(ticket_table_ref))
            .map_err(|e| RpcError::Text(e.to_string()))?;
        let ticket_id_input = ptb
            .pure(ticket_id)
            .map_err(|e| RpcError::Text(e.to_string()))?;
        let recipient_input = ptb
            .pure(recipient)
            .map_err(|e| RpcError::Text(e.to_string()))?;
        let amount_input = ptb
            .pure(amount)
            .map_err(|e| RpcError::Text(e.to_string()))?;

        let treasury_input = ptb
            .obj(ObjectArg::ImmOrOwnedObject(treasury_ref))
            .map_err(|e| RpcError::Text(e.to_string()))?;

        ptb.command(Command::move_call(
            package,
            module,
            mint_func,
            vec![type_arg],
            vec![
                owner_cap_input,
                ticket_table_input,
                ticket_id_input,
                recipient_input,
                amount_input,
                treasury_input,
            ],
        ));
        let pt = ptb.finish();

        self.build_and_send_tx(pt, gas_budget, forward).await
    }

    pub async fn mint_token(
        &self,
        action: SuiPortAction,
        sui_token: SuiToken,
        recipient: SuiAddress,
        amount: u64,
        gas_budget: Option<u64>,
        forward: Option<String>,
    ) -> RpcResult<SuiTransactionBlockResponse> {
        let package = action
            .package
            .parse::<ObjectID>()
            .map_err(|e| RpcError::Text(e.to_string()))?;
        let module = Identifier::new(action.module).map_err(|e| RpcError::Text(e.to_string()))?;
        let mint_func = Identifier::new(MINT_FUNC).map_err(|e| RpcError::Text(e.to_string()))?;
        let owner_cap = action
            .port_owner_cap
            .parse::<ObjectID>()
            .map_err(|e| RpcError::Text(e.to_string()))?;
        let owner_cap_ref = self
            .get_object(owner_cap, None, forward.to_owned())
            .await?
            .into_object()
            .map_err(|e| RpcError::Text(e.to_string()))?
            .object_ref();

        let treasury_cap = sui_token
            .treasury_cap
            .parse::<ObjectID>()
            .map_err(|e| RpcError::Text(e.to_string()))?;
        let treasury_ref = self
            .get_object(treasury_cap, None, forward.to_owned())
            .await?
            .into_object()
            .map_err(|e| RpcError::Text(e.to_string()))?
            .object_ref();
        let type_arg =
            TypeTag::from_str(&sui_token.type_tag).map_err(|e| RpcError::Text(e.to_string()))?;

        // programmable transactions allows the user to bundle a number of actions into one transaction
        let mut ptb = ProgrammableTransactionBuilder::new();

        let owner_cap_input = ptb
            .obj(ObjectArg::ImmOrOwnedObject(owner_cap_ref))
            .map_err(|e| RpcError::Text(e.to_string()))?;

        let recipient_input = ptb
            .pure(recipient)
            .map_err(|e| RpcError::Text(e.to_string()))?;
        let amount_input = ptb
            .pure(amount)
            .map_err(|e| RpcError::Text(e.to_string()))?;

        let treasury_input = ptb
            .obj(ObjectArg::ImmOrOwnedObject(treasury_ref))
            .map_err(|e| RpcError::Text(e.to_string()))?;
        ptb.command(Command::move_call(
            package,
            module,
            mint_func,
            vec![type_arg],
            vec![
                owner_cap_input,
                recipient_input,
                amount_input,
                treasury_input,
            ],
        ));
        let pt = ptb.finish();

        self.build_and_send_tx(pt, gas_budget, forward).await
    }

    pub async fn burn_token(
        &self,
        sui_token: SuiToken,
        burn_object_id: ObjectID,
        gas_budget: Option<u64>,
        forward: Option<String>,
    ) -> RpcResult<SuiTransactionBlockResponse> {
        let package = COIN_PKG_ID
            .parse::<ObjectID>()
            .map_err(|e| RpcError::Text(e.to_string()))?;
        let module = Identifier::new(COIN_MODULE).map_err(|e| RpcError::Text(e.to_string()))?;

        let burn_func = Identifier::new(BURN_FUNC).map_err(|e| RpcError::Text(e.to_string()))?;
        let treasury_cap = sui_token
            .treasury_cap
            .parse::<ObjectID>()
            .map_err(|e| RpcError::Text(e.to_string()))?;
        let treasury_ref = self
            .get_object(treasury_cap, None, forward.to_owned())
            .await?
            .into_object()
            .map_err(|e| RpcError::Text(e.to_string()))?
            .object_ref();
        log!(
            DEBUG,
            "[rpc_client::burn_token] treasury_ref: {:?} ",
            treasury_ref
        );
        let type_arg =
            TypeTag::from_str(&sui_token.type_tag).map_err(|e| RpcError::Text(e.to_string()))?;

        // programmable transactions allows the user to bundle a number of actions into one transaction
        let mut ptb = ProgrammableTransactionBuilder::new();

        let treasury_input = ptb
            .obj(ObjectArg::ImmOrOwnedObject(treasury_ref))
            .map_err(|e| RpcError::Text(e.to_string()))?;

        let burn_object_ref = self
            .get_object(burn_object_id, None, forward.to_owned())
            .await?
            .into_object()
            .map_err(|e| RpcError::Text(e.to_string()))?
            .object_ref();

        log!(
            DEBUG,
            "[rpc_client::burn_token] burn_object_ref: {:?} ",
            burn_object_ref
        );

        let burn_coin_input = ptb
            .obj(ObjectArg::ImmOrOwnedObject(burn_object_ref))
            .map_err(|e| RpcError::Text(e.to_string()))?;

        ptb.command(Command::move_call(
            package,
            module,
            burn_func,
            vec![type_arg],
            vec![treasury_input, burn_coin_input],
        ));
        let pt = ptb.finish();
        self.build_and_send_tx(pt, gas_budget, forward).await
    }

    pub async fn split_coin(
        &self,
        coin_id: ObjectID,
        amount: u64,
        recipient: SuiAddress,
        gas_budget: Option<u64>,
        forward: Option<String>,
    ) -> RpcResult<SuiTransactionBlockResponse> {
        // programmable transactions allows the user to bundle a number of actions into one transaction
        let mut ptb = ProgrammableTransactionBuilder::new();
        let coin_ref = self
            .get_object(coin_id, None, forward.to_owned())
            .await?
            .into_object()
            .map_err(|e| RpcError::Text(e.to_string()))?
            .object_ref();
        log!(DEBUG, "[rpc_client::split_coin] coin_ref: {:?} ", coin_ref);

        let coin_input = ptb
            .obj(ObjectArg::ImmOrOwnedObject(coin_ref))
            .map_err(|e| RpcError::Text(e.to_string()))?;
        let amt_input = ptb
            .pure(amount)
            .map_err(|e| RpcError::Text(e.to_string()))?;

        ptb.command(Command::SplitCoins(coin_input, vec![amt_input]));

        let recipient_input = ptb
            .pure(recipient)
            .map_err(|e| RpcError::Text(e.to_string()))?;
        ptb.command(Command::TransferObjects(
            vec![Argument::Result(0)],
            recipient_input,
        ));

        let pt = ptb.finish();
        self.build_and_send_tx(pt, gas_budget, forward).await
    }

    pub async fn merge_coin(
        &self,
        base_coin: ObjectID,
        merge_coins: Vec<ObjectID>,
        gas_budget: Option<u64>,
        forward: Option<String>,
    ) -> RpcResult<SuiTransactionBlockResponse> {
        // programmable transactions allows the user to bundle a number of actions into one transaction
        let mut ptb = ProgrammableTransactionBuilder::new();
        let base_coin_ref = self
            .get_object(base_coin, None, forward.to_owned())
            .await?
            .into_object()
            .map_err(|e| RpcError::Text(e.to_string()))?
            .object_ref();
        log!(
            DEBUG,
            "[rpc_client::merge_token] base_coin_ref: {:?} ",
            base_coin_ref
        );

        let base_coin_input = ptb
            .obj(ObjectArg::ImmOrOwnedObject(base_coin_ref))
            .map_err(|e| RpcError::Text(e.to_string()))?;

        let mut merge_coins_input = vec![];
        for merge_coin in merge_coins {
            let merge_coin_ref = self
                .get_object(merge_coin, None, forward.to_owned())
                .await?
                .into_object()
                .map_err(|e| RpcError::Text(e.to_string()))?
                .object_ref();
            let merge_coin_input = ptb
                .obj(ObjectArg::ImmOrOwnedObject(merge_coin_ref))
                .map_err(|e| RpcError::Text(e.to_string()))?;
            merge_coins_input.push(merge_coin_input);
        }

        ptb.command(Command::MergeCoins(base_coin_input, merge_coins_input));
        let pt = ptb.finish();
        self.build_and_send_tx(pt, gas_budget, forward).await
    }

    pub async fn create_ticket_table(
        &self,
        action: SuiPortAction,
        recipient: SuiAddress,
        gas_budget: Option<u64>,
        forward: Option<String>,
    ) -> RpcResult<SuiTransactionBlockResponse> {
        let package = action
            .package
            .parse::<ObjectID>()
            .map_err(|e| RpcError::Text(e.to_string()))?;
        let module = Identifier::new(action.module).map_err(|e| RpcError::Text(e.to_string()))?;
        let create_ticket_table_func =
            Identifier::new("create_ticket_table").map_err(|e| RpcError::Text(e.to_string()))?;

        let port_owner_cap = action
            .port_owner_cap
            .parse::<ObjectID>()
            .map_err(|e| RpcError::Text(e.to_string()))?;
        let port_owner_cap_ref = self
            .get_object(port_owner_cap, None, forward.to_owned())
            .await?
            .into_object()
            .map_err(|e| RpcError::Text(e.to_string()))?
            .object_ref();

        log!(
            DEBUG,
            "[rpc_client::create_ticket_table] port_owner_cap_ref: {:?} ",
            port_owner_cap_ref
        );

        // programmable transactions allows the user to bundle a number of actions into one transaction
        let mut ptb = ProgrammableTransactionBuilder::new();

        let recipient_input = ptb
            .pure(recipient)
            .map_err(|e| RpcError::Text(e.to_string()))?;

        let port_owner_cap_input = ptb
            .obj(ObjectArg::ImmOrOwnedObject(port_owner_cap_ref))
            .map_err(|e| RpcError::Text(e.to_string()))?;
        ptb.command(Command::move_call(
            package,
            module,
            create_ticket_table_func,
            vec![],
            vec![port_owner_cap_input, recipient_input],
        ));
        let pt = ptb.finish();

        self.build_and_send_tx(pt, gas_budget, forward).await
    }

    pub async fn drop_ticket_table(
        &self,
        action: SuiPortAction,
        gas_budget: Option<u64>,
        forward: Option<String>,
    ) -> RpcResult<SuiTransactionBlockResponse> {
        let package = action
            .package
            .parse::<ObjectID>()
            .map_err(|e| RpcError::Text(e.to_string()))?;
        let module = Identifier::new(action.module).map_err(|e| RpcError::Text(e.to_string()))?;
        let drop_ticket_table_func =
            Identifier::new("drop_ticket_table").map_err(|e| RpcError::Text(e.to_string()))?;

        let port_owner_cap = action
            .port_owner_cap
            .parse::<ObjectID>()
            .map_err(|e| RpcError::Text(e.to_string()))?;
        let port_owner_cap_ref = self
            .get_object(port_owner_cap, None, forward.to_owned())
            .await?
            .into_object()
            .map_err(|e| RpcError::Text(e.to_string()))?
            .object_ref();
        log!(
            DEBUG,
            "[rpc_client::drop_ticket_table] port_owner_cap_ref: {:?} ",
            port_owner_cap_ref
        );
        let ticket_table = action
            .ticket_table
            .parse::<ObjectID>()
            .map_err(|e| RpcError::Text(e.to_string()))?;
        let ticket_table_ref = self
            .get_object(ticket_table, None, forward.to_owned())
            .await?
            .into_object()
            .map_err(|e| RpcError::Text(e.to_string()))?
            .object_ref();

        // programmable transactions allows the user to bundle a number of actions into one transaction
        let mut ptb = ProgrammableTransactionBuilder::new();

        let port_owner_cap_input = ptb
            .obj(ObjectArg::ImmOrOwnedObject(port_owner_cap_ref))
            .map_err(|e| RpcError::Text(e.to_string()))?;

        let ticket_table_input = ptb
            .obj(ObjectArg::ImmOrOwnedObject(ticket_table_ref))
            .map_err(|e| RpcError::Text(e.to_string()))?;

        ptb.command(Command::move_call(
            package,
            module,
            drop_ticket_table_func,
            vec![],
            vec![port_owner_cap_input, ticket_table_input],
        ));
        let pt = ptb.finish();

        self.build_and_send_tx(pt, gas_budget, forward).await
    }

    pub async fn remove_ticket(
        &self,
        action: SuiPortAction,
        ticket_id: String,
        gas_budget: Option<u64>,
        forward: Option<String>,
    ) -> RpcResult<SuiTransactionBlockResponse> {
        let package = action
            .package
            .parse::<ObjectID>()
            .map_err(|e| RpcError::Text(e.to_string()))?;
        let module = Identifier::new(action.module).map_err(|e| RpcError::Text(e.to_string()))?;
        let remove_ticket_func =
            Identifier::new("remove_ticket").map_err(|e| RpcError::Text(e.to_string()))?;

        let port_owner_cap = action
            .port_owner_cap
            .parse::<ObjectID>()
            .map_err(|e| RpcError::Text(e.to_string()))?;
        let port_owner_cap_ref = self
            .get_object(port_owner_cap, None, forward.to_owned())
            .await?
            .into_object()
            .map_err(|e| RpcError::Text(e.to_string()))?
            .object_ref();
        log!(
            DEBUG,
            "[rpc_client::remove_ticket] port_owner_cap_ref: {:?} ",
            port_owner_cap_ref
        );
        let ticket_table = action
            .ticket_table
            .parse::<ObjectID>()
            .map_err(|e| RpcError::Text(e.to_string()))?;
        let ticket_table_ref = self
            .get_object(ticket_table, None, forward.to_owned())
            .await?
            .into_object()
            .map_err(|e| RpcError::Text(e.to_string()))?
            .object_ref();

        // programmable transactions allows the user to bundle a number of actions into one transaction
        let mut ptb = ProgrammableTransactionBuilder::new();

        let port_owner_cap_input = ptb
            .obj(ObjectArg::ImmOrOwnedObject(port_owner_cap_ref))
            .map_err(|e| RpcError::Text(e.to_string()))?;
        let ticket_table_input = ptb
            .obj(ObjectArg::ImmOrOwnedObject(ticket_table_ref))
            .map_err(|e| RpcError::Text(e.to_string()))?;

        let ticket_id_input = ptb
            .pure(ticket_id)
            .map_err(|e| RpcError::Text(e.to_string()))?;

        ptb.command(Command::move_call(
            package,
            module,
            remove_ticket_func,
            vec![],
            vec![port_owner_cap_input, ticket_table_input, ticket_id_input],
        ));
        let pt = ptb.finish();

        self.build_and_send_tx(pt, gas_budget, forward).await
    }
}

#[cfg(test)]
mod test {
    use crate::ic_sui::{
        fastcrypto::encoding::Encoding, rpc_client::JsonRpcResponse,
        sui_json_rpc_types::sui_transaction::SuiTransactionBlockResponse,
    };

    #[test]
    fn test_digest_encoding() {
        use crate::ic_sui::fastcrypto;

        let digest_str = "AG4WNemLgGKBsKTKrb73Pj7Y1DsPwJ2mC3dXXPWJaiTM";
        println!("digest str: {}", digest_str);
        let digest_bytes = fastcrypto::encoding::Base58::decode(digest_str).unwrap();
        let mut digest = [0; 32];
        digest.copy_from_slice(&digest_bytes);
        println!("digest_bytes:{:?}", digest);
        let encoded_digest = fastcrypto::encoding::Base58::encode(&digest);
        println!("encoded_digest:{:?}", encoded_digest);
    }

    #[test]
    fn test_parse_tx() {
        let json_str = r#" 
            {
                "jsonrpc": "2.0",
                "result": {
                    "digest": "ARMvttszMtbnKC92SpzBndL1XGwuCacEB4DY14kgzj8h",
                    "transaction": {
                    "data": {
                        "messageVersion": "v1",
                        "transaction": {
                        "kind": "ProgrammableTransaction",
                        "inputs": [
                            {
                            "type": "object",
                            "objectType": "sharedObject",
                            "objectId": "0x0000000000000000000000000000000000000000000000000000000000000006",
                            "initialSharedVersion": "1",
                            "mutable": false
                            },
                            {
                            "type": "object",
                            "objectType": "sharedObject",
                            "objectId": "0x03db251ba509a8d5d8777b6338836082335d93eecbdd09a11e190a1cff51c352",
                            "initialSharedVersion": "406496849",
                            "mutable": false
                            },
                            {
                            "type": "object",
                            "objectType": "sharedObject",
                            "objectId": "0x3b585786b13af1d8ea067ab37101b6513a05d2f90cfe60e8b1d9e1b46a63c4fa",
                            "initialSharedVersion": "406731547",
                            "mutable": true
                            },
                            {
                            "type": "object",
                            "objectType": "immOrOwnedObject",
                            "objectId": "0x6895b8156480b0ca4cfa1a7a791827380aad5704abefef3874f444b8175da736",
                            "version": "450987990",
                            "digest": "6bzHYyXH38T3yQRBUesGoyuGAcuEGg2kH28gcB1xx4jQ"
                            }
                        ],
                        "transactions": [
                            {
                            "MoveCall": {
                                "package": "0xa31282fc0a0ad50cf5f20908cfbb1539a143f5a38912eb8823a8dd6cbf98bc44",
                                "module": "gateway",
                                "function": "collect_fee",
                                "type_arguments": [
                                "0x2::sui::SUI",
                                "0xdba34672e30cb065b1f93e3ab55318768fd6fef66c15942c9f7cb846e2f900e7::usdc::USDC"
                                ],
                                "arguments": [
                                {
                                    "Input": 0
                                },
                                {
                                    "Input": 1
                                },
                                {
                                    "Input": 2
                                },
                                {
                                    "Input": 3
                                }
                                ]
                            }
                            },
                            {
                            "MoveCall": {
                                "package": "0xa31282fc0a0ad50cf5f20908cfbb1539a143f5a38912eb8823a8dd6cbf98bc44",
                                "module": "gateway",
                                "function": "collect_reward",
                                "type_arguments": [
                                "0x2::sui::SUI",
                                "0xdba34672e30cb065b1f93e3ab55318768fd6fef66c15942c9f7cb846e2f900e7::usdc::USDC",
                                "0x2::sui::SUI"
                                ],
                                "arguments": [
                                {
                                    "Input": 0
                                },
                                {
                                    "Input": 1
                                },
                                {
                                    "Input": 2
                                },
                                {
                                    "Input": 3
                                }
                                ]
                            }
                            },
                            {
                            "MoveCall": {
                                "package": "0xa31282fc0a0ad50cf5f20908cfbb1539a143f5a38912eb8823a8dd6cbf98bc44",
                                "module": "gateway",
                                "function": "collect_reward",
                                "type_arguments": [
                                "0x2::sui::SUI",
                                "0xdba34672e30cb065b1f93e3ab55318768fd6fef66c15942c9f7cb846e2f900e7::usdc::USDC",
                                "0xe1b45a0e641b9955a20aa0ad1c1f4ad86aad8afb07296d4085e349a50e90bdca::blue::BLUE"
                                ],
                                "arguments": [
                                {
                                    "Input": 0
                                },
                                {
                                    "Input": 1
                                },
                                {
                                    "Input": 2
                                },
                                {
                                    "Input": 3
                                }
                                ]
                            }
                            }
                        ]
                        },
                        "sender": "0xc8d919cef59bf118454313315950f8a10ddb49f799fcceac7479721891051e45",
                        "gasData": {
                        "payment": [
                            {
                            "objectId": "0xb077fcadb92d9b58ebcb94ae69f8eaa8d6ab2e1be6396cff0ef873944f6f1d8c",
                            "version": 450989448,
                            "digest": "GxDHwvd9dVSrDMTBgkLWoq3z31TTRBS5b2ZW58jEvT3V"
                            },
                            {
                            "objectId": "0x70981b3a89638852af4b9553142a35098931131bb4e8d3aebc10fe59f23c7aae",
                            "version": 450989448,
                            "digest": "ESbgykbbDYfaYsbpio4uNQPqrDqqpA5PyJmzrhrgVgCD"
                            }
                        ],
                        "owner": "0xc8d919cef59bf118454313315950f8a10ddb49f799fcceac7479721891051e45",
                        "price": "750",
                        "budget": "8067420"
                        }
                    },
                    "txSignatures": [
                        "BQNNMTUxODU2MDgzNTgwOTQyOTMxMTYxNDI4MzY2MjgxNjMwMzE5ODk0NjYzMjcxNzYzNDU4NjU5MTIxMjk3NjY3NTA1MTM4NzA0OTU3NDRMNzE1Nzg1NzY5OTI1MDM5MTUzNjAwMzQyMDkwMDI2Njc2MjYxNDk5NjEwNjY4MDg5ODQ3OTY2NDg3MjcxNzg5NTAzNTM4MzA4OTI2OAExAwJMNzA4NzkyMTQ1NTk0NTA1MDY3NjEzNDQ3MzE0MTAyOTY1NDIwNjUwMzY4MjExOTA3NjM5NzU0MDExNzg1OTMxNzU3Nzg3Nzc5MTI2N00xNTIzMjgzNjMyNTQxMDI5NDgxNTg1NTU2MTUzMzI5OTgzODA2NjIzMDIyNjQ5MzU0MTI2NzkyODgyOTkyMzEyNjE4Njk0NjY2ODU1MwJNMTU1NzY1ODg1MTUyMDk2NDM1NTA2MDM2NTc2NDIzNzYzNDIwODEwOTI4MjgyOTQ5NDM4OTAzNTc2MzkwMjk5OTA3NzE2MzgyNDQxOTBNMjAzNzEwNDM2NzYxNzA2NzQxMDE1MjMwMTc2NTAzNTgwNzU2Mjk3OTA1NDY4NDY2NzQ1NjIyMDU3Nzk5NzA3ODI5OTQ0MzgyODQ5MzYCATEBMANMMzk5MTkwNzU5OTAxMjA3NzgyMjc4NTM1MDY5NTExODYwNDkwNzI3NjI3OTYwMTU1ODg2MTkxNTk0NDIwNDI2MzM4MzM1Njk2NzgyOE0xNDUxMjk1NTk0MjQxMDUxNDYyNzU0NDM3NjkwMzc3MTk0Nzg1MjMwNTg2ODUyNzQwOTEwMjc5NTQ5MTg0NjAyNTU4OTQ5OTU0NTg5NAExMXlKcGMzTWlPaUpvZEhSd2N6b3ZMMkZqWTI5MWJuUnpMbWR2YjJkc1pTNWpiMjBpTEMBZmV5SmhiR2NpT2lKU1V6STFOaUlzSW10cFpDSTZJalUyTkdabFlXTmxZek5sWW1SbVlXRTNNekV4WWpsa09HVTNNMk0wTWpneE9HWXlPVEV5TmpRaUxDSjBlWEFpT2lKS1YxUWlmUU0xNDk3MjgzNTg1MzQzMjI4MDY4MTMwMDk0MDMyNDM2MTgzNjg0MDE1MTY2NTM4MzY2Nzc2ODU1Mzc4NDI3NTUxMzU4NTI0MDUyNjg5MmkCAAAAAAAAYQDjHqDOmJmCs9Ey0x9bqQwPHrLiqIFqZe1PndyUl8q9iwDo4q34JwxobeR/sJFjhpItq+ivouLAQyQucccYWpUItQGb55qCwbtfAFT3Nd7HCNoN+hgM52b9ACAQoY9jRbI="
                    ]
                    },
                    "events": [
                    {
                        "id": {
                        "txDigest": "ARMvttszMtbnKC92SpzBndL1XGwuCacEB4DY14kgzj8h",
                        "eventSeq": "0"
                        },
                        "packageId": "0xa31282fc0a0ad50cf5f20908cfbb1539a143f5a38912eb8823a8dd6cbf98bc44",
                        "transactionModule": "gateway",
                        "sender": "0xc8d919cef59bf118454313315950f8a10ddb49f799fcceac7479721891051e45",
                        "type": "0x3492c874c1e3b3e2984e8c41b589e642d4d0a5d6459e5a9cfc2d52fd7c89c267::events::UserFeeCollected",
                        "parsedJson": {
                        "coin_a_amount": "114988573",
                        "coin_b_amount": "427399",
                        "pool_coin_a_amount": "1256736132348030",
                        "pool_coin_b_amount": "4842363490862",
                        "pool_id": "0x3b585786b13af1d8ea067ab37101b6513a05d2f90cfe60e8b1d9e1b46a63c4fa",
                        "position_id": "0x6895b8156480b0ca4cfa1a7a791827380aad5704abefef3874f444b8175da736",
                        "sequence_number": "478484"
                        },
                        "bcsEncoding": "base64",
                        "bcs": "O1hXhrE68djqBnqzcQG2UToF0vkM/mDosdnhtGpjxPpolbgVZICwykz6Gnp5GCc4Cq1XBKvv7zh09ES4F12nNh2W2gYAAAAAh4UGAAAAAAB+uL6u/nYEAC4SW3NnBAAAFE0HAAAAAAAAAAAAAAAAAA=="
                    },
                    {
                        "id": {
                        "txDigest": "ARMvttszMtbnKC92SpzBndL1XGwuCacEB4DY14kgzj8h",
                        "eventSeq": "1"
                        },
                        "packageId": "0xa31282fc0a0ad50cf5f20908cfbb1539a143f5a38912eb8823a8dd6cbf98bc44",
                        "transactionModule": "gateway",
                        "sender": "0xc8d919cef59bf118454313315950f8a10ddb49f799fcceac7479721891051e45",
                        "type": "0x3492c874c1e3b3e2984e8c41b589e642d4d0a5d6459e5a9cfc2d52fd7c89c267::events::UserRewardCollected",
                        "parsedJson": {
                        "pool_id": "0x3b585786b13af1d8ea067ab37101b6513a05d2f90cfe60e8b1d9e1b46a63c4fa",
                        "position_id": "0x6895b8156480b0ca4cfa1a7a791827380aad5704abefef3874f444b8175da736",
                        "reward_amount": "21550920",
                        "reward_decimals": 9,
                        "reward_symbol": "SUI",
                        "reward_type": "0000000000000000000000000000000000000000000000000000000000000002::sui::SUI",
                        "sequence_number": "478485"
                        },
                        "bcsEncoding": "base64",
                        "bcs": "O1hXhrE68djqBnqzcQG2UToF0vkM/mDosdnhtGpjxPpolbgVZICwykz6Gnp5GCc4Cq1XBKvv7zh09ES4F12nNkowMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAyOjpzdWk6OlNVSQNTVUkJSNdIAQAAAAAVTQcAAAAAAAAAAAAAAAAA"
                    },
                    {
                        "id": {
                        "txDigest": "ARMvttszMtbnKC92SpzBndL1XGwuCacEB4DY14kgzj8h",
                        "eventSeq": "2"
                        },
                        "packageId": "0xa31282fc0a0ad50cf5f20908cfbb1539a143f5a38912eb8823a8dd6cbf98bc44",
                        "transactionModule": "gateway",
                        "sender": "0xc8d919cef59bf118454313315950f8a10ddb49f799fcceac7479721891051e45",
                        "type": "0x3492c874c1e3b3e2984e8c41b589e642d4d0a5d6459e5a9cfc2d52fd7c89c267::events::UserRewardCollected",
                        "parsedJson": {
                        "pool_id": "0x3b585786b13af1d8ea067ab37101b6513a05d2f90cfe60e8b1d9e1b46a63c4fa",
                        "position_id": "0x6895b8156480b0ca4cfa1a7a791827380aad5704abefef3874f444b8175da736",
                        "reward_amount": "626072172",
                        "reward_decimals": 9,
                        "reward_symbol": "BLUE",
                        "reward_type": "e1b45a0e641b9955a20aa0ad1c1f4ad86aad8afb07296d4085e349a50e90bdca::blue::BLUE",
                        "sequence_number": "478486"
                        },
                        "bcsEncoding": "base64",
                        "bcs": "O1hXhrE68djqBnqzcQG2UToF0vkM/mDosdnhtGpjxPpolbgVZICwykz6Gnp5GCc4Cq1XBKvv7zh09ES4F12nNkxlMWI0NWEwZTY0MWI5OTU1YTIwYWEwYWQxYzFmNGFkODZhYWQ4YWZiMDcyOTZkNDA4NWUzNDlhNTBlOTBiZGNhOjpibHVlOjpCTFVFBEJMVUUJbBpRJQAAAAAWTQcAAAAAAAAAAAAAAAAA"
                    }
                    ],
                    "objectChanges": [
                    {
                        "type": "mutated",
                        "sender": "0xc8d919cef59bf118454313315950f8a10ddb49f799fcceac7479721891051e45",
                        "owner": {
                        "Shared": {
                            "initial_shared_version": 406731547
                        }
                        },
                        "objectType": "0x3492c874c1e3b3e2984e8c41b589e642d4d0a5d6459e5a9cfc2d52fd7c89c267::pool::Pool<0x2::sui::SUI, 0xdba34672e30cb065b1f93e3ab55318768fd6fef66c15942c9f7cb846e2f900e7::usdc::USDC>",
                        "objectId": "0x3b585786b13af1d8ea067ab37101b6513a05d2f90cfe60e8b1d9e1b46a63c4fa",
                        "version": "451149778",
                        "previousVersion": "451149720",
                        "digest": "HJmA6uoL9W39zGoR9Vt2oqjfW8uks2SYgmeKFsYZSDrc"
                    },
                    {
                        "type": "mutated",
                        "sender": "0xc8d919cef59bf118454313315950f8a10ddb49f799fcceac7479721891051e45",
                        "owner": {
                        "AddressOwner": "0xc8d919cef59bf118454313315950f8a10ddb49f799fcceac7479721891051e45"
                        },
                        "objectType": "0x3492c874c1e3b3e2984e8c41b589e642d4d0a5d6459e5a9cfc2d52fd7c89c267::position::Position",
                        "objectId": "0x6895b8156480b0ca4cfa1a7a791827380aad5704abefef3874f444b8175da736",
                        "version": "451149778",
                        "previousVersion": "450987990",
                        "digest": "BBhz5SrGioujQs4nPAZQKTXdzGLnp5R4pT8aCgUVrfTQ"
                    },
                    {
                        "type": "mutated",
                        "sender": "0xc8d919cef59bf118454313315950f8a10ddb49f799fcceac7479721891051e45",
                        "owner": {
                        "AddressOwner": "0xc8d919cef59bf118454313315950f8a10ddb49f799fcceac7479721891051e45"
                        },
                        "objectType": "0x2::coin::Coin<0x2::sui::SUI>",
                        "objectId": "0xb077fcadb92d9b58ebcb94ae69f8eaa8d6ab2e1be6396cff0ef873944f6f1d8c",
                        "version": "451149778",
                        "previousVersion": "450989448",
                        "digest": "A2aP5ymw6R1pCEn7QFM6RdfVkZpEEscx4yKcLY6tJe8E"
                    },
                    {
                        "type": "mutated",
                        "sender": "0xc8d919cef59bf118454313315950f8a10ddb49f799fcceac7479721891051e45",
                        "owner": {
                        "ObjectOwner": "0x3b585786b13af1d8ea067ab37101b6513a05d2f90cfe60e8b1d9e1b46a63c4fa"
                        },
                        "objectType": "0x2::dynamic_field::Field<0x1::string::String, 0x2::balance::Balance<0x2::sui::SUI>>",
                        "objectId": "0xb59155bc6b699a8856772264fb9d309dea9a03b7e10d1208bb0c62174b53576d",
                        "version": "451149778",
                        "previousVersion": "451149315",
                        "digest": "BSrwMHK7UDTcAPyE9LPWKaya22znvKeK8tfYZBV6Jrid"
                    },
                    {
                        "type": "mutated",
                        "sender": "0xc8d919cef59bf118454313315950f8a10ddb49f799fcceac7479721891051e45",
                        "owner": {
                        "ObjectOwner": "0x3b585786b13af1d8ea067ab37101b6513a05d2f90cfe60e8b1d9e1b46a63c4fa"
                        },
                        "objectType": "0x2::dynamic_field::Field<0x1::string::String, 0x2::balance::Balance<0xe1b45a0e641b9955a20aa0ad1c1f4ad86aad8afb07296d4085e349a50e90bdca::blue::BLUE>>",
                        "objectId": "0xd935d479faf4d6f983555edf678d2e23dee1e109e48749a7ec2d999a7906409c",
                        "version": "451149778",
                        "previousVersion": "451149315",
                        "digest": "7iJAkhw2H6KXQwE639zerZYoADwKCFHqyiYCQ9P2K5Ex"
                    },
                    {
                        "type": "created",
                        "sender": "0xc8d919cef59bf118454313315950f8a10ddb49f799fcceac7479721891051e45",
                        "owner": {
                        "AddressOwner": "0xc8d919cef59bf118454313315950f8a10ddb49f799fcceac7479721891051e45"
                        },
                        "objectType": "0x2::coin::Coin<0x2::sui::SUI>",
                        "objectId": "0x2cb32bbbc5e5ce81030959473581b8515c5efaada4bc1cb717a5133a0d25f3b4",
                        "version": "451149778",
                        "digest": "skUVMRwhBbag4SpkitLHPga7rgBA3NGRt4NSsor5Pqx"
                    },
                    {
                        "type": "created",
                        "sender": "0xc8d919cef59bf118454313315950f8a10ddb49f799fcceac7479721891051e45",
                        "owner": {
                        "AddressOwner": "0xc8d919cef59bf118454313315950f8a10ddb49f799fcceac7479721891051e45"
                        },
                        "objectType": "0x2::coin::Coin<0xdba34672e30cb065b1f93e3ab55318768fd6fef66c15942c9f7cb846e2f900e7::usdc::USDC>",
                        "objectId": "0x5bcbddaedafe1d8b95e425eed2886c10a0c9d607d4a001b6c5d18001df0f881f",
                        "version": "451149778",
                        "digest": "5o6h41hVJ98wWPrddCertLeeiiHVKheeiZv7s2As3RFQ"
                    },
                    {
                        "type": "created",
                        "sender": "0xc8d919cef59bf118454313315950f8a10ddb49f799fcceac7479721891051e45",
                        "owner": {
                        "AddressOwner": "0xc8d919cef59bf118454313315950f8a10ddb49f799fcceac7479721891051e45"
                        },
                        "objectType": "0x2::coin::Coin<0xe1b45a0e641b9955a20aa0ad1c1f4ad86aad8afb07296d4085e349a50e90bdca::blue::BLUE>",
                        "objectId": "0xafeee074cc41b71ceacb5f6333439057ead7461021ed8b3e54aa07ffebc588a1",
                        "version": "451149778",
                        "digest": "8Br2d9GkcZ8zXUiY2QnCrJhZMKvJP7G34q2AY8EBfuM4"
                    },
                    {
                        "type": "created",
                        "sender": "0xc8d919cef59bf118454313315950f8a10ddb49f799fcceac7479721891051e45",
                        "owner": {
                        "AddressOwner": "0xc8d919cef59bf118454313315950f8a10ddb49f799fcceac7479721891051e45"
                        },
                        "objectType": "0x2::coin::Coin<0x2::sui::SUI>",
                        "objectId": "0xf50f1117e8fedb88441bc63d927d46c5afc85acbe771bcb44b045076b7f07cf9",
                        "version": "451149778",
                        "digest": "98YGNK4D436bLkW9N34afqyj6U5dvV7mJSZFHahu9qJ6"
                    }
                    ],
                    "balanceChanges": [
                    {
                        "owner": {
                        "AddressOwner": "0xc8d919cef59bf118454313315950f8a10ddb49f799fcceac7479721891051e45"
                        },
                        "coinType": "0x2::sui::SUI",
                        "amount": "131178313"
                    },
                    {
                        "owner": {
                        "AddressOwner": "0xc8d919cef59bf118454313315950f8a10ddb49f799fcceac7479721891051e45"
                        },
                        "coinType": "0xdba34672e30cb065b1f93e3ab55318768fd6fef66c15942c9f7cb846e2f900e7::usdc::USDC",
                        "amount": "427399"
                    },
                    {
                        "owner": {
                        "AddressOwner": "0xc8d919cef59bf118454313315950f8a10ddb49f799fcceac7479721891051e45"
                        },
                        "coinType": "0xe1b45a0e641b9955a20aa0ad1c1f4ad86aad8afb07296d4085e349a50e90bdca::blue::BLUE",
                        "amount": "626072172"
                    }
                    ],
                    "timestampMs": "1734500336136",
                    "checkpoint": "91680955"
                },
                "id": 1
            }
        "#;
        let json_response =
            serde_json::from_str::<JsonRpcResponse<SuiTransactionBlockResponse>>(json_str);
        println!("json_response: {:#?}", json_response);
    }

    #[tokio::test]
    async fn test_tx_sign() -> Result<(), anyhow::Error> {
        // mod utils;
        // use crate::utils::request_tokens_from_faucet;
        use anyhow::anyhow;
        use fastcrypto::encoding::Encoding;
        use fastcrypto::hash::HashFunction;
        use fastcrypto::{
            ed25519::Ed25519KeyPair,
            encoding::Base64,
            secp256k1::Secp256k1KeyPair,
            secp256r1::Secp256r1KeyPair,
            traits::{EncodeDecodeBase64, KeyPair},
        };
        use rand::{rngs::StdRng, SeedableRng};
        use shared_crypto::intent::{Intent, IntentMessage};
        use sui_sdk::{
            rpc_types::SuiTransactionBlockResponseOptions,
            types::{
                programmable_transaction_builder::ProgrammableTransactionBuilder,
                transaction::TransactionData,
            },
            SuiClientBuilder,
        };
        use sui_types::crypto::Signer;
        use sui_types::crypto::SuiSignature;
        use sui_types::crypto::ToFromBytes;
        use sui_types::signature::GenericSignature;
        use sui_types::{
            base_types::SuiAddress,
            crypto::{get_key_pair_from_rng, SuiKeyPair},
        };
        // set up sui client for the desired network.
        let sui_client = SuiClientBuilder::default().build_testnet().await?;

        // deterministically generate a keypair, testing only, do not use for mainnet,
        // use the next section to randomly generate a keypair instead.
        let skp_determ_0 =
            SuiKeyPair::Ed25519(Ed25519KeyPair::generate(&mut StdRng::from_seed([0; 32])));
        let _skp_determ_1 =
            SuiKeyPair::Secp256k1(Secp256k1KeyPair::generate(&mut StdRng::from_seed([0; 32])));
        let _skp_determ_2 =
            SuiKeyPair::Secp256r1(Secp256r1KeyPair::generate(&mut StdRng::from_seed([0; 32])));

        // randomly generate a keypair.
        let _skp_rand_0 = SuiKeyPair::Ed25519(get_key_pair_from_rng(&mut rand::rngs::OsRng).1);
        let _skp_rand_1 = SuiKeyPair::Secp256k1(get_key_pair_from_rng(&mut rand::rngs::OsRng).1);
        let _skp_rand_2 = SuiKeyPair::Secp256r1(get_key_pair_from_rng(&mut rand::rngs::OsRng).1);

        // import a keypair from a base64 encoded 32-byte `private key` assuming scheme is Ed25519.
        let _skp_import_no_flag_0 = SuiKeyPair::Ed25519(Ed25519KeyPair::from_bytes(
            &Base64::decode("1GPhHHkVlF6GrCty2IuBkM+tj/e0jn64ksJ1pc8KPoI=")
                .map_err(|_| anyhow!("Invalid base64"))?,
        )?);
        let _skp_import_no_flag_1 = SuiKeyPair::Ed25519(Ed25519KeyPair::from_bytes(
            &Base64::decode("1GPhHHkVlF6GrCty2IuBkM+tj/e0jn64ksJ1pc8KPoI=")
                .map_err(|_| anyhow!("Invalid base64"))?,
        )?);
        let _skp_import_no_flag_2 = SuiKeyPair::Ed25519(Ed25519KeyPair::from_bytes(
            &Base64::decode("1GPhHHkVlF6GrCty2IuBkM+tj/e0jn64ksJ1pc8KPoI=")
                .map_err(|_| anyhow!("Invalid base64"))?,
        )?);

        // import a keypair from a base64 encoded 33-byte `flag || private key`.
        // The signature scheme is determined by the flag.
        let _skp_import_with_flag_0 =
            SuiKeyPair::decode_base64("ANRj4Rx5FZRehqwrctiLgZDPrY/3tI5+uJLCdaXPCj6C")
                .map_err(|_| anyhow!("Invalid base64"))?;
        let _skp_import_with_flag_1 =
            SuiKeyPair::decode_base64("AdRj4Rx5FZRehqwrctiLgZDPrY/3tI5+uJLCdaXPCj6C")
                .map_err(|_| anyhow!("Invalid base64"))?;
        let _skp_import_with_flag_2 =
            SuiKeyPair::decode_base64("AtRj4Rx5FZRehqwrctiLgZDPrY/3tI5+uJLCdaXPCj6C")
                .map_err(|_| anyhow!("Invalid base64"))?;

        // import a keypair from a Bech32 encoded 33-byte `flag || private key`.
        // this is the format of a private key exported from Sui Wallet or sui.keystore.
        let _skp_import_with_flag_0 = SuiKeyPair::decode(
            "suiprivkey1qzdlfxn2qa2lj5uprl8pyhexs02sg2wrhdy7qaq50cqgnffw4c2477kg9h3",
        )
        .map_err(|_| anyhow!("Invalid Bech32"))?;
        let _skp_import_with_flag_1 = SuiKeyPair::decode(
            "suiprivkey1qqesr6xhua2dkt840v9yefely578q5ad90znnpmhhgpekfvwtxke6ef2xyg",
        )
        .map_err(|_| anyhow!("Invalid Bech32"))?;
        let _skp_import_with_flag_2 = SuiKeyPair::decode(
            "suiprivkey1qprzkcs823gcrk7n4hy8pzhntdxakpqk32qwjg9f2wyc3myj78egvtw3ecr",
        )
        .map_err(|_| anyhow!("Invalid Bech32"))?;

        // replace `skp_determ_0` with the variable names above
        let pk = skp_determ_0.public();
        let sender = SuiAddress::from(&pk);
        println!("Sender: {:?}", sender);

        // make sure the sender has a gas coin as an example.
        // request_tokens_from_faucet(sender, &sui_client).await?;
        let gas_coin = sui_client
            .coin_read_api()
            .get_coins(sender, None, None, None)
            .await?
            .data
            .into_iter()
            .next()
            .ok_or(anyhow!("No coins found for sender"))?;

        // construct an example programmable transaction.
        let pt = {
            let mut builder = ProgrammableTransactionBuilder::new();
            builder.pay_sui(vec![sender], vec![1])?;
            builder.finish()
        };

        let gas_budget = 5_000_000;
        let gas_price = sui_client.read_api().get_reference_gas_price().await?;

        // create the transaction data that will be sent to the network.
        let tx_data = TransactionData::new_programmable(
            sender,
            vec![gas_coin.object_ref()],
            pt,
            gas_budget,
            gas_price,
        );

        // derive the digest that the keypair should sign on,
        // i.e. the blake2b hash of `intent || tx_data`.
        let intent_msg = IntentMessage::new(Intent::sui_transaction(), tx_data);
        let raw_tx = bcs::to_bytes(&intent_msg).expect("bcs should not fail");
        let mut hasher = sui_types::crypto::DefaultHash::default();
        hasher.update(raw_tx.clone());
        let digest = hasher.finalize().digest;

        // use SuiKeyPair to sign the digest.
        let sui_sig = skp_determ_0.sign(&digest);

        // if you would like to verify the signature locally before submission, use this function.
        // if it fails to verify locally, the transaction will fail to execute in Sui.
        let res = sui_sig.verify_secure(
            &intent_msg,
            sender,
            sui_types::crypto::SignatureScheme::ED25519,
        );
        assert!(res.is_ok());

        // execute the transaction.
        let transaction_response = sui_client
            .quorum_driver_api()
            .execute_transaction_block(
                sui_types::transaction::Transaction::from_generic_sig_data(
                    intent_msg.value,
                    vec![GenericSignature::Signature(sui_sig)],
                ),
                SuiTransactionBlockResponseOptions::default(),
                None,
            )
            .await?;

        println!(
            "Transaction executed. Transaction digest: {}",
            transaction_response.digest.base58_encode()
        );
        println!("{transaction_response}");
        Ok(())
    }
}
