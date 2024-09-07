use cosmwasm::port::{PortContractExecutor, REDEEM_EVENT_KIND};
use memory::{get_redeem_ticket, insert_redeem_ticket, read_state};
use tendermint::abci::Event;

use crate::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedeemEvent {
    pub token_id: String,
    pub sender: String,
    pub receiver: String,
    pub amount: u128,
    pub target_chain: String,
}

pub async fn redeem_token_and_send_ticket(tx_hash: String)->Result<TicketId> {
    match get_redeem_ticket(&tx_hash) {
        Some(ticket_id) => {
            return Err(RouteError::CustomError(format!("ticket already redeemed, ticket_id: {:?}", ticket_id)))
        },
        None => {},
    }
    let port_contract_executor = PortContractExecutor::from_state()?;
    let event = port_contract_executor
        .query_redeem_token_event(tx_hash.clone())
        .await?;

    let (hub_principal, chain_id) = read_state(|s| (s.hub_principal, s.chain_id.clone()));
    let ticket = Ticket {
        ticket_id: tx_hash.clone(),
        ticket_type: omnity_types::TicketType::Normal,
        ticket_time: ic_cdk::api::time(),
        src_chain: chain_id,
        dst_chain: event.target_chain.clone(),
        action: omnity_types::TxAction::RedeemIcpChainKeyAssets(omnity_types::IcpChainKeyToken::CKBTC),
        token: event.token_id.clone(),
        amount: event.amount.to_string(),
        sender: Some(event.sender),
        receiver: event.receiver,
        memo: None,
    };

    log::info!(
        "try to send redeem ticket: {:?}, tx_hash: {:?}",
        ticket,
        tx_hash
    );

    hub::send_ticket(hub_principal, ticket.clone())
        .await?;
 

    insert_redeem_ticket(tx_hash, ticket.ticket_id.clone());

    Ok(ticket.ticket_id)

}

pub fn parse_redeem_event(redeem_event: Event) -> Result<RedeemEvent> {
    assert_eq!(
        redeem_event.kind,
        REDEEM_EVENT_KIND.to_string(),
        "Event kind is not RedeemRequested"
    );
    let token_id = find_attribute(&redeem_event, "token_id")?;
    let sender = find_attribute(&redeem_event, "sender")?;
    let receiver = find_attribute(&redeem_event, "receiver")?;
    let amount = find_attribute(&redeem_event, "amount")?
        .parse()
        .map_err(|e| RouteError::CustomError(format!("amount parse error: {}", e)))?;
    let target_chain = find_attribute(&redeem_event, "target_chain")?;
    Ok(RedeemEvent {
        token_id,
        sender,
        receiver,
        amount,
        target_chain,
    })
}

fn find_attribute(event: &Event, key: &str) -> Result<String> {
    for attr in &event.attributes {
        let key_str = attr.key_str().map_err(|e| {
            RouteError::AttributeParseError(event.kind.to_string(), key.to_string(), e.to_string())
        })?;
        if key_str.eq(key) {
            return attr
                .value_str()
                .map_err(|e| {
                    RouteError::AttributeParseError(
                        event.kind.to_string(),
                        key.to_string(),
                        e.to_string(),
                    )
                })
                .map(|v| v.to_string());
        }
    }
    Err(RouteError::AttributeNotFound(
        key.to_string(),
        event.kind.to_string(),
    ))
}
