use crate::state::{audit, read_state};
use crate::{hub, ICP_TRANSFER_FEE};
use candid::{CandidType, Deserialize, Nat, Principal};
use ic_cdk::caller;
use ic_ledger_types::{AccountIdentifier, Tokens, DEFAULT_SUBACCOUNT, MAINNET_LEDGER_CANISTER_ID};
use icrc_ledger_client_cdk::{CdkRuntime, ICRC1Client};
use icrc_ledger_types::icrc1::account::{Account, Subaccount};
use icrc_ledger_types::icrc2::transfer_from::{TransferFromArgs, TransferFromError};
use num_traits::cast::ToPrimitive;
use omnity_types::{ChainId, ChainState, Ticket, TxAction};
use serde::Serialize;

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
    // the index of the burn block on the ledger
    pub block_index: u64,
}

#[derive(CandidType, Clone, Debug, Deserialize, PartialEq, Eq)]
pub enum GenerateTicketError {
    UnsupportedToken(String),

    UnsupportedChainId(String),

    /// The withdrawal account does not hold the requested token amount.
    InsufficientFunds {
        balance: u64,
    },

    /// The caller didn't approve enough funds for spending.
    InsufficientAllowance {
        allowance: u64,
    },

    SendTicketErr(String),

    TemporarilyUnavailable(String),

    InsufficientRedeemFee {
        lack_balance: u64,
    },

    QueryBalanceError,

    TransferFailure,
}

pub async fn generate_ticket(
    req: GenerateTicketReq,
) -> Result<GenerateTicketOk, GenerateTicketError> {
    charge_redeem_fee(caller(), &req.target_chain_id).await?;

    if !read_state(|s| {
        s.counterparties
            .get(&req.target_chain_id)
            .is_some_and(|c| c.chain_state == ChainState::Active)
    }) {
        return Err(GenerateTicketError::UnsupportedChainId(
            req.target_chain_id.clone(),
        ));
    }

    let ledger_id = read_state(|s| match s.token_ledgers.get(&req.token_id) {
        Some(ledger_id) => Ok(ledger_id.clone()),
        None => Err(GenerateTicketError::UnsupportedToken(req.token_id.clone())),
    })?;

    let caller = ic_cdk::caller();
    let user = Account {
        owner: caller,
        subaccount: req.from_subaccount,
    };

    let block_index = burn_token_icrc2(ledger_id, user, req.amount).await?;

    let (hub_principal, chain_id) = read_state(|s| (s.hub_principal, s.chain_id.clone()));
    hub::send_ticket(
        hub_principal,
        Ticket {
            ticket_id: block_index.to_string(),
            ticket_time: ic_cdk::api::time(),
            src_chain: chain_id,
            dst_chain: req.target_chain_id.clone(),
            action: TxAction::Redeem,
            token: req.token_id.clone(),
            amount: req.amount.to_string(),
            sender: None,
            receiver: req.receiver.clone(),
            memo: None,
        },
    )
    .await
    .map_err(|err| GenerateTicketError::SendTicketErr(format!("{}", err)))?;

    audit::finalize_gen_ticket(block_index, req);
    Ok(GenerateTicketOk { block_index })
}

async fn burn_token_icrc2(
    ledger_id: Principal,
    user: Account,
    amount: u128,
) -> Result<u64, GenerateTicketError> {
    let client = ICRC1Client {
        runtime: CdkRuntime,
        ledger_canister_id: ledger_id,
    };
    let route = ic_cdk::id();
    let result = client
        .transfer_from(TransferFromArgs {
            spender_subaccount: None,
            from: user,
            to: Account {
                owner: route,
                subaccount: None,
            },
            amount: Nat::from(amount),
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
        Ok(block_index) => Ok(block_index.0.to_u64().expect("nat does not fit into u64")),
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

async fn charge_redeem_fee(
    charge_from: Principal,
    chain_id: &ChainId,
) -> Result<(), GenerateTicketError> {
    let redeem_fee = read_state(|s| {
        s.redeem_fees
            .get(chain_id)
            .expect("unreachable: redeem fee not found")
            .clone()
            .factor
            * (Tokens::SUBDIVIDABLE_BY as i64)
    }) as u64;

    let subaccount = principal_to_subaccount(&charge_from);

    let account_identifier = AccountIdentifier::new(&ic_cdk::api::id(), &subaccount);

    let balance_args = ic_ledger_types::AccountBalanceArgs { account: account_identifier };

    let balance = ic_ledger_types::account_balance(MAINNET_LEDGER_CANISTER_ID, balance_args)
        .await
        .map_err(|_| GenerateTicketError::QueryBalanceError)?;

    if balance.e8s() < redeem_fee + ICP_TRANSFER_FEE {
        return Err(GenerateTicketError::InsufficientRedeemFee { lack_balance: redeem_fee + ICP_TRANSFER_FEE - balance.e8s() });
    }

    let transfer_args = ic_ledger_types::TransferArgs {
        memo: ic_ledger_types::Memo(0),
        amount: Tokens::from_e8s(redeem_fee),
        fee: Tokens::from_e8s(ICP_TRANSFER_FEE),
        from_subaccount: Some(subaccount.clone()),
        to: AccountIdentifier::new(&ic_cdk::api::id(), &DEFAULT_SUBACCOUNT),
        created_at_time: None,
    };

    ic_ledger_types::transfer(MAINNET_LEDGER_CANISTER_ID, transfer_args)
    .await
    .map_err(|_| GenerateTicketError::TransferFailure)?
    .map_err(|_| GenerateTicketError::TransferFailure)?;

    Ok(())

}

pub fn principal_to_subaccount(principal_id: &Principal) -> ic_ledger_types::Subaccount {
    let mut subaccount = [0; std::mem::size_of::<Subaccount>()];
    let principal_id = principal_id.as_slice();
    subaccount[0] = principal_id.len().try_into().unwrap();
    subaccount[1..1 + principal_id.len()].copy_from_slice(principal_id);

    ic_ledger_types::Subaccount(subaccount)
}