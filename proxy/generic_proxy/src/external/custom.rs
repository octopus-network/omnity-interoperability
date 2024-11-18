
use types::OmnityAccount;

use crate::*;

#[derive(CandidType, Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct GenerateTicketOk {
    pub ticket_id: TicketId,
}

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct GenerateTicketReq {
    pub target_chain_id: String,
    pub receiver: String,
    pub token_id: String,
    pub amount: u128,
    // The subaccount to burn token from.
    pub from_subaccount: Option<Subaccount>,
}

pub async fn generate_ticket(
    token_id: String,
    amount: u128,
    omnity_account: OmnityAccount,
) -> Result<TicketId> {
    let req = GenerateTicketReq {
        target_chain_id: omnity_account.chain_id.clone(),
        receiver: omnity_account.account.clone(),
        token_id: token_id,
        amount: amount,
        from_subaccount: Some(omnity_account.get_mapping_subaccount()),
    };

    let icp_custom = state::get_state().icp_customs_principal;
    let result: (Result<GenerateTicketOk>,) =
        ic_cdk::api::call::call(icp_custom, "generate_ticket", (req,))
            .await
            .map_err(|(code, msg)| {
                Errors::CanisterCallError(
                    icp_custom.to_string(),
                    "generate_ticket".to_string(),
                    format!("{:?}", code),
                    msg,
                )
            })?;

    Ok(result.0?.ticket_id)
}
