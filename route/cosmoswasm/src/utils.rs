use base64::engine::general_purpose;
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
