use std::clone;

use crate::state::{audit, mutate_state, read_state};
use crate::{hub, ICP_TRANSFER_FEE};
use crate::{log, ERROR};
use candid::{CandidType, Deserialize, Nat, Principal};
use ic_crypto_sha2::Sha256;
use ic_ledger_types::{
    AccountIdentifier, Subaccount as IcSubaccount, Tokens, DEFAULT_SUBACCOUNT,
    MAINNET_LEDGER_CANISTER_ID,
};
use icrc_ledger_client_cdk::{CdkRuntime, ICRC1Client};
use icrc_ledger_types::icrc1::account::{Account, Subaccount};
use icrc_ledger_types::icrc2::transfer_from::{TransferFromArgs, TransferFromError};
use num_traits::cast::ToPrimitive;
use omnity_types::ic_log::INFO;
use omnity_types::{ChainId, ChainState, Memo, Ticket, TxAction};
use serde::Serialize;

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct GenerateTicketReq {
    pub target_chain_id: String,
    pub receiver: String,
    pub token_id: String,
    pub amount: u128,
    // The subaccount to burn token from.
    pub from_subaccount: Option<Subaccount>,
    pub action: TxAction,
}

#[derive(CandidType, Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct GenerateTicketOk {
    pub ticket_id: String,
}

#[derive(CandidType, Clone, Debug, Deserialize, PartialEq, Eq)]
pub enum GenerateTicketError {
    TemporarilyUnavailable(String),
    InvalidTicketAmount(u128),
    UnsupportedToken(String),
    UnsupportedChainId(String),
    /// The redeem account does not hold the requested token amount.
    InsufficientFunds {
        ledger_id: Principal,
        balance: Nat,
    },
    /// The caller didn't approve enough funds for spending.
    InsufficientAllowance {
        allowance: u64,
    },
    SendTicketErr(String),
    InsufficientRedeemFee {
        required: u64,
        provided: u64,
    },
    RedeemFeeNotSet,
    TransferFailure(String),
    UnsupportedAction(String),
    TokenNotFound(String),
}

