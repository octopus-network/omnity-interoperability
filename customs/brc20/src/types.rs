use std::str::FromStr;
use bitcoin::Amount;
use candid::{CandidType, Deserialize};
use ic_btc_interface::Txid;
use serde::Serialize;

use omnity_types::brc20::QueryBrc20TransferArgs;
use omnity_types::TokenId;
use crate::ord::builder::Utxo;

pub type Brc20Ticker = String;

#[derive(candid::CandidType, Clone, Debug, PartialEq, Eq, Deserialize)]
pub enum ReleaseTokenStatus {
    /// The custom has no data for this request.
    /// The request id is either invalid or too old.
    Unknown,
    /// The request is in the batch queue.
    Pending,
    /// Waiting for a signature on a transaction satisfy this request.
    Signing,
    /// Sending the transaction satisfying this request.
    Sending(String),
    /// Awaiting for confirmations on the transaction satisfying this request.
    Submitted(String),
    /// Confirmed a transaction satisfying this request.
    Confirmed(String),
}

#[derive(CandidType, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GenTicketStatus {
    /// The custom has no data for this request.
    /// The request is either invalid or too old.
    Unknown,
    /// The request is in the queue.
    Pending(LockTicketRequest),
    Confirmed(LockTicketRequest),
    Finalized(LockTicketRequest),
}

#[derive(candid::CandidType, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LockTicketRequest {
    pub target_chain_id: String,
    pub receiver: String,
    pub token_id: TokenId,
    pub ticker: Brc20Ticker,
    pub amount: String,
    pub txid: Txid,
    pub received_at: u64,
}

/// Unspent transaction output to be used as input of a transaction
#[derive(CandidType,Debug, Clone, Serialize, Deserialize)]
pub struct UtxoArgs {
    pub id: String,
    pub index: u32,
    pub amount: u64,
}

impl From<UtxoArgs> for Utxo {
    fn from(value: UtxoArgs) -> Self {
        Utxo {
            id: bitcoin::Txid::from_str(&value.id).unwrap(),
            index: value.index,
            amount: Amount::from_sat(value.amount),
        }
    }
}

pub fn create_query_brc20_transfer_args(
    gen_ticket_request: LockTicketRequest,
    deposit_addr: String,
    ticker_decimals: u8,
) -> QueryBrc20TransferArgs {
    QueryBrc20TransferArgs {
        tx_id: gen_ticket_request.txid.to_string(),
        ticker: gen_ticket_request.ticker,
        to_addr: deposit_addr,
        amt: gen_ticket_request.amount,
        decimals: ticker_decimals,
    }
}
