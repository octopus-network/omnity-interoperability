use crate::types::{ChainId, ChainState, Error, Seq, Ticket, TicketId, TicketType, TxAction};
use candid::{CandidType, Principal};
use ic_solana::types::Pubkey;
use ic_stable_structures::Storable;

use serde::{Deserialize, Serialize};

use super::sol_call::{get_or_create_ata, mint_to};
use crate::handler::sol_call::solana_client;

use crate::handler::sol_call::ParsedValue;
use crate::handler::sol_call::TransactionDetail;
use crate::handler::sol_call::{Burn, ParsedIns, Transfer};
use crate::{
    call_error::{CallError, Reason},
    state::{mutate_state, read_state},
};
use ic_solana::rpc_client::JsonRpcResponse;
use serde_json::from_value;

pub const TICKET_LIMIT_SIZE: u64 = 20;
pub const COUNTER_SIZE: u64 = 5;
use ic_canister_log::log;
use ic_solana::logs::{ERROR, INFO};

/// handler tickets from customs to solana
pub async fn query_tickets() {
    if read_state(|s| s.chain_state == ChainState::Deactive) {
        return;
    }

    let (hub_principal, offset) = read_state(|s| (s.hub_principal, s.next_ticket_seq));
    match inner_query_tickets(hub_principal, offset, TICKET_LIMIT_SIZE).await {
        Ok(tickets) => {
            let mut next_seq = offset;
            for (seq, ticket) in &tickets {
                if let Err(e) = Pubkey::try_from(ticket.receiver.as_str()) {
                    log!(
                        ERROR,
                        "[ticket::query_tickets] failed to parse ticket receiver: {}, error:{}",
                        ticket.receiver,
                        e.to_string()
                    );
                    next_seq = seq + 1;
                    continue;
                };
                if let Err(e) = ticket.amount.parse::<u64>() {
                    log!(
                        ERROR,
                        "[ticket::query_tickets] failed to parse ticket amount: {}, Error:{}",
                        ticket.amount,
                        e.to_string()
                    );
                    next_seq = seq + 1;
                    continue;
                };

                mutate_state(|s| s.tickets_queue.insert(*seq, ticket.to_owned()));
                next_seq = seq + 1;
            }
            mutate_state(|s| s.next_ticket_seq = next_seq)
        }
        Err(e) => {
            log!(
                ERROR,
                "[ticket::query_tickets] failed to query tickets, err: {}",
                e.to_string()
            );
        }
    }
}

/// query ticket from hub
pub async fn inner_query_tickets(
    hub_principal: Principal,
    offset: u64,
    limit: u64,
) -> Result<Vec<(Seq, Ticket)>, CallError> {
    let resp: (Result<Vec<(Seq, Ticket)>, Error>,) = ic_cdk::api::call::call(
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
    NotFoundToken(String),
    UnsupportedToken(String),
    AlreadyProcessed(TicketId),
    TemporarilyUnavailable(String),
}
#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct MintTokenRequest {
    pub ticket_id: TicketId,
    pub associated_account: String,
    pub amount: u64,
    pub token_mint: String,
}

pub async fn create_associated_account() {
    let mut creating_atas = vec![];
    read_state(|s| {
        for (_seq, ticket) in s.tickets_queue.iter() {
            if let Some(token_mint) = s.token_mint_map.get(&ticket.token) {
                // ata not exists, to be created
                if matches!(
                    s.associated_account
                        .get(&(ticket.receiver.to_string(), token_mint.to_string())),
                    None
                ) {
                    creating_atas.push((ticket.receiver.to_owned(), token_mint.to_owned()))
                }
            }
        }
    });

    let mut count = 0u64;
    for (owner, token_mint) in creating_atas.into_iter() {
        match get_or_create_ata(owner.to_owned(), token_mint.to_owned()).await {
            Ok(ata) => {
                log!(INFO,
                    "[ticket::create_associated_account] new associated_account {:?} based on {:} and {:?} ",
                    ata,
                    owner,
                    token_mint
                );
                // save the associated_account
                mutate_state(|s| s.associated_account.insert((owner, token_mint), ata));
                // Control foreach size, if >= COUNTER_SIZE, then break
                count += 1;
                if count >= COUNTER_SIZE {
                    break;
                }
            }
            Err(e) => {
                log!(
                    ERROR,
                    "[ticket::create_associated_account] get_or_create_ata error: {:?}  ",
                    e
                );
                continue;
            }
        }
    }
}

