use crate::{
    state::{
        get_finalized_mint_token_request, get_token_principal, insert_finalized_mint_token_request,
        read_state,
    },
    utils::{convert_u128_u64, nat_to_u64},
    ICP_TRANSFER_FEE,
};
use candid::{CandidType, Deserialize, Nat, Principal};
use ic_ledger_types::{
    AccountIdentifier, Subaccount as IcSubaccount, Tokens, MAINNET_LEDGER_CANISTER_ID,
};
use icrc_ledger_client_cdk::{CdkRuntime, ICRC1Client};
use icrc_ledger_types::{
    icrc1::{
        account::{Account, Subaccount, DEFAULT_SUBACCOUNT},
        transfer::{TransferArg, TransferError},
    },
    icrc2::approve::ApproveArgs,
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

    CustomError(String),
}

pub enum ErrorCode {
    ConfigurationError = 1,
}

impl From<TransferError> for MintTokenError {
    fn from(e: TransferError) -> Self {
        Self::TemporarilyUnavailable(format!("failed to mint tokens on the ledger: {:?}", e))
    }
}

/// The arguments of the [retrieve_btc_with_approval] endpoint.
#[derive(CandidType, Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct RetrieveBtcWithApprovalArgs {
    // amount to retrieve in satoshi
    pub amount: u64,

    // address where to send bitcoins
    pub address: String,

    // The subaccount to burn ckBTC from.
    pub from_subaccount: Option<Subaccount>,
}

#[derive(CandidType, Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct RetrieveBtcOk {
    // the index of the burn block on the ckbtc ledger
    pub block_index: u64,
}

#[derive(CandidType, Clone, Debug, Deserialize, PartialEq, Eq)]
pub enum RetrieveBtcWithApprovalError {
    /// There is another request for this principal.
    AlreadyProcessing,

    /// The withdrawal amount is too low.
    AmountTooLow(u64),

    /// The bitcoin address is not valid.
    MalformedAddress(String),

    /// The withdrawal account does not hold the requested ckBTC amount.
    InsufficientFunds { balance: u64 },

    /// The caller didn't approve enough funds for spending.
    InsufficientAllowance { allowance: u64 },

    /// There are too many concurrent requests, retry later.
    TemporarilyUnavailable(String),

    /// A generic error reserved for future extensions.
    GenericError {
        error_message: String,
        /// See the [ErrorCode] enum above for the list of possible values.
        error_code: u64,
    },
}

pub const CKBTC_TRANSFER_FEE: u64 = 10;

pub async fn retrieve_ckbtc(
    receiver: String,
    amount: Nat,
    ticket_id: TicketId
) -> Result<u64, MintTokenError> {

    if get_finalized_mint_token_request(&ticket_id).is_some() {
        return Err(MintTokenError::AlreadyProcessed(ticket_id.clone()));
    }
    let ckbtc_ledger_principal = read_state(|s| s.ckbtc_ledger_principal.clone());
    let ckbtc_minter_principal = read_state(|s| s.ckbtc_minter_principal.clone()).ok_or(MintTokenError::CustomError("ckbtc_minter_principal not found".to_string()))?;
    let client = ICRC1Client {
        runtime: CdkRuntime,
        ledger_canister_id: ckbtc_ledger_principal,
    };
    let approve_args = ApproveArgs {
        from_subaccount: None,
        spender: Account {
            owner: ckbtc_minter_principal,
            subaccount: None,
        },
        amount: Nat::from(amount.clone()),
        expected_allowance: None,
        expires_at: None,
        fee: None,
        memo: None,
        created_at_time: None,
    };

    client
        .approve(approve_args)
        .await
        .map_err(|e| MintTokenError::CustomError(format!("{:?}", e)))?
        .map_err(|e| MintTokenError::CustomError(format!("{:?}", e)))?;

    let arg = RetrieveBtcWithApprovalArgs {
        amount: nat_to_u64(amount),
        address: receiver,
        from_subaccount: None,
    };

    let result: (std::result::Result<RetrieveBtcOk, RetrieveBtcWithApprovalError >,) =
        ic_cdk::call(ckbtc_minter_principal, "retrieve_btc_with_approval", (arg,))
            .await
            .map_err(|e| MintTokenError::CustomError(format!("{:?}", e)))?;
    let retrieve_result = result.0.map_err(|e| MintTokenError::CustomError(format!("{:?}", e)))?;
    insert_finalized_mint_token_request(ticket_id, retrieve_result.block_index);
    Ok(retrieve_result.block_index)
}

pub async fn unlock_icp(req: &MintTokenRequest) -> Result<u64, MintTokenError> {
    if get_finalized_mint_token_request(&req.ticket_id).is_some() {
        return Err(MintTokenError::AlreadyProcessed(req.ticket_id.clone()));
    }
    let transfer_args = ic_ledger_types::TransferArgs {
        memo: ic_ledger_types::Memo(0),
        amount: Tokens::from_e8s(convert_u128_u64(req.amount) - ICP_TRANSFER_FEE),
        fee: Tokens::from_e8s(ICP_TRANSFER_FEE),
        from_subaccount: None,
        to: AccountIdentifier::new(
            &req.receiver.owner,
            &IcSubaccount(
                req.receiver
                    .subaccount
                    .unwrap_or(DEFAULT_SUBACCOUNT.clone()),
            ),
        ),
        created_at_time: None,
    };
    let block_index = ic_ledger_types::transfer(MAINNET_LEDGER_CANISTER_ID, transfer_args)
        .await
        .map_err(|(_, reason)| MintTokenError::CustomError(reason))?
        .map_err(|err| MintTokenError::CustomError(err.to_string()))?;
    insert_finalized_mint_token_request(req.ticket_id.clone(), block_index);
    Ok(block_index)

}

pub async fn mint_token(req: &MintTokenRequest) -> Result<u64, MintTokenError> {
    if get_finalized_mint_token_request(&req.ticket_id).is_some() {
        return Err(MintTokenError::AlreadyProcessed(req.ticket_id.clone()));
    }
    let ledger_id = get_token_principal(&req.token_id)
        .ok_or(MintTokenError::UnsupportedToken(req.token_id.clone()))?;
    let block_index = mint(ledger_id, req.amount, req.receiver).await?;
    insert_finalized_mint_token_request(req.ticket_id.clone(), block_index);
    Ok(block_index)
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
