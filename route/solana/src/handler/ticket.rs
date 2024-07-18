use candid::{CandidType, Principal};
use ic_stable_structures::Storable;
use log::{error, info};
use omnity_types::{ChainId, ChainState, Seq, Ticket, TicketId, TxAction};
use serde::{Deserialize, Serialize};

use crate::{
    call_error::{CallError, Reason},
    state::{mutate_state, read_state},
};

pub const TICKET_SIZE: u64 = 20;

/// handler tickets from customs to solana
pub async fn query_tickets() {
    if read_state(|s| s.chain_state == ChainState::Deactive) {
        return;
    }

    let (hub_principal, offset) = read_state(|s| (s.hub_principal, s.next_ticket_seq));
    match inner_query_tickets(hub_principal, offset, TICKET_SIZE).await {
        Ok(tickets) => {
            let mut next_seq = offset;
            for (seq, ticket) in &tickets {
                let amount: u128 = if let Ok(amount) = ticket.amount.parse() {
                    amount
                } else {
                    error!(
                        "[process tickets] failed to parse ticket amount: {}",
                        ticket.amount
                    );
                    next_seq = seq + 1;
                    continue;
                };
                match mint_token(&mut MintTokenRequest {
                    ticket_id: ticket.ticket_id.clone(),
                    token_id: ticket.token.clone(),
                    receiver: ticket.receiver.to_string(),
                    amount,
                })
                .await
                {
                    Ok(_) => {
                        info!(
                            "[process tickets] process successful for ticket id: {}",
                            ticket.ticket_id
                        );
                    }
                    Err(MintTokenError::TemporarilyUnavailable(desc)) => {
                        error!(
                            "[process tickets] failed to mint token for ticket id: {}, err: {}",
                            ticket.ticket_id, desc
                        );
                        break;
                    }
                    Err(err) => {
                        error!(
                            "[process tickets] process failure for ticket id: {}, err: {:?}",
                            ticket.ticket_id, err
                        );
                    }
                }
                next_seq = seq + 1;
            }
            mutate_state(|s| s.next_ticket_seq = next_seq)
        }
        Err(err) => {
            error!("[process tickets] failed to query tickets, err: {}", err);
        }
    }
}

/// query ticket from hub
pub async fn inner_query_tickets(
    hub_principal: Principal,
    offset: u64,
    limit: u64,
) -> Result<Vec<(Seq, Ticket)>, CallError> {
    let resp: (Result<Vec<(Seq, Ticket)>, omnity_types::Error>,) = ic_cdk::api::call::call(
        hub_principal,
        "query_tickets",
        (None::<Option<ChainId>>, offset, limit),
    )
    .await
    .map_err(|(code, message)| CallError {
        method: "query_tickets".to_string(),
        reason: Reason::from_reject(code, message),
    })?;
    let data = resp.0.map_err(|err| CallError {
        method: "query_tickets".to_string(),
        reason: Reason::CanisterError(err.to_string()),
    })?;
    Ok(data)
}

#[derive(CandidType, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum MintTokenError {
    UnsupportedToken(String),

    AlreadyProcessed(TicketId),

    TemporarilyUnavailable(String),
}

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct MintTokenRequest {
    pub ticket_id: TicketId,
    pub token_id: String,
    /// The owner of the account on the ledger.
    pub receiver: String,
    pub amount: u128,
}

/// send tx to solana for mint token
pub async fn mint_token(_req: &MintTokenRequest) -> Result<(), MintTokenError> {
    //TODO: check: if token account not exites, create mint token account and init token metadata
    //TODO: check receiver ata ,create it if not exites
    //TODO: mint token to receiver ata
    //TODO: save: tx signature for ticket id, the timer interval query signature status for finalized
    Ok(())
}

/// query solana tx signature / status and update txhash to hub
pub async fn get_signaute_status() -> Result<(), omnity_types::Error> {
    // query solana tx signature status
    //TODO: update tx hash to hub
    Ok(())
}

