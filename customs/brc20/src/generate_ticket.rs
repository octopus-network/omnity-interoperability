use std::str::FromStr;

use candid::{CandidType, Deserialize};
use ic_btc_interface::Txid;

use omnity_types::{ChainState, Ticket, TicketType, TxAction};

use crate::bitcoin_to_custom::check_transaction;
use crate::hub;
use crate::state::{mutate_state, read_state};
use crate::types::{GenTicketRequest, GenTicketStatus};

#[derive(CandidType, Clone, Debug, Deserialize, PartialEq, Eq)]
pub enum GenerateTicketError {
    TemporarilyUnavailable(String),
    AlreadySubmitted,
    AlreadyProcessed,
    NoNewUtxos,
    TxNotFoundInMemPool,
    InvalidRuneId(String),
    InvalidTxId,
    UnsupportedChainId(String),
    UnsupportedToken(String),
    SendTicketErr(String),
    RpcError(String),
    AmountIsZero,
    OrdTxError(String),
    NotBridgeTx,
    InvalidArgs,
}

#[derive(Clone, CandidType)]
pub struct GenerateTicketArgs {
    pub txid: String,
    pub amount: u128,
    pub target_chain_id: String,
    pub token_id: String,
    pub receiver: String,
}

pub async fn generate_ticket(args: GenerateTicketArgs) -> Result<(), GenerateTicketError>  {
    if read_state(|s| s.chain_state == ChainState::Deactive) {
        return Err(GenerateTicketError::TemporarilyUnavailable(
            "chain state is deactive!".into(),
        ));
    }
    if args.amount == 0 {
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
        s.tokens.get(&args.token_id).cloned().ok_or(GenerateTicketError::UnsupportedToken(args.token_id.clone()))
    })?;

    read_state(|s| match s.generate_ticket_status(txid) {
        GenTicketStatus::Pending(_) | GenTicketStatus::Confirmed(_) => {
            Err(GenerateTicketError::AlreadySubmitted)
        }
        GenTicketStatus::Finalized(_) => Err(GenerateTicketError::AlreadyProcessed),
        GenTicketStatus::Unknown => Ok(()),
    })?;
    let (chain_id, hub_principal) = read_state(|s| (s.chain_id.clone(), s.hub_principal));
    check_transaction(args.clone()).await?;
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
            amount: args.amount.to_string(),
            sender: None,
            receiver: args.receiver.clone(),
            memo: None,
        },
    )
        .await
        .map_err(|err| GenerateTicketError::SendTicketErr(format!("{}", err)))?;

    let request = GenTicketRequest {
        target_chain_id: args.target_chain_id,
        receiver: args.receiver,
        token_id: token.token_id,
        ticker: token.name,
        amount: args.amount,
        txid,
        received_at: ic_cdk::api::time(),
    };

    mutate_state(|s| {
        s
            .pending_gen_ticket_requests
            .insert(request.txid, request);
    });
    Ok(())

}