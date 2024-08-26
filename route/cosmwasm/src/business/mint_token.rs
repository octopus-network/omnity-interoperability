use candid::CandidType;
use omnity_types::Account;

use crate::*;

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct MintTokenRequest {
    pub ticket_id: TicketId,
    pub token_id: String,
    /// The owner of the account on the ledger.
    pub receiver: Account,
    pub amount: u128,
}
