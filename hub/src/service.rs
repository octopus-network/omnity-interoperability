use crate::memory::init_stable_log;
use candid::Principal;
use ic_canisters_http_types::{HttpRequest, HttpResponse};
use ic_cdk::{init, post_upgrade, pre_upgrade, query, update};
use ic_ledger_types::AccountIdentifier;
#[cfg(feature = "profiling")]
use ic_stable_structures::Memory;
use log::{debug, info};
use omnity_hub::auth::{auth_query, auth_update, is_admin, is_runes_oracle, set_perms, Permission};
use omnity_hub::event::{self, record_event, Event, GetEventsArg};
use omnity_hub::lifecycle::init::HubArg;
#[cfg(feature = "profiling")]
use omnity_hub::memory::get_profiling_memory;
use omnity_hub::metrics::{self, with_metrics};
use omnity_hub::self_help::{principal_to_subaccount, AddDestChainArgs, AddRunesTokenReq, FinalizeAddRunesArgs, SelfServiceError, ADD_CHAIN_FEE, ADD_TOKEN_FEE, LinkChainReq};
use omnity_hub::state::{with_state, with_state_mut};
use omnity_hub::types::{ChainMeta, TokenMeta, TxHash};
use omnity_hub::{proposal, self_help};

use omnity_hub::types::{
    TokenResp, {Proposal, Subscribers},
};
use omnity_hub::{lifecycle, memory};
use omnity_types::log::{init_log, LoggerConfigService, StableLogWriter};
use omnity_types::{
    Chain, ChainId, ChainState, ChainType, Directive, Error, Factor, Seq, Ticket, TicketId,
    TokenId, TokenOnChain, Topic,
};

use omnity_hub::state::HubState;

#[init]
fn init(args: HubArg) {
    match args {
        HubArg::Init(args) => {
            init_log(Some(init_stable_log()));
            info!("hub init args: {:?}", args);

            lifecycle::init(args.clone());
            record_event(&Event::Init(args));
        }
        HubArg::Upgrade(_) => {
            panic!("expected InitArgs got UpgradeArgs");
        }
    }
}

#[pre_upgrade]
fn pre_upgrade() {
    info!("begin to execute pre_upgrade ...");
    with_state(|hub_state| hub_state.pre_upgrade())
}

#[post_upgrade]
fn post_upgrade(args: Option<HubArg>) {
    init_log(Some(init_stable_log()));
    info!("begin to execute post_upgrade with :{:?}", args);
    // init log
    HubState::post_upgrade(args);
    info!("upgrade successfully!");
}

/// validate directive ,this method will be called by sns
#[query(guard = "is_admin")]
pub async fn validate_proposal(proposals: Vec<Proposal>) -> Result<Vec<String>, Error> {
    proposal::validate_proposal(&proposals).await
}
#[update(guard = "is_admin")]
pub async fn execute_proposal(proposals: Vec<Proposal>) -> Result<(), Error> {
    proposal::execute_proposal(proposals).await
}

#[update(guard = "is_admin")]
pub async fn handle_chain(proposals: Vec<Proposal>) -> Result<(), Error> {
    // The proposals must be AddToken or UpdateToken
    proposals.iter().try_for_each(|p| {
        if !matches!(p, Proposal::AddChain(_) | Proposal::UpdateChain(_)) {
            Err(Error::ProposalError(
                "The proposals must be AddChain or UpdateChain".to_string(),
            ))
        } else {
            Ok(())
        }
    })?;
    // validate proposal
    proposal::validate_proposal(&proposals).await?;
    // execution proposal and generate directives
    proposal::execute_proposal(proposals).await
}

#[update(guard = "is_admin")]
pub async fn handle_token(proposals: Vec<Proposal>) -> Result<(), Error> {
    // The proposals must be AddToken or UpdateToken
    proposals.iter().try_for_each(|p| {
        if !matches!(p, Proposal::AddToken(_) | Proposal::UpdateToken(_)) {
            Err(Error::ProposalError(
                "The proposals must be AddToken or UpdateToken".to_string(),
            ))
        } else {
            Ok(())
        }
    })?;
    // validate proposal
    proposal::validate_proposal(&proposals).await?;
    // exection proposal and generate directives
    proposal::execute_proposal(proposals).await
}

/// check and build update fee directive and push it to the directive queue
#[update(guard = "is_admin")]
pub async fn update_fee(factors: Vec<Factor>) -> Result<(), Error> {
    let proposals: Vec<Proposal> = factors.into_iter().map(Proposal::UpdateFee).collect();
    proposal::validate_proposal(&proposals).await?;
    proposal::execute_proposal(proposals).await
}

#[update(guard = "auth_update")]
pub async fn sub_directives(chain_id: Option<ChainId>, topics: Vec<Topic>) -> Result<(), Error> {
    debug!(
        "sub_topics for chain: {:?}, with topics: {:?} ",
        chain_id, topics
    );
    let dst_chain_id = metrics::get_chain_id(chain_id)?;
    debug!("get_chain_id:{:?}", dst_chain_id);
    with_state_mut(|hub_state| hub_state.sub_directives(&dst_chain_id, &topics))
}

#[update(guard = "auth_update")]
pub async fn unsub_directives(chain_id: Option<ChainId>, topics: Vec<Topic>) -> Result<(), Error> {
    debug!(
        "unsub_topics for chain: {:?}, with topics: {:?} ",
        chain_id, topics
    );
    let dst_chain_id = metrics::get_chain_id(chain_id)?;
    with_state_mut(|hub_state| hub_state.unsub_directives(&dst_chain_id, &topics))
}

#[query(guard = "auth_query")]
pub async fn query_subscribers(topic: Option<Topic>) -> Result<Vec<(Topic, Subscribers)>, Error> {
    debug!("query_subscribers for topic: {:?} ", topic);
    with_state(|hub_state| hub_state.query_subscribers(topic))
}

/// query directives for chain id filter by topic,this method will be called by route and custom
#[query(guard = "auth_query")]
pub async fn query_directives(
    chain_id: Option<ChainId>,
    topic: Option<Topic>,
    offset: usize,
    limit: usize,
) -> Result<Vec<(Seq, Directive)>, Error> {
    let dst_chain_id = metrics::get_chain_id(chain_id)?;
    with_state(|hub_state| hub_state.pull_directives(dst_chain_id, topic, offset, limit))
}

/// check and push ticket into queue
#[update(guard = "auth_update")]
pub async fn send_ticket(ticket: Ticket) -> Result<(), Error> {
    debug!("send_ticket: {:?}", ticket);

    with_state_mut(|hub_state| {
        // check ticket and update token on chain
        hub_state.check_and_update(&ticket)?;
        // push ticket into queue
        hub_state.push_ticket(ticket)
    })
}

#[update(guard = "auth_update")]
pub async fn resubmit_ticket(ticket: Ticket) -> Result<(), Error> {
    debug!("received resubmit ticket: {:?}", ticket);
    // No need to update the token since the old ticket has already added
    with_state_mut(|hub_state| hub_state.resubmit_ticket(ticket))
}

/// query tickets for chain id,this method will be called by route and custom
#[query(guard = "auth_query")]
pub async fn query_tickets(
    chain_id: Option<ChainId>,
    offset: usize,
    limit: usize,
) -> Result<Vec<(Seq, Ticket)>, Error> {
    let dst_chain_id = metrics::get_chain_id(chain_id)?;
    with_state(|hub_state| hub_state.pull_tickets(&dst_chain_id, offset, limit))
}

#[update(guard = "auth_update")]
pub async fn update_tx_hash(ticket_id: TicketId, tx_hash: String) -> Result<(), Error> {
    debug!("update tx({:?}) hash: {:?}", ticket_id, tx_hash);
    with_state_mut(|hub_state| hub_state.update_tx_hash(ticket_id, tx_hash))
}

#[update(guard = "auth_update")]
pub async fn batch_update_tx_hash(ticket_ids: Vec<TicketId>, tx_hash: String) -> Result<(), Error> {
    debug!("batch update tx({:?}) hash: {:?}", ticket_ids, tx_hash);
    for ticket_id in ticket_ids {
        with_state_mut(|hub_state| hub_state.update_tx_hash(ticket_id, tx_hash.clone()))?;
    }
    Ok(())
}

