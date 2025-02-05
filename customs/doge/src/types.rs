use std::borrow::Cow;

use bitcoin::consensus::{Decodable, Encodable, ReadExt};
use candid::{CandidType, Deserialize, Nat};
use ic_cdk::api::management_canister::http_request::{
    http_request, CanisterHttpRequestArgument, HttpResponse, TransformContext,
};
use ic_stable_structures::{storable::Bound, Storable};
use omnity_types::ic_log::{ERROR, INFO, WARNING};
use serde::Serialize;
// use std::str::FromStr;
use hex::prelude::*;

use ic_canister_log::log;
use omnity_types::{Token, TokenId};

use crate::{
    doge::{rpc::get_raw_transaction_by_rpc, transaction::TransactionJsonResult},
    errors::CustomsError,
};
use bitcoin::hashes::{sha256d, Hash};
use serde_bytes::ByteArray;

pub type ECDSAPublicKey = ic_cdk::api::management_canister::ecdsa::EcdsaPublicKeyResponse;

#[derive(CandidType, PartialEq, Eq, Clone, Default, Deserialize, Serialize, PartialOrd, Ord)]
pub struct Txid(pub ByteArray<32>);

impl std::str::FromStr for Txid {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let h = sha256d::Hash::from_str(s).map_err(|_| "invalid Txid")?;
        Ok(Self(h.to_byte_array().into()))
    }
}

impl Storable for Txid {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(bincode::serialize(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        bincode::deserialize(bytes.as_ref()).unwrap()
    }

    const BOUND: Bound = Bound::Unbounded;
}

impl std::fmt::Debug for Txid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        sha256d::Hash::from_bytes_ref(&self.0).fmt(f)
    }
}

impl std::fmt::Display for Txid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        sha256d::Hash::from_bytes_ref(&self.0).fmt(f)
    }
}

impl From<ByteArray<32>> for Txid {
    fn from(val: ByteArray<32>) -> Self {
        Self(val)
    }
}

impl From<[u8; 32]> for Txid {
    fn from(val: [u8; 32]) -> Self {
        Self(val.into())
    }
}

impl From<crate::doge::transaction::Txid> for Txid {
    fn from(txid: crate::doge::transaction::Txid) -> Self {
        Self((*txid).into())
    }
}

impl From<Txid> for crate::doge::transaction::Txid {
    fn from(txid: Txid) -> Self {
        Self::from_byte_array(*txid.0)
    }
}

#[derive(Serialize, CandidType, Deserialize, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Destination {
    pub target_chain_id: String,
    pub receiver: String,
    pub token: Option<String>,
}

impl Destination {
    pub fn new(target_chain_id: String, receiver: String, token: Option<String>) -> Self {
        Destination {
            target_chain_id,
            receiver,
            token,
        }
    }

    pub fn change_address() -> Destination {
        Destination::new(String::default(), String::default(), Option::None)
    }

    pub fn fee_payment_address() -> Destination {
        Destination::new(
            "fee_payment".to_string(),
            "fee_payment".to_string(),
            Option::None,
        )
    }

    #[inline]
    pub fn effective_token(&self) -> String {
        self.token.clone().unwrap_or(String::new())
    }

    pub fn derivation_path(&self) -> Vec<Vec<u8>> {
        const SCHEMA_V1: u8 = 1;
        vec![
            vec![SCHEMA_V1],
            self.target_chain_id.as_bytes().to_vec(),
            self.receiver.as_bytes().to_vec(),
            self.effective_token().as_bytes().to_vec(),
        ]
    }
}

#[derive(CandidType, Clone, Debug, Serialize, Deserialize)]
pub struct TokenResp {
    pub token_id: TokenId,
    pub symbol: String,
    pub decimals: u8,
    pub icon: Option<String>,
}

impl From<Token> for TokenResp {
    fn from(value: Token) -> Self {
        TokenResp {
            token_id: value.token_id,
            symbol: value.symbol,
            decimals: value.decimals,
            icon: value.icon,
        }
    }
}

#[derive(candid::CandidType, Clone, Debug, PartialEq, Eq, Deserialize)]
pub enum ReleaseTokenStatus {
    /// The custom has no data for this request.
    /// The request id is either invalid or too old.
    Unknown,
    /// The request is in the batch queue.
    Pending,
    /// Waiting for a signature on a transaction satisfy this request.
    Signing,
    /// Sending the transaction satisfying this request.
    Sending(String),
    /// Awaiting for confirmations on the transaction satisfying this request.
    Submitted(String),
    /// Confirmed a transaction satisfying this request.
    Confirmed(String),
}

#[derive(CandidType, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GenTicketStatus {
    /// The custom has no data for this request.
    /// The request is either invalid or too old.
    Unknown,
    /// The request is in the queue.
    Pending(LockTicketRequest),
    Confirmed(LockTicketRequest),
    Finalized(LockTicketRequest),
}

#[derive(candid::CandidType, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LockTicketRequest {
    pub target_chain_id: String,
    pub receiver: String,
    pub token_id: TokenId,
    pub amount: String,
    pub txid: Txid,
    pub received_at: u64,
    pub transaction_hex: String,
}

