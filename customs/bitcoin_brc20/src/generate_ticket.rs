use std::ops::Mul;
use std::str::FromStr;

use candid::{CandidType, Deserialize};
use ic_btc_interface::Txid;
use ic_canister_log::log;
use num_traits::{ToPrimitive, Zero};
use rust_decimal::Decimal;
use serde::Serialize;
use thiserror::Error;

use omnity_types::ic_log::INFO;
use omnity_types::{ChainState, Ticket, TicketType, TxAction};

use crate::bitcoin_to_custom::check_transaction;
use crate::hub;
use crate::state::{mutate_state, read_state};
use crate::types::{GenTicketStatus, LockTicketRequest};

#[derive(CandidType, Clone, Default, Debug, Deserialize, PartialEq, Eq, Error)]
pub enum GenerateTicketError {
    #[error("temp unavailable: {0}")]
    TemporarilyUnavailable(String),
    #[error("AlreadySubmitted")]
    AlreadySubmitted,
    #[error("AlreadyProcessed")]
    AlreadyProcessed,
    #[error("NoNewUtxos")]
    NoNewUtxos,
    #[error("TxNotFoundInMemPool")]
    TxNotFoundInMemPool,
    #[error("InvalidRuneId: {0}")]
    InvalidRuneId(String),
    #[error("InvalidTxId")]
    InvalidTxId,
    #[error("UnsupportedChainId: {0}")]
    UnsupportedChainId(String),
    #[error("UnsupportedToken: {0}")]
    UnsupportedToken(String),
    #[error("SendTicketErr: {0}")]
    SendTicketErr(String),
    #[error("RpcError: {0}")]
    RpcError(String),
    #[error("AmountIsZero")]
    AmountIsZero,
    #[error("OrdTxError: {0}")]
    OrdTxError(String),
    #[error("NotBridgeTx")]
    NotBridgeTx,
    #[error("InvalidArgs: {0}")]
    InvalidArgs(String),
    #[error("NotPayFees")]
    NotPayFees,
    #[default]
    #[error("Unknown")]
    Unknown,
}

#[derive(Clone, CandidType, Serialize, Deserialize, Debug)]
pub struct GenerateTicketArgs {
    pub txid: String,
    pub amount: String,
    pub target_chain_id: String,
    pub token_id: String,
    pub receiver: String,
}

pub async fn generate_ticket(args: GenerateTicketArgs) -> Result<(), GenerateTicketError> {
    log!(INFO, "received generate_ticket request: {:?}", args.clone());
    if read_state(|s| s.chain_state == ChainState::Deactive) {
        return Err(GenerateTicketError::TemporarilyUnavailable(
            "chain state is deactive!".into(),
        ));
    }
    let amt = Decimal::from_str(&args.amount);
    if amt.is_err() {
        return Err(GenerateTicketError::InvalidArgs(format!(
            "amount format error {}",
            args.amount
        )));
    }
    if amt.unwrap() == Decimal::zero() {
        return Err(GenerateTicketError::AmountIsZero);
    }
    let txid = Txid::from_str(&args.txid).map_err(|_| GenerateTicketError::InvalidTxId)?;
    if !read_state(|s| {
        s.counterparties
            .get(&args.target_chain_id)
            .is_some_and(|c| c.chain_state == ChainState::Active)
    }) {
        return Err(GenerateTicketError::UnsupportedChainId(
            args.target_chain_id.clone(),
        ));
    }

    let token = read_state(|s| {
        s.tokens
            .get(&args.token_id)
            .cloned()
            .ok_or(GenerateTicketError::UnsupportedToken(args.token_id.clone()))
    })?;
    read_state(|s| match s.generate_ticket_status(txid) {
        GenTicketStatus::Pending(_) | GenTicketStatus::Confirmed(_) => {
            Err(GenerateTicketError::AlreadySubmitted)
        }
        GenTicketStatus::Finalized(_) => Err(GenerateTicketError::AlreadyProcessed),
        GenTicketStatus::Unknown => Ok(()),
    })?;
    let (chain_id, hub_principal) = read_state(|s| (s.chain_id.clone(), s.hub_principal));
    let transfer = check_transaction(args.clone()).await?;

    let ticket_amount: u128 = Decimal::from_str(&transfer.amt)
        .unwrap()
        .mul(Decimal::from(10u128.pow(token.decimals as u32)))
        .to_u128()
        .unwrap();

    // let (fee, _) = read_state(|s|s.get_transfer_fee_info(&args.target_chain_id));
    // let bridge_fee = Fee {bridge_fee: fee.unwrap_or_default()};
    // let memo = bridge_fee.into_memo(None).unwrap_or_default();
    
    hub::pending_ticket(
        hub_principal,
        Ticket {
            ticket_id: args.txid.clone(),
            ticket_type: TicketType::Normal,
            ticket_time: ic_cdk::api::time(),
            src_chain: chain_id,
            dst_chain: args.target_chain_id.clone(),
            action: TxAction::Transfer,
            token: token.token_id.clone(),
            amount: ticket_amount.to_string(),
            sender: None,
            receiver: args.receiver.clone(),
            // memo: memo.to_owned().map(|m| m.to_bytes().to_vec())
            memo: None,
        },
    )
    .await
    .map_err(|err| GenerateTicketError::SendTicketErr(format!("{}", err)))?;
    let request = LockTicketRequest {
        target_chain_id: args.target_chain_id,
        receiver: args.receiver,
        token_id: token.token_id,
        ticker: token.name,
        amount: args.amount,
        txid,
        received_at: ic_cdk::api::time(),
    };
    mutate_state(|s| {
        s.pending_lock_ticket_requests.insert(request.txid, request);
    });
    Ok(())
}