#[query(guard = "auth_query")]
pub async fn query_tx_hash(ticket_id: TicketId) -> Result<TxHash, Error> {
    with_state(|hub_state| hub_state.get_tx_hash(&ticket_id))
}

#[update(guard = "is_admin")]
pub async fn set_logger_filter(filter: String) {
    LoggerConfigService::default().set_logger_filter(&filter);
}

#[update(guard = "is_admin")]
pub async fn set_permissions(caller: Principal, perm: Permission) {
    set_perms(caller.to_string(), perm)
}

#[update(guard = "is_admin")]
fn set_runes_oracle(oracle: Principal) {
    with_state_mut(|s| s.runes_oracles.insert(oracle));
}

#[update(guard = "is_admin")]
fn remove_runes_oracle(oracle: Principal) {
    with_state_mut(|s| s.runes_oracles.remove(&oracle));
}

#[update]
pub async fn add_runes_token(args: AddRunesTokenReq) -> Result<(), SelfServiceError> {
    self_help::add_runes_token(args).await
}

#[update]
pub async fn link_chains(args: LinkChainReq) -> Result<(), SelfServiceError> {
    self_help::link_chains(args).await
}



#[update(guard = "is_runes_oracle")]
pub async fn finalize_add_runes_token_req(
    args: FinalizeAddRunesArgs,
) -> Result<(), SelfServiceError> {
    self_help::finalize_add_runes_token(args).await
}

#[update]
pub async fn add_dest_chain_for_token(args: AddDestChainArgs) -> Result<(), SelfServiceError> {
    self_help::add_dest_chain_for_token(args).await
}

#[query]
pub fn get_add_runes_token_requests() -> Vec<AddRunesTokenReq> {
    with_state(|s| {
        s.add_runes_token_requests
            .iter()
            .map(|(_, req)| req.clone())
            .collect()
    })
}

#[derive(candid::CandidType, serde::Serialize, Clone)]
pub struct SelfServiceFee {
    pub add_token_fee: u64,
    pub add_chain_fee: u64,
}

#[query]
pub fn get_self_service_fee() -> SelfServiceFee {
    SelfServiceFee {
        add_token_fee: ADD_TOKEN_FEE,
        add_chain_fee: ADD_CHAIN_FEE,
    }
}

#[query]
pub fn get_fee_account(principal: Option<Principal>) -> AccountIdentifier {
    let principal = principal.unwrap_or(ic_cdk::caller());
    AccountIdentifier::new(&ic_cdk::api::id(), &principal_to_subaccount(&principal))
}

#[query]
pub async fn get_chains(
    chain_type: Option<ChainType>,
    chain_state: Option<ChainState>,
    offset: usize,
    limit: usize,
) -> Result<Vec<Chain>, Error> {
    metrics::get_chains(chain_type, chain_state, offset, limit).await
}

#[query]
pub async fn get_chain(chain_id: String) -> Result<Chain, Error> {
    metrics::get_chain(chain_id).await
}

#[query]
pub async fn get_tokens(
    chain_id: Option<ChainId>,
    token_id: Option<TokenId>,
    offset: usize,
    limit: usize,
) -> Result<Vec<TokenResp>, Error> {
    metrics::get_tokens(chain_id, token_id, offset, limit)
        .await
        .map(|tokens| tokens.iter().map(|t| t.clone().into()).collect())
}

#[query(guard = "auth_query")]
pub async fn get_fees(
    chain_id: Option<ChainId>,
    token_id: Option<TokenId>,
    offset: usize,
    limit: usize,
) -> Result<Vec<(ChainId, TokenId, u128)>, Error> {
    metrics::get_fees(chain_id, token_id, offset, limit).await
}

#[query]
pub async fn get_chain_tokens(
    chain_id: Option<ChainId>,
    token_id: Option<TokenId>,
    offset: usize,
    limit: usize,
) -> Result<Vec<TokenOnChain>, Error> {
    metrics::get_chain_tokens(chain_id, token_id, offset, limit).await
}

#[query]
pub async fn get_token_position_size() -> Result<u64, Error> {
    metrics::get_token_position_size().await
}

#[query]
pub async fn get_tx(ticket_id: TicketId) -> Result<Ticket, Error> {
    metrics::get_tx(ticket_id).await
}

#[query]
pub async fn get_txs_with_chain(
    src_chain: Option<ChainId>,
    dst_chain: Option<ChainId>,
    token_id: Option<TokenId>,
    time_range: Option<(u64, u64)>,
    offset: usize,
    limit: usize,
) -> Result<Vec<Ticket>, Error> {
    metrics::get_txs_with_chain(src_chain, dst_chain, token_id, time_range, offset, limit).await
}

#[query]
pub async fn get_txs_with_account(
    sender: Option<ChainId>,
    receiver: Option<ChainId>,
    token_id: Option<TokenId>,
    time_range: Option<(u64, u64)>,
    offset: usize,
    limit: usize,
) -> Result<Vec<Ticket>, Error> {
    metrics::get_txs_with_account(sender, receiver, token_id, time_range, offset, limit).await
}

#[query]
pub async fn get_txs(offset: usize, limit: usize) -> Result<Vec<Ticket>, Error> {
    metrics::get_txs(offset, limit).await
}

#[query]
pub async fn get_total_tx() -> Result<u64, Error> {
    metrics::get_total_tx().await
}

#[query(hidden = true)]
fn http_request(req: HttpRequest) -> HttpResponse {
    StableLogWriter::http_request(req)
}

#[query(guard = "auth_query")]
pub async fn get_logs(time: Option<u64>, offset: usize, limit: usize) -> Vec<String> {
    let max_skip_timestamp = time.unwrap_or(0);
    StableLogWriter::get_logs(max_skip_timestamp, offset, limit)
}

#[query(guard = "auth_query")]
fn get_events(args: GetEventsArg) -> Vec<Event> {
    event::events(args)
}

#[query(guard = "auth_query")]
pub async fn get_chain_metas(offset: usize, limit: usize) -> Result<Vec<ChainMeta>, Error> {
    metrics::get_chain_metas(offset, limit).await
}

#[query(guard = "auth_query")]
pub async fn get_chain_size() -> Result<u64, Error> {
    metrics::get_chain_size().await
}

#[query(guard = "auth_query")]
pub async fn get_token_metas(offset: usize, limit: usize) -> Result<Vec<TokenMeta>, Error> {
    metrics::get_token_metas(offset, limit).await
}

#[query(guard = "auth_query")]
pub async fn get_token_size() -> Result<u64, Error> {
    metrics::get_token_size().await
}

#[query(guard = "auth_query")]
pub async fn get_directive_size() -> Result<u64, Error> {
    metrics::get_directive_size().await
}

#[query(guard = "auth_query")]
pub async fn get_directives(offset: usize, limit: usize) -> Result<Vec<Directive>, Error> {
    metrics::get_directives(offset, limit).await
}

#[query(guard = "auth_query")]
pub async fn sync_ticket_size() -> Result<u64, Error> {
    with_metrics(|metrics| metrics.sync_ticket_size())
}

#[query(guard = "auth_query")]
pub async fn sync_tickets(offset: usize, limit: usize) -> Result<Vec<(u64, Ticket)>, Error> {
    with_metrics(|metrics| metrics.sync_tickets(offset, limit))
}

#[query(guard = "auth_query")]
pub async fn get_tx_hash_size() -> Result<u64, Error> {
    with_state(|hub_state| {
        let tx_hash_size = hub_state.tx_hashes.len();
        Ok(tx_hash_size)
    })
}

#[query(guard = "auth_query")]
pub async fn get_tx_hashes(offset: usize, limit: usize) -> Result<Vec<(TicketId, TxHash)>, Error> {
    let tx_hashes = with_state(|hub_state| {
        hub_state
            .tx_hashes
            .iter()
            .skip(offset)
            .take(limit)
            .map(|(ticket_id, tx_hash)| (ticket_id, tx_hash))
            .collect::<Vec<_>>()
    });
    Ok(tx_hashes)
}

#[update(guard = "auth_update")]
pub async fn pending_ticket(ticket: Ticket) -> Result<(), Error> {
    debug!("pending_ticket: {:?}", ticket);
    with_state_mut(|hub_state| hub_state.pending_ticket(ticket))
}

