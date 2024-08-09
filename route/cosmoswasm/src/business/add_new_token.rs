use cw::client::CosmosWasmClient;
use ic_cdk::api::management_canister::http_request::HttpResponse;
use schnorr::cw_schnorr_public_key;
use state::get_contract_id;

use crate::*;

// const CHAIN_ID: &str = "localosmosis";
// const RPC_PORT: u16 = 26657;
const ACCOUNT_PREFIX: &str = "osmo";
// const MEMO: &str = "test memo";
// const ACCOUNT_NUMBER: AccountNumber = 1;

pub async fn add_new_token(token: Token) -> Result<HttpResponse> {
    let msg = ExecuteMsg::ExecDirective {
        seq: 0,
        directive: Directive::AddToken {
            settlement_chain: "settlement_chain".to_string(),
            token_id: "token_id".to_string(),
            name: "token_name".to_string(),
        },
    };

    let rpc_url = state::read_state(|state| state.cw_rpc_url.clone());
    let rest_url = state::read_state(|state| state.cw_rest_url.clone());
    let chain_id = state::read_state(|state| state.chain_id.clone());
    let client = CosmosWasmClient::new(rpc_url, rest_url, chain_id);
    let contract_id = get_contract_id();

    let schnorr_public_key = cw_schnorr_public_key().await?;
    log::info!("schnorr_public_key: {:?}", schnorr_public_key);
    let tendermint_public_key = tendermint::public_key::PublicKey::from_raw_secp256k1(
        schnorr_public_key.public_key.as_slice(),
    )
    .unwrap();

    log::info!("tendermint_public_key: {:?}", tendermint_public_key);
    let sender_public_key = cosmrs::crypto::PublicKey::from(tendermint_public_key);
    let sender_account_id = sender_public_key.account_id(ACCOUNT_PREFIX).unwrap();

    log::info!("sender_public_key: {:?}", sender_public_key);

    log::info!("sender_account_id: {:?}", sender_account_id);

    client
        .execute_msg(
            contract_id,
            msg,
            sender_public_key,
            sender_account_id,
            SchnorrKeyIds::TestKey1.to_key_id(),
        )
        .await
}

// pub async fn build_tx_raw() -> Result<Raw> {
//     let sign_doc = build_sign_doc().await?;
//     let sign_doc_bytes = sign_doc.clone().into_bytes().unwrap();
//     let schnorr_canister_principal: candid::Principal =
//         state::read_state(|state| state.schnorr_canister_principal);
//     let derivation_path = state::read_state(|state| state.derivation_path);
//     let sign_result =
//         sign_by_schnorr_canister(sign_doc_bytes, derivation_path, schnorr_canister_principal)
//             .await?;
//     let raw: Raw = proto::cosmos::tx::v1beta1::TxRaw {
//         body_bytes: sign_doc.body_bytes.clone(),
//         auth_info_bytes: sign_doc.auth_info_bytes.clone(),
//         signatures: vec![sign_result.signature.to_vec()],
//     }
//     .into();

//     Ok(raw)
// }

// pub async fn build_sign_doc(chain_id: String) -> Result<SignDoc> {
//     let tx_body = build_tx_body().await?;
//     let auth_info = build_auth_info().await?;
//     let sign_doc = SignDoc::new(&tx_body, &auth_info, &chain_id, ACCOUNT_NUMBER).unwrap();
//     todo!()
// }

// pub async fn build_tx_body() -> Result<tx::Body> {

//     let tx_body = tx::BodyBuilder::new().msg(msg_execute).memo(MEMO).finish();
//     Ok(tx_body)
// }

// pub async fn build_auth_info() -> Result<tx::AuthInfo> {
//     todo!()
// }

// pub async fn add_new_token_tmp(token: Token) -> Result<()> {
//     log::info!("add_new_token: {:?}", token);
//     dbg!(&token);
//     let schnorr_canister_principal: candid::Principal =
//         state::read_state(|state| state.schnorr_canister_principal);

//     let derivation_path: Vec<ByteBuf> = [vec![1u8; 4]] // Example derivation path for signing
//         .iter()
//         .map(|v| ByteBuf::from(v.clone()))
//         .collect();

//     let public_arg = SchnorrPublicKeyArgs {
//         canister_id: Some(ic_cdk::api::id()),
//         derivation_path: derivation_path.clone(),
//         key_id: SchnorrKeyIds::TestKey1.to_key_id(),
//     };

//     let res: (Result<SchnorrPublicKeyResult, String>,) = ic_cdk::api::call::call(
//         schnorr_canister_principal,
//         "schnorr_public_key",
//         (public_arg,),
//     )
//     .await
//     .map_err(|(code, message)| {
//         RouteError::CallError(
//             "schnorr_public_key".to_string(),
//             schnorr_canister_principal,
//             code,
//             message,
//         )
//         format!(
//             "Error calling schnorr canister: code: {:?}, message: {:?}",
//             code, message
//         )
//         // ic_cdk::api::trap(format!("Error calling schnorr canister: code: {:?}, message: {:?}", code, message))
//     })?;
//     dbg!(&res);
//     log::info!("res: {:?}", res);
//     let schnorr_public_key = res.0.map_err(|err| {
//         err
//         // ic_cdk::api::trap(format!("Error calling schnorr canister: {:?}", err))
//     })?;

