use std::str::FromStr;

use crate::{constants::KB100, errors::CustomsError, types::http_request_with_retry};
use ic_cdk::api::management_canister::http_request::{CanisterHttpRequestArgument, HttpHeader, HttpMethod, TransformContext, TransformFunc};
use serde::{Deserialize, Serialize};

use super::{rpc::DogeRpc, transaction::Txid};


pub struct TatumDogeRpc {
    doge_rpc: DogeRpc
} 

impl TatumDogeRpc {
    pub fn new(tatum_rpc_url: String, tatum_api_key: Option<String>) -> Self {
        let doge_rpc = DogeRpc {
            url: tatum_rpc_url,
            api_key: tatum_api_key,
        };
        Self {
            doge_rpc
        }
    }

    pub async fn get_transactions_by_address(&self, address: String )-> Result<Vec<Txid>, CustomsError>{
        let mut headers = vec![
            HttpHeader {
                name: "Content-Type".to_string(),
                value: "application/json".to_string(),
            },
        ]; 

        if let Some(api_key) = self.doge_rpc.api_key.clone() {
            headers.push(HttpHeader {
                name: "x-api-key".to_string(),
                value: api_key,
            });
        };

        let full_url = format!("{}/v3/dogecoin/transaction/address/{}?pageSize=3&txType=incoming", self.doge_rpc.url, address);
        let request = CanisterHttpRequestArgument {
            url: full_url,
            method: HttpMethod::GET,
            body: None,
            max_response_bytes: Some(KB100),
            transform: Some(TransformContext {
                function: TransformFunc(candid::Func {
                    principal: ic_cdk::api::id(),
                    method: "transform".to_string(),
                }),
                context: vec![],
            }),
            headers
        }; 

        // let txs: Vec<Transaction> = serde_json::from_slice(&response.body).map_err(|e| {
        //     log!(ERROR, "json error {:?}", e);
        //     CustomsError::RpcError(
        //         "failed to desc result from json".to_string(),
        //     )
        // })?;

        let response = http_request_with_retry(request.clone()).await?;

        let rpc_response: Vec<Transaction> = serde_json::from_slice(&response.body).map_err(|_| {
            CustomsError::RpcError(
                "failed to desc result from json".to_string(),
            )
        })?;

        let mut txids = vec![];
        for e in rpc_response.iter() {
            txids.push(Txid::from_str(e.hash.as_str()).map_err(
                |e| CustomsError::RpcError(
                    format!("failed to parse txid: {:?}", e)
                )
            )?);
        }

        // let v: Vec<Txid> = rpc_response.iter().map(|tx| Txid::from_str(tx.hash.as_str())).collect();
        Ok(txids)
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Transaction {
    // pub version: u32,
    // pub lock_time: u32,
    // pub input: Vec<TxIn>,
    pub hash: String,
    // pub outputs: Vec<TxOut>,
}

#[test]
pub fn test() {

    let r = r#"
    [
  {
    "blockNumber": 5535077,
    "fee": "0.373",
    "hash": "2a68d0319985d7a35eec7b97ce707f3a5b2872e173bb8e9dd21890fdccd0c172",
    "hex": "02000000023d1a3ac89095b058f7cf1ab6df64a8d34ce19289b04b438a892bf5a004350114010000006b48304502210099f5ee230866637643700a94bac827df7bc03f8528613d94697c459aef1bb9f602201763b28fe4f0702af69e683f21ad3da82df01e1a6b16a1394bba661664b624cf0121020c191f5ab73a4e5694240a58841f8295438f052888e8b86d9c26a206b58e7026ffffffff43b82a5d4eeff0db2eaa3db7350cf45b55ba5d64f56ae63c77c5e5b63b2bbfae000000006a47304402203cf939c7d546ad6a2b9bf1c6674e14d13e2a5a7a83ad83ffb0f0ffe7c5eed31c02204a42edfbf27a0faac30f1c04eee881d20edb98174ad06558fbe914b733529a220121020c191f5ab73a4e5694240a58841f8295438f052888e8b86d9c26a206b58e7026ffffffff0200e1f505000000001976a91457996f3bd447eb254ed59e832a0aceedf842497f88aca0a28eb9010000001976a9143131fbf0980ccc6429e6b67b8a642d4e2b33db7588ac00000000",
    "index": 8,
    "inputs": [
      {
        "prevout": {
          "hash": "14013504a0f52b898a434bb08992e14cd3a864dfb61acff758b09590c83a1a3d",
          "index": 1
        },
        "sequence": 4294967295,
        "script": "48304502210099f5ee230866637643700a94bac827df7bc03f8528613d94697c459aef1bb9f602201763b28fe4f0702af69e683f21ad3da82df01e1a6b16a1394bba661664b624cf0121020c191f5ab73a4e5694240a58841f8295438f052888e8b86d9c26a206b58e7026",
        "coin": {
          "version": 2,
          "height": 5534613,
          "value": "74.454",
          "script": "76a9143131fbf0980ccc6429e6b67b8a642d4e2b33db7588ac",
          "address": "D9dDXck2s276Kyi4DRRhBuWojJKNanjiEW",
          "type": null,
          "reqSigs": null,
          "coinbase": false
        }
      },
      {
        "prevout": {
          "hash": "aebf2b3bb6e5c5773ce66af5645dba555bf40c35b73daa2edbf0ef4e5d2ab843",
          "index": 0
        },
        "sequence": 4294967295,
        "script": "47304402203cf939c7d546ad6a2b9bf1c6674e14d13e2a5a7a83ad83ffb0f0ffe7c5eed31c02204a42edfbf27a0faac30f1c04eee881d20edb98174ad06558fbe914b733529a220121020c191f5ab73a4e5694240a58841f8295438f052888e8b86d9c26a206b58e7026",
        "coin": {
          "version": 1,
          "height": 5534995,
          "value": "1",
          "script": "76a9143131fbf0980ccc6429e6b67b8a642d4e2b33db7588ac",
          "address": "D9dDXck2s276Kyi4DRRhBuWojJKNanjiEW",
          "type": null,
          "reqSigs": null,
          "coinbase": false
        }
      }
    ],
    "locktime": 0,
    "outputs": [
      {
        "value": "1",
        "script": "76a91457996f3bd447eb254ed59e832a0aceedf842497f88ac",
        "address": "DD8H8mGhwhztaT6431VA58gEMu8vacL8Y7",
        "scriptPubKey": {
          "type": "pubkeyhash",
          "reqSigs": 1
        }
      },
      {
        "value": "74.081",
        "script": "76a9143131fbf0980ccc6429e6b67b8a642d4e2b33db7588ac",
        "address": "D9dDXck2s276Kyi4DRRhBuWojJKNanjiEW",
        "scriptPubKey": {
          "type": "pubkeyhash",
          "reqSigs": 1
        }
      }
    ],
    "size": 373,
    "time": 1736246223,
    "version": 2,
    "vsize": 373,
    "witnessHash": "2a68d0319985d7a35eec7b97ce707f3a5b2872e173bb8e9dd21890fdccd0c172"
  },
  {
    "blockNumber": 5534613,
    "fee": "1.125",
    "hash": "14013504a0f52b898a434bb08992e14cd3a864dfb61acff758b09590c83a1a3d",
    "hex": "02000000018073e3a7567120b22a6b37010b7ec50d1cd3b1413a2b5a0429d029e806f7d1f4010000006b483045022100fea54ebe0e61b121d6056c88521d8b1c8313f5758adfa5de9a873f738382997402207549dadb5761bb64a76e826e2f25a1a29f227d73d30e31cdef5e1c443cd63c210121020c191f5ab73a4e5694240a58841f8295438f052888e8b86d9c26a206b58e7026ffffffff0200e1f505000000001976a91457996f3bd447eb254ed59e832a0aceedf842497f88acc0c9c7bb010000001976a9143131fbf0980ccc6429e6b67b8a642d4e2b33db7588ac00000000",
    "index": 57,
    "inputs": [
      {
        "prevout": {
          "hash": "f4d1f706e829d029045a2b3a41b1d31c0dc57e0b01376b2ab2207156a7e37380",
          "index": 1
        },
        "sequence": 4294967295,
        "script": "483045022100fea54ebe0e61b121d6056c88521d8b1c8313f5758adfa5de9a873f738382997402207549dadb5761bb64a76e826e2f25a1a29f227d73d30e31cdef5e1c443cd63c210121020c191f5ab73a4e5694240a58841f8295438f052888e8b86d9c26a206b58e7026",
        "coin": {
          "version": 2,
          "height": 5534606,
          "value": "76.579",
          "script": "76a9143131fbf0980ccc6429e6b67b8a642d4e2b33db7588ac",
          "address": "D9dDXck2s276Kyi4DRRhBuWojJKNanjiEW",
          "type": null,
          "reqSigs": null,
          "coinbase": false
        }
      }
    ],
    "locktime": 0,
    "outputs": [
      {
        "value": "1",
        "script": "76a91457996f3bd447eb254ed59e832a0aceedf842497f88ac",
        "address": "DD8H8mGhwhztaT6431VA58gEMu8vacL8Y7",
        "scriptPubKey": {
          "type": "pubkeyhash",
          "reqSigs": 1
        }
      },
      {
        "value": "74.454",
        "script": "76a9143131fbf0980ccc6429e6b67b8a642d4e2b33db7588ac",
        "address": "D9dDXck2s276Kyi4DRRhBuWojJKNanjiEW",
        "scriptPubKey": {
          "type": "pubkeyhash",
          "reqSigs": 1
        }
      }
    ],
    "size": 226,
    "time": 1736215756,
    "version": 2,
    "vsize": 226,
    "witnessHash": "14013504a0f52b898a434bb08992e14cd3a864dfb61acff758b09590c83a1a3d"
  },
  {
    "blockNumber": 5526416,
    "fee": "0.225",
    "hash": "508a6242603d54f63ed8e17553836cd66c6f019ebdf5f85d4acd4b5a3d56d354",
    "hex": "0200000001e8ee6ba4aedaabbce0d3431ca7ec1450a928aad27b5f61fd73119c5ac1f99467010000006a4730440220254a3b496005bf0dcca9bfbb792191490e126cd146decae2c3b36e47ebc0391202206218cef80b975c21779ee9387ba6b16cabc03b462103425010b525aeb3b882ec0121020c191f5ab73a4e5694240a58841f8295438f052888e8b86d9c26a206b58e7026ffffffff0200c2eb0b000000001976a91457996f3bd447eb254ed59e832a0aceedf842497f88acc0554e03020000001976a9143131fbf0980ccc6429e6b67b8a642d4e2b33db7588ac00000000",
    "index": 2,
    "inputs": [
      {
        "prevout": {
          "hash": "6794f9c15a9c1173fd615f7bd2aa28a95014eca71c43d3e0bcabdaaea46beee8",
          "index": 1
        },
        "sequence": 4294967295,
        "script": "4730440220254a3b496005bf0dcca9bfbb792191490e126cd146decae2c3b36e47ebc0391202206218cef80b975c21779ee9387ba6b16cabc03b462103425010b525aeb3b882ec0121020c191f5ab73a4e5694240a58841f8295438f052888e8b86d9c26a206b58e7026",
        "coin": {
          "version": 2,
          "height": 5525087,
          "value": "88.679",
          "script": "76a9143131fbf0980ccc6429e6b67b8a642d4e2b33db7588ac",
          "address": "D9dDXck2s276Kyi4DRRhBuWojJKNanjiEW",
          "type": null,
          "reqSigs": null,
          "coinbase": false
        }
      }
    ],
    "locktime": 0,
    "outputs": [
      {
        "value": "2",
        "script": "76a91457996f3bd447eb254ed59e832a0aceedf842497f88ac",
        "address": "DD8H8mGhwhztaT6431VA58gEMu8vacL8Y7",
        "scriptPubKey": {
          "type": "pubkeyhash",
          "reqSigs": 1
        }
      },
      {
        "value": "86.454",
        "script": "76a9143131fbf0980ccc6429e6b67b8a642d4e2b33db7588ac",
        "address": "D9dDXck2s276Kyi4DRRhBuWojJKNanjiEW",
        "scriptPubKey": {
          "type": "pubkeyhash",
          "reqSigs": 1
        }
      }
    ],
    "size": 225,
    "time": 1735698111,
    "version": 2,
    "vsize": 225,
    "witnessHash": "508a6242603d54f63ed8e17553836cd66c6f019ebdf5f85d4acd4b5a3d56d354"
  },
  {
    "blockNumber": 5525087,
    "fee": "0.225",
    "hash": "6794f9c15a9c1173fd615f7bd2aa28a95014eca71c43d3e0bcabdaaea46beee8",
    "hex": "0200000001896cd225c2809b94c3a00adcd083425c46f4a05e0b91606ccb669081c930c2b5010000006946304302201c99c97d8f7a1ca20c0669a2e0bcd2c957e6a863c57705a417c1d7c7d753b1bf021f1239739b6a8fecf450e6fc0f610752d707a67e1fe1c0924caf9b650abf800f0121020c191f5ab73a4e5694240a58841f8295438f052888e8b86d9c26a206b58e7026ffffffff0200c2eb0b000000001976a91457996f3bd447eb254ed59e832a0aceedf842497f88ac606a9110020000001976a9143131fbf0980ccc6429e6b67b8a642d4e2b33db7588ac00000000",
    "index": 58,
    "inputs": [
      {
        "prevout": {
          "hash": "b5c230c9819066cb6c60910b5ea0f4465c4283d0dc0aa0c3949b80c225d26c89",
          "index": 1
        },
        "sequence": 4294967295,
        "script": "46304302201c99c97d8f7a1ca20c0669a2e0bcd2c957e6a863c57705a417c1d7c7d753b1bf021f1239739b6a8fecf450e6fc0f610752d707a67e1fe1c0924caf9b650abf800f0121020c191f5ab73a4e5694240a58841f8295438f052888e8b86d9c26a206b58e7026",
        "coin": {
          "version": 2,
          "height": 5525048,
          "value": "90.904",
          "script": "76a9143131fbf0980ccc6429e6b67b8a642d4e2b33db7588ac",
          "address": "D9dDXck2s276Kyi4DRRhBuWojJKNanjiEW",
          "type": null,
          "reqSigs": null,
          "coinbase": false
        }
      }
    ],
    "locktime": 0,
    "outputs": [
      {
        "value": "2",
        "script": "76a91457996f3bd447eb254ed59e832a0aceedf842497f88ac",
        "address": "DD8H8mGhwhztaT6431VA58gEMu8vacL8Y7",
        "scriptPubKey": {
          "type": "pubkeyhash",
          "reqSigs": 1
        }
      },
      {
        "value": "88.679",
        "script": "76a9143131fbf0980ccc6429e6b67b8a642d4e2b33db7588ac",
        "address": "D9dDXck2s276Kyi4DRRhBuWojJKNanjiEW",
        "scriptPubKey": {
          "type": "pubkeyhash",
          "reqSigs": 1
        }
      }
    ],
    "size": 224,
    "time": 1735613142,
    "version": 2,
    "vsize": 224,
    "witnessHash": "6794f9c15a9c1173fd615f7bd2aa28a95014eca71c43d3e0bcabdaaea46beee8"
  }
]
    "#;

    let txs: Vec<Transaction> = serde_json::from_str(r).unwrap();
    dbg!(&txs);

}