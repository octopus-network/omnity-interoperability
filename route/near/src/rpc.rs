use ic_cdk::api::management_canister::http_request::*;
use near_primitives::{types::BlockHeight, views::BlockView};
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
}

impl RpcEndpoint {
    /// Get a static string which represents this method name
    fn method(&self) -> &'static str {
        match self {
            RpcEndpoint::Block(_) => "block",
        }
    }

    pub fn params(&self) -> impl Into<serde_json::Value> {
        match self {
            RpcEndpoint::Block(height) => {
                serde_json::json!([height])
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

pub(crate) async fn fetch_block(url: &str, height: BlockHeight) -> Result<BlockView, RpcError> {
    let block = make_rpc::<BlockView>(url, RpcEndpoint::Block(height)).await?;
    Ok(block)
}
