use ic_cdk::api::management_canister::http_request::HttpResponse;
use proto::tendermint::crypto::public_key;
use schnorr::{sign_with_schnorr, SchnorrKeyId};
use serde_json::Value;
use utils::{bytes_to_base64, http_request_with_status_check};

use crate::*;

// const CHAIN_ID: &str = "localosmosis";
// const RPC_PORT: u16 = 26657;
// const OSMO_ACCOUNT_PREFIX: &str = "osmo";
const DENOM: &str = "uosmo";
const MEMO: &str = "test memo";
// const ACCOUNT_NUMBER: AccountNumber = 96638;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CosmosWasmClient {
    pub rpc_url: String,
    pub rest_url: String,
    pub chain_id: ChainId,
}

impl CosmosWasmClient {
    pub fn new(rpc_url: String, rest_url: String, chain_id: ChainId) -> Self {
        Self {
            rpc_url,
            rest_url,
            chain_id,
        }
    }

    pub async fn query_account_number_and_sequence(
        &self,
        address: String,
    ) -> Result<(AccountNumber, u64)> {
        // https://lcd.testnet.osmosis.zone/cosmos/auth/v1beta1/account_info/osmo1x6ctqf5fwy37tx9vdhh9y7kxk5puvwsdnl0acw

        let full_url = format!(
            "{}/cosmos/auth/v1beta1/account_info/{}",
            self.rest_url, address
        )
        .to_string();

        let request_headers = vec![HttpHeader {
            name: "content-type".to_string(),
            value: "application/json".to_string(),
        }];

        let request = CanisterHttpRequestArgument {
            url: full_url,
            max_response_bytes: None,
            method: HttpMethod::GET,
            headers: request_headers,
            body: None,
            transform: None,
        };

        let response = http_request_with_status_check(request).await?;

        let json_value: Value = serde_json::from_slice(&response.body).map_err(|e| {
            RouteError::CustomError(format!("Failed to parse account info: {:?}", e.to_string()))
        })?;

        let account_number = json_value["info"]["account_number"]
            .as_str()
            .ok_or_else(|| RouteError::CustomError("Failed to parse account number".to_string()))?
            .parse::<u64>()
            .map_err(|e| {
                RouteError::CustomError(format!(
                    "Failed to parse account number: {:?}",
                    e.to_string()
                ))
            })?;

        let sequence = json_value["info"]["sequence"]
            .as_str()
            .ok_or_else(|| RouteError::CustomError("Failed to parse sequence".to_string()))?
            .parse::<u64>()
            .map_err(|e| {
                RouteError::CustomError(format!("Failed to parse sequence: {:?}", e.to_string()))
            })?;

        Ok((account_number, sequence))
    }

    pub async fn broadcast_tx_commit(&self, raw: Raw) -> Result<HttpResponse> {
        let raw_bytes = raw.to_bytes().unwrap();
        let raw_base64 = bytes_to_base64(&raw_bytes);

        log::info!("tx_raw_base64: {:?}", raw_base64);

        let request_headers = vec![HttpHeader {
            name: "content-type".to_string(),
            value: "application/json".to_string(),
        }];

        let request_body = json!({
            "jsonrpc": "2.0",
            "method": "broadcast_tx_commit",
            "params": {
                "tx": raw_base64,
            },
            "id": Id::uuid_v4(),
        });

        let request = CanisterHttpRequestArgument {
            url: self.rpc_url.clone(),
            max_response_bytes: None,
            method: HttpMethod::POST,
            headers: request_headers,
            body: Some(request_body.to_string().into_bytes()),
            transform: None,
            // transform: None, //optional for request
        };

        let http_response = http_request_with_status_check(request).await?;
        // Response::from_string(response.body)
        // DialectResponse::from_string(response)
        // response.body
        // let respone = http_request(request, 100_000_000_000)
        //     .await
        //     .map_err(|(code, message)| {
        //         RouteError::HttpOutCallError(format!("{:?}", code).to_string(), message)
        //     })?;

        // dbg!(&respone);
        // let tx_response: Response = serde_json::from_slice(&http_response.body).map_err(
        //     |e| RouteError::CustomError(format!("Failed to parse tx response: {:?}", e.to_string())),
        // )?;

        dbg!(&http_response);

        Ok(http_response)
    }

    pub async fn execute_msg(
        &self,
        contract_id: AccountId,
        msg: ExecuteMsg,
        sender_public_key: cosmrs::crypto::PublicKey,
        sender_account_id: AccountId,
        key_id: SchnorrKeyId,
    ) -> Result<HttpResponse> {
        let (account_number, sequence) = self.query_account_number_and_sequence(sender_account_id.to_string()).await?;

        log::info!("account_number: {:?}, sequence: {:?}", account_number, sequence);
        // let sequence_number = 0u64;
        let gas = 100_000u64;
        let amount = Coin {
            amount: 10000u128.into(),
            denom: DENOM.parse().unwrap(),
        };
        let fee = Fee::from_amount_and_gas(amount, gas);

        let msg_execute = MsgExecuteContract {
            sender: sender_account_id,
            contract: contract_id,
            msg: serde_json::to_string(&msg).unwrap().into_bytes(),
            funds: vec![],
        }
        .to_any()
        .unwrap();

        let tx_body = tx::BodyBuilder::new().msg(msg_execute).memo(MEMO).finish();

        let auth_info =
            SignerInfo::single_direct(Some(sender_public_key), sequence).auth_info(fee);

        log::info!("auth_info: {:?}", auth_info);

        let chain_id = self
            .chain_id
            .clone()
            .parse::<tendermint::chain::Id>()
            .map_err(|e| {
                RouteError::CustomError(format!("Failed to parse chain id: {:?}", e.to_string()))
            })?;
        let sign_doc = SignDoc::new(&tx_body, &auth_info, &chain_id, account_number).unwrap();
        log::info!("sign_doc: {:?}", sign_doc);

        let sign_result =
            sign_with_schnorr(&sign_doc.clone().into_bytes().unwrap(), key_id).await?;

        log::info!("sign_result: {:?}", sign_result);

        let raw: Raw = proto::cosmos::tx::v1beta1::TxRaw {
            body_bytes: sign_doc.body_bytes.clone(),
            auth_info_bytes: sign_doc.auth_info_bytes.clone(),
            signatures: vec![sign_result.signature.to_vec()],
        }
        .into();

        self.broadcast_tx_commit(raw).await
    }
}

