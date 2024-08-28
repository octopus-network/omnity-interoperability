use cosmwasm::port::REDEEM_EVENT_KIND;
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