#[derive(CandidType, Clone, Debug, Deserialize, PartialEq, Eq)]
pub enum GenerateTicketError {
    TemporarilyUnavailable(String),
    UnsupportedToken(String),
    UnsupportedChainId(String),
    /// The redeem account does not hold the requested token amount.
    InsufficientFunds {
        balance: u64,
    },
    /// The caller didn't approve enough funds for spending.
    InsufficientAllowance {
        allowance: u64,
    },
    SendTicketErr(String),
    InsufficientRedeemFee {
        required: u64,
        provided: u64,
    },
    RedeemFeeNotSet,
    TransferFailure(String),
    UnsupportedAction(String),
}

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct GenerateTicketReq {
    pub tx_signature: String,
    pub target_chain_id: String,
    pub sender: String,
    pub receiver: String,
    pub token_id: String,
    pub amount: u128,
    pub action: TxAction,
    pub memo: Option<String>,
}

#[derive(CandidType, Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct GenerateTicketOk {
    pub ticket_id: String,
}

pub async fn generate_ticket(
    req: GenerateTicketReq,
) -> Result<GenerateTicketOk, GenerateTicketError> {
    if read_state(|s| s.chain_state == ChainState::Deactive) {
        return Err(GenerateTicketError::TemporarilyUnavailable(
            "chain state is deactive!".into(),
        ));
    }

    if !read_state(|s| {
        s.counterparties
            .get(&req.target_chain_id)
            .is_some_and(|c| c.chain_state == ChainState::Active)
    }) {
        return Err(GenerateTicketError::UnsupportedChainId(
            req.target_chain_id.clone(),
        ));
    }
    if !read_state(|s| s.tokens.get(&req.token_id.to_string()).is_none()) {
        return Err(GenerateTicketError::UnsupportedToken(req.token_id.clone()));
    }

    if !matches!(req.action, TxAction::Redeem) {
        return Err(GenerateTicketError::UnsupportedAction(
            "Transfer action is not supported".into(),
        ));
    }

    let (hub_principal, chain_id) = read_state(|s| (s.hub_principal, s.chain_id.clone()));
    let action = req.action.clone();

    //TODO: check solana sigature status

    let ticket = Ticket {
        ticket_id: req.tx_signature.to_string(),
        ticket_type: omnity_types::TicketType::Normal,
        ticket_time: ic_cdk::api::time(),
        src_chain: chain_id,
        dst_chain: req.target_chain_id.clone(),
        action,
        token: req.token_id.clone(),
        amount: req.amount.to_string(),
        sender: Some(req.sender.clone()),
        receiver: req.receiver.clone(),
        memo: req.memo.clone().map(|m| m.to_bytes().to_vec()),
    };
    match send_ticket(hub_principal, ticket.clone()).await {
        Err(err) => {
            mutate_state(|s| {
                s.failed_tickets.push(ticket.clone());
            });
            log::error!("failed to send ticket: {}", req.tx_signature.to_string());
            Err(GenerateTicketError::SendTicketErr(format!("{}", err)))
        }
        Ok(()) => {
            mutate_state(|s| s.finalize_gen_ticket(req.tx_signature.to_string(), req.clone()));

            Ok(GenerateTicketOk {
                ticket_id: req.tx_signature.to_string(),
            })
        }
    }
}

/// send ticket to hub
pub async fn send_ticket(hub_principal: Principal, ticket: Ticket) -> Result<(), CallError> {
    // TODO determine how many cycle it will cost.
    let cost_cycles = 4_000_000_000_u64;

    let resp: (Result<(), omnity_types::Error>,) =
        ic_cdk::api::call::call_with_payment(hub_principal, "send_ticket", (ticket,), cost_cycles)
            .await
            .map_err(|(code, message)| CallError {
                method: "send_ticket".to_string(),
                reason: Reason::from_reject(code, message),
            })?;
    let data = resp.0.map_err(|err| CallError {
        method: "send_ticket".to_string(),
        reason: Reason::CanisterError(err.to_string()),
    })?;
    Ok(data)
}
