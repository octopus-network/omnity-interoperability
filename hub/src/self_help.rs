use crate::{
    proposal::{execute_proposal, validate_proposal},
    state::{with_state, with_state_mut}
};
use omnity_types::hub_types::{Proposal, TokenMeta};
use candid::{CandidType, Deserialize, Principal};
use ic_ledger_types::{
    account_balance, AccountBalanceArgs, AccountIdentifier, Memo, Subaccount, Tokens, TransferArgs,
    DEFAULT_SUBACCOUNT, MAINNET_LEDGER_CANISTER_ID,
};
use omnity_types::{rune_id::RuneId, ChainId};
use serde::Serialize;
use std::{collections::HashMap, str::FromStr};
use crate::self_help::SelfServiceError::LinkError;
use omnity_types::hub_types::Proposal::UpdateChain;

pub const ADD_TOKEN_FEE: u64 = 1_000_000_000;
pub const ADD_CHAIN_FEE: u64 = 300_000_000;
const ICP_TRANSFER_FEE: u64 = 10_000;
const BITCOIN_CHAIN: &str = "Bitcoin";

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct AddRunesTokenReq {
    pub rune_id: String,
    pub symbol: String,
    pub icon: String,
    pub dest_chain: ChainId,
}

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct LinkChainReq{
    pub chain1: ChainId,
    pub chain2: ChainId,
}

#[derive(CandidType, Debug, Deserialize)]
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
    ChainNotFound(String),
    LinkError(omnity_types::Error),
    ChainsAlreadyLinked,
}

pub async fn link_chains(args: LinkChainReq) -> Result<(), SelfServiceError> {
    with_state(|s| {
        if !s.chains.contains_key(&args.chain1) {
            Err(SelfServiceError::ChainNotFound(args.chain1.clone()))
        }else if !s.chains.contains_key(&args.chain2) {
            Err(SelfServiceError::ChainNotFound(args.chain2.clone()))
        }else {
            Ok(())
        }
    })?;
    let mut chain1 = with_state(|s|s.chain(&args.chain1)).unwrap();
    let mut chain2 = with_state(|s|s.chain(&args.chain2)).unwrap();
    if chain1.contains_counterparty(&args.chain2) && chain2.contains_counterparty(&args.chain1) {
        return Err(SelfServiceError::ChainsAlreadyLinked);
    }
    chain1.add_counterparty(args.chain2.clone());
    chain2.add_counterparty(args.chain1.clone());
    execute_proposal(vec![UpdateChain(chain1)]).await.map_err(|e| LinkError(e))?;
    execute_proposal(vec![UpdateChain(chain2)]).await.map_err(|e| LinkError(e))?;
    Ok(())
}

pub async fn add_runes_token(args: AddRunesTokenReq) -> Result<(), SelfServiceError> {
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

    let bitcoin = with_state(|s| s.chain(&BITCOIN_CHAIN.to_string()))
        .map_err(|_| SelfServiceError::ChainNotFound(BITCOIN_CHAIN.to_string()))?;

    if !bitcoin
        .counterparties
        .is_some_and(|c| c.contains(&args.dest_chain))
    {
        return Err(SelfServiceError::ChainNotFound(args.dest_chain));
    }

    charge_fee(ADD_TOKEN_FEE).await?;

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
}

pub async fn finalize_add_runes_token(args: FinalizeAddRunesArgs) -> Result<(), SelfServiceError> {
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
        dst_chains: vec![BITCOIN_CHAIN.into(), request.dest_chain],
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
    pub dest_chain: ChainId,
}

pub async fn add_dest_chain_for_token(args: AddDestChainArgs) -> Result<(), SelfServiceError> {
    let mut token_meta = with_state(|s| {
        s.token(&args.token_id)
            .map_err(|_| SelfServiceError::TokenNotFound)
    })?;

    let issue_chain = with_state(|s| {
        s.chain(&token_meta.issue_chain)
            .map_err(|_| SelfServiceError::ChainNotFound(token_meta.issue_chain.clone()))
    })?;

    if !issue_chain
        .counterparties
        .is_some_and(|c| c.contains(&args.dest_chain))
    {
        return Err(SelfServiceError::ChainNotFound(args.dest_chain));
    }

    token_meta.dst_chains.push(args.dest_chain);

    let proposal = vec![Proposal::UpdateToken(token_meta)];
    validate_proposal(&proposal)
        .await
        .map_err(|err| SelfServiceError::InvalidProposal(err.to_string()))?;

    charge_fee(ADD_CHAIN_FEE).await?;

    execute_proposal(proposal)
        .await
        .map_err(|err| SelfServiceError::InvalidProposal(err.to_string()))?;

    Ok(())
}

async fn charge_fee(fee_amount: u64) -> Result<(), SelfServiceError> {
    let subaccount = principal_to_subaccount(&ic_cdk::caller());
    let balance = ic_balance_of(&subaccount).await?.e8s();
    if balance < fee_amount {
        return Err(SelfServiceError::InsufficientFee {
            required: fee_amount,
            provided: balance,
        });
    }

    transfer_fee(&subaccount, fee_amount).await?;
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
        amount: Tokens::from_e8s(fee_amount - ICP_TRANSFER_FEE),
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
