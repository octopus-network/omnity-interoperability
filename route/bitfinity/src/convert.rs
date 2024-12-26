use did::transaction::TransactionReceiptLog;
use ic_cdk::api::management_canister::ecdsa::{EcdsaCurve, EcdsaKeyId};
use omnity_types::{Directive, Factor, Ticket, TicketType, ToggleAction, TxAction};
use crate::contract_types::{RunesMintRequested, TokenBurned, TokenTransportRequested};
use crate::contracts::{PortContractCommandIndex, PortContractFactorTypeIndex};
use crate::state::read_state;
use ic_stable_structures::Storable;

pub fn ticket_from_burn_event(log_entry: &TransactionReceiptLog, token_burned: TokenBurned, fee_token: Option<String>, bridge_fee: Option<u128>) -> Ticket {
    let src_chain = read_state(|s| s.omnity_chain_id.clone());
    let token = read_state(|s| {
        s.tokens
            .get(&token_burned.token_id.to_string())
            .expect("token not found")
            .clone()
    });
    let dst_chain = token.token_id_info()[0].to_string();
    let tx_action = if token_burned.receiver == "0" {
        TxAction::Burn
    } else {
        TxAction::Redeem
    };

    let memo = Some("fee_token: ".to_string()+ fee_token.unwrap_or_default().as_str() + ", bridge_fee: " + bridge_fee.unwrap_or_default().to_string().as_str() + "Wei");

    Ticket {
        ticket_id: log_entry.transaction_hash.to_hex_str(),
        ticket_time: ic_cdk::api::time(),
        ticket_type: TicketType::Normal,
        src_chain,
        dst_chain,
        action: tx_action,
        token: token_burned.token_id,
        amount: token_burned.amount.to_string(),
        sender: Some(format!(
            "0x{}",
            hex::encode(token_burned.sender.0.as_slice())
        )),
        receiver: token_burned.receiver,
        memo: memo.to_owned().map(|m| m.to_bytes().to_vec()),
    }
}

pub fn ticket_from_runes_mint_event(log_entry: &TransactionReceiptLog, runes_mint: RunesMintRequested, fee_token: Option<String>, bridge_fee: Option<u128>) -> Ticket {
    let src_chain = read_state(|s| s.omnity_chain_id.clone());
    let token = read_state(|s| {
        s.tokens
            .get(&runes_mint.token_id.to_string())
            .expect("token not found")
            .clone()
    });
    let dst_chain = token.token_id_info()[0].to_string();

    let memo = Some("fee_token: ".to_string()+ fee_token.unwrap_or_default().as_str() + ", bridge_fee: " + bridge_fee.unwrap_or_default().to_string().as_str() + "Wei");

    Ticket {
        ticket_id: log_entry.transaction_hash.to_hex_str(),
        ticket_time: ic_cdk::api::time(),
        ticket_type: TicketType::Normal,
        src_chain,
        dst_chain,
        action: TxAction::Mint,
        token: runes_mint.token_id,
        amount: "0".to_string(),
        sender: Some(format!("0x{}", hex::encode(runes_mint.sender.0.as_slice()))),
        receiver: format!("0x{}", hex::encode(runes_mint.receiver.0.as_slice())),
        memo: memo.to_owned().map(|m| m.to_bytes().to_vec()),
    }
}

pub fn ticket_from_transport_event(log_entry: &TransactionReceiptLog,
                                   token_transport_requested: TokenTransportRequested,
                                   fee_token: Option<String>, bridge_fee: Option<u128>) -> Ticket {
    let src_chain = read_state(|s| s.omnity_chain_id.clone());
    let dst_chain = token_transport_requested.dst_chain_id;

    let memo = Some("fee_token: ".to_string()+ fee_token.unwrap_or_default().as_str() + ", bridge_fee: " + bridge_fee.unwrap_or_default().to_string().as_str() + "Wei");
    
    Ticket {
        ticket_id: log_entry.transaction_hash.to_hex_str(),
        ticket_time: ic_cdk::api::time(),
        ticket_type: TicketType::Normal,
        src_chain,
        dst_chain,
        action: TxAction::Transfer,
        token: token_transport_requested.token_id.to_string(),
        amount: token_transport_requested.amount.to_string(),
        sender: Some(format!(
            "0x{}",
            hex::encode(token_transport_requested.sender.0.as_slice())
        )),
        receiver: token_transport_requested.receiver,
        memo: memo.to_owned().map(|m| m.to_bytes().to_vec()),
    }
}

pub fn convert_ecdsa_key_id(k: &omnity_types::EcdsaKeyId) -> ic_cdk::api::management_canister::ecdsa::EcdsaKeyId {
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


pub fn  directive_to_port_command_index(directive: &Directive) ->  Option<PortContractCommandIndex> {
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