pub async fn handle_mint_token() {
    // take 5 tickets to mint,very time
    let tickets = read_state(|s| {
        s.tickets_queue
            .iter()
            .take(5)
            .map(|(seq, ticket)| (seq, ticket))
            .collect::<Vec<_>>()
    });

    for (seq, ticket) in tickets.into_iter() {
        // first,check token mint
        let token_mint = read_state(|s| s.token_mint_map.get(&ticket.token).cloned());
        let token_mint = match token_mint {
            Some(token_mint) => token_mint,
            None => {
                log!(INFO,
                 "[ticket::handle_mint_token] the token({:}) mint account is not exists, waiting for create token mint ...",
                 ticket.token
             );
                continue;
            }
        };

        // secord,check ata
        let associated_account = read_state(|s| {
            s.associated_account
                .get(&(ticket.receiver.to_owned(), token_mint.to_owned()))
                .cloned()
        });
        let associated_account = match associated_account {
            Some(associated_account) => associated_account,
            None => {
                log!(INFO,
                 "[ticket::handle_mint_token] the associated_account based on {} and {} is not exists,waiting for create associated account ...",
                 ticket.receiver.to_string(),token_mint.to_string()
             );
                continue;
            }
        };

        let req = MintTokenRequest {
            ticket_id: ticket.ticket_id.to_owned(),
            associated_account: associated_account,
            amount: ticket.amount.parse::<u64>().unwrap(),
            token_mint: token_mint,
        };

        match mint_token(req).await {
            Ok(signature) => {
                log!(INFO,
                 "[ticket::handle_mint_token] process successful for ticket id: {} and tx hash :{}",
                 ticket.ticket_id,signature
             );
                // if ok, remove the handled ticket from queue
                mutate_state(|s| s.tickets_queue.remove(&seq));

                // update txhash to hub
                let hub_principal = read_state(|s| s.hub_principal);
                if let Err(err) =
                    update_tx_hash(hub_principal, ticket.ticket_id.to_string(), signature).await
                {
                    log!(
                        ERROR,
                        "[tickets::handle_mint_token] failed to update tx hash after mint token:{}",
                        err
                    );
                }
            }
            Err(e) => {
                log!(
                    ERROR,
                    "[ticket::handle_mint_token] failed to mint token for ticket id: {}, err: {:?}",
                    ticket.ticket_id,
                    e
                );

                continue;
            }
        }
    }
}

/// send tx to solana for mint token
pub async fn mint_token(req: MintTokenRequest) -> Result<String, MintTokenError> {
    if read_state(|s| s.finalized_mint_token_requests.contains_key(&req.ticket_id)) {
        return Err(MintTokenError::AlreadyProcessed(req.ticket_id));
    }
    let signuate = mint_to(req.associated_account, req.amount, req.token_mint)
        .await
        .map_err(|e| MintTokenError::TemporarilyUnavailable(e.to_string()))?;
    mutate_state(|s| s.finalize_mint_token_req(req.ticket_id.to_owned(), signuate.to_owned()));

    Ok(signuate)
}

