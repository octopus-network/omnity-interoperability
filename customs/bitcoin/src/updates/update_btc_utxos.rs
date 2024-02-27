use super::get_btc_address::init_ecdsa_public_key;
use crate::{
    address::main_destination,
    management,
    state::{audit, mutate_state, read_state},
    updates::get_btc_address::destination_to_p2wpkh_address_from_state,
    BTC_TOKEN,
};
use candid::{CandidType, Deserialize};
use ic_btc_interface::Utxo;

#[derive(CandidType, Clone, Debug, Deserialize, PartialEq, Eq)]
pub enum UpdateBtcUtxosErr {
    TemporarilyUnavailable(String),
}

pub async fn update_btc_utxos() -> Result<Vec<Utxo>, UpdateBtcUtxosErr> {
    init_ecdsa_public_key().await;

    let destination = main_destination(String::from(BTC_TOKEN));
    let address = read_state(|s| destination_to_p2wpkh_address_from_state(s, &destination));

    let (btc_network, min_confirmations) = read_state(|s| (s.btc_network, s.min_confirmations));

    let resp = management::get_utxos(
        btc_network,
        &address,
        min_confirmations,
        management::CallSource::Custom,
    )
    .await
    .map_err(|err| {
        UpdateBtcUtxosErr::TemporarilyUnavailable(format!(
            "Failed to call bitcoin canister: {}",
            err
        ))
    })?;

    let new_utxos =
        read_state(|s| s.new_utxos_for_destination(resp.utxos.clone(), &destination, None));
    if new_utxos.len() == 0 {
        return Ok(vec![]);
    }

    mutate_state(|s| audit::add_utxos(s, destination, new_utxos.clone(), false));

    Ok(new_utxos)
}
