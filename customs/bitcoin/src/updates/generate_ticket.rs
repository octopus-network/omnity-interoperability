use crate::destination::Destination;
use crate::guard::{generate_ticket_guard, GuardError};
use crate::management::{get_utxos, CallSource};
use crate::state::{audit, mutate_state, read_state, GenTicketRequest, GenTicketStatus, RuneId};
use crate::updates::get_btc_address::{
    destination_to_p2wpkh_address_from_state, init_ecdsa_public_key,
};
use candid::{CandidType, Deserialize};
use ic_btc_interface::Txid;
use omnity_types::ChainState;
use serde::Serialize;
use std::str::FromStr;

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct GenerateTicketArgs {
    pub target_chain_id: String,
    pub receiver: String,
    pub rune_id: String,
    pub amount: u128,
    pub txid: String,
}

#[derive(CandidType, Clone, Debug, Deserialize, PartialEq, Eq)]
pub enum GenerateTicketError {
    TemporarilyUnavailable(String),
    AlreadySubmitted,
    AleardyProcessed,
    NoNewUtxos,
    InvalidRuneId(String),
    UnsupportedChainId(String),
    UnsupportedToken(String),
}

impl From<GuardError> for GenerateTicketError {
    fn from(e: GuardError) -> Self {
        match e {
            GuardError::TooManyConcurrentRequests => {
                Self::TemporarilyUnavailable("too many concurrent requests".to_string())
            }
        }
    }
}

pub async fn generate_ticket(args: GenerateTicketArgs) -> Result<(), GenerateTicketError> {
    read_state(|s| s.mode.is_transport_available_for())
        .map_err(GenerateTicketError::TemporarilyUnavailable)?;

    init_ecdsa_public_key().await;
    let _guard = generate_ticket_guard()?;

    let rune_id = RuneId::from_str(&args.rune_id)
        .map_err(|e| GenerateTicketError::InvalidRuneId(e.to_string()))?;

    let txid = Txid::from_str(&args.txid)
        .map_err(|_| GenerateTicketError::TemporarilyUnavailable("Invalid txid".to_string()))?;

    read_state(|s| {
        if !s.counterparties
            .get(&args.target_chain_id)
            .is_some_and(|c| c.chain_state == ChainState::Active)
        {
            Err(GenerateTicketError::UnsupportedChainId(
                args.target_chain_id.clone(),
            ))
        } else if !s.tokens.contains_key(&args.rune_id) {
            Err(GenerateTicketError::UnsupportedToken(args.rune_id))
        } else {
            Ok(())
        }
    })?;

    read_state(|s| match s.generate_ticket_status(txid) {
        GenTicketStatus::Pending(_) => Err(GenerateTicketError::AlreadySubmitted),
        GenTicketStatus::Invalid | GenTicketStatus::Finalized => {
            Err(GenerateTicketError::AleardyProcessed)
        }
        GenTicketStatus::Unknown => Ok(()),
    })?;

    let (btc_network, min_confirmations) = read_state(|s| (s.btc_network, s.min_confirmations));

    let destination = Destination {
        target_chain_id: args.target_chain_id.clone(),
        receiver: args.receiver.clone(),
        token: None,
    };

    let address = read_state(|s| destination_to_p2wpkh_address_from_state(s, &destination));

    // In order to prevent the memory from being exhausted,
    // ensure that the user has transferred token to this address.
    let utxos = get_utxos(btc_network, &address, min_confirmations, CallSource::Client)
        .await
        .map_err(|call_err| {
            GenerateTicketError::TemporarilyUnavailable(format!(
                "Failed to call bitcoin canister: {}",
                call_err
            ))
        })?
        .utxos;

    let new_utxos = read_state(|s| s.new_utxos_for_destination(utxos, &destination, Some(txid)));
    if new_utxos.len() == 0 {
        return Err(GenerateTicketError::NoNewUtxos);
    }

    let request = GenTicketRequest {
        address,
        target_chain_id: args.target_chain_id,
        receiver: args.receiver,
        rune_id,
        amount: args.amount,
        txid,
        received_at: ic_cdk::api::time(),
    };

    mutate_state(|s| {
        audit::accept_generate_ticket_request(s, request);
        audit::add_utxos(s, destination, new_utxos, true);
    });
    Ok(())
}
