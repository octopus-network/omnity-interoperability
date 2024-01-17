use crate::destination::Destination;
use crate::state::{mutate_state, read_state, TransportTokenRequest};
use crate::updates::get_btc_address::destination_to_p2wpkh_address_from_state;
use candid::{CandidType, Deserialize};
use ic_btc_interface::Txid;
use serde::Serialize;

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct TransportTokenArgs {
    pub target_chain_id: String,
    pub receiver: String,
    pub token: String,
    pub amount: u128,
    pub tx_id: Txid,
}

#[derive(CandidType, Clone, Debug, Deserialize, PartialEq, Eq)]
pub enum TransportTokenError {
    TemporarilyUnavailable(String),
    AlreadyProcessing,
}

pub async fn transport_token(args: TransportTokenArgs) -> Result<(), TransportTokenError> {
    read_state(|s| s.mode.is_transport_available_for())
        .map_err(TransportTokenError::TemporarilyUnavailable)?;

    // TODO invoke hub canister, check if the token and target_chain_id is in whitelist

    read_state(|s| {
        if s.pending_transport_token_request.contains_key(&args.tx_id) {
            Err(TransportTokenError::AlreadyProcessing)
        } else {
            Ok(())
        }
    })?;

    let address = read_state(|s| {
        destination_to_p2wpkh_address_from_state(
            s,
            &Destination {
                target_chain_id: args.target_chain_id.clone(),
                receiver: args.receiver.clone(),
            },
        )
    });

    let request = TransportTokenRequest {
        address,
        target_chain_id: args.target_chain_id,
        receiver: args.receiver,
        token: args.token,
        amount: args.amount,
        tx_id: args.tx_id,
    };

    mutate_state(|s| {
        s.pending_transport_token_request
            .insert(args.tx_id, request)
    });
    Ok(())
}
