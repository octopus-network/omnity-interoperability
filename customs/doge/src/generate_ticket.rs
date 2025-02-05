use std::str::FromStr;

use candid::{CandidType, Deserialize};
use ic_canister_log::log;
use serde::Serialize;

use omnity_types::ic_log::INFO;
use omnity_types::{ChainState, Ticket, TicketType, TxAction};

use crate::doge::tatum_rpc;
use crate::doge::transaction::Txid;
use crate::dogeoin_to_custom::check_transaction;
use crate::errors::CustomsError;
use crate::hub;
use crate::state::{mutate_state, read_state};
use crate::types::{serialize_hex, Destination, GenTicketStatus, LockTicketRequest};

#[derive(Clone, CandidType, Serialize, Deserialize, Debug)]
pub struct GenerateTicketWithTxidArgs {
    pub txid: String,
    pub target_chain_id: String,
    pub token_id: String,
    pub receiver: String,
}

#[derive(Clone, CandidType, Serialize, Deserialize, Debug)]
pub struct GenerateTicketArgs {
    pub target_chain_id: String,
    pub token_id: String,
    pub receiver: String,
}

pub async fn get_ungenerated_txids(args: GenerateTicketArgs) -> Result<Vec<Txid>, CustomsError> {
    let dest = Destination::new(args.target_chain_id, args.receiver, None);
    let deposit_address = read_state(|s| s.get_address(dest)).map(|a| a.0.to_string())?;

    let tatum_rpc_config = read_state(|s| s.tatum_api_config.clone());
    let tatum_rpc = tatum_rpc::TatumDogeRpc::new(tatum_rpc_config.url, tatum_rpc_config.api_key);
    let txids = tatum_rpc
        .get_transactions_by_address(deposit_address)
        .await
        .map_err(|e| CustomsError::RpcError(format!("{}", e)))?;

    let filtered_txids: Vec<Txid> = txids
        .into_iter()
        .filter(|txid| {
            read_state(|s| {
                let type_txid: crate::types::Txid = txid.to_owned().into();
                s.generate_ticket_status(&type_txid) == GenTicketStatus::Unknown
            })
        })
        .collect();

    Ok(filtered_txids)
}

pub async fn generate_ticket(args: GenerateTicketWithTxidArgs) -> Result<(), CustomsError> {
    log!(INFO, "received generate_ticket: {:?}", args.clone());
    if read_state(|s| s.chain_state == ChainState::Deactive) {
        return Err(CustomsError::TemporarilyUnavailable(
            "chain state is deactive!".into(),
        ));
    }

    let txid = Txid::from_str(&args.txid).map_err(|_| CustomsError::InvalidTxId)?;
    if !read_state(|s| {
        s.counterparties
            .get(&args.target_chain_id)
            .is_some_and(|c| c.chain_state == ChainState::Active)
    }) {
        return Err(CustomsError::UnsupportedChainId(
            args.target_chain_id.clone(),
        ));
    }

    read_state(|s| {
        s.tokens
            .get(&args.token_id)
            .cloned()
            .ok_or(CustomsError::UnsupportedToken(args.token_id.clone()))
    })?;

    read_state(|s| match s.generate_ticket_status(&(txid.clone().into())) {
        GenTicketStatus::Pending(_) | GenTicketStatus::Confirmed(_) => {
            Err(CustomsError::AlreadySubmitted)
        }
        GenTicketStatus::Finalized(_) => Err(CustomsError::AlreadyProcessed),
        GenTicketStatus::Unknown => Ok(()),
    })?;
    let (chain_id, hub_principal, min_deposit_amount) =
        read_state(|s| (s.chain_id.clone(), s.hub_principal, s.min_deposit_amount));
    let (transaction, amount, sender) = check_transaction(args.clone()).await?;

    if amount < min_deposit_amount {
        return Err(CustomsError::CustomError(format!(
            "The amount of the transaction is less than the minimum deposit amount: {}",
            min_deposit_amount
        )));
    }

    hub::pending_ticket(
        hub_principal,
        Ticket {
            ticket_id: args.txid.clone(),
            ticket_type: TicketType::Normal,
            ticket_time: ic_cdk::api::time(),
            src_chain: chain_id,
            dst_chain: args.target_chain_id.clone(),
            action: TxAction::Transfer,
            token: args.token_id.clone(),
            amount: amount.to_string(),
            sender: Some(sender),
            receiver: args.receiver.clone(),
            memo: None,
        },
    )
    .await
    .map_err(|err| CustomsError::SendTicketErr(format!("{}", err)))?;
    let request = LockTicketRequest {
        target_chain_id: args.target_chain_id,
        receiver: args.receiver,
        token_id: args.token_id,
        amount: amount.to_string(),
        txid: txid.into(),
        received_at: ic_cdk::api::time(),
        transaction_hex: serialize_hex(&transaction),
    };
    mutate_state(|s| {
        s.pending_lock_ticket_requests
            .insert(request.txid.clone().into(), request);
    });
    Ok(())
}