//     // // VerifyingKey::from_bytes(bytes)
//     // let verifying_key = k256::schnorr::VerifyingKey
//     let public = tendermint::public_key::PublicKey::from_raw_secp256k1(
//         schnorr_public_key.public_key.as_slice(),
//     )
//     .unwrap();
//     let sender_public_key = cosmrs::crypto::PublicKey::from(public);
//     let sender_account_id = sender_public_key.account_id(ACCOUNT_PREFIX).unwrap();

//     let contract_id = "osmo1cyyzpxplxdzkeea7kwsydadg87357qnahakaks"
//         .parse::<AccountId>()
//         .unwrap();

//     let execute_msg = ExecuteMsg::ExecDirective {
//         seq: 0,
//         directive: Directive::AddToken {
//             settlement_chain: "settlement_chain".to_string(),
//             token_id: "token_id".to_string(),
//             name: "token_name".to_string(),
//         },
//         signature: vec![],
//     };

//     let msg_execute = MsgExecuteContract {
//         sender: sender_account_id,
//         contract: contract_id,
//         msg: serde_json::to_string(&execute_msg).unwrap().into_bytes(),
//         funds: vec![],
//     }
//     .to_any()
//     .unwrap();

//     let chain_id: tendermint::chain::Id = CHAIN_ID.parse().unwrap();
//     let sequence_number = 0u64;
//     let gas = 100_000u64;
//     let amount = Coin {
//         amount: 1u8.into(),
//         denom: DENOM.parse().unwrap(),
//     };
//     let fee = Fee::from_amount_and_gas(amount, gas);

//     let tx_body = tx::BodyBuilder::new().msg(msg_execute).memo(MEMO).finish();
//     let auth_info =
//         SignerInfo::single_direct(Some(sender_public_key), sequence_number).auth_info(fee);
//     let sign_doc = SignDoc::new(&tx_body, &auth_info, &chain_id, ACCOUNT_NUMBER).unwrap();

//     let sign_doc_bytes = sign_doc.clone().into_bytes().unwrap();

//     let sign_result =
//         sign_by_schnorr_canister(sign_doc_bytes, derivation_path, schnorr_canister_principal)
//             .await?;

//     let raw: Raw = proto::cosmos::tx::v1beta1::TxRaw {
//         body_bytes: sign_doc.body_bytes.clone(),
//         auth_info_bytes: sign_doc.auth_info_bytes.clone(),
//         signatures: vec![sign_result.signature.to_vec()],
//     }
//     .into();

//     // raw.into

//     send_tx_raw_by_http_outcall(raw).await;

//     // let tx_raw = sign_doc.sign(&sender_private_key).unwrap();

//     // SignDoc::new(body, auth_info, chain_id, account_number)

//     Ok(())
// }

// async fn send_tx_raw_by_http_outcall(raw: Raw) {
//     let raw_bytes = raw.to_bytes().unwrap();
//     let raw_base64 = bytes_to_base64(&raw_bytes);

//     // let host = "http://localhost:{}";
//     let url = format!("http://localhost:{}", RPC_PORT);

//     let request_headers = vec![HttpHeader {
//         name: "content-type".to_string(),
//         value: "application/json".to_string(),
//     }];

//     let request_body = json!({
//         "jsonrpc": "2.0",
//         "method": "broadcast_tx_commit",
//         "params": {
//             "tx": raw_base64,
//         },
//         "id": Id::uuid_v4(),
//     });

//     log::info!("request_body: {:?}", request_body);
//     dbg!(request_body.to_string());

//     let request = CanisterHttpRequestArgument {
//         url: url.to_string(),
//         max_response_bytes: None, //optional for request
//         method: HttpMethod::POST,
//         headers: request_headers,
//         body: Some(request_body.to_string().into_bytes()),
//         transform: None,
//         // transform: None, //optional for request
//     };

//     let respone = http_request(request, 49_140_000).await;
//     dbg!(&respone);
// }

// use base64::{engine::general_purpose, Engine as _};
// use state::read_state;
// use tx::Body;

// async fn sign_by_schnorr_canister(
//     message: Vec<u8>,
//     derivation_path: Vec<ByteBuf>,
//     schnorr_canister_principal: candid::Principal,
// ) -> Result<SignWithSchnorrResult> {
//     let sign_with_schnorr_args = SignWithSchnorrArgs {
//         message: ByteBuf::from(message),
//         derivation_path,
//         key_id: SchnorrKeyIds::TestKey1.to_key_id(),
//     };

//     let res: (Result<SignWithSchnorrResult, String>,) = ic_cdk::api::call::call(
//         schnorr_canister_principal,
//         "sign_with_schnorr",
//         (sign_with_schnorr_args,),
//     )
//     .await
//     .map_err(|(code, message)| message)?;
//     res.0.map_err(|err| err)
// }