pub async fn generate_ticket(
    req: GenerateTicketReq,
    is_charge_icp_fee_by_icrc: bool,
) -> Result<GenerateTicketOk, GenerateTicketError> {
    if read_state(|s| s.chain_state == ChainState::Deactive) {
        return Err(GenerateTicketError::TemporarilyUnavailable(
            "chain state is deactive!".into(),
        ));
    }

    if !read_state(|s| {
        s.counterparties
            .get(&req.target_chain_id)
            .is_some_and(|c| c.chain_state == ChainState::Active)
    }) {
        return Err(GenerateTicketError::UnsupportedChainId(
            req.target_chain_id.clone(),
        ));
    }

    if req.amount <= 0 {
        return Err(GenerateTicketError::InvalidTicketAmount(req.amount));
    }

    let ledger_id = read_state(|s| match s.token_ledgers.get(&req.token_id) {
        Some(ledger_id) => Ok(ledger_id.clone()),
        None => Err(GenerateTicketError::UnsupportedToken(req.token_id.clone())),
    })?;

    let caller = ic_cdk::caller();
    let user = Account {
        owner: caller.clone(),
        subaccount: req.from_subaccount,
    };

    ensure_balance_enough(&req).await?;

    let ticket_id = match req.action {
        TxAction::Mint => {
            let ledger_id = ic_cdk::id().to_string();
            let ticket_id =
                Sha256::hash(format!("MINT_{}_{}", ledger_id, ic_cdk::api::time()).as_bytes());

            Ok(hex::encode(&ticket_id))
        }
        TxAction::Burn
        | TxAction::Redeem
        | TxAction::RedeemIcpChainKeyAssets(_)
        | TxAction::Transfer => {
            let block_index = burn_token_icrc2(ledger_id, user, req.amount).await?;
            let ticket_id = format!("{}_{}", ledger_id.to_string(), block_index.to_string());
            Ok(ticket_id)
        }
    }?;

    log!(INFO, "successfully get ticket_id: {:?}", ticket_id);

    let charge_icp_result = if is_charge_icp_fee_by_icrc {
        charge_icp_fee_by_icrc(user.clone(), &req.target_chain_id).await
    } else {
        charge_icp_fee(caller, &req.target_chain_id).await
    };

    match charge_icp_result {
        Ok(block_index) => {
            log!(
                INFO,
                "successfully charged icp fee, block_index:{}",
                block_index
            );
        }
        Err(err) => {
            // just record log, not block process
            log!(ERROR, "Failed to charge icp fee: {:?}", err);
        }
    }

    let (hub_principal, chain_id) = read_state(|s| (s.hub_principal, s.chain_id.clone()));
    let action = req.action.clone();

    let fee = icp_get_redeem_fee(req.target_chain_id.clone());
    let memo_json = Memo {
        memo: None,
        bridge_fee: fee.unwrap_or_default() as u128,
    }
    .convert_to_memo_json()
    .unwrap_or_default();

    let ticket = Ticket {
        ticket_id: ticket_id.clone(),
        ticket_type: omnity_types::TicketType::Normal,
        ticket_time: ic_cdk::api::time(),
        src_chain: chain_id,
        dst_chain: req.target_chain_id.clone(),
        action,
        token: req.token_id.clone(),
        amount: req.amount.to_string(),
        sender: Some(caller.to_string()),
        receiver: req.receiver.clone(),
        memo: Some(memo_json.as_bytes().to_vec()),
    };

    let scope_guard = scopeguard::guard(
        ticket.clone(),
         |ticket| {
        log!(
            ERROR,
            "failed to send ticket trigger in scope_guard: {:?}",
            ticket.clone()
        );
        mutate_state(|s| {
            s.failed_tickets.push(ticket.clone());
        });
    }); 
    let r = match hub::send_ticket(hub_principal, ticket.clone()).await {
        Err(err) => {
            mutate_state(|s| {
                s.failed_tickets.push(ticket.clone());
            });
            log!(
                ERROR,
                "failed to send ticket: {}, err: {:?}",
                ticket_id,
                err
            );
            Err(GenerateTicketError::SendTicketErr(format!("{}", err)))
        }
        Ok(()) => {
            audit::finalize_gen_ticket(ticket_id.clone(), req);
            log!(INFO, "successfully send ticket: {}", ticket_id);
            Ok(GenerateTicketOk { ticket_id })
        }
    };
    scopeguard::ScopeGuard::into_inner(scope_guard);
    r
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
            ledger_id: ledger_id,
            balance: balance
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

pub async fn charge_icp_fee(
    from: Principal,
    chain_id: &ChainId,
) -> Result<Nat, GenerateTicketError> {
    let redeem_fee = read_state(|s| match s.target_chain_factor.get(chain_id) {
        Some(target_chain_factor) => s.fee_token_factor.map_or(
            Err(GenerateTicketError::RedeemFeeNotSet),
            |fee_token_factor| Ok((target_chain_factor * fee_token_factor) as u64),
        ),
        None => Err(GenerateTicketError::RedeemFeeNotSet),
    })?;

    let subaccount = principal_to_subaccount(&from);
    let ic_balance = ic_balance_of(&subaccount).await?;

    if ic_balance.e8s() < redeem_fee + ICP_TRANSFER_FEE {
        return Err(GenerateTicketError::InsufficientRedeemFee {
            required: redeem_fee + ICP_TRANSFER_FEE,
            provided: ic_balance.e8s(),
        });
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
        .map_err(|(_, reason)| GenerateTicketError::TemporarilyUnavailable(reason))?
        .map_err(|err| GenerateTicketError::TransferFailure(err.to_string()))
        .map(|block_index| block_index.into())
}

pub async fn ensure_balance_enough(req: &GenerateTicketReq) -> Result<(), GenerateTicketError> {
    let ledger_id = read_state(|s| s.token_ledgers.get(&req.token_id).cloned())
        .ok_or(GenerateTicketError::TokenNotFound(req.token_id.clone()))?;

    let icrc_client = ICRC1Client {
        runtime: CdkRuntime,
        ledger_canister_id: ledger_id,
    };

    let redeem_fee = read_state(|s| match s.target_chain_factor.get(&req.target_chain_id) {
        Some(target_chain_factor) => s.fee_token_factor.map_or(
            Err(GenerateTicketError::RedeemFeeNotSet),
            |fee_token_factor| Ok((target_chain_factor * fee_token_factor) as u64),
        ),
        None => Err(GenerateTicketError::RedeemFeeNotSet),
    })?;

    let (icp_balance_res, icrc_transfer_fee_res, icrc_balance_res) = futures::future::join3(
        async {
            ic_balance_of(
                &req.from_subaccount
                    .map(|e| IcSubaccount(e))
                    .unwrap_or(DEFAULT_SUBACCOUNT),
            )
            .await
        },
        async { icrc_client.fee().await },
        async {
            icrc_client
                .balance_of(Account {
                    owner: ic_cdk::caller(),
                    subaccount: req.from_subaccount.clone(),
                })
                .await
        },
    )
    .await;

    let icp_balance = icp_balance_res?;
    let icrc_transfer_fee = icrc_transfer_fee_res.map_err(|(code, msg)| {
        GenerateTicketError::TemporarilyUnavailable(format!(
            "cannot query icrc transfer fee: {} (reject_code = {})",
            msg, code
        ))
    })?;
    let icrc_balance = icrc_balance_res.map_err(|(code, msg)| {
        GenerateTicketError::TemporarilyUnavailable(format!(
            "cannot query icrc balance_of transaction: {} (reject_code = {})",
            msg, code
        ))
    })?;

    if icp_balance.e8s() < redeem_fee + ICP_TRANSFER_FEE {
        return Err(GenerateTicketError::InsufficientRedeemFee {
            required: redeem_fee + ICP_TRANSFER_FEE,
            provided: icp_balance.e8s(),
        });
    }

    if icrc_balance < Nat::from(req.amount) + icrc_transfer_fee {
        return Err(GenerateTicketError::InsufficientFunds {
            ledger_id,
            balance: icrc_balance,
        });
    }

    Ok(())
}

pub async fn charge_icp_fee_by_icrc(
    user: Account,
    chain_id: &ChainId,
) -> Result<Nat, GenerateTicketError> {
    let redeem_fee = read_state(|s| match s.target_chain_factor.get(chain_id) {
        Some(target_chain_factor) => s.fee_token_factor.map_or(
            Err(GenerateTicketError::RedeemFeeNotSet),
            |fee_token_factor| Ok((target_chain_factor * fee_token_factor) as u64),
        ),
        None => Err(GenerateTicketError::RedeemFeeNotSet),
    })?;

    let client = ICRC1Client {
        runtime: CdkRuntime,
        ledger_canister_id: MAINNET_LEDGER_CANISTER_ID,
    };
    let route = ic_cdk::id();

    client
        .transfer_from(TransferFromArgs {
            spender_subaccount: None,
            from: user,
            to: Account {
                owner: route,
                subaccount: None,
            },
            amount: Nat::from(redeem_fee),
            fee: None,
            memo: None,
            created_at_time: None,
        })
        .await
        .map_err(|(code, msg)| {
            GenerateTicketError::TemporarilyUnavailable(format!(
                "cannot enqueue a icp transferFrom transaction: {} (reject_code = {})",
                msg, code
            ))
        })?
        .map_err(|err| {
            GenerateTicketError::TransferFailure(format!("Failed to transferFrom icp {:?}", err))
        })
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

pub fn principal_to_subaccount(principal_id: &Principal) -> IcSubaccount {
    let mut subaccount = [0; std::mem::size_of::<IcSubaccount>()];
    let principal_id = principal_id.as_slice();
    subaccount[0] = principal_id.len().try_into().unwrap();
    subaccount[1..1 + principal_id.len()].copy_from_slice(principal_id);

    IcSubaccount(subaccount)
}

pub fn icp_get_redeem_fee(chain_id: ChainId) -> Option<u64> {
    read_state(|s| {
        s.target_chain_factor
            .get(&chain_id)
            // Add an additional transfer fee to make users bear the cost of transferring from route subaccount to route default account
            .map_or(None, |target_chain_factor| {
                s.fee_token_factor.map(|fee_token_factor| {
                    (target_chain_factor * fee_token_factor) as u64 + ICP_TRANSFER_FEE
                })
            })
    })
}