pub fn raw_response()-> String {
    return r#"
    {
        "header": {
          "chain_id": "osmo-test-5",
          "timestamp": "2024-08-05T05:56:07Z"
        },
        "data": {
          "height": "10705947",
          "txhash": "B7CC0D163449D2BFD8BC775015B91214749817292766D3251816B4828A11A221",
          "codespace": "",
          "code": 0,
          "data": "122D0A2B2F6962632E636F72652E636C69656E742E76312E4D7367557064617465436C69656E74526573706F6E7365",
          "logs": [
            {
              "msg_index": 0,
              "events": [
                {
                  "type": "message",
                  "attributes": [
                    {
                      "key": "action",
                      "value": "/ibc.core.client.v1.MsgUpdateClient",
                      "index": true
                    },
                    {
                      "key": "sender",
                      "value": "osmo1j73g96rdw2vlwvkuu733tcejzyvhkp4nlsdptg",
                      "index": true
                    },
                    {
                      "key": "msg_index",
                      "value": "0",
                      "index": true
                    }
                  ]
                },
                {
                  "type": "update_client",
                  "attributes": [
                    {
                      "key": "client_id",
                      "value": "07-tendermint-3894",
                      "index": true
                    },
                    {
                      "key": "client_type",
                      "value": "07-tendermint",
                      "index": true
                    },
                    {
                      "key": "consensus_height",
                      "value": "0-129871",
                      "index": true
                    },
                    {
                      "key": "consensus_heights",
                      "value": "0-129871",
                      "index": true
                    },
                    {
                      "key": "header",
                      "value": "0a262f6962632e6c69676874636c69656e74732e74656e6465726d696e742e76312e48656164657212d81d0ac5080aa6030a02080b121e686f757365666972652d656e76656c6f70652e623866393535373230616218cff607220c08f2d2c1b50610df8fa2ed022a480a205f19884febafb35adaf7e4c2c5e1b1adeafd2db6c5262745f5214785679dea331224080112201cc2cda8f43a7682f8f689e4258e3926461ed7765e5c5700b987096eee96f4ca3220df599a31ab2d68743a5e00f6202bb41e1b8bbdd6e2bb0ce2911ec4b1ab8f923a3a20e4b5babb1544562fecaa9c9e38132873d3c65ecc616d2b68ebfbd1fce63583e24220fd0c016e3718ee5ac1272bb0faaa21523753aad5cac400a0f6273b3c55d897bc4a20fd0c016e3718ee5ac1272bb0faaa21523753aad5cac400a0f6273b3c55d897bc52204b53c13521d2126d4e199e783963d0e8b2729f143eb01a7f3817a1d1e079193f5a20669bdc11dbe73c9ecdc47244826fb7eab0c94153244112d240c9388307aafbfd6220e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b8556a20e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b8557214f67db88bccccd418cedca1583f59d4777e7c424712990508cff6071a480a20e033b656e4d046d55c352ad8dbaddc0fcec24c585982ccf2e9dda631cab200451224080112207fdf259742a35ae76166197bed9a2665a3e1fae1621e71a55ef95d4819003fab22670802121432ac584b1f1ef09e4a7c985d10fc7b7c4f50741a1a0b08f9d2c1b50610fcd8a6282240b32371497f5df7d4d801e867eab51adffa7c5f94589c44055022bad2b86035c0662b17208a8aa3c45b34a28b09bc1608fc9e916e50fa5023729045756435ce06226708021214f67db88bccccd418cedca1583f59d4777e7c42471a0b08f9d2c1b50610ac84c32b2240655349e15a639553b87d05d83d4cce5f216866d8902926c998da3969ffd72d626914b8d0058e60e76ac8af20489cd7d17f3e3666980a8cf45d2cfbd1a1a0ff08220f08011a0b088092b8c398feffffff01220f08011a0b088092b8c398feffffff01220f08011a0b088092b8c398feffffff01220f08011a0b088092b8c398feffffff01220f08011a0b088092b8c398feffffff01220f08011a0b088092b8c398feffffff01220f08011a0b088092b8c398feffffff01220f08011a0b088092b8c398feffffff01220f08011a0b088092b8c398feffffff01220f08011a0b088092b8c398feffffff01220f08011a0b088092b8c398feffffff01226708021214a73aa53a3d18823b811e7476b2c8fe157d5c969e1a0b08f9d2c1b50610d0d78f5e22400b944396c6bc0b9e89a07ece7b29958d8b930142a0c45b1420871238df0cd2706b6f6d2cda21449e4763c67f0aabf6e78e12d24969a05ece4bb781837df8ee07220f08011a0b088092b8c398feffffff01220f08011a0b088092b8c398feffffff01220f08011a0b088092b8c398feffffff01220f08011a0b088092b8c398feffffff01220f08011a0b088092b8c398feffffff0112c20a0a430a1432ac584b1f1ef09e4a7c985d10fc7b7c4f50741a12220a20ec4e3380295dd1c132feb3d23d5c60ae6851a5df62dcc658b9b0c43480e7f6ce18e9d4c3d2ecafe3010a430a14f67db88bccccd418cedca1583f59d4777e7c424712220a20cc77f5696302dbe149d2c64aae9933649ad99dd6e81d853e497a5d4c44c91e3c18afc4c1eeeaafe3010a410a14d487e850e53f442cee553ad5c6882d8cb1008c0212220a201f5c13e30f9198f64b665b024c6a46b00ab04761d05dd325b593c3f98a79d40118c8c09be0951d0a410a1439b917deb17599f02676edf853034442cebddc7012220a20c76e33a6e008b44c9dcf0c743b8a56afaadc97d55224e41ee31ffe7d708826fc18cbe1caba911d0a410a1413c50423b7cf6ef7eb7c69f3c964044507f8820012220a2052dcc663641232ef96a4713ad1a166329db77c1e848cf0d48a0100212ba1c100189de493dc8f1d0a410a14bfe24bfc69a782bd1cb9c76f1c45dd8a0d4889a712220a20c371ab99f712736d4537a9205c14c91806c7b2cb007c4268bd396a7689022e0418d69f98d18d1d0a410a14223690b82da6a32b97d0892d7322aac7d82349d912220a20c8714fd84d12abe0e0694eade440a46f3ac580b5f355097d9683724520f3eaf218e9ca99f09d1a0a410a140b665441a8b0fabb118da79007fab319c674db7a12220a2058833eacb1d3c1d8a89a9cf1990b6453499a0257936e39e906c3e577f4b38ff518f29debed9d1a0a410a146cbf7eba2000c7db61f166635fa0804c1a0ae98112220a202a8c29a4d97631686545189273ad32253a35fa55b171864a90d4443a0e0991a918c08cd6a59a1a0a410a143dbf6ccf5de1e1c8cddc2afef95386c2e749628e12220a2019519ac4160d2e396f178c40dab8ca001e6b958080cb3b2ee7856094eb3153be18cd9ad6e7df010a400a14e203fe9da0aae81c4efcf44d3ff9d87f17cfedff12220a2029f5f8d968637821efb8d37134cc388de0dcb8c424715727d0922b9fb9bf2ed418e6baecfd290a400a14eaef17fb3ed004e8b13cfce282c55a0f703b077312220a2035da8e0a5f8e4c91cea48e5917947d7289bce661babd4c783e93a6f508527abe18a5f480ed150a400a1473253ea1063c7a18fe35d598c20bb753bcc868fa12220a2072af80fca43a1721de15470e6a2e9a2d47d5a9841af5921af7bfda348792d1511880aecdbe140a400a14a73aa53a3d18823b811e7476b2c8fe157d5c969e12220a20ebe1ff8676e319c19257251354de9a747c92b871496670f4d9f0c7d08d22678f18c2fcaf87080a400a148f55c7278a5e0a063bedc381ce2044bc923c3a4b12220a20d8b064fd4215f3be944ea97bfc56d99ddf0d0c92d692ee59276ca92a31e1f6f518c2d6b8ec050a400a1423ff4cbfeff27a529533bf2bf0b756d517f9283712220a201ea97bc04c5d507b12b3f5f59b14239c699779640e4c71e3b7051b50f0cc5d9818c3e4f99b040a400a14d141667bfa4d9428a72518972683c9b2bceff83f12220a20959e47c0dfd4bda0ae8d0f8208e3cc3d3e0a76331b138d271cf51966280f85d518c098a8dd030a400a1458c8aa3fe9180532b7ce909e1541d835e8a5c7f712220a20d6017344b8d7277876661e70ff3291d504e4f0e273349cf33caa26a04bfb15231880e0e1c9030a400a14d825a8f9e8350d41b632998985b7c829a90ac8f512220a20114724bf6321a844289f9cb1f00241fb38426dc847372f09cca83dcfd8b7dcd318b2efa3e60112430a14f67db88bccccd418cedca1583f59d4777e7c424712220a20cc77f5696302dbe149d2c64aae9933649ad99dd6e81d853e497a5d4c44c91e3c18afc4c1eeeaafe30118cacb93bbc1a5c8031a041083f60722c20a0a430a1432ac584b1f1ef09e4a7c985d10fc7b7c4f50741a12220a20ec4e3380295dd1c132feb3d23d5c60ae6851a5df62dcc658b9b0c43480e7f6ce18e9d4c3d2ecafe3010a430a14f67db88bccccd418cedca1583f59d4777e7c424712220a20cc77f5696302dbe149d2c64aae9933649ad99dd6e81d853e497a5d4c44c91e3c18afc4c1eeeaafe3010a410a14d487e850e53f442cee553ad5c6882d8cb1008c0212220a201f5c13e30f9198f64b665b024c6a46b00ab04761d05dd325b593c3f98a79d40118c8c09be0951d0a410a1439b917deb17599f02676edf853034442cebddc7012220a20c76e33a6e008b44c9dcf0c743b8a56afaadc97d55224e41ee31ffe7d708826fc18cbe1caba911d0a410a1413c50423b7cf6ef7eb7c69f3c964044507f8820012220a2052dcc663641232ef96a4713ad1a166329db77c1e848cf0d48a0100212ba1c100189de493dc8f1d0a410a14bfe24bfc69a782bd1cb9c76f1c45dd8a0d4889a712220a20c371ab99f712736d4537a9205c14c91806c7b2cb007c4268bd396a7689022e0418d69f98d18d1d0a410a14223690b82da6a32b97d0892d7322aac7d82349d912220a20c8714fd84d12abe0e0694eade440a46f3ac580b5f355097d9683724520f3eaf218e9ca99f09d1a0a410a140b665441a8b0fabb118da79007fab319c674db7a12220a2058833eacb1d3c1d8a89a9cf1990b6453499a0257936e39e906c3e577f4b38ff518f29debed9d1a0a410a146cbf7eba2000c7db61f166635fa0804c1a0ae98112220a202a8c29a4d97631686545189273ad32253a35fa55b171864a90d4443a0e0991a918c08cd6a59a1a0a410a143dbf6ccf5de1e1c8cddc2afef95386c2e749628e12220a2019519ac4160d2e396f178c40dab8ca001e6b958080cb3b2ee7856094eb3153be18cd9ad6e7df010a400a14e203fe9da0aae81c4efcf44d3ff9d87f17cfedff12220a2029f5f8d968637821efb8d37134cc388de0dcb8c424715727d0922b9fb9bf2ed418e6baecfd290a400a14eaef17fb3ed004e8b13cfce282c55a0f703b077312220a2035da8e0a5f8e4c91cea48e5917947d7289bce661babd4c783e93a6f508527abe18a5f480ed150a400a1473253ea1063c7a18fe35d598c20bb753bcc868fa12220a2072af80fca43a1721de15470e6a2e9a2d47d5a9841af5921af7bfda348792d1511880aecdbe140a400a14a73aa53a3d18823b811e7476b2c8fe157d5c969e12220a20ebe1ff8676e319c19257251354de9a747c92b871496670f4d9f0c7d08d22678f18c2fcaf87080a400a148f55c7278a5e0a063bedc381ce2044bc923c3a4b12220a20d8b064fd4215f3be944ea97bfc56d99ddf0d0c92d692ee59276ca92a31e1f6f518c2d6b8ec050a400a1423ff4cbfeff27a529533bf2bf0b756d517f9283712220a201ea97bc04c5d507b12b3f5f59b14239c699779640e4c71e3b7051b50f0cc5d9818c3e4f99b040a400a14d141667bfa4d9428a72518972683c9b2bceff83f12220a20959e47c0dfd4bda0ae8d0f8208e3cc3d3e0a76331b138d271cf51966280f85d518c098a8dd030a400a1458c8aa3fe9180532b7ce909e1541d835e8a5c7f712220a20d6017344b8d7277876661e70ff3291d504e4f0e273349cf33caa26a04bfb15231880e0e1c9030a400a14d825a8f9e8350d41b632998985b7c829a90ac8f512220a20114724bf6321a844289f9cb1f00241fb38426dc847372f09cca83dcfd8b7dcd318b2efa3e60112430a1432ac584b1f1ef09e4a7c985d10fc7b7c4f50741a12220a20ec4e3380295dd1c132feb3d23d5c60ae6851a5df62dcc658b9b0c43480e7f6ce18e9d4c3d2ecafe30118cacb93bbc1a5c803",
                      "index": true
                    },
                    {
                      "key": "msg_index",
                      "value": "0",
                      "index": true
                    }
                  ]
                },
                {
                  "type": "message",
                  "attributes": [
                    {
                      "key": "module",
                      "value": "ibc_client",
                      "index": true
                    },
                    {
                      "key": "msg_index",
                      "value": "0",
                      "index": true
                    }
                  ]
                }
              ]
            },
            {
              "msg_index": null,
              "events": [
                {
                  "type": "coin_spent",
                  "attributes": [
                    {
                      "key": "spender",
                      "value": "osmo1j73g96rdw2vlwvkuu733tcejzyvhkp4nlsdptg",
                      "index": true
                    },
                    {
                      "key": "amount",
                      "value": "19283uosmo",
                      "index": true
                    }
                  ]
                },
                {
                  "type": "coin_received",
                  "attributes": [
                    {
                      "key": "receiver",
                      "value": "osmo17xpfvakm2amg962yls6f84z3kell8c5lczssa0",
                      "index": true
                    },
                    {
                      "key": "amount",
                      "value": "19283uosmo",
                      "index": true
                    }
                  ]
                },
                {
                  "type": "transfer",
                  "attributes": [
                    {
                      "key": "recipient",
                      "value": "osmo17xpfvakm2amg962yls6f84z3kell8c5lczssa0",
                      "index": true
                    },
                    {
                      "key": "sender",
                      "value": "osmo1j73g96rdw2vlwvkuu733tcejzyvhkp4nlsdptg",
                      "index": true
                    },
                    {
                      "key": "amount",
                      "value": "19283uosmo",
                      "index": true
                    }
                  ]
                },
                {
                  "type": "message",
                  "attributes": [
                    {
                      "key": "sender",
                      "value": "osmo1j73g96rdw2vlwvkuu733tcejzyvhkp4nlsdptg",
                      "index": true
                    }
                  ]
                },
                {
                  "type": "tx",
                  "attributes": [
                    {
                      "key": "fee",
                      "value": "19283uosmo",
                      "index": true
                    }
                  ]
                },
                {
                  "type": "tx",
                  "attributes": [
                    {
                      "key": "acc_seq",
                      "value": "osmo1j73g96rdw2vlwvkuu733tcejzyvhkp4nlsdptg/3927",
                      "index": true
                    }
                  ]
                },
                {
                  "type": "tx",
                  "attributes": [
                    {
                      "key": "signature",
                      "value": "pnPaX3Kr4JA5XMTb02PF0/+ChNg+7rLaYfS/zW9ZbV9uWwLHMuMNz4AYZhtftPNkFglxu55jzqqeBN+LdJtB8A==",
                      "index": true
                    }
                  ]
                }
              ]
            }
          ],
          "info": "",
          "gas_wanted": "192824",
          "gas_used": "182724",
          "tx": {
            "@type": "/cosmos.tx.v1beta1.Tx",
            "body": {
              "messages": [
                {
                  "@type": "/ibc.core.client.v1.MsgUpdateClient",
                  "client_id": "07-tendermint-3894",
                  "client_message": {
                    "@type": "/ibc.lightclients.tendermint.v1.Header",
                    "signed_header": {
                      "header": {
                        "version": {
                          "block": "11",
                          "app": "0"
                        },
                        "chain_id": "housefire-envelope.b8f955720ab",
                        "height": "129871",
                        "time": "2024-08-05T05:56:02.766019551Z",
                        "last_block_id": {
                          "hash": "XxmIT+uvs1ra9+TCxeGxrer9LbbFJidF9SFHhWed6jM=",
                          "part_set_header": {
                            "total": 1,
                            "hash": "HMLNqPQ6doL49onkJY45JkYe13ZeXFcAuYcJbu6W9Mo="
                          }
                        },
                        "last_commit_hash": "31maMastaHQ6XgD2ICu0HhuLvdbiuwzikR7EsauPkjo=",
                        "data_hash": "5LW6uxVEVi/sqpyeOBMoc9PGXsxhbSto6/vR/OY1g+I=",
                        "validators_hash": "/QwBbjcY7lrBJyuw+qohUjdTqtXKxACg9ic7PFXYl7w=",
                        "next_validators_hash": "/QwBbjcY7lrBJyuw+qohUjdTqtXKxACg9ic7PFXYl7w=",
                        "consensus_hash": "S1PBNSHSEm1OGZ54OWPQ6LJynxQ+sBp/OBeh0eB5GT8=",
                        "app_hash": "ZpvcEdvnPJ7NxHJEgm+36rDJQVMkQRLSQMk4gweq+/0=",
                        "last_results_hash": "47DEQpj8HBSa+/TImW+5JCeuQeRkm5NMpJWZG3hSuFU=",
                        "evidence_hash": "47DEQpj8HBSa+/TImW+5JCeuQeRkm5NMpJWZG3hSuFU=",
                        "proposer_address": "9n24i8zM1BjO3KFYP1nUd358Qkc="
                      },
                      "commit": {
                        "height": "129871",
                        "round": 0,
                        "block_id": {
                          "hash": "4DO2VuTQRtVcNSrY263cD87CTFhZgszy6d2mMcqyAEU=",
                          "part_set_header": {
                            "total": 1,
                            "hash": "f98ll0KjWudhZhl77ZomZaPh+uFiHnGlXvldSBkAP6s="
                          }
                        },
                        "signatures": [
                          {
                            "block_id_flag": "BLOCK_ID_FLAG_COMMIT",
                            "validator_address": "MqxYSx8e8J5KfJhdEPx7fE9QdBo=",
                            "timestamp": "2024-08-05T05:56:09.084520060Z",
                            "signature": "syNxSX9d99TYAehn6rUa3/p8X5RYnEQFUCK60rhgNcBmKxcgioqjxFs0oosJvBYI/J6RblD6UCNykEV1ZDXOBg=="
                          },
                          {
                            "block_id_flag": "BLOCK_ID_FLAG_COMMIT",
                            "validator_address": "9n24i8zM1BjO3KFYP1nUd358Qkc=",
                            "timestamp": "2024-08-05T05:56:09.091275820Z",
                            "signature": "ZVNJ4VpjlVO4fQXYPUzOXyFoZtiQKSbJmNo5af/XLWJpFLjQBY5g52rIryBInNfRfz42ZpgKjPRdLPvRoaD/CA=="
                          },
                          {
                            "block_id_flag": "BLOCK_ID_FLAG_ABSENT",
                            "validator_address": null,
                            "timestamp": "0001-01-01T00:00:00Z",
                            "signature": null
                          },
                          {
                            "block_id_flag": "BLOCK_ID_FLAG_ABSENT",
                            "validator_address": null,
                            "timestamp": "0001-01-01T00:00:00Z",
                            "signature": null
                          },
                          {
                            "block_id_flag": "BLOCK_ID_FLAG_ABSENT",
                            "validator_address": null,
                            "timestamp": "0001-01-01T00:00:00Z",
                            "signature": null
                          },
                          {
                            "block_id_flag": "BLOCK_ID_FLAG_ABSENT",
                            "validator_address": null,
                            "timestamp": "0001-01-01T00:00:00Z",
                            "signature": null
                          },
                          {
                            "block_id_flag": "BLOCK_ID_FLAG_ABSENT",
                            "validator_address": null,
                            "timestamp": "0001-01-01T00:00:00Z",
                            "signature": null
                          },
                          {
                            "block_id_flag": "BLOCK_ID_FLAG_ABSENT",
                            "validator_address": null,
                            "timestamp": "0001-01-01T00:00:00Z",
                            "signature": null
                          },
                          {
                            "block_id_flag": "BLOCK_ID_FLAG_ABSENT",
                            "validator_address": null,
                            "timestamp": "0001-01-01T00:00:00Z",
                            "signature": null
                          },
                          {
                            "block_id_flag": "BLOCK_ID_FLAG_ABSENT",
                            "validator_address": null,
                            "timestamp": "0001-01-01T00:00:00Z",
                            "signature": null
                          },
                          {
                            "block_id_flag": "BLOCK_ID_FLAG_ABSENT",
                            "validator_address": null,
                            "timestamp": "0001-01-01T00:00:00Z",
                            "signature": null
                          },
                          {
                            "block_id_flag": "BLOCK_ID_FLAG_ABSENT",
                            "validator_address": null,
                            "timestamp": "0001-01-01T00:00:00Z",
                            "signature": null
                          },
                          {
                            "block_id_flag": "BLOCK_ID_FLAG_ABSENT",
                            "validator_address": null,
                            "timestamp": "0001-01-01T00:00:00Z",
                            "signature": null
                          },
                          {
                            "block_id_flag": "BLOCK_ID_FLAG_COMMIT",
                            "validator_address": "pzqlOj0YgjuBHnR2ssj+FX1clp4=",
                            "timestamp": "2024-08-05T05:56:09.197389264Z",
                            "signature": "C5RDlsa8C56JoH7OeymVjYuTAUKgxFsUIIcSON8M0nBrb20s2iFEnkdjxn8Kq/bnjhLSSWmgXs5Lt4GDffjuBw=="
                          },
                          {
                            "block_id_flag": "BLOCK_ID_FLAG_ABSENT",
                            "validator_address": null,
                            "timestamp": "0001-01-01T00:00:00Z",
                            "signature": null
                          },
                          {
                            "block_id_flag": "BLOCK_ID_FLAG_ABSENT",
                            "validator_address": null,
                            "timestamp": "0001-01-01T00:00:00Z",
                            "signature": null
                          },
                          {
                            "block_id_flag": "BLOCK_ID_FLAG_ABSENT",
                            "validator_address": null,
                            "timestamp": "0001-01-01T00:00:00Z",
                            "signature": null
                          },
                          {
                            "block_id_flag": "BLOCK_ID_FLAG_ABSENT",
                            "validator_address": null,
                            "timestamp": "0001-01-01T00:00:00Z",
                            "signature": null
                          },
                          {
                            "block_id_flag": "BLOCK_ID_FLAG_ABSENT",
                            "validator_address": null,
                            "timestamp": "0001-01-01T00:00:00Z",
                            "signature": null
                          }
                        ]
                      }
                    },
                    "validator_set": {
                      "validators": [
                        {
                          "address": "MqxYSx8e8J5KfJhdEPx7fE9QdBo=",
                          "pub_key": {
                            "ed25519": "7E4zgCld0cEy/rPSPVxgrmhRpd9i3MZYubDENIDn9s4="
                          },
                          "voting_power": "1000000629828201",
                          "proposer_priority": "0"
                        },
                        {
                          "address": "9n24i8zM1BjO3KFYP1nUd358Qkc=",
                          "pub_key": {
                            "ed25519": "zHf1aWMC2+FJ0sZKrpkzZJrZndboHYU+SXpdTETJHjw="
                          },
                          "voting_power": "1000000151642671",
                          "proposer_priority": "0"
                        },
                        {
                          "address": "1IfoUOU/RCzuVTrVxogtjLEAjAI=",
                          "pub_key": {
                            "ed25519": "H1wT4w+RmPZLZlsCTGpGsAqwR2HQXdMltZPD+Yp51AE="
                          },
                          "voting_power": "1002271334472",
                          "proposer_priority": "0"
                        },
                        {
                          "address": "ObkX3rF1mfAmdu34UwNEQs693HA=",
                          "pub_key": {
                            "ed25519": "x24zpuAItEydzwx0O4pWr6rcl9VSJOQe4x/+fXCIJvw="
                          },
                          "voting_power": "1001118675147",
                          "proposer_priority": "0"
                        },
                        {
                          "address": "E8UEI7fPbvfrfGnzyWQERQf4ggA=",
                          "pub_key": {
                            "ed25519": "UtzGY2QSMu+WpHE60aFmMp23fB6EjPDUigEAISuhwQA="
                          },
                          "voting_power": "1000652206621",
                          "proposer_priority": "0"
                        },
                        {
                          "address": "v+JL/Gmngr0cucdvHEXdig1Iiac=",
                          "pub_key": {
                            "ed25519": "w3GrmfcSc21FN6kgXBTJGAbHsssAfEJovTlqdokCLgQ="
                          },
                          "voting_power": "1000092340182",
                          "proposer_priority": "0"
                        },
                        {
                          "address": "IjaQuC2moyuX0IktcyKqx9gjSdk=",
                          "pub_key": {
                            "ed25519": "yHFP2E0Sq+DgaU6t5ECkbzrFgLXzVQl9loNyRSDz6vI="
                          },
                          "voting_power": "901373125993",
                          "proposer_priority": "0"
                        },
                        {
                          "address": "C2ZUQaiw+rsRjaeQB/qzGcZ023o=",
                          "pub_key": {
                            "ed25519": "WIM+rLHTwdiompzxmQtkU0maAleTbjnpBsPld/Szj/U="
                          },
                          "voting_power": "901368172274",
                          "proposer_priority": "0"
                        },
                        {
                          "address": "bL9+uiAAx9th8WZjX6CATBoK6YE=",
                          "pub_key": {
                            "ed25519": "KowppNl2MWhlRRiSc60yJTo1+lWxcYZKkNREOg4Jkak="
                          },
                          "voting_power": "900411524672",
                          "proposer_priority": "0"
                        },
                        {
                          "address": "Pb9sz13h4cjN3Cr++VOGwudJYo4=",
                          "pub_key": {
                            "ed25519": "GVGaxBYNLjlvF4xA2rjKAB5rlYCAyzsu54VglOsxU74="
                          },
                          "voting_power": "60078525773",
                          "proposer_priority": "0"
                        },
                        {
                          "address": "4gP+naCq6BxO/PRNP/nYfxfP7f8=",
                          "pub_key": {
                            "ed25519": "KfX42WhjeCHvuNNxNMw4jeDcuMQkcVcn0JIrn7m/LtQ="
                          },
                          "voting_power": "11269774694",
                          "proposer_priority": "0"
                        },
                        {
                          "address": "6u8X+z7QBOixPPzigsVaD3A7B3M=",
                          "pub_key": {
                            "ed25519": "NdqOCl+OTJHOpI5ZF5R9com85mG6vUx4PpOm9QhSer4="
                          },
                          "voting_power": "5865749029",
                          "proposer_priority": "0"
                        },
                        {
                          "address": "cyU+oQY8ehj+NdWYwgu3U7zIaPo=",
                          "pub_key": {
                            "ed25519": "cq+A/KQ6FyHeFUcOai6aLUfVqYQa9ZIa97/aNIeS0VE="
                          },
                          "voting_power": "5500000000",
                          "proposer_priority": "0"
                        },
                        {
                          "address": "pzqlOj0YgjuBHnR2ssj+FX1clp4=",
                          "pub_key": {
                            "ed25519": "6+H/hnbjGcGSVyUTVN6adHySuHFJZnD02fDH0I0iZ48="
                          },
                          "voting_power": "2162949698",
                          "proposer_priority": "0"
                        },
                        {
                          "address": "j1XHJ4peCgY77cOBziBEvJI8Oks=",
                          "pub_key": {
                            "ed25519": "2LBk/UIV876UTql7/FbZnd8NDJLWku5ZJ2ypKjHh9vU="
                          },
                          "voting_power": "1569598274",
                          "proposer_priority": "0"
                        },
                        {
                          "address": "I/9Mv+/yelKVM78r8LdW1Rf5KDc=",
                          "pub_key": {
                            "ed25519": "Hql7wExdUHsSs/X1mxQjnGmXeWQOTHHjtwUbUPDMXZg="
                          },
                          "voting_power": "1132360259",
                          "proposer_priority": "0"
                        },
                        {
                          "address": "0UFme/pNlCinJRiXJoPJsrzv+D8=",
                          "pub_key": {
                            "ed25519": "lZ5HwN/UvaCujQ+CCOPMPT4KdjMbE40nHPUZZigPhdU="
                          },
                          "voting_power": "1001000000",
                          "proposer_priority": "0"
                        },
                        {
                          "address": "WMiqP+kYBTK3zpCeFUHYNeilx/c=",
                          "pub_key": {
                            "ed25519": "1gFzRLjXJ3h2Zh5w/zKR1QTk8OJzNJzzPKomoEv7FSM="
                          },
                          "voting_power": "960000000",
                          "proposer_priority": "0"
                        },
                        {
                          "address": "2CWo+eg1DUG2MpmJhbfIKakKyPU=",
                          "pub_key": {
                            "ed25519": "EUckv2MhqEQon5yx8AJB+zhCbchHNy8JzKg9z9i33NM="
                          },
                          "voting_power": "482932658",
                          "proposer_priority": "0"
                        }
                      ],
                      "proposer": {
                        "address": "9n24i8zM1BjO3KFYP1nUd358Qkc=",
                        "pub_key": {
                          "ed25519": "zHf1aWMC2+FJ0sZKrpkzZJrZndboHYU+SXpdTETJHjw="
                        },
                        "voting_power": "1000000151642671",
                        "proposer_priority": "0"
                      },
                      "total_voting_power": "2006798091740618"
                    },
                    "trusted_height": {
                      "revision_number": "0",
                      "revision_height": "129795"
                    },
                    "trusted_validators": {
                      "validators": [
                        {
                          "address": "MqxYSx8e8J5KfJhdEPx7fE9QdBo=",
                          "pub_key": {
                            "ed25519": "7E4zgCld0cEy/rPSPVxgrmhRpd9i3MZYubDENIDn9s4="
                          },
                          "voting_power": "1000000629828201",
                          "proposer_priority": "0"
                        },
                        {
                          "address": "9n24i8zM1BjO3KFYP1nUd358Qkc=",
                          "pub_key": {
                            "ed25519": "zHf1aWMC2+FJ0sZKrpkzZJrZndboHYU+SXpdTETJHjw="
                          },
                          "voting_power": "1000000151642671",
                          "proposer_priority": "0"
                        },
                        {
                          "address": "1IfoUOU/RCzuVTrVxogtjLEAjAI=",
                          "pub_key": {
                            "ed25519": "H1wT4w+RmPZLZlsCTGpGsAqwR2HQXdMltZPD+Yp51AE="
                          },
                          "voting_power": "1002271334472",
                          "proposer_priority": "0"
                        },
                        {
                          "address": "ObkX3rF1mfAmdu34UwNEQs693HA=",
                          "pub_key": {
                            "ed25519": "x24zpuAItEydzwx0O4pWr6rcl9VSJOQe4x/+fXCIJvw="
                          },
                          "voting_power": "1001118675147",
                          "proposer_priority": "0"
                        },
                        {
                          "address": "E8UEI7fPbvfrfGnzyWQERQf4ggA=",
                          "pub_key": {
                            "ed25519": "UtzGY2QSMu+WpHE60aFmMp23fB6EjPDUigEAISuhwQA="
                          },
                          "voting_power": "1000652206621",
                          "proposer_priority": "0"
                        },
                        {
                          "address": "v+JL/Gmngr0cucdvHEXdig1Iiac=",
                          "pub_key": {
                            "ed25519": "w3GrmfcSc21FN6kgXBTJGAbHsssAfEJovTlqdokCLgQ="
                          },
                          "voting_power": "1000092340182",
                          "proposer_priority": "0"
                        },
                        {
                          "address": "IjaQuC2moyuX0IktcyKqx9gjSdk=",
                          "pub_key": {
                            "ed25519": "yHFP2E0Sq+DgaU6t5ECkbzrFgLXzVQl9loNyRSDz6vI="
                          },
                          "voting_power": "901373125993",
                          "proposer_priority": "0"
                        },
                        {
                          "address": "C2ZUQaiw+rsRjaeQB/qzGcZ023o=",
                          "pub_key": {
                            "ed25519": "WIM+rLHTwdiompzxmQtkU0maAleTbjnpBsPld/Szj/U="
                          },
                          "voting_power": "901368172274",
                          "proposer_priority": "0"
                        },
                        {
                          "address": "bL9+uiAAx9th8WZjX6CATBoK6YE=",
                          "pub_key": {
                            "ed25519": "KowppNl2MWhlRRiSc60yJTo1+lWxcYZKkNREOg4Jkak="
                          },
                          "voting_power": "900411524672",
                          "proposer_priority": "0"
                        },
                        {
                          "address": "Pb9sz13h4cjN3Cr++VOGwudJYo4=",
                          "pub_key": {
                            "ed25519": "GVGaxBYNLjlvF4xA2rjKAB5rlYCAyzsu54VglOsxU74="
                          },
                          "voting_power": "60078525773",
                          "proposer_priority": "0"
                        },
                        {
                          "address": "4gP+naCq6BxO/PRNP/nYfxfP7f8=",
                          "pub_key": {
                            "ed25519": "KfX42WhjeCHvuNNxNMw4jeDcuMQkcVcn0JIrn7m/LtQ="
                          },
                          "voting_power": "11269774694",
                          "proposer_priority": "0"
                        },
                        {
                          "address": "6u8X+z7QBOixPPzigsVaD3A7B3M=",
                          "pub_key": {
                            "ed25519": "NdqOCl+OTJHOpI5ZF5R9com85mG6vUx4PpOm9QhSer4="
                          },
                          "voting_power": "5865749029",
                          "proposer_priority": "0"
                        },
                        {
                          "address": "cyU+oQY8ehj+NdWYwgu3U7zIaPo=",
                          "pub_key": {
                            "ed25519": "cq+A/KQ6FyHeFUcOai6aLUfVqYQa9ZIa97/aNIeS0VE="
                          },
                          "voting_power": "5500000000",
                          "proposer_priority": "0"
                        },
                        {
                          "address": "pzqlOj0YgjuBHnR2ssj+FX1clp4=",
                          "pub_key": {
                            "ed25519": "6+H/hnbjGcGSVyUTVN6adHySuHFJZnD02fDH0I0iZ48="
                          },
                          "voting_power": "2162949698",
                          "proposer_priority": "0"
                        },
                        {
                          "address": "j1XHJ4peCgY77cOBziBEvJI8Oks=",
                          "pub_key": {
                            "ed25519": "2LBk/UIV876UTql7/FbZnd8NDJLWku5ZJ2ypKjHh9vU="
                          },
                          "voting_power": "1569598274",
                          "proposer_priority": "0"
                        },
                        {
                          "address": "I/9Mv+/yelKVM78r8LdW1Rf5KDc=",
                          "pub_key": {
                            "ed25519": "Hql7wExdUHsSs/X1mxQjnGmXeWQOTHHjtwUbUPDMXZg="
                          },
                          "voting_power": "1132360259",
                          "proposer_priority": "0"
                        },
                        {
                          "address": "0UFme/pNlCinJRiXJoPJsrzv+D8=",
                          "pub_key": {
                            "ed25519": "lZ5HwN/UvaCujQ+CCOPMPT4KdjMbE40nHPUZZigPhdU="
                          },
                          "voting_power": "1001000000",
                          "proposer_priority": "0"
                        },
                        {
                          "address": "WMiqP+kYBTK3zpCeFUHYNeilx/c=",
                          "pub_key": {
                            "ed25519": "1gFzRLjXJ3h2Zh5w/zKR1QTk8OJzNJzzPKomoEv7FSM="
                          },
                          "voting_power": "960000000",
                          "proposer_priority": "0"
                        },
                        {
                          "address": "2CWo+eg1DUG2MpmJhbfIKakKyPU=",
                          "pub_key": {
                            "ed25519": "EUckv2MhqEQon5yx8AJB+zhCbchHNy8JzKg9z9i33NM="
                          },
                          "voting_power": "482932658",
                          "proposer_priority": "0"
                        }
                      ],
                      "proposer": {
                        "address": "MqxYSx8e8J5KfJhdEPx7fE9QdBo=",
                        "pub_key": {
                          "ed25519": "7E4zgCld0cEy/rPSPVxgrmhRpd9i3MZYubDENIDn9s4="
                        },
                        "voting_power": "1000000629828201",
                        "proposer_priority": "0"
                      },
                      "total_voting_power": "2006798091740618"
                    }
                  },
                  "signer": "osmo1j73g96rdw2vlwvkuu733tcejzyvhkp4nlsdptg"
                }
              ],
              "memo": "Relayed by anodeofzen! | hermes 1.10.0+b3d458d (https://hermes.informal.systems)",
              "timeout_height": "0",
              "extension_options": [],
              "non_critical_extension_options": []
            },
            "auth_info": {
              "signer_infos": [
                {
                  "public_key": {
                    "@type": "/cosmos.crypto.secp256k1.PubKey",
                    "key": "AkSGe/9QaBYOcUxivUAlYQfTsz4XArQpCF3sh2v7bsbo"
                  },
                  "mode_info": {
                    "single": {
                      "mode": "SIGN_MODE_DIRECT"
                    }
                  },
                  "sequence": "3927"
                }
              ],
              "fee": {
                "amount": [
                  {
                    "denom": "uosmo",
                    "amount": "19283"
                  }
                ],
                "gas_limit": "192824",
                "payer": "",
                "granter": ""
              },
              "tip": null
            },
            "signatures": [
              "pnPaX3Kr4JA5XMTb02PF0/+ChNg+7rLaYfS/zW9ZbV9uWwLHMuMNz4AYZhtftPNkFglxu55jzqqeBN+LdJtB8A=="
            ]
          },
          "timestamp": "2024-08-05T05:56:07Z",
          "events": []
        }
      }
    "#.to_string()
}