#[update(guard = "auth_update")]
pub async fn finalize_ticket(ticket_id: String) -> Result<(), Error> {
    debug!("finaize_ticket: {:?}", ticket_id);

    with_state_mut(|hub_state| hub_state.finalize_ticket(&ticket_id))
}

#[query(guard = "auth_query")]
pub async fn get_pending_ticket_size() -> Result<u64, Error> {
    with_state(|hub_state| {
        let pending_size = hub_state.pending_tickets.len();
        Ok(pending_size)
    })
}

#[query(guard = "auth_query")]
pub async fn get_pending_tickets(
    offset: usize,
    limit: usize,
) -> Result<Vec<(TicketId, Ticket)>, Error> {
    let pending_tickets = with_state(|hub_state| {
        hub_state
            .pending_tickets
            .iter()
            .skip(offset)
            .take(limit)
            .map(|(ticket_id, tickets)| (ticket_id, tickets))
            .collect::<Vec<_>>()
    });
    Ok(pending_tickets)
}

fn main() {}

ic_cdk::export_candid!();

#[cfg(test)]
mod tests {

    use super::*;

    use ic_base_types::PrincipalId;
    use omnity_hub::{
        lifecycle::init::InitArgs,
        types::{ChainMeta, TokenMeta},
    };
    use omnity_types::{
        ChainType, Factor, FeeTokenFactor, TargetChainFactor, Ticket, TicketType, ToggleAction,
        ToggleState, TxAction,
    };

    // use env_logger;
    // use log::LevelFilter;
    use std::{
        collections::HashMap,
        time::{SystemTime, UNIX_EPOCH},
    };
    use uuid::Uuid;

    async fn init_hub() {
        let arg = HubArg::Init(InitArgs {
            admin: PrincipalId::new_user_test_id(1).0,
        });
        init(arg);
        set_logger_filter("debug".to_string()).await;
    }
    pub fn get_logs(
        max_skip_timestamp: &Option<u64>,
        offset: &usize,
        limit: &usize,
    ) -> Vec<String> {
        let url = if let Some(max_skip_timestamp) = max_skip_timestamp {
            format!(
                "/logs?time={}&offset={}&limit={}",
                max_skip_timestamp, offset, limit
            )
        } else {
            format!("/logs?offset={}&limit={}", offset, limit)
        };

        let request = HttpRequest {
            method: "".to_string(),
            url: url,
            headers: vec![],
            body: serde_bytes::ByteBuf::new(),
        };

        let response = http_request(request);
        serde_json::from_slice(&response.body).expect("failed to parse hub log")
    }

    fn get_timestamp() -> u64 {
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        since_the_epoch.as_nanos() as u64
    }

    fn chain_ids() -> Vec<String> {
        vec![
            "Bitcoin".to_string(),
            "Ethereum".to_string(),
            "ICP".to_string(),
            "EVM-Arbitrum".to_string(),
            "EVM-Optimistic".to_string(),
            "EVM-Starknet".to_string(),
        ]
    }

    fn token_ids() -> Vec<String> {
        vec![
            "BTC".to_string(),
            "Bitcoin-RUNES-150:1".to_string(),
            "Bitcoin-RUNES-WTF".to_string(),
            "ETH".to_string(),
            "Ethereum-ERC20-ARB".to_string(),
            "Ethereum-ERC20-OP".to_string(),
            "Ethereum-ERC20-Starknet".to_string(),
            "ICP".to_string(),
            "ICP-ICRC2-XXX".to_string(),
            "ICP-ICRC2-XXY".to_string(),
        ]
    }
    fn default_topic() -> Vec<Topic> {
        vec![
            Topic::AddChain,
            Topic::UpdateChain,
            Topic::AddToken,
            Topic::UpdateToken,
            Topic::UpdateFee,
            Topic::ToggleChainState,
        ]
    }
    async fn sub_dires() {
        for chain_id in chain_ids() {
            let result = sub_directives(Some(chain_id.to_string()), default_topic()).await;
            println!("chain({}) sub topic result: {:?}", chain_id, result)
        }
    }
    async fn unsub_dires() {
        for chain_id in chain_ids() {
            let result = unsub_directives(Some(chain_id.to_string()), default_topic()).await;
            println!("chain({}) unsub topic result: {:?}", chain_id, result)
        }
    }
    async fn add_chains() {
        let btc = ChainMeta {
            chain_id: "Bitcoin".to_string(),
            chain_type: ChainType::SettlementChain,
            chain_state: ChainState::Active,
            canister_id: "bkyz2-fmaaa-aaaaa-qaaaq-cai".to_string(),
            contract_address: None,
            counterparties: None,
            fee_token: None,
        };

        // validate proposal
        let result = validate_proposal(vec![Proposal::AddChain(btc.clone())]).await;
        assert!(result.is_ok());
        println!(
            "validate_proposal for Proposal::AddChain(chain_info) result:{:#?}",
            result
        );
        // build directive
        let result = execute_proposal(vec![Proposal::AddChain(btc)]).await;
        assert!(result.is_ok());

        let btc = ChainMeta {
            chain_id: "Bitcoin".to_string(),
            chain_type: ChainType::SettlementChain,
            chain_state: ChainState::Active,
            canister_id: "bkyz2-fmaaa-aaaaa-qaaaq-cai".to_string(),
            contract_address: None,
            counterparties: None,
            fee_token: None,
        };

        // validate proposal
        let result = validate_proposal(vec![Proposal::AddChain(btc.clone())]).await;
        println!(
            "validate_proposal for Proposal::AddChain(chain_info) result:{:#?}",
            result
        );
        assert!(result.is_err());
        // build directive
        // let result = execute_proposal(vec![Proposal::AddChain(btc)]).await;
        // assert!(result.is_ok());

        let ethereum = ChainMeta {
            chain_id: "Ethereum".to_string(),
            chain_type: ChainType::SettlementChain,
            chain_state: ChainState::Active,
            canister_id: "bkyz2-fmaaa-aaaaa-qaaab-cai".to_string(),
            contract_address: Some("Ethereum constract address".to_string()),
            counterparties: Some(vec!["Bitcoin".to_string()]),
            fee_token: None,
        };
        let result = validate_proposal(vec![Proposal::AddChain(ethereum.clone())]).await;
        assert!(result.is_ok());
        println!(
            "validate_proposal for Proposal::AddChain(chain_info) result:{:#?}",
            result
        );
        let result = execute_proposal(vec![Proposal::AddChain(ethereum)]).await;
        assert!(result.is_ok());

        let icp = ChainMeta {
            chain_id: "ICP".to_string(),
            chain_type: ChainType::ExecutionChain,
            chain_state: ChainState::Active,
            canister_id: "bkyz2-fmaaa-aaaaa-qadaab-cai".to_string(),
            contract_address: Some("bkyz2-fmaaa-aaafa-qadaab-cai".to_string()),
            counterparties: Some(vec!["Bitcoin".to_string(), "Ethereum".to_string()]),
            fee_token: Some("ICP".to_owned()),
        };

        let result = validate_proposal(vec![Proposal::AddChain(icp.clone())]).await;
        assert!(result.is_ok());
        println!(
            "validate_proposal for Proposal::AddChain(chain_info) result:{:#?}",
            result
        );
        let result = execute_proposal(vec![Proposal::AddChain(icp)]).await;
        assert!(result.is_ok());

        let arbitrum = ChainMeta {
            chain_id: "EVM-Arbitrum".to_string(),
            chain_type: ChainType::ExecutionChain,
            chain_state: ChainState::Active,
            canister_id: "bkyz2-fmaaa-aaasaa-qadaab-cai".to_string(),
            contract_address: Some("Arbitrum constract address".to_string()),
            counterparties: Some(vec![
                "Bitcoin".to_string(),
                "Ethereum".to_string(),
                "ICP".to_string(),
            ]),
            fee_token: Some("Ethereum-ERC20-ARB".to_owned()),
        };
        let result = validate_proposal(vec![Proposal::AddChain(arbitrum.clone())]).await;
        assert!(result.is_ok());
        println!(
            "validate_proposal for Proposal::AddChain(chain_info) result:{:#?}",
            result
        );
        let result = execute_proposal(vec![Proposal::AddChain(arbitrum)]).await;
        assert!(result.is_ok());

        let optimistic = ChainMeta {
            chain_id: "EVM-Optimistic".to_string(),
            chain_type: ChainType::ExecutionChain,
            chain_state: ChainState::Active,
            canister_id: "bkyz2-fdmaaa-aaasaa-qadaab-cai".to_string(),
            contract_address: Some("Optimistic constract address".to_string()),
            counterparties: Some(vec![
                "Bitcoin".to_string(),
                "Ethereum".to_string(),
                "ICP".to_string(),
                "EVM-Arbitrum".to_string(),
            ]),
            fee_token: Some("Ethereum-ERC20-OP".to_owned()),
        };

        let result = validate_proposal(vec![Proposal::AddChain(optimistic.clone())]).await;
        assert!(result.is_ok());
        println!(
            "validate_proposal for Proposal::AddChain(chain_info) result:{:#?}",
            result
        );
        let result = execute_proposal(vec![Proposal::AddChain(optimistic)]).await;
        assert!(result.is_ok());

        let starknet = ChainMeta {
            chain_id: "EVM-Starknet".to_string(),
            chain_type: ChainType::ExecutionChain,
            chain_state: ChainState::Active,
            canister_id: "bkyz2-fddmaaa-aaasaa-qadaab-cai".to_string(),
            contract_address: Some("Starknet constract address".to_string()),
            counterparties: Some(vec![
                "Bitcoin".to_string(),
                "Ethereum".to_string(),
                "ICP".to_string(),
                "EVM-Arbitrum".to_string(),
                "EVM-Optimistic".to_string(),
            ]),
            fee_token: Some("Ethereum-ERC20-StarkNet".to_owned()),
        };
        let result = validate_proposal(vec![Proposal::AddChain(starknet.clone())]).await;
        assert!(result.is_ok());
        println!(
            "validate_proposal for Proposal::AddChain(chain_info) result:{:#?}",
            result
        );
        let result = execute_proposal(vec![Proposal::AddChain(starknet)]).await;
        assert!(result.is_ok());
    }

