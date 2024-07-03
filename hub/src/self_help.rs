use crate::{
    proposal::{execute_proposal, validate_proposal},
    state::{with_state, with_state_mut},
    types::{Proposal, TokenMeta},
};
use candid::{CandidType, Deserialize, Principal};
use ic_ledger_types::{
    account_balance, AccountBalanceArgs, AccountIdentifier, Memo, Subaccount, Tokens, TransferArgs,
    DEFAULT_SUBACCOUNT, MAINNET_LEDGER_CANISTER_ID,
};
use omnity_types::{rune_id::RuneId, ChainId};
use serde::Serialize;
use std::{collections::HashMap, str::FromStr};

const SELF_SERVICE_FEE: u64 = 1_000_000_000_000;
const ICP_TRANSFER_FEE: u64 = 10_000;
const BITCOIN_CHAIN: &str = "Bitcoin";

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct AddRunesTokenArgs {
    pub rune_id: String,
    pub symbol: String,
    pub icon: String,
}

#[derive(CandidType, Clone, Debug, Deserialize, PartialEq, Eq)]
pub enum SelfServiceError {
    TemporarilyUnavailable(String),
    InvalidRuneId(String),
    EmptyArgument,
    TokenAlreadyExisting,
    InsufficientFee { required: u64, provided: u64 },
    TransferFailure(String),
    RequestNotFound,
    InvalidProposal(String),
    TokenNotFound,
    ChainNotAvailable,
}

pub async fn add_runes_token(args: AddRunesTokenArgs) -> Result<(), SelfServiceError> {
    let _ = RuneId::from_str(&args.rune_id)
        .map_err(|e| SelfServiceError::InvalidRuneId(e.to_string()))?;

    if args.symbol.is_empty() || args.icon.is_empty() {
        return Err(SelfServiceError::EmptyArgument);
    }

    if with_state(|s| {
        s.tokens.iter().any(|(_, tokenmeta)| {
            tokenmeta.symbol == args.symbol
                || tokenmeta
                    .metadata
                    .get("rune_id")
                    .cloned()
                    .is_some_and(|rune_id| rune_id == args.rune_id)
        })
    }) {
        return Err(SelfServiceError::TokenAlreadyExisting);
    }

    charge_fee().await?;

    with_state_mut(|s| {
        s.add_runes_token_requests
            .insert(args.rune_id.clone(), args)
    });

    Ok(())
}

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct FinalizeAddRunesArgs {
    pub rune_id: String,
    pub name: String,
    pub decimal: u8,
    pub dst_chain: ChainId,
}

pub async fn finalize_add_runes_token_req(
    args: FinalizeAddRunesArgs,
) -> Result<(), SelfServiceError> {
    let request = with_state(|s| match s.add_runes_token_requests.get(&args.rune_id) {
        Some(req) => Ok(req.clone()),
        None => Err(SelfServiceError::RequestNotFound),
    })?;

    let token_meta = TokenMeta {
        token_id: format!("{}-{}", "Bitcoin-runes", args.name),
        name: args.name,
        symbol: request.symbol,
        issue_chain: BITCOIN_CHAIN.into(),
        decimals: args.decimal,
        icon: Some(request.icon),
        metadata: HashMap::from_iter(vec![("rune_id".to_string(), args.rune_id.clone())]),
        dst_chains: vec![BITCOIN_CHAIN.into(), args.dst_chain],
    };

    let proposal = vec![Proposal::AddToken(token_meta)];
    validate_proposal(&proposal)
        .await
        .map_err(|err| SelfServiceError::InvalidProposal(err.to_string()))?;
    execute_proposal(proposal)
        .await
        .map_err(|err| SelfServiceError::InvalidProposal(err.to_string()))?;

    with_state_mut(|s| s.add_runes_token_requests.remove(&args.rune_id));
    Ok(())
}

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct AddDestChainArgs {
    pub token_id: String,
    pub dst_chain: ChainId,
}

pub async fn add_dest_chain_for_token(args: AddDestChainArgs) -> Result<(), SelfServiceError> {
    let mut token_meta = with_state(|s| {
        s.token(&args.token_id)
            .map_err(|_| SelfServiceError::TokenNotFound)
    })?;

    let issue_chain = with_state(|s| {
        s.chain(&token_meta.issue_chain)
            .map_err(|_| SelfServiceError::ChainNotAvailable)
    })?;

    if !issue_chain
        .counterparties
        .is_some_and(|c| c.contains(&args.dst_chain))
    {
        return Err(SelfServiceError::ChainNotAvailable);
    }

    token_meta.dst_chains.push(args.dst_chain);

    let proposal = vec![Proposal::UpdateToken(token_meta)];
    validate_proposal(&proposal)
        .await
        .map_err(|err| SelfServiceError::InvalidProposal(err.to_string()))?;

    charge_fee().await?;

    execute_proposal(proposal)
        .await
        .map_err(|err| SelfServiceError::InvalidProposal(err.to_string()))?;

    Ok(())
}

async fn charge_fee() -> Result<(), SelfServiceError> {
    let subaccount = principal_to_subaccount(&ic_cdk::caller());
    let balance = ic_balance_of(&subaccount).await?.e8s();
    if balance < SELF_SERVICE_FEE {
        return Err(SelfServiceError::InsufficientFee {
            required: SELF_SERVICE_FEE,
            provided: balance,
        });
    }

    transfer_fee(&subaccount, SELF_SERVICE_FEE).await?;
    Ok(())
}

async fn ic_balance_of(subaccount: &Subaccount) -> Result<Tokens, SelfServiceError> {
    let account_identifier = AccountIdentifier::new(&ic_cdk::api::id(), &subaccount);
    let balance_args = AccountBalanceArgs {
        account: account_identifier,
    };
    account_balance(MAINNET_LEDGER_CANISTER_ID, balance_args)
        .await
        .map_err(|(_, reason)| SelfServiceError::TemporarilyUnavailable(reason))
}

async fn transfer_fee(subaccount: &Subaccount, fee_amount: u64) -> Result<(), SelfServiceError> {
    let transfer_args = TransferArgs {
        memo: Memo(0),
        amount: Tokens::from_e8s(fee_amount),
        fee: Tokens::from_e8s(ICP_TRANSFER_FEE),
        from_subaccount: Some(subaccount.clone()),
        to: AccountIdentifier::new(&ic_cdk::api::id(), &DEFAULT_SUBACCOUNT),
        created_at_time: None,
    };

    ic_ledger_types::transfer(MAINNET_LEDGER_CANISTER_ID, transfer_args)
        .await
        .map_err(|(_, reason)| SelfServiceError::TemporarilyUnavailable(reason))?
        .map_err(|err| SelfServiceError::TransferFailure(err.to_string()))?;

    Ok(())
}

pub fn principal_to_subaccount(principal_id: &Principal) -> Subaccount {
    let mut subaccount = [0; std::mem::size_of::<Subaccount>()];
    let principal_id = principal_id.as_slice();
    subaccount[0] = principal_id.len().try_into().unwrap();
    subaccount[1..1 + principal_id.len()].copy_from_slice(principal_id);

    Subaccount(subaccount)
}