#[test]
pub fn test() {
    let public_key_bytes = vec![2,244,211,246,208,6,119,55,46,52,239,207,151,152,143,4,205,148,37,126,72,103,37,205,171,29,228,80,245,104,131,219,109];
    dbg!(&bytes_to_base64(&public_key_bytes));
    let tendermint_public_key = tendermint::public_key::PublicKey::from_raw_secp256k1(
        public_key_bytes.as_slice(),
    )
    .unwrap();
    dbg!(&tendermint_public_key);
    dbg!(&tendermint_public_key.to_hex());
    let sender_public_key_from_tendermint = cosmrs::crypto::PublicKey::from(tendermint_public_key);

    dbg!(&sender_public_key_from_tendermint);
}

#[test]
pub fn test_serde() {
    let public_key_bytes = vec![2,244,211,246,208,6,119,55,46,52,239,207,151,152,143,4,205,148,37,126,72,103,37,205,171,29,228,80,245,104,131,219,109];
    dbg!(&bytes_to_base64(&public_key_bytes));
    let tendermint_public_key = tendermint::public_key::PublicKey::from_raw_secp256k1(
        public_key_bytes.as_slice(),
    )
    .unwrap();

    // tendermint_public_key.to_bech32(hrp)

    let s = serde_json::to_string(&tendermint_public_key).unwrap();
    dbg!(&s);
}

#[test]
pub fn test_de() {
    let s =r#"{\"type\":\"tendermint/PubKeySecp256k1\",\"value\":\"AvTT9tAGdzcuNO/Pl5iPBM2UJX5IZyXNqx3kUPVog9tt\"}"#;
    let public_key = serde_json::from_str::<tendermint::public_key::PublicKey>(s).unwrap();
    dbg!(&public_key);

}