pub async fn update_tx_hash(
    hub_principal: Principal,
    ticket_id: TicketId,
    mint_tx_hash: String,
) -> Result<(), CallError> {
    let resp: (Result<(), Error>,) =
        ic_cdk::api::call::call(hub_principal, "update_tx_hash", (ticket_id, mint_tx_hash))
            .await
            .map_err(|(code, message)| CallError {
                method: "update_tx_hash".to_string(),
                reason: Reason::from_reject(code, message),
            })?;
    resp.0.map_err(|err| CallError {
        method: "update_tx_hash".to_string(),
        reason: Reason::CanisterError(err.to_string()),
    })?;
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
    pub signature: String,
    pub target_chain_id: String,
    pub sender: String,
    pub receiver: String,
    pub token_id: String,
    pub amount: u64,
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
    log!(INFO, "generate_ticket req: {:#?}", req);

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

    if !read_state(|s| s.tokens.contains_key(&req.token_id.to_string())) {
        return Err(GenerateTicketError::UnsupportedToken(req.token_id.clone()));
    }

    if !matches!(req.action, TxAction::Redeem) {
        return Err(GenerateTicketError::UnsupportedAction(
            "Transfer action is not supported".into(),
        ));
    }

    let (hub_principal, chain_id) = read_state(|s| (s.hub_principal, s.chain_id.to_owned()));

    //parsed tx via signature
    let mut receiver = String::from("");
    let client = solana_client().await;
    let tx = client
        .query_transaction(req.signature.to_owned())
        .await
        .map_err(|e| GenerateTicketError::TemporarilyUnavailable(e.to_string()))?;

    let json_response = serde_json::from_str::<JsonRpcResponse<TransactionDetail>>(&tx)
        .map_err(|e| GenerateTicketError::TemporarilyUnavailable(e.to_string()))?;

    if let Some(e) = json_response.error {
        return Err(GenerateTicketError::TemporarilyUnavailable(e.message));
    } else {
        let tx_detail = json_response
            .result
            .ok_or(GenerateTicketError::TemporarilyUnavailable(
                "tx result is None".to_string(),
            ))?;
        // parse instruction
        // TODO: check instruction size == 3(must includes: transfer,burned and memo)
        for instruction in &tx_detail.transaction.message.instructions {
            if let Ok(parsed_value) = from_value::<ParsedValue>(instruction.parsed.to_owned()) {
                if let Ok(pi) = from_value::<ParsedIns>(parsed_value.parsed.to_owned()) {
                    log!(INFO, "Parsed instruction: {:#?}", pi);
                    if pi.instr_type.eq("transfer") {
                        let transfer = from_value::<Transfer>(pi.info.to_owned()).map_err(|e| {
                            GenerateTicketError::TemporarilyUnavailable(e.to_string())
                        })?;
                        log!(INFO, "Parsed transfer: {:#?}", transfer);
                        let fee = read_state(|s| s.get_fee(req.target_chain_id.clone())).ok_or(
                            GenerateTicketError::TemporarilyUnavailable(format!(
                                "No found fee for {}",
                                req.target_chain_id
                            )),
                        )?;
                        let lamports = transfer.lamports as u128;
                        //TODO: verify: transfer.destination == omnity.solana_fee_account and transfer.lamports == omnity.solana_fee_account_received_amount
                        if !(transfer.source.eq(&req.sender) && lamports == fee) {
                            return Err(GenerateTicketError::TemporarilyUnavailable(format!(
                                "Unable to verify the fee info",
                            )));
                        }
                    }
                    if pi.instr_type.eq("burnChecked") {
                        let burn = from_value::<Burn>(pi.info.to_owned()).map_err(|e| {
                            GenerateTicketError::TemporarilyUnavailable(e.to_string())
                        })?;
                        log!(INFO, "Parsed burn: {:#?}", burn);
                        let burned_amount = burn
                            .token_amount
                            .ui_amount_string
                            .parse::<u64>()
                            .map_err(|e| {
                                GenerateTicketError::TemporarilyUnavailable(e.to_string())
                            })?;
                        let mint_address =
                            read_state(|s| s.token_mint_map.get(&req.token_id).cloned()).ok_or(
                                GenerateTicketError::TemporarilyUnavailable(format!(
                                    "No found token mint address for {}",
                                    req.token_id
                                )),
                            )?;
                        if !(burn.authority.eq(&req.sender)
                            && burn.mint.eq(&mint_address)
                            && burned_amount == req.amount)
                        {
                            return Err(GenerateTicketError::TemporarilyUnavailable(format!(
                                "Unable to verify the token burned info",
                            )));
                        }
                    }
                } else if let Ok(memo) = from_value::<String>(parsed_value.parsed.to_owned()) {
                    log!(INFO, "Parsed memo: {:?}", memo);
                    //verify memo.eq(req.receiver.)
                    if memo.eq(&req.receiver) {
                        receiver = memo;
                    } else {
                        return Err(GenerateTicketError::TemporarilyUnavailable(format!(
                            "receiver({}) from memo not match req.receiver({})",
                            memo, req.receiver,
                        )));
                    }
                } else {
                    log!(
                        INFO,
                        "Unknown Parsed instruction: {:#?}",
                        parsed_value.parsed
                    );
                }
            } else {
                log!(INFO, "Unknown Parsed Value: {:#?}", instruction.parsed);
                return Err(GenerateTicketError::TemporarilyUnavailable(format!(
                    "tx parsed error:{}",
                    tx
                )));
            }
        }
    }

    let ticket = Ticket {
        ticket_id: req.signature.to_string(),
        ticket_type: TicketType::Normal,
        ticket_time: ic_cdk::api::time(),
        src_chain: chain_id,
        dst_chain: req.target_chain_id.to_owned(),
        action: req.action.to_owned(),
        token: req.token_id.to_owned(),
        amount: req.amount.to_string(),
        sender: Some(req.sender.to_owned()),
        receiver: receiver.to_string(),
        memo: req.memo.to_owned().map(|m| m.to_bytes().to_vec()),
    };

    match send_ticket(hub_principal, ticket.to_owned()).await {
        Err(err) => {
            mutate_state(|s| {
                s.failed_tickets.push(ticket.clone());
            });
            log!(
                ERROR,
                "failed to send ticket: {}",
                req.signature.to_string()
            );
            Err(GenerateTicketError::SendTicketErr(format!("{}", err)))
        }
        Ok(()) => {
            log!(INFO, "successful to send ticket: {:?}", ticket);
            Ok(GenerateTicketOk {
                ticket_id: req.signature.to_string(),
            })
        }
    }
}

/// send ticket to hub
pub async fn send_ticket(hub_principal: Principal, ticket: Ticket) -> Result<(), CallError> {
    let resp: (Result<(), Error>,) =
        ic_cdk::api::call::call(hub_principal, "send_ticket", (ticket,))
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