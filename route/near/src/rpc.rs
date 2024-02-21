use ic_cdk::api::management_canister::http_request::*;
use near_client::{
    near_types::{
        hash::CryptoHash,
        signature::{ED25519PublicKey, PublicKey, Signature},
        *,
    },
    types::*,
};
use near_primitives::views::{
    validator_stake_view::ValidatorStakeView as ValidatorStakeViewN, BlockView as BlockViewN,
    LightClientBlockView as LightClientBlockViewN,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RpcError {
    #[error("IO error occured while calling {0} onto {1} due to {2}.")]
    Io(&'static str, String, String),
    #[error("Decoding response of {0} from {1} failed due to {2}.")]
    Decode(&'static str, String, String),
    #[error("Received an error of endpoint {0} from {1}: {2}.")]
    Endpoint(&'static str, String, String),
}

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
enum RpcEndpoint {
    Block(BlockHeight),
    NextLightBlock(CryptoHash),
}

impl RpcEndpoint {
    /// Get a static string which represents this method name
    fn method(&self) -> &'static str {
        match self {
            RpcEndpoint::Block(_) => "block",
            RpcEndpoint::NextLightBlock(_) => "next_light_client_block",
        }
    }

    pub fn params(&self) -> impl Into<serde_json::Value> {
        match self {
            RpcEndpoint::Block(height) => {
                serde_json::json!([height])
            }
            RpcEndpoint::NextLightBlock(hash) => {
                serde_json::json!([format!("{}", hash)])
            }
        }
    }
}

#[derive(Serialize, Debug)]
struct Payload {
    pub jsonrpc: &'static str,
    pub id: i64,
    pub method: &'static str,
    pub params: serde_json::Value,
}

#[derive(Deserialize, Debug)]
struct Reply<R> {
    #[allow(dead_code)]
    pub jsonrpc: String,
    #[allow(dead_code)]
    pub id: i64,
    pub error: Option<ErrorMsg>,
    pub result: Option<R>,
}

#[derive(Deserialize, Debug)]
struct ErrorMsg {
    #[allow(dead_code)]
    code: i64,
    message: String,
}

async fn make_rpc<R>(url: impl AsRef<str>, endpoint: RpcEndpoint) -> Result<R, RpcError>
where
    R: DeserializeOwned,
{
    let payload = Payload {
        jsonrpc: "2.0",
        id: 1,
        method: endpoint.method(),
        params: endpoint.params().into(),
    };
    let body = serde_json::to_vec(&payload).unwrap();
    let url = url.as_ref();
    let args = CanisterHttpRequestArgument {
        url: url.to_string(),
        method: HttpMethod::POST,
        body: Some(body),
        max_response_bytes: None,
        transform: None,
        headers: vec![
            HttpHeader {
                name: "Content-Type".to_string(),
                value: "application/json".to_string(),
            },
            HttpHeader {
                name: "User-Agent".to_string(),
                value: format!("ic_tendermint_lightclient/{}", env!("CARGO_PKG_VERSION")),
            },
        ],
    };
    // TODO max cycle
    let (response,) = http_request(args, 100000)
        .await
        .map_err(|(_, e)| RpcError::Io(endpoint.method(), url.to_string(), e))?;
    let reply: Reply<R> = serde_json::from_slice(response.body.as_slice())
        .map_err(|e| RpcError::Decode(endpoint.method(), url.to_string(), e.to_string()))?;
    if reply.error.is_some() {
        return Err(RpcError::Endpoint(
            endpoint.method(),
            url.to_string(),
            reply.error.map(|e| e.message).unwrap(),
        ));
    }
    return Ok(reply.result.unwrap());
}

async fn fetch_block(url: &str, height: BlockHeight) -> Result<BlockViewN, RpcError> {
    let block = make_rpc::<BlockViewN>(url, RpcEndpoint::Block(height)).await?;
    Ok(block)
}

async fn fetch_next_light_block(
    url: &str,
    hash: CryptoHash,
) -> Result<LightClientBlockViewN, RpcError> {
    let light_block =
        make_rpc::<LightClientBlockViewN>(url, RpcEndpoint::NextLightBlock(hash)).await?;
    Ok(light_block)
}

pub(crate) async fn fetch_header(url: &str, height: BlockHeight) -> Result<Header, RpcError> {
    let block = fetch_block(url, height).await?;
    let light_block = fetch_next_light_block(url, CryptoHash(block.header.hash.0)).await?;
    Ok(Header {
        light_client_block: LightClientBlock {
            prev_block_hash: CryptoHash(light_block.prev_block_hash.0),
            next_block_inner_hash: CryptoHash(light_block.next_block_inner_hash.0),
            inner_lite: BlockHeaderInnerLite {
                height: light_block.inner_lite.height,
                epoch_id: EpochId(CryptoHash(light_block.inner_lite.epoch_id.0)),
                next_epoch_id: EpochId(CryptoHash(light_block.inner_lite.next_epoch_id.0)),
                prev_state_root: CryptoHash(light_block.inner_lite.prev_state_root.0),
                outcome_root: CryptoHash(light_block.inner_lite.outcome_root.0),
                timestamp: light_block.inner_lite.timestamp_nanosec,
                next_bp_hash: CryptoHash(light_block.inner_lite.next_bp_hash.0),
                block_merkle_root: CryptoHash(light_block.inner_lite.block_merkle_root.0),
            },
            inner_rest_hash: CryptoHash(light_block.inner_rest_hash.0),
            next_bps: Some(
                light_block
                    .next_bps
                    .as_ref()
                    .unwrap()
                    .iter()
                    .map(|f| match f {
                        ValidatorStakeViewN::V1(v) => {
                            ValidatorStakeView::V1(ValidatorStakeViewV1 {
                                account_id: v.account_id.to_string(),
                                public_key: match &v.public_key {
                                    near_crypto::PublicKey::ED25519(data) => {
                                        PublicKey::ED25519(ED25519PublicKey(data.clone().0))
                                    }
                                    _ => panic!("Unsupported publickey in next block producers."),
                                },
                                stake: v.stake,
                            })
                        }
                    })
                    .collect(),
            ),
            approvals_after_next: light_block
                .approvals_after_next
                .iter()
                .map(|f| {
                    f.as_ref().map(|s| match **s {
                        near_crypto::Signature::ED25519(data) => {
                            Signature::ED25519(data.to_bytes().to_vec())
                        }
                        _ => panic!("Unsupported signature in approvals after next."),
                    })
                })
                .collect(),
        },
        prev_state_root_of_chunks: block
            .chunks
            .iter()
            .map(|header| CryptoHash(header.prev_state_root.0))
            .collect(),
    })
}
