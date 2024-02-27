use ic_cdk::query;

use omnity_types::{
    ChainCondition, ChainId, ChainInfo, ChainState, ChainType, DireQueue, Directive, Error, Fee,
    LockedToken, Proposal, Seq, StateAction, Ticket, TicketId, TicketQueue, TokenCondition,
    TokenId, TokenMetaData, Topic, TxAction, TxCondition,
};

#[query]
pub async fn get_chain_list(
    condition: Option<ChainCondition>,
    from: u64,
    num: u64,
) -> Result<Vec<ChainInfo>, Error> {
    Ok(Vec::new())
}

#[query]
pub async fn get_token_list(
    condition: Option<TokenCondition>,
    from: u64,
    num: u64,
) -> Result<Vec<ChainInfo>, Error> {
    Ok(Vec::new())
}

#[query]
pub async fn get_all_locked_tokens() -> Result<Vec<ChainInfo>, Error> {
    Ok(Vec::new())
}

#[query]
pub async fn get_locked_tokens(
    condition: Option<TokenCondition>,
    from: u64,
    num: u64,
) -> Result<Vec<LockedToken>, Error> {
    Ok(Vec::new())
}

#[query]
pub async fn get_tx_list(
    condition: Option<TxCondition>,
    from: u64,
    num: u64,
) -> Result<Vec<Ticket>, Error> {
    Ok(Vec::new())
}

#[query]
pub async fn get_total_tx() -> Result<u64, Error> {
    Ok(0u64)
}
