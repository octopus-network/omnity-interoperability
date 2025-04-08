use crate::contract_types::{RunesMintRequested, TokenBurned, TokenTransportRequested};
use crate::contracts::{PortContractCommandIndex, PortContractFactorTypeIndex};
use crate::evm_log::LogEntry;
use crate::traits::{get_memo, StateProvider};
use const_hex::ToHexExt;
use ic_cdk::api::management_canister::ecdsa::{EcdsaCurve, EcdsaKeyId};
use ic_stable_structures::Storable;
use omnity_types::{Directive, Factor, Ticket, TicketType, ToggleAction, TxAction};

pub fn ticket_from_burn_event<P: StateProvider>(
    log_entry: &LogEntry,
    token_burned: TokenBurned,
    has_memo: bool,
) -> Ticket {
    let src_chain = P::chain_info().ommnity_chain_id.clone();
    let token = P::get_token(&token_burned.token_id)
        .expect("token not found")
        .clone();
    let dst_chain = token.token_id_info()[0].to_string();
    let tx_action = if token_burned.receiver == "0" {
        TxAction::Burn
    } else {
        TxAction::Redeem
    };

    let memo = has_memo
        .then(|| get_memo::<P>(None, dst_chain.clone()))
        .unwrap_or_default();

    Ticket {
        ticket_id: log_entry.transaction_hash.encode_hex_with_prefix(),
        ticket_time: ic_cdk::api::time(),
        ticket_type: TicketType::Normal,
        src_chain,
        dst_chain,
        action: tx_action,
        token: token_burned.token_id,
        amount: token_burned.amount.to_string(),
        sender: Some(token_burned.sender.encode_hex_with_prefix()),
        receiver: token_burned.receiver,
        memo: memo.map(|m| m.to_bytes().to_vec()),
    }
}

pub fn ticket_from_runes_mint_event<P: StateProvider>(
    log_entry: &LogEntry,
    runes_mint: RunesMintRequested,
    has_memo: bool,
) -> Ticket {
    let src_chain = P::chain_info().ommnity_chain_id.clone();
    let token = P::get_token(&runes_mint.token_id)
        .expect("token not found")
        .clone();
    let dst_chain = token.token_id_info()[0].to_string();
    let memo = has_memo
        .then(|| get_memo::<P>(None, dst_chain.clone()))
        .unwrap_or_default();

    Ticket {
        ticket_id: log_entry.transaction_hash.clone().encode_hex_with_prefix(),
        ticket_time: ic_cdk::api::time(),
        ticket_type: TicketType::Normal,
        src_chain,
        dst_chain,
        action: TxAction::Mint,
        token: runes_mint.token_id,
        amount: "0".to_string(),
        sender: Some(runes_mint.sender.encode_hex_with_prefix()),
        receiver: runes_mint.receiver.encode_hex_with_prefix(),
        memo: memo.map(|m| m.to_bytes().to_vec()),
    }
}

pub fn ticket_from_transport_event<P: StateProvider>(
    log_entry: &LogEntry,
    token_transport_requested: TokenTransportRequested,
    has_memo: bool,
) -> Ticket {
    let src_chain = P::chain_info().ommnity_chain_id.clone();
    let dst_chain = token_transport_requested.dst_chain_id;
    let memo = has_memo
        .then(|| get_memo::<P>(Some(token_transport_requested.memo), dst_chain.clone()))
        .unwrap_or_default();

    Ticket {
        ticket_id: log_entry.transaction_hash.encode_hex_with_prefix(),
        ticket_time: ic_cdk::api::time(),
        ticket_type: TicketType::Normal,
        src_chain,
        dst_chain,
        action: TxAction::Transfer,
        token: token_transport_requested.token_id.to_string(),
        amount: token_transport_requested.amount.to_string(),
        sender: Some(token_transport_requested.sender.encode_hex_with_prefix()),
        receiver: token_transport_requested.receiver,
        memo: memo.map(|m| m.to_bytes().to_vec()),
    }
}

pub fn convert_ecdsa_key_id(
    k: &omnity_types::EcdsaKeyId,
) -> ic_cdk::api::management_canister::ecdsa::EcdsaKeyId {
    EcdsaKeyId {
        curve: EcdsaCurve::Secp256k1,
        name: k.name.clone(),
    }
}

pub fn convert_factor_to_port_factor_type_index(f: &Factor) -> PortContractFactorTypeIndex {
    match f {
        Factor::UpdateTargetChainFactor(_) => 0,
        Factor::UpdateFeeTokenFactor(_) => 1,
    }
}

pub fn directive_to_port_command_index(directive: &Directive) -> Option<PortContractCommandIndex> {
    match directive {
        Directive::AddChain(_) => None,
        Directive::AddToken(_) => Some(0u8),
        Directive::UpdateFee(_) => Some(1u8),
        Directive::ToggleChainState(t) => match t.action {
            ToggleAction::Activate => Some(3u8),
            ToggleAction::Deactivate => Some(2u8),
        },
        Directive::UpdateChain(_) => None,
        Directive::UpdateToken(_) => None,
    }
}
