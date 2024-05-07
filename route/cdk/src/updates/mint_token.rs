use crate::state::{mutate_state, read_state};
use crate::audit;
use candid::{CandidType, Deserialize, Nat, Principal};
use crate::types::{TicketId, TokenId};
use serde::Serialize;

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct MintTokenRequest {
    pub ticket_id: TicketId,
    pub token_id: String,
    pub receiver: String,
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

/*impl From<TransferError> for MintTokenError {
    fn from(e: TransferError) -> Self {
        Self::TemporarilyUnavailable(format!("failed to mint tokens on the ledger: {:?}", e))
    }
}*/

pub async fn mint_token(req: &MintTokenRequest) -> Result<(), MintTokenError> {
    if read_state(|s| s.finalized_mint_token_requests.contains_key(&req.ticket_id)) {
        return Err(MintTokenError::AlreadyProcessed(req.ticket_id.clone()));
    }


   // let block_index = mint(ledger_id, req.amount, account).await?;

    mutate_state(|s| audit::finalize_mint_token_req(s, req.ticket_id.clone(), 0));
    Ok(())
}

async fn mint(token_id: TokenId, amount: u128, to: String) -> Result<u64, MintTokenError> {
   /* let client = ICRC1Client {
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
        })??;*/
    Ok(0)
    //Ok(block_index.0.to_u64().expect("nat does not fit into u64"))
}
