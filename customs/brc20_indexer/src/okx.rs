use candid::Deserialize;
use serde::Serialize;

pub const BASE_URL: &str = "https://www.oklink.com";
pub const URI: &str = "/api/v5/explorer/btc/transaction-list";



#[derive(Clone, Debug, Eq, PartialEq, Default, Serialize, Deserialize)]
struct CommonResponse<T> {
    pub code: i32,
    pub msg: String,
    pub data: T
}

struct PageInfo<T> {
    pub page: String,
    pub limit: String,
    pub totalPage: String,
    pub totalTransaction: String,
    pub InscriptionsList: T,
}
struct Brc20TransferEvent {
    pub txId: String,
    pub blockHeight: String,
    pub state: String,
    pub tokenType: String,
    pub actionType: String,
    pub fromAddress: String,
    pub toAddress: String,
    pub amount: String,
    pub token: String,
    pub inscriptionId:String,
    pub inscriptionNumber: String,
    pub index: String,
    pub location: String,
    pub msg: String,
    pub time: String,
}