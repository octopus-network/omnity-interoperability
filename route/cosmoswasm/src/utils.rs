use crate::*;
use base64::engine::general_purpose;
use candid::Nat;
use ic_cdk::api::management_canister::http_request::HttpResponse;
use rand::Rng;
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
    let bytes: [u8; 16] = [0; 16];
    let uuid = uuid::Builder::from_random_bytes(bytes).into_uuid();
    uuid.to_string()
}

pub fn bytes_to_base64(bytes: &[u8]) -> String {
    base64::Engine::encode(&general_purpose::STANDARD, bytes)
}

pub async fn http_request_with_status_check(
    request: CanisterHttpRequestArgument,
) ->Result<HttpResponse> {
    let response =  http_request(request.clone(), 100_000_000_000)
            .await
            .map_err(|(code, message)| {
                RouteError::HttpOutCallError(format!("{:?}", code).to_string(), message)
            })?.0;
    if response.status != Nat::from(200u64) {
        return Err(RouteError::HttpStatusError(response.status.clone(), request.url.clone(), String::from_utf8_lossy(&response.body).to_string()));
    }
    Ok(response)
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
    let hex = hex::encode(bytes);
    dbg!(&hex);
}

#[test]
pub fn test_show_string() {
    let raw_vec: Vec<u8> = vec![123, 34, 106, 115, 111, 110, 114, 112, 99, 34, 58, 34, 50, 46, 48, 34, 44, 34, 105, 100, 34, 58, 34, 48, 48, 48, 48, 48, 48, 48, 48, 45, 48, 48, 48, 48, 45, 52, 48, 48, 48, 45, 56, 48, 48, 48, 45, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 34, 44, 34, 114, 101, 115, 117, 108, 116, 34, 58, 123, 34, 99, 104, 101, 99, 107, 95, 116, 120, 34, 58, 123, 34, 99, 111, 100, 101, 34, 58, 52, 44, 34, 100, 97, 116, 97, 34, 58, 110, 117, 108, 108, 44, 34, 108, 111, 103, 34, 58, 34, 115, 105, 103, 110, 97, 116, 117, 114, 101, 32, 118, 101, 114, 105, 102, 105, 99, 97, 116, 105, 111, 110, 32, 102, 97, 105, 108, 101, 100, 59, 32, 112, 108, 101, 97, 115, 101, 32, 118, 101, 114, 105, 102, 121, 32, 97, 99, 99, 111, 117, 110, 116, 32, 110, 117, 109, 98, 101, 114, 32, 40, 57, 55, 52, 48, 54, 41, 32, 97, 110, 100, 32, 99, 104, 97, 105, 110, 45, 105, 100, 32, 40, 111, 115, 109, 111, 45, 116, 101, 115, 116, 45, 53, 41, 58, 32, 117, 110, 97, 117, 116, 104, 111, 114, 105, 122, 101, 100, 34, 44, 34, 105, 110, 102, 111, 34, 58, 34, 34, 44, 34, 103, 97, 115, 95, 119, 97, 110, 116, 101, 100, 34, 58, 34, 49, 48, 48, 48, 48, 48, 34, 44, 34, 103, 97, 115, 95, 117, 115, 101, 100, 34, 58, 34, 53, 55, 55, 54, 50, 34, 44, 34, 101, 118, 101, 110, 116, 115, 34, 58, 91, 93, 44, 34, 99, 111, 100, 101, 115, 112, 97, 99, 101, 34, 58, 34, 115, 100, 107, 34, 44, 34, 115, 101, 110, 100, 101, 114, 34, 58, 34, 34, 44, 34, 112, 114, 105, 111, 114, 105, 116, 121, 34, 58, 34, 48, 34, 44, 34, 109, 101, 109, 112, 111, 111, 108, 69, 114, 114, 111, 114, 34, 58, 34, 34, 125, 44, 34, 100, 101, 108, 105, 118, 101, 114, 95, 116, 120, 34, 58, 123, 34, 99, 111, 100, 101, 34, 58, 48, 44, 34, 100, 97, 116, 97, 34, 58, 110, 117, 108, 108, 44, 34, 108, 111, 103, 34, 58, 34, 34, 44, 34, 105, 110, 102, 111, 34, 58, 34, 34, 44, 34, 103, 97, 115, 95, 119, 97, 110, 116, 101, 100, 34, 58, 34, 48, 34, 44, 34, 103, 97, 115, 95, 117, 115, 101, 100, 34, 58, 34, 48, 34, 44, 34, 101, 118, 101, 110, 116, 115, 34, 58, 91, 93, 44, 34, 99, 111, 100, 101, 115, 112, 97, 99, 101, 34, 58, 34, 34, 125, 44, 34, 104, 97, 115, 104, 34, 58, 34, 67, 52, 51, 68, 65, 51, 65, 55, 68, 68, 50, 65, 50, 68, 49, 67, 51, 56, 66, 66, 54, 51, 67, 55, 48, 52, 65, 55, 57, 56, 54, 52, 56, 55, 57, 50, 53, 54, 66, 54, 65, 69, 70, 57, 57, 65, 57, 56, 53, 51, 57, 50, 54, 54, 52, 66, 57, 48, 52, 53, 69, 51, 67, 53, 34, 44, 34, 104, 101, 105, 103, 104, 116, 34, 58, 34, 48, 34, 125, 125];
    let s = String::from_utf8(raw_vec).unwrap();
    dbg!(&s);
}

#[test]
pub fn ttt() {
    let base_64 = bytes_to_base64(&vec![2,
        223,
        31,
        241,
        136,
        223,
        252,
        34,
        203,
        195,
        185,
        234,
        74,
        34,
        162,
        79,
        226,
        251,
        245,
        51,
        94,
        235,
        169,
        161,
        110,
        161,
        139,
        207,
        252,
        163,
        144,
        202,
        122]);
    dbg!(&base_64);
}