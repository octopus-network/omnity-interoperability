use crate::*;

pub const OSMO_ACCOUNT_PREFIX: &str = "osmo";
const DENOM: &str = "uosmo";
const MEMO: &str = "memo";

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

    pub fn cosmos_wasm_port_client() -> CosmosWasmClient {
        let (rpc_url, rest_url, chain_id) = memory::read_state(|state| {
            (
                state.cw_rpc_url.clone(),
                state.cw_rest_url.clone(),
                state.chain_id.clone(),
            )
        });
        let client = CosmosWasmClient::new(rpc_url, rest_url, chain_id);
        client
    }

    pub async fn query_account_number_and_sequence(
        &self,
        address: String,
    ) -> Result<(AccountNumber, u64)> {
        // eg: https://lcd.testnet.osmosis.zone/cosmos/auth/v1beta1/account_info/osmo1x6ctqf5fwy37tx9vdhh9y7kxk5puvwsdnl0acw
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

        log::info!("response: {:?}", response);

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
        };

        http_request_with_status_check(request).await
    }

    pub async fn query_tx_by_hash(&self, tx_hash: TxHash) -> Result<HttpResponse> {
        // https://rpc.testnet.osmosis.zone/tx?hash=0xFE14C9EAD18A6990FF426F4782894C1719A4A2C4B62D2F6B8A53AD945D7FFE34
        let request_url = format!("{}/tx?hash=0x{}", self.rpc_url, tx_hash);
        let request_headers = vec![HttpHeader {
            name: "content-type".to_string(),
            value: "application/json".to_string(),
        }];

        let request = CanisterHttpRequestArgument {
            url: request_url,
            max_response_bytes: None,
            method: HttpMethod::POST,
            headers: request_headers,
            body: None,
            transform: None,
        };

        http_request_with_status_check(request).await
    }

    pub async fn execute_msg(
        &self,
        contract_id: AccountId,
        msg: ExecuteMsg,
        tendermint_public_key: tendermint::public_key::PublicKey,
    ) -> Result<HttpResponse> {
        let sender_public_key = cosmrs::crypto::PublicKey::from(tendermint_public_key);
        let sender_account_id = sender_public_key.account_id(OSMO_ACCOUNT_PREFIX).unwrap();

        let (account_number, sequence) = self
            .query_account_number_and_sequence(sender_account_id.to_string())
            .await?;

        log::info!(
            "account_number: {:?}, sequence: {:?}",
            account_number,
            sequence
        );
        // let sequence_number = 0u64;
        let gas = 2_000_000u64;
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
        log::info!("tx_body: {:?}", tx_body);

        let auth_info = SignerInfo::single_direct(Some(sender_public_key), sequence).auth_info(fee);

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

        let sign_result = sign_with_cw_key(
            sign_doc
                .clone()
                .into_bytes()
                .expect("Sign doc into bytes failed"),
        )
        .await?;

        log::info!("sign_result: {:?}", sign_result);

        let raw: Raw = proto::cosmos::tx::v1beta1::TxRaw {
            body_bytes: sign_doc.body_bytes.clone(),
            auth_info_bytes: sign_doc.auth_info_bytes.clone(),
            signatures: vec![sign_result.signature.to_vec()],
        }
        .into();

        log::info!("raw: {:?}", raw);

        self.broadcast_tx_commit(raw).await
    }
}

pub async fn cw_chain_key_arg() -> EcdsaChainKeyArg {
    let test_key_local = EcdsaKeyIds::TestKeyLocalDevelopment.to_key_id();
    let cw_chain_key_derivation_path =
        memory::read_state(|state| state.cw_chain_key_derivation_path.clone());

    EcdsaChainKeyArg {
        derivation_path: cw_chain_key_derivation_path
            .iter()
            .map(|e| e.clone().into_vec())
            .collect(),
        key_id: EcdsaKeyId {
            curve: ic_cdk::api::management_canister::ecdsa::EcdsaCurve::Secp256k1,
            name: test_key_local.name,
        },
    }
}

pub async fn query_cw_public_key() -> Result<EcdsaPublicKeyResponse> {
    let key_arg = cw_chain_key_arg().await;

    let request = EcdsaPublicKeyArgument {
        canister_id: ic_cdk::api::id().into(),
        derivation_path: key_arg.derivation_path,
        key_id: key_arg.key_id,
    };

    let (response,) = ecdsa_public_key(request).await.map_err(|(code, msg)| {
        RouteError::CallError(
            "ecdsa_public_key".to_string(),
            Principal::management_canister(),
            format!("{:?}", code).to_string(),
            msg,
        )
    })?;

    Ok(response)
}

pub async fn sign_with_cw_key(message: Vec<u8>) -> Result<SignWithEcdsaResponse> {
    let key_arg = cw_chain_key_arg().await;
    let request = SignWithEcdsaArgument {
        message_hash: sha256(message).to_vec(),
        derivation_path: key_arg.derivation_path,
        key_id: key_arg.key_id,
    };

    let (response,) = sign_with_ecdsa(request)
        .await
        .map_err(|e| RouteError::SignWithEcdsaError(format!("{:?}", e.0), e.1))?;
    Ok(response)
}

#[test]
pub fn test() {
    let public_key_bytes = vec![
        2, 244, 211, 246, 208, 6, 119, 55, 46, 52, 239, 207, 151, 152, 143, 4, 205, 148, 37, 126,
        72, 103, 37, 205, 171, 29, 228, 80, 245, 104, 131, 219, 109,
    ];
    dbg!(&bytes_to_base64(&public_key_bytes));
    let tendermint_public_key =
        tendermint::public_key::PublicKey::from_raw_secp256k1(public_key_bytes.as_slice()).unwrap();
    dbg!(&tendermint_public_key);
    dbg!(&tendermint_public_key.to_hex());
    let sender_public_key_from_tendermint = cosmrs::crypto::PublicKey::from(tendermint_public_key);

    dbg!(&sender_public_key_from_tendermint);
}

#[test]
pub fn test_serde() {
    let public_key_bytes = vec![
        2, 244, 211, 246, 208, 6, 119, 55, 46, 52, 239, 207, 151, 152, 143, 4, 205, 148, 37, 126,
        72, 103, 37, 205, 171, 29, 228, 80, 245, 104, 131, 219, 109,
    ];
    dbg!(&bytes_to_base64(&public_key_bytes));
    let tendermint_public_key =
        tendermint::public_key::PublicKey::from_raw_secp256k1(public_key_bytes.as_slice()).unwrap();

    // tendermint_public_key.to_bech32(hrp)

    let s = serde_json::to_string(&tendermint_public_key).unwrap();
    dbg!(&s);
}

#[test]
pub fn test_de() {
    let s = r#"{\"type\":\"tendermint/PubKeySecp256k1\",\"value\":\"AvTT9tAGdzcuNO/Pl5iPBM2UJX5IZyXNqx3kUPVog9tt\"}"#;
    let public_key = serde_json::from_str::<tendermint::public_key::PublicKey>(s).unwrap();
    dbg!(&public_key);
}
