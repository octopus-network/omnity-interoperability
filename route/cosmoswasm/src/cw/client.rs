use ic_cdk::api::management_canister::http_request::HttpResponse;
use schnorr::{sign_with_schnorr, SchnorrKeyId};
use utils::bytes_to_base64;

use crate::*;

// const CHAIN_ID: &str = "localosmosis";
// const RPC_PORT: u16 = 26657;
// const OSMO_ACCOUNT_PREFIX: &str = "osmo";
const DENOM: &str = "uosmo";
const MEMO: &str = "test memo";
const ACCOUNT_NUMBER: AccountNumber = 96638;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CosmosWasmClient {
    pub url: String,
    pub chain_id: ChainId,
}

impl CosmosWasmClient {
    pub fn new(url: String, chain_id: ChainId) -> Self {
        Self { url, chain_id }
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
            url: self.url.clone(),
            max_response_bytes: None,
            method: HttpMethod::POST,
            headers: request_headers,
            body: Some(request_body.to_string().into_bytes()),
            transform: None,
            // transform: None, //optional for request
        };
        let respone = http_request(request, 100_000_000_000)
            .await
            .map_err(|(code, message)| {
                RouteError::HttpOutCallError(format!("{:?}", code).to_string(), message)
            })?;
        dbg!(&respone);

        Ok(respone.0)
    }

    pub async fn execute_msg(
        &self,
        contract_id: AccountId,
        msg: ExecuteMsg,
        sender_public_key: cosmrs::crypto::PublicKey,
        sender_account_id: AccountId,
        key_id: SchnorrKeyId,
    ) -> Result<HttpResponse> {
        let sequence_number = 0u64;
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
            SignerInfo::single_direct(Some(sender_public_key), sequence_number).auth_info(fee);

        let sign_doc = SignDoc::new(&tx_body, &auth_info, &self.chain_id, ACCOUNT_NUMBER).unwrap();

        let sign_result =
            sign_with_schnorr(&sign_doc.clone().into_bytes().unwrap(), key_id).await?;

        let raw: Raw = proto::cosmos::tx::v1beta1::TxRaw {
            body_bytes: sign_doc.body_bytes.clone(),
            auth_info_bytes: sign_doc.auth_info_bytes.clone(),
            signatures: vec![sign_result.signature.to_vec()],
        }
        .into();

        self.broadcast_tx_commit(raw).await
    }
}
