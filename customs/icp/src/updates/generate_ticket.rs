use candid::{CandidType, Deserialize, Nat, Principal};
use icrc_ledger_client_cdk::{CdkRuntime, ICRC1Client};
use icrc_ledger_types::{
    icrc1::account::{Account, Subaccount},
    icrc2::transfer_from::{TransferFromArgs, TransferFromError},
};
use ic_ledger_types::{AccountIdentifier, BlockIndex, Subaccount as IcSubaccount, Tokens, DEFAULT_SUBACCOUNT, MAINNET_LEDGER_CANISTER_ID};
use num_traits::cast::ToPrimitive;
use omnity_types::{ Ticket, TxAction};
use serde::Serialize;
use ic_canister_log::log;
use omnity_types::ic_log::INFO;

use crate::{hub, state::{get_counterparty, get_token_principal, is_icp, read_state}, utils::convert_u128_u64, ICP_TRANSFER_FEE};

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct GenerateTicketReq {
    pub target_chain_id: String,
    pub receiver: String,
    pub token_id: String,
    pub amount: u128,
    // The subaccount to burn token from.
    pub from_subaccount: Option<Subaccount>,
}

#[derive(CandidType, Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct GenerateTicketOk {
    pub ticket_id: String,
}

#[derive(CandidType, Clone, Debug, Deserialize, PartialEq, Eq)]
pub enum GenerateTicketError {
    TemporarilyUnavailable(String),
    UnsupportedToken(String),
    UnsupportedChainId(String),
    /// The redeem account does not hold the requested token amount.
    InsufficientFunds {
        balance: u64,
    },
    InsufficientIcp {
        required: u64,
        provided: u64,
    },
    /// The caller didn't approve enough funds for spending.
    InsufficientAllowance {
        allowance: u64,
    },
    SendTicketErr(String),
    TransferIcpFailure(String),
    CustomError(String),
}

pub async fn refund_icp_from_subaccount(
    principal: Principal,
)->Result<(BlockIndex, u64), String>{
    let subaccount = IcSubaccount::from(principal); 
    let ic_balance = ic_balance_of(&subaccount).await.map_err(
        |err| format!("Failed to get ic balance: {:?}", err)
    )?;
    let transfer_args = ic_ledger_types::TransferArgs {
        memo: ic_ledger_types::Memo(0),
        amount: Tokens::from_e8s(ic_balance.e8s() - ICP_TRANSFER_FEE),
        fee: Tokens::from_e8s(ICP_TRANSFER_FEE),
        from_subaccount: Some(subaccount.clone()),
        to: AccountIdentifier::new(&principal, &DEFAULT_SUBACCOUNT),
        created_at_time: None,
    };
    let index = ic_ledger_types::transfer(MAINNET_LEDGER_CANISTER_ID, transfer_args)
        .await
        .map_err(
            |(_, reason)| format!("Failed to transfer icp: {:?}", reason)
        )?
        .map_err(
            |err| format!("Failed to transfer icp: {:?}", err)
        )?;

    log!(INFO, "Success to refund {} icp to {}", ic_balance, principal);
    Ok((index, ic_balance.e8s()))
}

pub async fn generate_ticket(
    req: GenerateTicketReq,
) -> Result<GenerateTicketOk, GenerateTicketError> {

    if get_counterparty(&req.target_chain_id).is_none() {
        return Err(GenerateTicketError::UnsupportedChainId(
            req.target_chain_id.clone(),
        ));
    }

    let (ticket_id, ticket_amount) = if is_icp(&req.token_id) {
        let (block_index, ticket_amount ) = lock_icp(ic_cdk::caller(), convert_u128_u64(req.amount)).await?;
        let ticket_id = format!("{}_{}", MAINNET_LEDGER_CANISTER_ID.to_string(), block_index.to_string());
        (ticket_id, ticket_amount as u128)
    } else {
        let ledger_id = get_token_principal(&req.token_id).ok_or(GenerateTicketError::UnsupportedToken(req.token_id.clone()))?;

        let user = Account {
            owner: ic_cdk::caller(),
            subaccount: req.from_subaccount,
        };
    
        let (block_index, ticket_amount) = burn_token_icrc2(ledger_id, user, req.amount).await?;
        let ticket_id = format!("{}_{}", ledger_id.to_string(), block_index.to_string());
        (ticket_id, ticket_amount)
    };

    let (hub_principal, chain_id) = read_state(|s| (s.hub_principal, s.chain_id.clone()));

    hub::send_ticket(
        hub_principal,
        Ticket {
            ticket_id: ticket_id.clone(),
            ticket_type: omnity_types::TicketType::Normal,
            ticket_time: ic_cdk::api::time(),
            src_chain: chain_id,
            dst_chain: req.target_chain_id.clone(),
            action: TxAction::Transfer,
            token: req.token_id.clone(),
            amount: ticket_amount.to_string(),
            sender: Some(ic_cdk::caller().to_text()),
            receiver: req.receiver.clone(),
            memo: None,
        },
    )
    .await
    .map_err(|err| GenerateTicketError::SendTicketErr(format!("{}", err)))?;
    log!(INFO, "Success to generate ticket: {}", ticket_id);
    Ok(GenerateTicketOk { ticket_id })
}

