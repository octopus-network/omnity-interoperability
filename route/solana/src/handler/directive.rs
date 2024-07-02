use candid::{CandidType, Principal};
use log::{error, info};
use omnity_types::{ChainId, Directive, Error, Seq, Token, TokenId, Topic};
use serde::{Deserialize, Serialize};

// use updates::mint_token::{MintTokenError, MintTokenRequest};

use crate::{
    call_error::{CallError, Reason},
    state::{mutate_state, read_state},
};
pub const DIRECTIVE_SIZE: u64 = 20;

/// query directives from hub and save to route state
pub async fn query_directives() {
    let (hub_principal, offset) = read_state(|s| (s.hub_principal, s.next_directive_seq));
    match inner_query_directives(hub_principal, offset, DIRECTIVE_SIZE).await {
        Ok(directives) => {
            for (_, directive) in &directives {
                match directive {
                    Directive::AddChain(chain) | Directive::UpdateChain(chain) => {
                        mutate_state(|s| s.add_chain(chain.clone()));
                    }
                    Directive::AddToken(token) | Directive::UpdateToken(token) => {
                        mutate_state(|s| s.add_token(token.clone()));
                    }
                    Directive::ToggleChainState(toggle) => {
                        mutate_state(|s| s.toggle_chain_state(toggle.clone()));
                    }
                    Directive::UpdateFee(fee) => {
                        mutate_state(|s| s.update_fee(fee.clone()));
                        info!("[process_directives] success to update fee, fee: {}", fee);
                    }
                }
            }
            let next_seq = directives.last().map_or(offset, |(seq, _)| seq + 1);
            mutate_state(|s| {
                s.next_directive_seq = next_seq;
            });
        }
        Err(err) => {
            error!(
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

pub async fn inner_query_directives(
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

/// call the solana port to execute directives
pub async fn execute_directives() -> Result<(), Error> {
    Ok(())
}
