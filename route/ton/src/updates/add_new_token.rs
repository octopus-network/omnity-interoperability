use crate::audit;
use crate::state::{mutate_state, read_state};
use candid::{CandidType, Deserialize};
use omnity_types::Token;

#[derive(CandidType, Clone, Debug, Deserialize, PartialEq, Eq)]
pub enum AddNewTokenError {
    AlreadyAdded(String),
    CreateLedgerErr(String),
}

pub async fn add_new_token(token: Token) -> Result<(), AddNewTokenError> {
    if read_state(|s| s.tokens.contains_key(&token.token_id)) {
        return Err(AddNewTokenError::AlreadyAdded(token.token_id));
    }

    mutate_state(|s| {
        audit::add_token(s, token);
    });
    Ok(())
}
