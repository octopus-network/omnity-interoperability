use crate::state::{mutate_state, read_state};
use candid::{CandidType, Deserialize, Nat, Principal};
use icrc_ledger_client_cdk::{CdkRuntime, ICRC1Client};
use icrc_ledger_types::icrc1::{
    account::Account,
    transfer::{TransferArg, TransferError},
};
use num_traits::cast::ToPrimitive;
use omnity_types::TicketId;
use serde::Serialize;

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct MintTokenRequest {
    pub ticket_id: TicketId,
    pub token_id: String,
    /// The owner of the account on the ledger.
    pub receiver: Account,
    pub amount: u128,
}

#[derive(CandidType, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum MintTokenError {
    UnsupportedToken(String),

    AlreadyProcessed(TicketId),

    TemporarilyUnavailable(String),
}

pub enum ErrorCode {
    ConfigurationError = 1,
}

impl From<TransferError> for MintTokenError {
    fn from(e: TransferError) -> Self {
        Self::TemporarilyUnavailable(format!("failed to mint tokens on the ledger: {:?}", e))
    }
}

pub async fn mint_token(req: &MintTokenRequest) -> Result<(), MintTokenError> {
    if read_state(|s| s.finalized_mint_token_requests.contains_key(&req.ticket_id)) {
        return Err(MintTokenError::AlreadyProcessed(req.ticket_id.clone()));
    }

    let ledger_id = read_state(|s| match s.tokens.get(&req.token_id) {
        Some((_, ledger_id)) => Ok(ledger_id.clone()),
        None => Err(MintTokenError::UnsupportedToken(req.token_id.clone())),
    })?;

    let block_index = mint(ledger_id, req.amount, req.receiver).await?;

    mutate_state(|s| {
        s.finalized_mint_token_requests
            .insert(req.ticket_id.clone(), block_index)
    });
    Ok(())
}

async fn mint(ledger_id: Principal, amount: u128, to: Account) -> Result<u64, MintTokenError> {
    let client = ICRC1Client {
        runtime: CdkRuntime,
        ledger_canister_id: ledger_id,
    };
    let block_index = client
        .transfer(TransferArg {
            from_subaccount: None,
            to,
            fee: None,
            created_at_time: None,
            memo: None,
            amount: Nat::from(amount),
        })
        .await
        .map_err(|(code, msg)| {
            MintTokenError::TemporarilyUnavailable(format!(
                "cannot mint token: {} (reject_code = {})",
                msg, code
            ))
        })??;
    Ok(block_index.0.to_u64().expect("nat does not fit into u64"))
}
