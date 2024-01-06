use ic_cdk::api::management_canister::http_request::*;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use thiserror::Error;
use tm_verifier::types::*;

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
    Commit(Option<Height>),
    Genesis,
    Header(Option<Height>),
    Validators(Height, Option<u32>, Option<u8>),
}

impl RpcEndpoint {
    /// Get a static string which represents this method name
    fn method(&self) -> &'static str {
        match self {
            RpcEndpoint::Commit(_) => "commit",
            RpcEndpoint::Genesis => "genesis",
            RpcEndpoint::Header(_) => "header",
            RpcEndpoint::Validators(..) => "validators",
        }
    }

    pub fn params(&self) -> impl Into<serde_json::Value> {
        match self {
            RpcEndpoint::Commit(height) => {
                serde_json::json!({
                    "height": height
                })
            }
            RpcEndpoint::Genesis => serde_json::Value::Null,
            RpcEndpoint::Header(height) => {
                serde_json::json!({
                    "height": height
                })
            }
            RpcEndpoint::Validators(height, page, per_page) => {
                serde_json::json!({
                    "height": height,
                    "page": page.map(|x| x.to_string()),
                    "per_page": per_page.map(|x| x.to_string()),
                })
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
    let (response,) = http_request(args)
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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CommitResponse {
    pub signed_header: SignedHeader,
    pub canonical: bool,
}

pub(crate) async fn fetch_signed_header(
    url: &str,
    height: Option<Height>,
) -> Result<SignedHeader, RpcError> {
    let commit = make_rpc::<CommitResponse>(url, RpcEndpoint::Commit(height)).await?;
    Ok(commit.signed_header)
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ValidatorResponse {
    pub block_height: Height,
    pub validators: Vec<Validator>,
    pub total: String,
}

pub(crate) async fn fetch_validator_set(
    url: &str,
    height: Height,
    proposer: Option<ValidatorAddress>,
) -> Result<ValidatorSet, RpcError> {
    let mut validators = Vec::new();
    let mut page = 1;
    let per_page = 30;
    loop {
        let v = make_rpc::<ValidatorResponse>(
            url,
            RpcEndpoint::Validators(height, Some(page), Some(per_page)),
        )
        .await?;
        validators.extend(v.validators);
        let total = v
            .total
            .parse::<usize>()
            .map_err(|e| RpcError::Decode("validators", url.to_string(), e.to_string()))?;
        if validators.len() == total {
            break;
        }
        page += 1;
    }
    let validators = match proposer {
        Some(addr) => ValidatorSet::with_proposer(validators, addr)
            .map_err(|e| RpcError::Decode("validators", url.to_string(), e.to_string()))?,
        None => ValidatorSet::without_proposer(validators),
    };
    Ok(validators)
}

// TODO if we would like to make an RPC from canister, we'd better take care of the networking issues
pub(crate) async fn fetch_block(rpc: &str, height: Option<Height>) -> Result<LightBlock, RpcError> {
    let signed_header = fetch_signed_header(rpc, height).await?;
    let at = signed_header.header.height;
    let proposer = signed_header.header.proposer_address;
    let validator_set = fetch_validator_set(rpc, at, Some(proposer)).await?;
    let next_validator_set = fetch_validator_set(rpc, at.increment(), None).await?;
    let light_block = LightBlock::new(
        signed_header,
        validator_set,
        next_validator_set,
        PeerId::new([0u8; 20]),
    );
    Ok(light_block)
}
