use candid::{CandidType, Principal};

use log::{self};
use omnity_types::{ChainId, Directive, Seq, Token, TokenId, Topic};
use serde::{Deserialize, Serialize};


// use updates::mint_token::{MintTokenError, MintTokenRequest};

use crate::{
    audit,
    call_error::{CallError, Reason},
    state::{mutate_state, read_state},
    BATCH_QUERY_LIMIT,
};

/// handler directives from hub to solana
pub async fn handle_directives() {
    let (hub_principal, offset) = read_state(|s| (s.hub_principal, s.next_directive_seq));
    match query_directives(hub_principal, offset, BATCH_QUERY_LIMIT).await {
        Ok(directives) => {
            for (_, directive) in &directives {
                match directive {
                    Directive::AddChain(chain) | Directive::UpdateChain(chain) => {
                        mutate_state(|s| audit::add_chain(s, chain.clone()));
                    }
                    Directive::AddToken(token) | Directive::UpdateToken(token) => {
                        match add_new_token(token.clone()).await {
                            Ok(_) => {
                                log::info!(
                                    "[process directives] add token successful, token id: {}",
                                    token.token_id
                                );
                            }
                            Err(err) => {
                                log::error!(
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
                        log::info!("[process_directives] success to update fee, fee: {}", fee);
                    }
                }
            }
            let next_seq = directives.last().map_or(offset, |(seq, _)| seq + 1);
            mutate_state(|s| {
                s.next_directive_seq = next_seq;
            });
        }
        Err(err) => {
            log::error!(
                "[process directives] failed to query directives, err: {:?}",
                err
            );
        }
    };
}

#[derive(CandidType, Clone, Debug, Serialize, Deserialize)]
pub struct TokenResp {
    pub token_id: TokenId,
    pub symbol: String,
    pub decimals: u8,
    pub icon: Option<String>,
    pub rune_id: Option<String>,
}

impl From<Token> for TokenResp {
    fn from(value: Token) -> Self {
        TokenResp {
            token_id: value.token_id,
            symbol: value.symbol,
            decimals: value.decimals,
            icon: value.icon,
            rune_id: value.metadata.get("rune_id").map(|rune_id| rune_id.clone()),
        }
    }
}

pub async fn query_directives(
    hub_principal: Principal,
    offset: u64,
    limit: u64,
) -> Result<Vec<(Seq, Directive)>, CallError> {
    let resp: (Result<Vec<(Seq, Directive)>, omnity_types::Error>,) = ic_cdk::api::call::call(
        hub_principal,
        "query_directives",
        (
            None::<Option<ChainId>>,
            None::<Option<Topic>>,
            offset,
            limit,
        ),
    )
    .await
    .map_err(|(code, message)| CallError {
        method: "query_directives".to_string(),
        reason: Reason::from_reject(code, message),
    })?;
    let data = resp.0.map_err(|err| CallError {
        method: "query_directives".to_string(),
        reason: Reason::CanisterError(err.to_string()),
    })?;
    Ok(data)
}

#[derive(CandidType, Clone, Debug, Deserialize, PartialEq, Eq)]
pub enum AddNewTokenError {
    AlreadyAdded(String),
    CreateLedgerErr(String),
}

pub async fn add_new_token(token: Token) -> Result<(), AddNewTokenError> {
    if read_state(|s| s.tokens.contains_key(&token.token_id)) {
        return Err(AddNewTokenError::AlreadyAdded(token.token_id));
    }

    
    //TODO: send tx to solana for add token

    //TODO: fetch tx status from solana and update route state

    // mutate_state(|s| {
    //     audit::add_token(s, token, record.canister_id);
    // });
    Ok(())
}