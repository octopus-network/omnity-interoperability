use candid::{CandidType, Principal};
use icrc_ledger_types::icrc1::account::{Account, Subaccount};
use omnity_types::{ChainState, Directive, Token, TokenId};
use serde::{Deserialize, Serialize};
use state::{audit, mutate_state, read_state};
use std::str::FromStr;
use updates::mint_token::{MintTokenError, MintTokenRequest};
pub use ic_canister_log::log;
pub use omnity_types::ic_log::{INFO, ERROR};

pub mod call_error;
pub mod guard;
pub mod hub;
pub mod lifecycle;
pub mod memory;
pub mod state;
pub mod storage;
pub mod updates;

pub const INTERVAL_QUERY_DIRECTIVE: u64 = 60;
pub const INTERVAL_QUERY_TICKET: u64 = 5;
pub const BATCH_QUERY_LIMIT: u64 = 20;
pub const ICRC2_WASM: &[u8] = include_bytes!("../ic-icrc1-ledger.wasm");
pub const ICP_TRANSFER_FEE: u64 = 10_000;
pub const FEE_COLLECTOR_SUB_ACCOUNT: &Subaccount = &[1; 32];
pub const BLOCK_HOLE_ADDRESS: &str = "e3mmv-5qaaa-aaaah-aadma-cai";

async fn process_tickets() {
    if read_state(|s| s.chain_state == ChainState::Deactive) {
        return;
    }

    let (hub_principal, offset) = read_state(|s| (s.hub_principal, s.next_ticket_seq));
    match hub::query_tickets(hub_principal, offset, BATCH_QUERY_LIMIT).await {
        Ok(tickets) => {
            log!(INFO, "[Consolidation]ICP Route: pull tickets: {:?}", tickets);
            let mut next_seq = offset;
            for (seq, ticket) in &tickets {
                let receiver_parse_result = if ticket.receiver.contains(".") {
                    Account::from_str(ticket.receiver.as_str()).map_err(|e| e.to_string())
                } else {
                    Principal::from_str(ticket.receiver.as_str())
                        .map(|owner| Account {
                            owner,
                            subaccount: None,
                        })
                        .map_err(|e| e.to_string())
                };

                let receiver = match receiver_parse_result {
                    Ok(receiver) => receiver,
                    Err(err) => {
                        log!(ERROR,
                            "[process tickets] failed to parse ticket receiver: {}, err: {}",
                            ticket.receiver,
                            err
                        );
                        next_seq = seq + 1;
                        continue;
                    }
                };

                let amount: u128 = if let Ok(amount) = ticket.amount.parse() {
                    amount
                } else {
                    log!(ERROR,
                        "[process tickets] failed to parse ticket amount: {}",
                        ticket.amount
                    );
                    
                    next_seq = seq + 1;
                    continue;
                };
                match updates::mint_token(&mut MintTokenRequest {
                    ticket_id: ticket.ticket_id.clone(),
                    token_id: ticket.token.clone(),
                    receiver,
                    amount,
                })
                .await
                {
                    Ok(_) => {
                        log!(INFO,
                            "[process tickets] process successful for ticket id: {}",
                            ticket.ticket_id
                        );
                    }
                    Err(MintTokenError::TemporarilyUnavailable(desc)) => {
                        log!(ERROR,
                            "[process tickets] failed to mint token for ticket id: {}, err: {}",
                            ticket.ticket_id,
                            desc
                        );
                        break;
                    }
                    Err(err) => {
                        log!(ERROR,
                            "[process tickets] process failure for ticket id: {}, err: {:?}",
                            ticket.ticket_id,
                            err
                        );
                    }
                }
                next_seq = seq + 1;
            }
            mutate_state(|s| s.next_ticket_seq = next_seq)
        }
        Err(err) => {
            log!(ERROR, "[process tickets] failed to query tickets, err: {}", err);
        }
    }
}

async fn process_directives() {
    let (hub_principal, offset) = read_state(|s| (s.hub_principal, s.next_directive_seq));
    match hub::query_directives(hub_principal, offset, BATCH_QUERY_LIMIT).await {
        Ok(directives) => {
            for (_, directive) in &directives {
                match directive {
                    Directive::AddChain(chain) | Directive::UpdateChain(chain) => {
                        mutate_state(|s| audit::add_chain(s, chain.clone()));
                    }
                    Directive::AddToken(token) | Directive::UpdateToken(token) => {
                        match updates::add_new_token(token.clone()).await {
                            Ok(_) => {
                                log!(INFO,
                                    "[process directives] add token successful, token id: {}",
                                    token.token_id
                                );
                            }
                            Err(err) => {
                                log!(ERROR,
                                    "[process directives] failed to add token: token id: {}, err: {:?}",
                                    token.token_id,
                                    err
                                );
                            }
                        }
                    }
                    Directive::ToggleChainState(toggle) => {
                        mutate_state(|s| audit::toggle_chain_state(s, toggle.clone()));
                    }
                    Directive::UpdateFee(fee) => {
                        mutate_state(|s| audit::update_fee(s, fee.clone()));
                        log!(INFO,
                            "[process directives] success to update fee, fee: {}",
                            fee
                        );
                    }
                }
            }
            let next_seq = directives.last().map_or(offset, |(seq, _)| seq + 1);
            mutate_state(|s| {
                s.next_directive_seq = next_seq;
            });
        }
        Err(err) => {
            log!(ERROR,
                "[process directives] failed to query directives, err: {:?}",
                err
            );
        }
    };
}

pub fn process_directive_msg_task() {
    ic_cdk::spawn(async {
        // Considering that the directive is queried once a minute, guard protection is not needed.
        process_directives().await;
    });
}

pub fn process_ticket_msg_task() {
    ic_cdk::spawn(async {
        let _guard = match crate::guard::TimerLogicGuard::new() {
            Some(guard) => guard,
            None => return,
        };

        process_tickets().await;
    });
}

#[derive(CandidType, Clone, Debug, Serialize, Deserialize)]
pub struct TokenResp {
    pub token_id: TokenId,
    pub symbol: String,
    pub decimals: u8,
    pub icon: Option<String>,
    pub rune_id: Option<String>,
    pub principal: Option<Principal>,
}

impl From<Token> for TokenResp {
    fn from(value: Token) -> Self {
        TokenResp {
            token_id: value.token_id,
            symbol: value.symbol,
            decimals: value.decimals,
            icon: value.icon,
            rune_id: value.metadata.get("rune_id").map(|rune_id| rune_id.clone()),
            principal: None
        }
    }
}