async fn ic_balance_of(subaccount: &IcSubaccount) -> Result<Tokens, GenerateTicketError> {
    let account_identifier = AccountIdentifier::new(&ic_cdk::api::id(), &subaccount);
    let balance_args = ic_ledger_types::AccountBalanceArgs {
        account: account_identifier,
    };
    ic_ledger_types::account_balance(MAINNET_LEDGER_CANISTER_ID, balance_args)
        .await
        .map_err(|(_, reason)| GenerateTicketError::TemporarilyUnavailable(reason))
}


async fn lock_icp(
    user: Principal,
    amount: u64
)->Result<(BlockIndex, u64), GenerateTicketError>{

    let subaccount =  IcSubaccount::from(user);
    let ic_balance = ic_balance_of(&subaccount).await?;

    if ic_balance.e8s() < amount + ICP_TRANSFER_FEE {
        return Err(GenerateTicketError::InsufficientIcp { 
            required: amount + ICP_TRANSFER_FEE,
            provided: ic_balance.e8s(),
        });
    }

    let transfer_args = ic_ledger_types::TransferArgs {
        memo: ic_ledger_types::Memo(0),
        amount: Tokens::from_e8s(amount),
        fee: Tokens::from_e8s(ICP_TRANSFER_FEE),
        from_subaccount: Some(subaccount.clone()),
        to: AccountIdentifier::new(&ic_cdk::api::id(), &DEFAULT_SUBACCOUNT),
        created_at_time: None,
    };

    let index = ic_ledger_types::transfer(MAINNET_LEDGER_CANISTER_ID, transfer_args)
        .await
        .map_err(|(_, reason)| GenerateTicketError::TemporarilyUnavailable(reason))?
        .map_err(|err| GenerateTicketError::TransferIcpFailure(err.to_string()))?;
    Ok((index, amount))
}

async fn burn_token_icrc2(
    ledger_id: Principal,
    user: Account,
    amount: u128,
) -> Result<(BlockIndex, u128), GenerateTicketError> {
    let client = ICRC1Client {
        runtime: CdkRuntime,
        ledger_canister_id: ledger_id,
    };
    let fee = client.fee()
    .await
    .map_err(|e| 
        GenerateTicketError::CustomError(
            format!("Failed to get icrc fee, error: {:?}", e).to_string(), 
    ))?;
    let route = ic_cdk::id();
    let transfer_amount = Nat::from(amount) - fee;
    let result = client
        .transfer_from(TransferFromArgs {
            spender_subaccount: None,
            from: user,
            to: Account {
                owner: route,
                subaccount: None,
            },
            amount: transfer_amount,
            fee: None,
            memo: None,
            created_at_time: Some(ic_cdk::api::time()),
        })
        .await
        .map_err(|(code, msg)| {
            GenerateTicketError::TemporarilyUnavailable(format!(
                "cannot enqueue a burn transaction: {} (reject_code = {})",
                msg, code
            ))
        })?;

    match result {
        Ok(block_index) => Ok((
            block_index.0
            .to_u64()
            .ok_or(
                GenerateTicketError::CustomError("block index does not fit into u64".to_string())
            )?, 
            amount
            .to_u128()
            .ok_or(
                GenerateTicketError::CustomError("amount does not fit into u64".to_string())
            )?
        )),
        Err(TransferFromError::InsufficientFunds { balance }) => Err(GenerateTicketError::InsufficientFunds {
            balance: balance.0.to_u64().expect("unreachable: ledger balance does not fit into u64")
        }),
        Err(TransferFromError::InsufficientAllowance { allowance }) => Err(GenerateTicketError::InsufficientAllowance {
            allowance: allowance.0.to_u64().expect("unreachable: ledger balance does not fit into u64")
        }),
        Err(TransferFromError::TemporarilyUnavailable) => {
            Err(GenerateTicketError::TemporarilyUnavailable(
                "cannot burn token: the ledger is busy".to_string(),
            ))
        }
        Err(TransferFromError::GenericError { error_code, message }) => {
            Err(GenerateTicketError::TemporarilyUnavailable(format!(
                "cannot burn token: the ledger fails with: {} (error code {})", message, error_code
            )))
        }
        Err(TransferFromError::BadFee { expected_fee }) => ic_cdk::trap(&format!(
            "unreachable: the ledger demands the fee of {} even though the fee field is unset",
            expected_fee
        )),
        Err(TransferFromError::Duplicate { duplicate_of }) => ic_cdk::trap(&format!(
            "unreachable: the ledger reports duplicate ({}) even though the create_at_time field is unset",
            duplicate_of
        )),
        Err(TransferFromError::CreatedInFuture {..}) => ic_cdk::trap(
            "unreachable: the ledger reports CreatedInFuture even though the create_at_time field is unset"
        ),
        Err(TransferFromError::TooOld) => ic_cdk::trap(
            "unreachable: the ledger reports TooOld even though the create_at_time field is unset"
        ),
        Err(TransferFromError::BadBurn { min_burn_amount }) => ic_cdk::trap(&format!(
            "the burn amount {} is less than ledger's min_burn_amount {}",
            amount,
            min_burn_amount
        )),
    }
}