    async fn add_tokens() {
        let btc = TokenMeta {
            token_id: "BTC".to_string(),
            name: "BTC".to_owned(),
            symbol: "BTC".to_owned(),
            issue_chain: "Bitcoin".to_string(),
            decimals: 18,
            icon: None,
            metadata: HashMap::default(),
            dst_chains: vec![
                "Ethereum".to_string(),
                "ICP".to_string(),
                "EVM-Arbitrum".to_string(),
                "EVM-Optimistic".to_string(),
                "EVM-Starknet".to_string(),
            ],
        };

        // validate proposal
        let result = validate_proposal(vec![Proposal::AddToken(btc.clone())]).await;
        println!(
            "validate_proposal for Proposal::AddToken(token) result:{:#?}",
            result
        );
        assert!(result.is_ok());
        // build directive
        let result = execute_proposal(vec![Proposal::AddToken(btc)]).await;
        assert!(result.is_ok());

        let runse_150 = TokenMeta {
            token_id: "Bitcoin-RUNES-150:1".to_string(),
            name: "150:1".to_owned(),
            symbol: "150:1".to_owned(),
            issue_chain: "Bitcoin".to_string(),
            decimals: 18,
            icon: None,
            metadata: HashMap::from([("rune_id".to_string(), "150:1".to_string())]),
            dst_chains: vec![
                "Ethereum".to_string(),
                "ICP".to_string(),
                "EVM-Arbitrum".to_string(),
                "EVM-Optimistic".to_string(),
                "EVM-Starknet".to_string(),
            ],
        };
        let result = validate_proposal(vec![Proposal::AddToken(runse_150.clone())]).await;
        println!(
            "validate_proposal for Proposal::AddToken(token) result:{:#?}",
            result
        );
        assert!(result.is_ok());

        let result = execute_proposal(vec![Proposal::AddToken(runse_150)]).await;
        assert!(result.is_ok());

        let runes_wtf = TokenMeta {
            token_id: "Bitcoin-RUNES-WTF".to_string(),
            name: "BTC".to_owned(),
            symbol: "BTC".to_owned(),
            issue_chain: "Bitcoin".to_string(),
            decimals: 18,
            icon: None,
            metadata: HashMap::from([("rune_id".to_string(), "WTF".to_string())]),
            dst_chains: vec![
                "Ethereum".to_string(),
                "ICP".to_string(),
                "EVM-Arbitrum".to_string(),
                "EVM-Optimistic".to_string(),
                "EVM-Starknet".to_string(),
            ],
        };
        let result = validate_proposal(vec![Proposal::AddToken(runes_wtf.clone())]).await;
        println!(
            "validate_proposal for Proposal::AddToken(token) result:{:#?}",
            result
        );
        assert!(result.is_ok());

        let result = execute_proposal(vec![Proposal::AddToken(runes_wtf)]).await;
        assert!(result.is_ok());

        let eth = TokenMeta {
            token_id: "ETH".to_string(),
            name: "ETH".to_owned(),
            symbol: "ETH".to_owned(),
            issue_chain: "Ethereum".to_string(),
            decimals: 18,
            icon: None,
            metadata: HashMap::default(),
            dst_chains: vec![
                "Bitcoin".to_string(),
                "ICP".to_string(),
                "EVM-Arbitrum".to_string(),
                "EVM-Optimistic".to_string(),
                "EVM-Starknet".to_string(),
            ],
        };
        let result = validate_proposal(vec![Proposal::AddToken(eth.clone())]).await;
        assert!(result.is_ok());
        println!(
            "validate_proposal for Proposal::AddToken(token) result:{:#?}",
            result
        );
        let result = execute_proposal(vec![Proposal::AddToken(eth)]).await;
        assert!(result.is_ok());

        let icp = TokenMeta {
            token_id: "ICP".to_string(),
            name: "ICP".to_owned(),
            symbol: "ICP".to_owned(),
            issue_chain: "ICP".to_string(),
            decimals: 18,
            icon: None,
            metadata: HashMap::default(),
            dst_chains: vec![
                "Bitcoin".to_string(),
                "Ethereum".to_string(),
                "EVM-Arbitrum".to_string(),
                "EVM-Optimistic".to_string(),
                "EVM-Starknet".to_string(),
            ],
        };
        let result = validate_proposal(vec![Proposal::AddToken(icp.clone())]).await;
        assert!(result.is_ok());
        println!(
            "validate_proposal for Proposal::AddToken(token) result:{:#?}",
            result
        );
        let result = execute_proposal(vec![Proposal::AddToken(icp)]).await;
        assert!(result.is_ok());

        let arb = TokenMeta {
            token_id: "Ethereum-ERC20-ARB".to_string(),
            name: "ARB".to_owned(),
            symbol: "ARB".to_owned(),
            issue_chain: "Ethereum".to_string(),
            decimals: 18,
            icon: None,
            metadata: HashMap::default(),
            dst_chains: vec![
                "Bitcoin".to_string(),
                "Ethereum".to_string(),
                "ICP".to_string(),
                "EVM-Optimistic".to_string(),
                "EVM-Starknet".to_string(),
            ],
        };
        let result = validate_proposal(vec![Proposal::AddToken(arb.clone())]).await;
        assert!(result.is_ok());
        println!(
            "validate_proposal for Proposal::AddToken(token) result:{:#?}",
            result
        );
        let result = execute_proposal(vec![Proposal::AddToken(arb)]).await;
        assert!(result.is_ok());

        let op = TokenMeta {
            token_id: "Ethereum-ERC20-OP".to_string(),
            name: "OP".to_owned(),
            symbol: "OP".to_owned(),
            issue_chain: "Ethereum".to_string(),
            decimals: 18,
            icon: None,
            metadata: HashMap::default(),
            dst_chains: vec![
                "Bitcoin".to_string(),
                "Ethereum".to_string(),
                "ICP".to_string(),
                "EVM-Arbitrum".to_string(),
                "EVM-Starknet".to_string(),
            ],
        };
        let result = validate_proposal(vec![Proposal::AddToken(op.clone())]).await;
        assert!(result.is_ok());
        println!(
            "validate_proposal for Proposal::AddToken(token) result:{:#?}",
            result
        );
        let result = execute_proposal(vec![Proposal::AddToken(op)]).await;
        assert!(result.is_ok());

        let starknet = TokenMeta {
            token_id: "Ethereum-ERC20-StarkNet".to_string(),
            name: "StarkNet".to_owned(),
            symbol: "StarkNet".to_owned(),
            issue_chain: "Ethereum".to_string(),
            decimals: 18,
            icon: None,
            metadata: HashMap::default(),
            dst_chains: vec![
                "Bitcoin".to_string(),
                "Ethereum".to_string(),
                "ICP".to_string(),
                "EVM-Arbitrum".to_string(),
                "EVM-Optimistic".to_string(),
            ],
        };
        let result = validate_proposal(vec![Proposal::AddToken(starknet.clone())]).await;
        assert!(result.is_ok());
        println!(
            "validate_proposal for Proposal::AddToken(token) result:{:#?}",
            result
        );
        let result = execute_proposal(vec![Proposal::AddToken(starknet)]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_sub_unsub() {
        init_hub().await;
        sub_dires().await;
        let result = query_subscribers(None).await;
        println!("query_subscribers result: {:?}", result);
        unsub_dires().await;
        let result = query_subscribers(None).await;
        println!("query_subscribers result: {:?}", result);
    }

    #[tokio::test]
    async fn test_add_chain() {
        init_hub().await;

        // sub_dires().await;
        let result = sub_directives(Some("Bitcoin".to_string()), vec![Topic::AddChain]).await;
        println!(
            "chain({}) sub topic result: {:?}",
            "Bitcoin".to_string(),
            result
        );

        let topic_subs = query_subscribers(None).await.unwrap();
        for (topic, subs) in topic_subs.iter() {
            println!("topic:{:?},subs:{:?}", topic, subs)
        }
        // add chain
        add_chains().await;

        // print directives
        with_state(|hub_state| {
            hub_state.directives.iter().for_each(|(k, v)| {
                println!("directive -> {}, value -> {:?}", k, v);
            })
        });

        let result =
            query_directives(Some("Bitcoin".to_string()), Some(Topic::AddChain), 0, 20).await;
        println!(
            "query_directives for {:} dires: {:#?}",
            "Bitcoin".to_string(),
            result
        );
        assert!(result.is_ok());
        let chain = get_chain("Bitcoin".to_string()).await;
        println!(
            "get chain for {:} chain: {:#?}",
            "Bitcoin".to_string(),
            chain
        );

        // new subscribers
        println!("---- add new subscribers -------");
        sub_dires().await;

        let topic_subs = query_subscribers(None).await.unwrap();
        for (topic, subs) in topic_subs.iter() {
            println!("topic:{:?},subs:{:?}", topic, subs)
        }

        for chain_id in vec![
            "Bitcoin".to_string(),
            "Ethereum".to_string(),
            "ICP".to_string(),
        ] {
            let result =
                query_directives(Some(chain_id.to_string()), Some(Topic::AddChain), 0, 20).await;
            println!("query_directives for {:} dires: {:#?}", chain_id, result);
            assert!(result.is_ok());
            let chain = get_chain(chain_id.to_string()).await;
            println!("get chain for {:} chain: {:#?}", chain_id, chain);
        }

        let result = get_chains(None, None, 0, 10).await;
        println!("get_chains result : {:#?}", result);
        assert!(result.is_ok());

        let result = get_chains(Some(ChainType::ExecutionChain), None, 0, 10).await;
        println!("get_chains result by chain type: {:#?}", result);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_chain() {
        init_hub().await;
        sub_dires().await;
        // add chain
        add_chains().await;

        let icp = ChainMeta {
            chain_id: "ICP".to_string(),
            chain_type: ChainType::ExecutionChain,
            chain_state: ChainState::Active,
            canister_id: "bkyz2-fmaaa-aaaaa-qadaab-cai".to_string(),
            contract_address: Some("bkyz2-fmaaa-aaafa-qadaab-cai".to_string()),
            counterparties: Some(vec![
                "Bitcoin".to_string(),
                "Ethereum".to_string(),
                "EVM-Arbitrum".to_string(),
                "EVM-Optimistic".to_string(),
                "EVM-Starknet".to_string(),
            ]),
            fee_token: Some("ICP".to_owned()),
        };

        let result = handle_chain(vec![Proposal::UpdateChain(icp)]).await;
        println!("handle_chain reuslt : {:#?}", result);
        assert!(result.is_err());

        let chain = get_chain("ICP".to_string()).await;
        println!("before update ,the chain info : {:#?}", chain);
        let icp = ChainMeta {
            chain_id: "ICP".to_string(),
            chain_type: ChainType::ExecutionChain,
            chain_state: ChainState::Active,
            canister_id: "bkyz2-fmaaa-aaaaa-qadaab-cai".to_string(),
            contract_address: Some("bkyz2-fmaaa-aaafa-qadaab-cai".to_string()),
            counterparties: Some(vec![
                "Bitcoin".to_string(),
                "Ethereum".to_string(),
                "bevm_testnet".to_string(),
            ]),
            fee_token: Some("ICP".to_owned()),
        };

        let result = handle_chain(vec![Proposal::UpdateChain(icp)]).await;
        println!("handle_chain reuslt : {:#?}", result);
        assert!(result.is_ok());

        // let chain = get_chain("ICP".to_string()).await;
        // println!("after update ,the chain info : {:#?}", chain);
        for chain_id in chain_ids() {
            let result =
                query_directives(Some(chain_id.to_string()), Some(Topic::UpdateChain), 0, 20).await;
            println!("query_directives for {:} dires: {:#?}", chain_id, result);
            assert!(result.is_ok());
            let chain = get_chain(chain_id.to_string()).await;
            println!("get chain for {:} chain: {:#?}", chain_id, chain);
        }
    }

    #[tokio::test]
    async fn test_add_token() {
        init_hub().await;
        sub_dires().await;

        // sub special token id
        let result = sub_directives(Some("ICP".to_string()), vec![Topic::AddToken]).await;
        println!(
            "chain({}) sub topic result: {:?}",
            "ICP".to_string(),
            result
        );
        //check sub result
        let topic_subs = query_subscribers(None).await.unwrap();
        for (topic, subs) in topic_subs.iter() {
            println!("topic:{:?},subs:{:?}", topic, subs)
        }

        // // add chain
        add_chains().await;
        // add token
        add_tokens().await;

        for chain_id in chain_ids() {
            let result =
                query_directives(Some(chain_id.to_string()), Some(Topic::AddToken), 0, 50).await;
            println!("query_directives for {:} dires: {:#?}", chain_id, result);
            assert!(result.is_ok());
        }

        for token_id in token_ids() {
            let result = get_tokens(None, Some(token_id.to_string()), 0, 5).await;
            println!("get tokens for {:} tokens: {:#?}", token_id, result);
            assert!(result.is_ok());
        }
        let result = get_tokens(None, None, 0, 10).await;
        assert!(result.is_ok());
        println!("get_tokens result : {:#?}", result);

        let result = get_tokens(Some("Bitcoin".to_string()), None, 0, 10).await;
        assert!(result.is_ok());
        println!("get_tokens result by chain_id: {:#?}", result);
        let result = get_tokens(Some("ICP".to_string()), Some("ICP".to_string()), 0, 10).await;
        assert!(result.is_ok());
        println!("get_tokens result by chain_id and token id: {:#?}", result);
    }

    #[tokio::test]
    async fn test_handle_token() {
        init_hub().await;
        sub_dires().await;
        // add chain
        add_chains().await;
        // add token
        add_tokens().await;

        let bevm = ChainMeta {
            chain_id: "bevm_testnet".to_string(),
            chain_type: ChainType::ExecutionChain,
            chain_state: ChainState::Active,
            canister_id: "bonsh-yiaaa-aaaap-qhlhq-cai".to_string(),
            contract_address: Some("bevm constract address".to_string()),
            counterparties: Some(vec![
                "Bitcoin".to_string(),
                "Ethereum".to_string(),
                "ICP".to_string(),
                "EVM-Arbitrum".to_string(),
                "EVM-Optimistic".to_string(),
                "EVM-Starknet".to_string(),
            ]),
            fee_token: Some("Bitcoin-BRC20-BEVM".to_owned()),
        };

        let result = handle_chain(vec![Proposal::AddChain(bevm)]).await;
        println!("handle_chain result:{:#?}", result);
        assert!(result.is_ok());
        let chain = get_chain("bevm_testnet".to_string()).await;
        println!(
            "get chain for {:} chain: {:#?}",
            "bevm_testnet".to_string(),
            chain
        );

        let result = with_state(|hub_state| hub_state.token(&"ICP".to_string()));

        println!("before update,the token info: {:#?}", result);
        assert!(result.is_ok());

        let icp_token = TokenMeta {
            token_id: "ICP".to_string(),
            name: "ICP".to_owned(),
            symbol: "ICP".to_owned(),
            issue_chain: "ICP".to_string(),
            decimals: 18,
            icon: None,
            metadata: HashMap::default(),
            dst_chains: vec![
                "Bitcoin".to_string(),
                "Ethereum".to_string(),
                "EVM-Arbitrum".to_string(),
                "EVM-Optimistic".to_string(),
                "EVM-Starknet".to_string(),
                "bevm_testnet".to_string(),
            ],
        };
        let result = handle_token(vec![Proposal::UpdateToken(icp_token.clone())]).await;
        println!("handle_token result:{:#?}", result);
        assert!(result.is_ok());

        let result = get_tokens(Some("ICP".to_string()), Some("ICP".to_string()), 0, 10).await;
        println!("get_tokens result by chain_id and token id: {:#?}", result);
        assert!(result.is_ok());
        for chain_id in chain_ids() {
            let result =
                query_directives(Some(chain_id.to_string()), Some(Topic::UpdateToken), 0, 20).await;
            println!("query_directives for {:} dires: {:#?}", chain_id, result);
            assert!(result.is_ok());
            let chain = get_chain(chain_id.to_string()).await;
            println!("get chain for {:} chain: {:#?}", chain_id, chain);
        }
    }

    #[tokio::test]
    async fn test_toggle_chain_state() {
        init_hub().await;
        sub_dires().await;
        // add chain
        add_chains().await;
        // add token
        add_tokens().await;

        println!("------------ state switch: from active to deactive -----------------");
        // change chain state to deactivate
        let chain_state = ToggleState {
            chain_id: "EVM-Optimistic".to_string(),
            action: ToggleAction::Deactivate,
        };

        let toggle_state = Proposal::ToggleChainState(chain_state);
        let result = validate_proposal(vec![toggle_state.clone()]).await;
        println!(
            "validate_proposal for Proposal::ToggleChainState(chain_state) result:{:#?}",
            result
        );
        assert!(result.is_ok());
        let result = execute_proposal(vec![toggle_state]).await;
        println!(
            "execute_proposal for Proposal::ToggleChainState(chain_state) result:{:#?}",
            result
        );
        assert!(result.is_ok());

        // query directives for chain id
        for chain_id in chain_ids() {
            let result = query_directives(
                Some(chain_id.to_string()),
                // None,
                Some(Topic::ToggleChainState),
                0,
                5,
            )
            .await;
            println!("query_directives for {:} dires: {:#?}", chain_id, result);
            assert!(result.is_ok());
        }

        let result = get_chains(None, Some(ChainState::Deactive), 0, 10).await;
        // let result = get_chains(None, None, 0, 10).await;
        assert!(result.is_ok());
        println!(
            "get_chains result by chain type and chain state: {:#?}",
            result
        );

        println!("------------ state switch: from deactive to active -----------------");
        // change chain state to active
        let chain_state = ToggleState {
            chain_id: "EVM-Optimistic".to_string(),
            action: ToggleAction::Activate,
        };

        let toggle_state = Proposal::ToggleChainState(chain_state);
        let result = validate_proposal(vec![toggle_state.clone()]).await;
        println!(
            "validate_proposal for Proposal::ToggleChainState(chain_state) result:{:#?}",
            result
        );
        assert!(result.is_ok());
        let result = execute_proposal(vec![toggle_state]).await;
        println!(
            "execute_proposal for Proposal::ToggleChainState(chain_state) result:{:#?}",
            result
        );
        assert!(result.is_ok());

        // query directives for chain id
        for chain_id in chain_ids() {
            let result = query_directives(
                Some(chain_id.to_string()),
                // None,
                Some(Topic::ToggleChainState),
                0,
                5,
            )
            .await;
            println!("query_directives for {:} dires: {:#?}", chain_id, result);
            assert!(result.is_ok());
        }

        let result = get_chains(None, Some(ChainState::Deactive), 0, 10).await;
        // let result = get_chains(None, None, 0, 10).await;
        assert!(result.is_ok());
        println!(
            "get_chains result by chain type and chain state: {:#?}",
            result
        );
        let chain = get_chain("EVM-Optimistic".to_string()).await;
        println!(
            "get chain for {:} chain: {:#?}",
            "EVM-Optimistic".to_string(),
            chain
        );
    }

    #[tokio::test]
    async fn test_update_fee() {
        init_hub().await;

        sub_dires().await;
        // add chain
        add_chains().await;
        // add token
        add_tokens().await;

        // sub special token id
        let result = sub_directives(Some("ICP".to_string()), vec![Topic::UpdateFee]).await;
        println!(
            "chain({}) sub topic result: {:?}",
            "ICP".to_string(),
            result
        );

        //  chain factor
        let chain_factor = Factor::UpdateTargetChainFactor(TargetChainFactor {
            target_chain_id: "Bitcoin".to_string(),
            target_chain_factor: 10000,
        });

        //  token factor
        let token_factor = Factor::UpdateFeeTokenFactor(FeeTokenFactor {
            fee_token: "ICP".to_string(),
            fee_token_factor: 60_000_000_000,
        });

        let result = update_fee(vec![chain_factor, token_factor]).await;
        assert!(result.is_ok());
        println!("update_fee result:{:?}", result);

        // query directives for chain id
        for chain_id in chain_ids() {
            let result =
                query_directives(Some(chain_id.to_string()), Some(Topic::UpdateFee), 0, 5).await;
            println!("query_directives for {:} dires: {:#?}", chain_id, result);
            assert!(result.is_ok());
        }
        for chain_id in chain_ids() {
            let result =
                query_directives(Some(chain_id.to_string()), Some(Topic::UpdateFee), 0, 5).await;
            println!("query_directives for {:} dires: {:#?}", chain_id, result);
            assert!(result.is_ok());
        }

        assert!(result.is_ok());
        let result = get_fees(None, None, 0, 10).await;
        assert!(result.is_ok());
        println!("get_fees result : {:#?}", result);

        let result = get_fees(None, Some("ICP".to_string()), 0, 12).await;
        assert!(result.is_ok());
        println!("get_fees result filter by token id : {:#?}", result);

        let result = query_directives(Some("ICP".to_string()), None, 0, 20).await;
        println!(
            "query_directives for {:} dires: {:#?}",
            "ICP".to_string(),
            result
        );
        //unsub all the topic for icp
        let result = unsub_directives(Some("ICP".to_string()), default_topic()).await;
        println!(
            "chain({}) unsub topic result: {:?}",
            "ICP".to_string(),
            result
        );
        let topic_subs = query_subscribers(None).await.unwrap();
        for (topic, subs) in topic_subs.iter() {
            println!("topic:{:?},subs:{:?}", topic, subs)
        }
        let result = query_directives(Some("ICP".to_string()), None, 0, 20).await;
        println!(
            "query_directives for {:} dires: {:#?}",
            "ICP".to_string(),
            result
        );
    }

    #[tokio::test]
    async fn test_new_subscriber() {
        init_hub().await;
        // sub_dires().await;
        let chains = vec![
            "Bitcoin".to_string(),
            "Ethereum".to_string(),
            // "ICP".to_string(),
            "EVM-Arbitrum".to_string(),
            "EVM-Optimistic".to_string(),
            "EVM-Starknet".to_string(),
        ];
        let topics = vec![
            Topic::AddChain,
            Topic::UpdateChain,
            Topic::AddToken,
            Topic::UpdateToken,
            Topic::UpdateFee,
            Topic::ToggleChainState,
        ];
        for chain_id in chains.iter() {
            let result = sub_directives(Some(chain_id.to_string()), topics.to_vec()).await;
            println!("chain({}) sub topic result: {:?}", chain_id, result)
        }

        // add chain
        add_chains().await;
        // add token
        add_tokens().await;

        let bevm = ChainMeta {
            chain_id: "bevm_testnet".to_string(),
            chain_type: ChainType::ExecutionChain,
            chain_state: ChainState::Active,
            canister_id: "bonsh-yiaaa-aaaap-qhlhq-cai".to_string(),
            contract_address: Some("bevm constract address".to_string()),
            counterparties: Some(vec![
                "Bitcoin".to_string(),
                "Ethereum".to_string(),
                "ICP".to_string(),
                "EVM-Arbitrum".to_string(),
                "EVM-Optimistic".to_string(),
                "EVM-Starknet".to_string(),
            ]),
            fee_token: Some("Bitcoin-BRC20-BEVM".to_owned()),
        };

        let result = handle_chain(vec![Proposal::AddChain(bevm)]).await;
        println!("handle_chain result:{:#?}", result);
        assert!(result.is_ok());
        let chain = get_chain("bevm_testnet".to_string()).await;
        println!(
            "get chain for {:} chain: {:#?}",
            "bevm_testnet".to_string(),
            chain
        );

        let icp_token = TokenMeta {
            token_id: "ICP".to_string(),
            name: "ICP".to_owned(),
            symbol: "ICP".to_owned(),
            issue_chain: "ICP".to_string(),
            decimals: 18,
            icon: None,
            metadata: HashMap::default(),
            dst_chains: vec![
                "Bitcoin".to_string(),
                "Ethereum".to_string(),
                "EVM-Arbitrum".to_string(),
                "EVM-Optimistic".to_string(),
                "EVM-Starknet".to_string(),
                "bevm_testnet".to_string(),
            ],
        };
        let result = handle_token(vec![Proposal::UpdateToken(icp_token.clone())]).await;
        println!("handle_token result:{:#?}", result);
        assert!(result.is_ok());

        for chain_id in chains.iter() {
            let result = query_directives(Some(chain_id.to_string()), None, 0, 50).await;
            println!("query_directives for {:} dires: {:#?}", chain_id, result);
            assert!(result.is_ok());
        }
        let result = query_directives(Some("ICP".to_string()), None, 0, 50).await;
        println!(
            "query_directives for {:} dires: {:#?}",
            "ICP".to_string(),
            result
        );
        assert!(result.is_ok());

        // add new subscriber
        let result = sub_directives(Some("ICP".to_string()), topics.to_vec()).await;
        println!(
            "chain({}) sub topic result: {:?}",
            "ICP".to_string(),
            result
        );

        let result = query_directives(Some("ICP".to_string()), None, 0, 50).await;
        println!(
            "query_directives for {:} dires: {:#?}",
            "ICP".to_string(),
            result
        );
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_a_b_tx_ticket() {
        init_hub().await;
        sub_dires().await;
        // add chain
        add_chains().await;
        // add token
        add_tokens().await;

        // A->B: `transfer` ticket
        let src_chain = "Bitcoin";
        // let dst_chain = "Bitcoin";
        let dst_chain = "EVM-Arbitrum";
        let sender = "address_on_Bitcoin";
        let receiver = "address_on_Arbitrum";
        let token = "Bitcoin-RUNES-150:1".to_string();

        let transfer_ticket = Ticket {
            ticket_id: Uuid::new_v4().to_string(),
            ticket_type: TicketType::Normal,
            ticket_time: get_timestamp(),
            src_chain: src_chain.to_string(),
            dst_chain: dst_chain.to_string(),
            action: TxAction::Transfer,
            token: token.clone(),
            amount: 88888.to_string(),
            sender: Some(sender.to_string()),
            receiver: receiver.to_string(),
            memo: None,
        };

        println!(
            " {} -> {} ticket:{:#?}",
            src_chain, dst_chain, transfer_ticket
        );
        let result = send_ticket(transfer_ticket).await;
        println!(
            "{} -> {} transfer result:{:?}",
            src_chain, dst_chain, result
        );
        assert!(result.is_ok());

        with_state(|hus_state| {
            hus_state.ticket_map.iter().for_each(|(seq_key, ticket)| {
                println!(" seq key: {:?} ticket: {:?}", seq_key, ticket)
            })
        });

        // query tickets for chain id
        let result = query_tickets(Some(dst_chain.to_string()), 0, 5).await;
        println!("query tickets for {:} tickets: {:#?}", dst_chain, result);
        assert!(result.is_ok());

        // query token on chain
        let result = get_chain_tokens(None, None, 0, 5).await;
        println!("get_chain_tokens result: {:#?}", result);
        assert!(result.is_ok());

        // query tx from get_txs
        let result = get_txs_with_chain(Some(src_chain.to_string()), None, None, None, 0, 10).await;
        println!(
            "get_txs by src chain({}) result: {:#?}",
            src_chain.to_string(),
            result
        );
        assert!(result.is_ok());

        // B->A: `redeem` ticket
        let src_chain = "EVM-Arbitrum";
        let dst_chain = "Bitcoin";
        let sender = "address_on_Arbitrum";
        let receiver = "address_on_Bitcoin";

        let redeem_ticket = Ticket {
            ticket_id: Uuid::new_v4().to_string(),
            ticket_type: TicketType::Normal,
            ticket_time: get_timestamp(),
            src_chain: src_chain.to_string(),
            dst_chain: dst_chain.to_string(),
            action: TxAction::Redeem,
            token: token.clone(),
            amount: 22222.to_string(),
            sender: Some(sender.to_string()),
            receiver: receiver.to_string(),
            memo: None,
        };

        println!(
            " {} -> {} ticket:{:#?}",
            src_chain, dst_chain, redeem_ticket
        );
        let result = send_ticket(redeem_ticket).await;
        println!("{} -> {} redeem result:{:?}", src_chain, dst_chain, result);
        assert!(result.is_ok());

        with_state(|hus_state| {
            hus_state.ticket_map.iter().for_each(|(seq_key, ticket)| {
                println!(" seq key: {:?} ticket: {:?}", seq_key, ticket)
            })
        });

        // query tickets for chain id
        let result = query_tickets(Some(dst_chain.to_string()), 0, 5).await;
        assert!(result.is_ok());
        println!("query tickets for {:} tickets: {:#?}", dst_chain, result);
        // query token on chain
        let result = get_chain_tokens(None, None, 0, 5).await;
        println!("get_chain_tokens result: {:#?}", result);
        assert!(result.is_ok());

        // query tx from get_txs
        let result = get_txs_with_chain(None, Some(dst_chain.to_string()), None, None, 0, 10).await;
        println!(
            "get_txs by dst chain({}) result: {:#?}",
            dst_chain.to_string(),
            result
        );
        assert!(result.is_ok());

        // query tx from get_txs
        let result = get_txs_with_chain(
            None,
            None,
            Some("Bitcoin-RUNES-150:1".to_string()),
            None,
            0,
            10,
        )
        .await;
        println!(
            "get_txs by token ({}) result: {:#?}",
            "BTC".to_string(),
            result
        );
        assert!(result.is_ok());

        //print ticket seq
        with_state(|hub_state| {
            hub_state
                .ticket_seq
                .iter()
                .for_each(|(chain_id, latest_seq)| {
                    println!("chain:{},latest seq:{}", chain_id, latest_seq)
                })
        })
    }

    #[tokio::test]
    async fn test_a_b_c_tx_ticket() {
        init_hub().await;
        // add chain
        add_chains().await;
        // add token
        add_tokens().await;

        // transfer
        // A->B: `transfer` ticket
        let src_chain = "Ethereum";
        let dst_chain = "EVM-Optimistic";
        let sender = "address_on_Ethereum";
        let receiver = "address_on_Optimistic";
        let token = "ETH".to_string();

        let a_2_b_ticket = Ticket {
            ticket_id: Uuid::new_v4().to_string(),
            ticket_type: TicketType::Normal,
            ticket_time: get_timestamp(),
            src_chain: src_chain.to_string(),
            dst_chain: dst_chain.to_string(),
            action: TxAction::Transfer,
            token: token.clone(),
            amount: 6666.to_string(),
            sender: Some(sender.to_string()),
            receiver: receiver.to_string(),
            memo: None,
        };

        println!(" {} -> {} ticket:{:#?}", src_chain, dst_chain, a_2_b_ticket);
        let result = send_ticket(a_2_b_ticket).await;
        println!(
            "{} -> {} transfer result:{:?}",
            src_chain, dst_chain, result
        );
        assert!(result.is_ok());
        // query tickets for chain id
        let result = query_tickets(Some(dst_chain.to_string()), 0, 5).await;
        println!("query tickets for {:} tickets: {:#?}", dst_chain, result);
        assert!(result.is_ok());

        // query token on chain
        let result = get_chain_tokens(None, None, 0, 5).await;
        println!("get_chain_tokens result: {:#?}", result);
        assert!(result.is_ok());

        // query txs
        let result =
            get_txs_with_account(None, Some(receiver.to_string()), None, None, 0, 10).await;
        println!(
            "get_txs_with_account({}) result: {:#?}",
            receiver.to_string(),
            result
        );
        assert!(result.is_ok());

        // B->C: `transfer` ticket
        let sender = "address_on_Optimistic";
        let receiver = "address_on_Starknet";
        let src_chain = "EVM-Optimistic";
        let dst_chain = "EVM-Starknet";

        let b_2_c_ticket = Ticket {
            ticket_id: Uuid::new_v4().to_string(),
            ticket_type: TicketType::Normal,
            ticket_time: get_timestamp(),
            src_chain: src_chain.to_string(),
            dst_chain: dst_chain.to_string(),
            action: TxAction::Transfer,
            token: token.clone(),
            amount: 1111.to_string(),
            sender: Some(sender.to_string()),
            receiver: receiver.to_string(),
            memo: None,
        };

        println!(" {} -> {} ticket:{:#?}", src_chain, dst_chain, b_2_c_ticket);
        assert!(result.is_ok());

        let result = send_ticket(b_2_c_ticket).await;
        println!(
            "{} -> {} transfer result:{:?}",
            src_chain, dst_chain, result
        );

        // query tickets for chain id
        let result = query_tickets(Some(dst_chain.to_string()), 0, 5).await;
        println!("query tickets for {:} tickets: {:#?}", dst_chain, result);
        assert!(result.is_ok());

        // query token on chain
        let result = get_chain_tokens(None, None, 0, 5).await;
        println!("get_chain_tokens result: {:#?}", result);
        assert!(result.is_ok());

        let result =
            get_txs_with_account(None, Some(receiver.to_string()), None, None, 0, 10).await;
        println!(
            "get_txs_with_account({}) result: {:#?}",
            receiver.to_string(),
            result
        );
        assert!(result.is_ok());
        // redeem
        // C->B: `redeem` ticket
        let src_chain = "EVM-Starknet";
        let dst_chain = "EVM-Optimistic";
        let sender = "address_on_Starknet";
        let receiver = "address_on_Optimistic";

        let c_2_b_ticket = Ticket {
            ticket_id: Uuid::new_v4().to_string(),
            ticket_type: TicketType::Normal,
            ticket_time: get_timestamp(),
            src_chain: src_chain.to_string(),
            dst_chain: dst_chain.to_string(),
            action: TxAction::Redeem,
            token: token.clone(),
            amount: 555.to_string(),
            sender: Some(sender.to_string()),
            receiver: receiver.to_string(),
            memo: None,
        };

        println!(" {} -> {} ticket:{:#?}", src_chain, dst_chain, c_2_b_ticket);

        let result = send_ticket(c_2_b_ticket).await;
        println!("{} -> {} redeem result:{:?}", src_chain, dst_chain, result);
        assert!(result.is_ok());
        // query tickets for chain id
        let result = query_tickets(Some(dst_chain.to_string()), 0, 5).await;
        println!("query tickets for {:} tickets: {:#?}", dst_chain, result);
        assert!(result.is_ok());
        // query token on chain
        let result = get_chain_tokens(None, None, 0, 5).await;
        println!("get_chain_tokens result: {:#?}", result);
        assert!(result.is_ok());
        let result =
            get_txs_with_account(None, Some(receiver.to_string()), None, None, 0, 10).await;
        println!(
            "get_txs_with_account({}) result: {:#?}",
            receiver.to_string(),
            result
        );
        assert!(result.is_ok());

        // B->A: `redeem` ticket
        let sender = "address_on_Optimistic";
        let receiver = "address_on_Ethereum";
        let src_chain = "EVM-Optimistic";
        let dst_chain = "Ethereum";

        let b_2_a_ticket = Ticket {
            ticket_id: Uuid::new_v4().to_string(),
            ticket_type: TicketType::Normal,
            ticket_time: get_timestamp(),
            src_chain: src_chain.to_string(),
            dst_chain: dst_chain.to_string(),
            action: TxAction::Redeem,
            token: token.clone(),
            amount: 222.to_string(),
            sender: Some(sender.to_string()),
            receiver: receiver.to_string(),
            memo: None,
        };
        println!(" {} -> {} ticket:{:#?}", src_chain, dst_chain, b_2_a_ticket);

        let result = send_ticket(b_2_a_ticket).await;
        println!("{} -> {} redeem result:{:?}", src_chain, dst_chain, result);
        assert!(result.is_ok());

        // query tickets for chain id
        let result = query_tickets(Some(dst_chain.to_string()), 0, 5).await;
        println!("query tickets for {:} tickets: {:#?}", dst_chain, result);
        assert!(result.is_ok());

        // query token on chain
        let result = get_chain_tokens(None, None, 0, 5).await;
        println!("get_chain_tokens result: {:#?}", result);
        assert!(result.is_ok());

        let result =
            get_txs_with_account(None, Some(receiver.to_string()), None, None, 0, 10).await;
        println!(
            "get_txs_with_account({}) result: {:#?}",
            receiver.to_string(),
            result
        );
        assert!(result.is_ok());

        // query txs
        let result = get_txs_with_chain(None, None, None, None, 0, 10).await;
        println!("get_txs result: {:#?}", result);
        assert!(result.is_ok());

        // print log
        let logs = StableLogWriter::get_logs(0, 0, 50);
        for r in logs.iter() {
            print!("stable log: {}", r)
        }
        let logs = get_logs(&None, &0, &50);
        for r in logs.iter() {
            print!("http request stable log: {}", r)
        }
        //print ticket seq
        with_state(|hub_state| {
            hub_state
                .ticket_seq
                .iter()
                .for_each(|(chain_id, latest_seq)| {
                    println!("chain:{},latest seq:{}", chain_id, latest_seq)
                })
        })
    }

    #[tokio::test]
    async fn test_storage() {
        // // add chain
        // build_chains().await;
        // // add token
        // build_tokens().await;
        //
        // A->B: `transfer` ticket
        let src_chain = "Bitcoin";
        let dst_chain = "EVM-Arbitrum";
        let sender = "address_on_Bitcoinxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx";
        let receiver = "address_on_Arbitrumxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx";
        let token = "Bitcoin-RUNES-150:1".to_string();

        let transfer_ticket = Ticket {
            ticket_id: Uuid::new_v4().to_string(),
            ticket_type: TicketType::Normal,
            ticket_time: get_timestamp(),
            src_chain: src_chain.to_string(),
            dst_chain: dst_chain.to_string(),
            action: TxAction::Transfer,
            token: token.clone(),
            amount: 88888.to_string(),
            sender: Some(sender.to_string()),
            receiver: receiver.to_string(),
            memo: None,
        };

        // Serialize the ticket.
        let mut state_bytes = vec![];
        let _ = ciborium::ser::into_writer(&transfer_ticket, &mut state_bytes);
        let ticket_len = state_bytes.len() as u128;
        // let ticket_len = 1024 as u128;
        let daily_ticket_storage = 100000 as u128 * ticket_len;
        let total_storage = 500 * 1024 * 1024 * 1024 as u128;
        let days = total_storage / daily_ticket_storage;

        println!(
            "Ticket_len:{} bytes \ndaily_ticket_storage:{} MB \nStorable Time: {} days,about {} years ",
            ticket_len,
            daily_ticket_storage / 1024 / 1024,
            days,
            days / 365,
        );
    }
}
