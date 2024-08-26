use std::str::FromStr;

use candid::{CandidType, Deserialize, Principal};
use omnity_types::Token;

use crate::state::{get_token, insert_token};

#[derive(CandidType, Clone, Debug, Deserialize, PartialEq, Eq)]
pub enum AddNewTokenError {
    AlreadyAdded(String),
    LedgerIdNotSpecified,
    InvalidLedgerId(String),
}

pub async fn add_new_token(token: Token) -> Result<(), AddNewTokenError> {
    if get_token(&token.token_id).is_some() {
        return Err(AddNewTokenError::AlreadyAdded(token.token_id));
    }

    let ledger_id = match token.metadata.get("ledger_id") {
        Some(ledger_id) => Ok(ledger_id.clone()),
        None => Err(AddNewTokenError::LedgerIdNotSpecified),
    }?;

    let principal = Principal::from_str(&ledger_id)
        .map_err(|_| AddNewTokenError::InvalidLedgerId(ledger_id))?;

    insert_token(token, principal);
    Ok(())
}
