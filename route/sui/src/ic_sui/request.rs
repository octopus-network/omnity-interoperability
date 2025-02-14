use serde_json::{json, Value};
use std::fmt;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum RpcRequest {
    Custom { method: &'static str },
    GetCoins,
    GetEvents,
    GetObject,
    GetBalance,
    GetOwnedObjects,
    GetReferenceGasPrice,
    ExecuteTransactionBlock,
    GetTransactionBlock,
}

#[allow(deprecated)]
impl fmt::Display for RpcRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let method = match self {
            RpcRequest::Custom { method } => method,
            RpcRequest::GetCoins => "suix_getCoins",
            RpcRequest::GetEvents => "sui_getEvents",
            RpcRequest::GetObject => "sui_getObject",
            RpcRequest::GetBalance => "suix_getBalance",
            RpcRequest::GetOwnedObjects => "suix_getOwnedObjects",
            RpcRequest::GetReferenceGasPrice => "suix_getReferenceGasPrice",
            RpcRequest::ExecuteTransactionBlock => "sui_executeTransactionBlock",
            RpcRequest::GetTransactionBlock => "sui_getTransactionBlock",
        };

        write!(f, "{method}")
    }
}

impl RpcRequest {
    pub fn build_request_json(self, id: u64, params: Value) -> Value {
        let jsonrpc = "2.0";
        json!({
           "jsonrpc": jsonrpc,
           "id": id,
           "method": format!("{self}"),
           "params": params,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_object_request() {
        let test_request = RpcRequest::GetObject;
        let addr = json!("deadbeefXjn8o3yroDHxUtKsZZgoy4GPkPPXfouKNHhx");
        let request = test_request.build_request_json(1, json!([addr]));
        assert_eq!(request["method"], "sui_getObject");
        assert_eq!(request["params"], json!([addr]));
    }
    #[test]
    fn test_build_request_json() {
        let test_request = RpcRequest::GetOwnedObjects;
        let struct_type = "0x2::coin::Coin<0x2::sui::SUI>";
        let address_owner = "0xaf9306cac62396be300b175046140c392eed876bd8ac0efac6301cea286fa272";
        let query = json!({
            "filter": {
                "MatchAll": [
                    { "StructType": struct_type },
                    { "AddressOwner": address_owner }
                ]
            }
        });
        // let options = json!({
        //     "options": null
        // });
        let options: Option<serde_json::Value> = None;
        let request = test_request.build_request_json(1, json!([address_owner, query, options]));
        println!("request:{}", serde_json::to_string(&request).unwrap());
        assert_eq!(request["method"], "suix_getOwnedObjects");
        assert_eq!(request["params"], json!([address_owner, query, options]));
    }
}
