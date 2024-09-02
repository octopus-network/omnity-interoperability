pub mod business;
pub mod cosmwasm;
pub mod error;
pub mod hub;
pub mod lifecycle;
pub mod memory;
pub mod service;
pub mod state;
pub mod utils;
pub mod guard;

pub use candid::Principal;
pub use error::Result;
pub use error::RouteError;
pub use serde::{Deserialize, Serialize};

pub use cosmrs::{
    cosmwasm::MsgExecuteContract,
    proto, tendermint,
    tx::{self, AccountNumber, Fee, Msg, Raw, SignDoc, SignerInfo},
    AccountId, Coin,
};
pub use ic_cdk::api::{
    call::RejectionCode,
    management_canister::http_request::{
        http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod,
    },
};
pub use num_traits::sign;
pub use omnity_types::Token;
pub use serde_bytes::ByteBuf;
pub use serde_json::json;
pub use state::*;

pub use crate::{
    utils::Id,
};
pub use cosmwasm::TxHash;

pub use omnity_types::TicketId;
pub use omnity_types::Topic;
pub use omnity_types::{self, ChainId, Seq, Ticket};

pub use ic_cdk::api::management_canister::{
    ecdsa::{
        ecdsa_public_key, sign_with_ecdsa, EcdsaKeyId, EcdsaPublicKeyArgument,
        EcdsaPublicKeyResponse, SignWithEcdsaArgument, SignWithEcdsaResponse,
    },
    http_request::HttpResponse,
};
pub use omnity_types::EcdsaKeyIds;
pub use serde_json::Value;
pub use utils::*;

pub use cosmwasm::client::{cw_chain_key_arg, query_cw_public_key, CosmWasmClient};
pub use memory::get_contract_id;

pub type DerivationPath = Vec<Vec<u8>>;
pub struct EcdsaChainKeyArg {
    pub derivation_path: DerivationPath,
    pub key_id: EcdsaKeyId,
}

pub mod const_args {
    pub const FETCH_HUB_TICKET_NAME: &str = "FETCH_HUB_TICKET";
    pub const FETCH_HUB_DIRECTIVE_NAME: &str = "FETCH_HUB_DIRECTIVE";
    pub const BATCH_QUERY_LIMIT: u64 = 20;
    pub const VERSION: &str = "0.1.4";
    pub const INTERVAL_QUERY_DIRECTIVE: u64 = 60;
    pub const INTERVAL_QUERY_TICKET: u64 = 60;
}