impl Storable for LockTicketRequest {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(bincode::serialize(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        bincode::deserialize(bytes.as_ref()).unwrap()
    }

    const BOUND: Bound = Bound::Unbounded;
}

pub fn err_string(err: impl std::fmt::Display) -> String {
    err.to_string()
}

pub fn wrap_to_customs_error(err: impl std::fmt::Display) -> CustomsError {
    CustomsError::CustomError(err.to_string())
}

pub fn serialize_hex<T: Encodable>(v: &T) -> String {
    let mut buf = Vec::new();
    v.consensus_encode(&mut buf)
        .expect("serialize_hex: encode failed");
    buf.to_lower_hex_string()
}

pub fn deserialize_hex<T: Decodable>(hex: &str) -> Result<T, String> {
    let data = Vec::from_hex(hex).map_err(err_string)?;
    let mut reader = &data[..];
    let object = Decodable::consensus_decode_from_finite_reader(&mut reader).map_err(err_string)?;
    if reader.read_u8().is_ok() {
        Err("decode_hex: data not consumed entirely".to_string())
    } else {
        Ok(object)
    }
}

pub async fn http_request_with_retry(
    mut request: CanisterHttpRequestArgument,
) -> Result<HttpResponse, CustomsError> {
    request.transform = Some(TransformContext::from_name("transform".to_owned(), vec![]));

    for _ in 0..3 {
        let response = match http_request(request.clone(), 60_000_000_000).await {
            Ok((response,)) => response,
            Err(e) => {
                log!(
                    ERROR,
                    "http request error, request: {:?} \n, error {:?}",
                    request,
                    e
                );
                continue;
            }
        };

        log!(
            INFO,
            "httpoutcall request:{:?} response: {:?}",
            request,
            response
        );
        if response.status == Nat::from(200u64) {
            return Ok(response);
        } else {
            log!(WARNING, "http request error: {:?}", response);
        }
    }
    Err(CustomsError::HttpOutExceedRetryLimit)
}

#[derive(CandidType, Clone, Debug, Default, Deserialize, Serialize)]
pub struct Utxo {
    pub txid: Txid,
    pub vout: u32,
    pub value: u64,
}

impl From<Utxo> for crate::doge::transaction::TxIn {
    fn from(val: Utxo) -> Self {
        Self::with_outpoint(crate::doge::transaction::OutPoint {
            txid: val.txid.into(),
            vout: val.vout,
        })
    }
}

#[derive(CandidType, Clone, Debug, Default, Deserialize, Serialize)]
pub struct RpcConfig {
    pub url: String,
    pub api_key: Option<String>,
}

impl From<RpcConfig> for crate::doge::rpc::DogeRpc {
    fn from(val: RpcConfig) -> Self {
        Self {
            url: val.url,
            api_key: val.api_key,
        }
    }
}

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, Default)]
pub struct MultiRpcConfig {
    pub rpc_list: Vec<RpcConfig>,
    pub minimum_response_count: u32,
}

impl MultiRpcConfig {
    pub fn check_config_valid(&self) -> Result<(), CustomsError> {
        if self.minimum_response_count == 0 {
            return Err(CustomsError::CustomError(
                "minimum_response_count should be greater than 0".to_string(),
            ));
        }
        if self.rpc_list.len() < self.minimum_response_count as usize {
            return Err(CustomsError::CustomError(
                "rpc_list length should be greater than minimum_response_count".to_string(),
            ));
        }
        Ok(())
    }

    pub async fn get_raw_transaction_json_data(
        &self,
        txid: &str,
    ) -> Result<TransactionJsonResult, CustomsError> {
        self.check_config_valid()?;

        let mut fut = Vec::with_capacity(self.rpc_list.len());
        for rpc_config in self.rpc_list.iter() {
            // let doge_rpc = DogeRpc::from(rpc_url.clone());
            fut.push(get_raw_transaction_by_rpc(txid, rpc_config.clone()));
        }

        let res = futures::future::join_all(fut).await;

        self.valid_and_get_transaction(res)
    }

    fn valid_and_get_transaction(
        &self,
        res: Vec<Result<TransactionJsonResult, CustomsError>>,
    ) -> Result<TransactionJsonResult, CustomsError> {
        let success_res = res
            .iter()
            .filter_map(|r| r.as_ref().ok())
            .collect::<Vec<_>>();
        self.check_config_valid()?;
        if success_res.len() < self.minimum_response_count as usize {
            return Err(CustomsError::CustomError(format!(
                "Success res count({}) less than minimum_response_count: {}",
                success_res.len(),
                self.minimum_response_count
            )));
        }

        for i in 1..success_res.len() {
            if success_res[i] != success_res[i - 1] {
                return Err(CustomsError::CustomError(format!(
                    "Success res not equal: {:?} != {:?}",
                    success_res[i],
                    success_res[i - 1]
                )));
            }
        }

        Ok(success_res[0].clone())
    }
}
