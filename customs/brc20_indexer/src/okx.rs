use candid::Deserialize;
use serde::Serialize;

pub const BASE_URL: &str = "https://www.oklink.com/api/v5/explorer/btc/transaction-list?txId=";

pub async fn query() {


}

#[derive(Clone, Debug, Eq, PartialEq, Default, Serialize, Deserialize)]
struct CommonResponse<T> {
    pub code: i32,
    pub msg: String,
    pub data: T
}

#[derive(Clone, Debug, Eq, PartialEq, Default, Serialize, Deserialize)]
struct PageInfo<T> {
    pub page: String,
    pub limit: String,
    #[serde(rename = "totalPage")]
    pub total_page: String,
    #[serde(rename = "totalTransaction")]
    pub total_transaction: String,
    #[serde(rename = "inscriptionsList")]
    pub inscriptions_list: T,
}

#[derive(Clone, Debug, Eq, PartialEq, Default, Serialize, Deserialize)]
struct Brc20TransferEvent {
    #[serde(rename = "txId")]
    pub tx_id: String,
    #[serde(rename = "blockHeight")]
    pub block_height: String,
    pub state: String,
    #[serde(rename = "tokenType")]
    pub token_type: String,
    #[serde(rename = "actionType")]
    pub action_type: String,
    #[serde(rename = "fromAddress")]
    pub from_address: String,
    #[serde(rename = "toAddress")]
    pub to_address: String,
    pub amount: String,
    pub token: String,
    #[serde(rename = "inscriptionId")]
    pub inscription_id:String,
    #[serde(rename = "inscriptionNumber")]
    pub inscription_number: String,
    pub index: String,
    pub location: String,
    pub msg: String,
    pub time: String,
}