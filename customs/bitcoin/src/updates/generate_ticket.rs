use crate::destination::Destination;
use crate::guard::{generate_ticket_guard, GuardError};
use crate::management::{get_utxos, CallSource};
use crate::state::{
    audit, mutate_state, read_state, GenTicketRequest, GenTicketStatus, RuneId, RUNES_TOKEN,
};
use crate::updates::get_btc_address::{
    destination_to_p2wpkh_address_from_state, init_ecdsa_public_key,
};
use candid::{CandidType, Deserialize};
use ic_btc_interface::{Network, Txid, Utxo};
use omnity_types::ChainState;
use serde::Serialize;
use std::str::FromStr;

use super::get_btc_address::destination_to_p2wpkh_address_from_state_v0;

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
    AlreadyProcessed,
    NoNewUtxos,
    InvalidRuneId(String),
    InvalidTxId,
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
    if read_state(|s| s.chain_state == ChainState::Deactive) {
        return Err(GenerateTicketError::TemporarilyUnavailable(
            "chain state is deactive!".into(),
        ));
    }

    init_ecdsa_public_key().await;
    let _guard = generate_ticket_guard()?;

    let rune_id = RuneId::from_str(&args.rune_id)
        .map_err(|e| GenerateTicketError::InvalidRuneId(e.to_string()))?;

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

    let token_id = read_state(|s| {
        if let Some((token_id, _)) = s.tokens.iter().find(|(_, (r, _))| rune_id.eq(r)) {
            Ok(token_id.clone())
        } else {
            Err(GenerateTicketError::UnsupportedToken(args.rune_id))
        }
    })?;

    read_state(|s| match s.generate_ticket_status(txid) {
        GenTicketStatus::Pending(_) => Err(GenerateTicketError::AlreadySubmitted),
        GenTicketStatus::Finalized => {
            Err(GenerateTicketError::AlreadyProcessed)
        }
        GenTicketStatus::Unknown => Ok(()),
    })?;

    let (btc_network, min_confirmations) = read_state(|s| (s.btc_network, s.min_confirmations));

    let mut destination = Destination {
        target_chain_id: args.target_chain_id.clone(),
        receiver: args.receiver.clone(),
        token: Some(RUNES_TOKEN.into()),
    };

    let mut address = read_state(|s| destination_to_p2wpkh_address_from_state(s, &destination));

    // In order to prevent the memory from being exhausted,
    // ensure that the user has transferred token to this address.
    let mut new_utxos = fetch_new_utxos(btc_network, min_confirmations, &address, txid).await?;

    if new_utxos.len() == 0 {
        // We have migrated the key. It is possible that some users transferred
        // the token to the old address before the migration.
        destination.token = None;
        address = read_state(|s| destination_to_p2wpkh_address_from_state_v0(s, &destination));
        new_utxos = fetch_new_utxos(btc_network, min_confirmations, &address, txid).await?;
        if new_utxos.len() == 0 {
            return Err(GenerateTicketError::NoNewUtxos);
        }
    }

    let request = GenTicketRequest {
        address,
        target_chain_id: args.target_chain_id,
        receiver: args.receiver,
        token_id,
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

async fn fetch_new_utxos(
    btc_network: Network,
    min_confirmations: u32,
    address: &String,
    txid: Txid,
) -> Result<Vec<Utxo>, GenerateTicketError> {
    let utxos = get_utxos(btc_network, address, min_confirmations, CallSource::Client)
        .await
        .map_err(|call_err| {
            GenerateTicketError::TemporarilyUnavailable(format!(
                "Failed to call bitcoin canister: {}",
                call_err
            ))
        })?
        .utxos;

    Ok(read_state(|s| s.new_utxos(utxos, Some(txid))))
}
