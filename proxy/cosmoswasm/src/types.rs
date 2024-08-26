use crate::*;
use subtle_encoding::bech32;

pub const TENDERMINT_ADDRESS_LENGTH: usize = 20;
pub type TicketId = String;

pub struct AddressData(pub [u8; 20]);

impl AddressData {
    pub const PREFIX: &'static str = "osmo";

    pub fn to_bytes(&self) -> &[u8; 20] {
        &self.0
    }

    pub fn to_cosmos_address(&self) -> String {
        bech32::encode(AddressData::PREFIX, self.0.to_vec())
    }
}

// https://github.com/tendermint/spec/blob/master/spec/core/encoding.md
impl TryFrom<&str> for AddressData {
    type Error = Errors;

    fn try_from(account_id: &str) -> Result<Self> {
        let a = bech32::decode(account_id)
            .map_err(|err| Errors::AccountIdParseError(account_id.to_string(), err.to_string()))?;
        let bytes: [u8; 20] = a.1.try_into().map_err(|_| {
            Errors::AccountIdParseError(
                account_id.to_string(),
                "Invalid address length".to_string(),
            )
        })?;
        Ok(AddressData(bytes))
    }
}

impl From<AddressData> for Subaccount {
    fn from(data: AddressData) -> Self {
        let mut subaccount_bytes = [0_u8; 32];
        subaccount_bytes[..20].copy_from_slice(data.to_bytes());
        // Subaccount(subaccount_bytes)
        subaccount_bytes
    }
}

impl From<Subaccount> for AddressData {
    fn from(value: Subaccount) -> Self {
        let mut bytes = [0_u8; 20];
        bytes.copy_from_slice(&value[..20]);
        AddressData(bytes)
    }
}

// pub mod index_ng {
//     use candid::Nat;

//     use candid::{CandidType, Deserialize, Principal};
//     use icrc_ledger_types::icrc1::account::{Account, Subaccount};
//     use icrc_ledger_types::icrc1::transfer::BlockIndex;
//     use icrc_ledger_types::icrc3::transactions::Transaction;

//     #[derive(CandidType, Clone, Debug, Deserialize, PartialEq, Eq)]
//     pub struct TransactionWithId {
//         pub id: BlockIndex,
//         pub transaction: Transaction,
//     }

//     #[derive(CandidType, Debug, Deserialize, PartialEq, Eq)]
//     pub struct GetAccountTransactionsError {
//         pub message: String,
//     }

//     #[derive(CandidType, Debug, Deserialize, PartialEq, Eq)]
//     pub struct GetAccountTransactionsResponse {
//         pub balance: Nat,
//         pub transactions: Vec<TransactionWithId>,
//         // The txid of the oldest transaction the account has
//         pub oldest_tx_id: Option<BlockIndex>,
//     }

//     pub type GetAccountTransactionsResult =
//         Result<GetAccountTransactionsResponse, GetAccountTransactionsError>;

//     #[derive(CandidType, Debug, Deserialize, PartialEq, Eq)]
//     pub struct GetAccountTransactionsArgs {
//         pub account: Account,
//         // The txid of the last transaction seen by the client.
//         // If None then the results will start from the most recent
//         // txid. If set then the results will start from the next
//         // most recent txid after start (start won't be included).
//         pub start: Option<BlockIndex>,
//         // Maximum number of transactions to fetch.
//         pub max_results: Nat,
//     }
// }
