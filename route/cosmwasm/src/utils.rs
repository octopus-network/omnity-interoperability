use crate::*;
use base64::engine::general_purpose;
use candid::Nat;
use ic_cdk::api::management_canister::http_request::{HttpResponse, TransformContext};
use serde::{Deserialize, Serialize};

/// JSON-RPC ID: request-specific identifier
#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq, Ord, PartialOrd)]
#[serde(untagged)]
pub enum Id {
    /// Numerical JSON ID
    Num(i64),
    /// String JSON ID
    Str(String),
    /// null JSON ID
    None,
}

impl Id {
    /// Create a JSON-RPC ID containing a UUID v4 (i.e. random)
    pub fn uuid_v4() -> Self {
        Self::Str(uuid_str())
    }
}

pub fn uuid_str() -> String {
    // let bytes: [u8; 16] = rand::thread_rng().gen();
    // todo use icp native random number generator
    let bytes: [u8; 16] = [1; 16];
    let uuid = uuid::Builder::from_random_bytes(bytes).into_uuid();
    uuid.to_string()
}

pub fn bytes_to_base64(bytes: &[u8]) -> String {
    base64::Engine::encode(&general_purpose::STANDARD, bytes)
}

pub fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

pub async fn http_request_with_status_check(
    mut request: CanisterHttpRequestArgument,
) -> Result<HttpResponse> {

    request.transform = Some(TransformContext::from_name(
        "cleanup_response".to_owned(),
        vec![],
    ));

    // let cycles = http_request_required_cycles(&request, 13);
    let response = http_request(request.clone(), 50_000_000_000)
        .await
        .map_err(|(code, message)| {
            RouteError::HttpOutCallError(format!("{:?}", code).to_string(), message, format!("{:?}", request))
        })?
        .0;
    log::info!(
        "Http status code: {:?}, url: {}, response body: {}",
        response.status,
        request.url,
        String::from_utf8_lossy(&response.body)
    );
    if response.status != Nat::from(200u64) {
        return Err(RouteError::HttpStatusError(
            response.status.clone(),
            request.url.clone(),
            String::from_utf8_lossy(&response.body).to_string(),
        ));
    }
    Ok(response)
}

pub fn sha256(input: Vec<u8>) -> [u8; 32] {
    use sha2::Digest;
    let mut hasher = sha2::Sha256::new();
    hasher.update(input);
    hasher.finalize().into()
}

/// Calculates the baseline cost of sending a JSON-RPC request using HTTP outcalls.
pub fn http_request_required_cycles(
    arg: &CanisterHttpRequestArgument,
    nodes_in_subnet: u32,
) -> u128 {
    const HTTP_OUTCALL_REQUEST_BASE_COST: u128 = 3_000_000;
    const HTTP_OUTCALL_REQUEST_PER_NODE_COST: u128 = 60_000;
    const HTTP_OUTCALL_REQUEST_COST_PER_BYTE: u128 = 400;
    const HTTP_OUTCALL_RESPONSE_COST_PER_BYTE: u128 = 800;
    let max_response_bytes = match arg.max_response_bytes {
        Some(ref n) => *n as u128,
        None => 2 * 1024 * 1024, // default 2MiB
    };
    let nodes_in_subnet = nodes_in_subnet as u128;

    // The coefficients can be found in [this page](https://internetcomputer.org/docs/current/developer-docs/production/computation-and-storage-costs).
    // 12 is "http_request".len().

    let request_bytes = candid::utils::encode_args((arg,))
        .expect("Failed to encode arguments.")
        .len() as u128
        + 12;

    (HTTP_OUTCALL_REQUEST_BASE_COST
        + HTTP_OUTCALL_REQUEST_PER_NODE_COST * nodes_in_subnet
        + HTTP_OUTCALL_REQUEST_COST_PER_BYTE * request_bytes
        + HTTP_OUTCALL_RESPONSE_COST_PER_BYTE * max_response_bytes)
        * nodes_in_subnet * 3
}

#[test]
pub fn test_show_address() {
    #[derive(Serialize, Deserialize)]
    struct Account {
        pub address: String,
    };

    let account = Account {
        address: "osmo1cyyzpxplxdzkeea7kwsydadg87357qnahakaks".to_string(),
    };
    let bytes = serde_json::to_string(&account).unwrap().into_bytes();
    let hex = bytes_to_hex(&bytes);
    dbg!(&hex);
}