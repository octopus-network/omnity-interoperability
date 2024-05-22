use crate::memory::init_stable_log;
use ic_canisters_http_types::{HttpRequest, HttpResponse};
use ic_cdk::{init, post_upgrade, pre_upgrade, query, update};
use log::info;
use omnity_hub::auth::{auth, is_admin};
use omnity_hub::event::{self, record_event, Event, GetEventsArg};
use omnity_hub::lifecycle::init::HubArg;
use omnity_hub::metrics::{self, with_metrics};
use omnity_hub::proposal;
use omnity_hub::state::{with_state, with_state_mut};
use omnity_hub::types::{ChainMeta, TokenMeta};
use omnity_hub::types::{
    TokenResp, {Proposal, Subscribers},
};
use omnity_hub::{lifecycle, memory};
use omnity_types::log::{init_log, LoggerConfigService, StableLogWriter};
use omnity_types::{
    Chain, ChainId, ChainState, ChainType, Directive, Error, Factor, Seq, Ticket, TicketId, TokenId, TokenOnChain, Topic
};

use omnity_hub::state::HubState;

#[init]
fn init(args: HubArg) {
    info!("hub init args: {:?}", args);
    match args {
        HubArg::Init(args) => {
            init_log(Some(init_stable_log()));

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
    info!("begin to execute post_upgrade with :{:?}", args);
    // init log
    init_log(Some(init_stable_log()));
    HubState::post_upgrade(args);
    info!("upgrade successfully!");
}

/// validate directive ,this method will be called by sns
#[query(guard = "auth")]
pub async fn validate_proposal(proposals: Vec<Proposal>) -> Result<Vec<String>, Error> {
    proposal::validate_proposal(&proposals).await
}
#[update(guard = "auth")]
pub async fn execute_proposal(proposals: Vec<Proposal>) -> Result<(), Error> {
    proposal::execute_proposal(proposals).await
}

#[update(guard = "auth")]
pub async fn add_token(tokens: Vec<TokenMeta>) -> Result<(), Error> {
    let proposals: Vec<Proposal> = tokens.into_iter().map(Proposal::AddToken).collect();

    // validate proposal
    proposal::validate_proposal(&proposals).await?;
    // exection proposal and generate directives
    proposal::execute_proposal(proposals).await
}

/// check and build update fee directive and push it to the directive queue
#[update(guard = "auth")]
pub async fn update_fee(factors: Vec<Factor>) -> Result<(), Error> {
    let proposals: Vec<Proposal> = factors.into_iter().map(Proposal::UpdateFee).collect();

    // validate proposal
    proposal::validate_proposal(&proposals).await?;
    // exection proposal and generate directives
    proposal::execute_proposal(proposals).await
}

#[update(guard = "auth")]
pub async fn sub_directives(chain_id: Option<ChainId>, topics: Vec<Topic>) -> Result<(), Error> {
    info!(
        "sub_topics for chain: {:?}, with topics: {:?} ",
        chain_id, topics
    );
    let dst_chain_id = metrics::get_chain_id(chain_id)?;
    with_state_mut(|hub_state| hub_state.sub_directives(&dst_chain_id, &topics))
}

#[update(guard = "auth")]
pub async fn unsub_directives(chain_id: Option<ChainId>, topics: Vec<Topic>) -> Result<(), Error> {
    info!(
        "unsub_topics for chain: {:?}, with topics: {:?} ",
        chain_id, topics
    );
    let dst_chain_id = metrics::get_chain_id(chain_id)?;
    with_state_mut(|hub_state| hub_state.unsub_directives(&dst_chain_id, &topics))
}

#[query(guard = "auth")]
pub async fn query_subscribers(topic: Option<Topic>) -> Result<Vec<(Topic, Subscribers)>, Error> {
    info!("query_subscribers for topic: {:?} ", topic);
    with_state(|hub_state| hub_state.query_subscribers(topic))
}

/// query directives for chain id filter by topic,this method will be called by route and custom
#[query(guard = "auth")]
pub async fn query_directives(
    chain_id: Option<ChainId>,
    topic: Option<Topic>,
    offset: usize,
    limit: usize,
) -> Result<Vec<(Seq, Directive)>, Error> {
    info!(
        "query directive for chain: {:?}, with topic: {:?} ",
        chain_id, topic
    );

    let dst_chain_id = metrics::get_chain_id(chain_id)?;
    with_state(|hub_state| hub_state.pull_directives(dst_chain_id, topic, offset, limit))
}

/// check and push ticket into queue
#[update(guard = "auth")]
pub async fn send_ticket(ticket: Ticket) -> Result<(), Error> {
    info!("send_ticket: {:?}", ticket);

    with_state_mut(|hub_state| {
        // checke ticket and update token on chain
        hub_state.check_and_update(&ticket)?;
        // push ticket into queue
        hub_state.push_ticket(ticket)
    })
}

#[update(guard = "auth")]
pub async fn resubmit_ticket(ticket: Ticket) -> Result<(), Error> {
    info!("received resubmit ticket: {:?}", ticket);
    // No need to update the token since the old ticket has already added
    with_state_mut(|hub_state| hub_state.resubmit_ticket(ticket))
}

/// query tickets for chain id,this method will be called by route and custom
#[query(guard = "auth")]
pub async fn query_tickets(
    chain_id: Option<ChainId>,
    offset: usize,
    limit: usize,
) -> Result<Vec<(Seq, Ticket)>, Error> {
    info!("query_tickets: {:?},{offset},{limit}", chain_id);
    // let end = from + num;
    let dst_chain_id = metrics::get_chain_id(chain_id)?;
    with_state(|hub_state| hub_state.pull_tickets(&dst_chain_id, offset, limit))
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
pub async fn get_chain_metas(offset: usize, limit: usize) -> Result<Vec<ChainMeta>, Error> {
    metrics::get_chain_metas(offset, limit).await
}

#[query]
pub async fn get_chain_size() -> Result<u64, Error> {
    metrics::get_chain_size().await
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

#[query]
pub async fn get_token_metas(offset: usize, limit: usize) -> Result<Vec<TokenMeta>, Error> {
    metrics::get_token_metas(offset, limit).await
}

#[query]
pub async fn get_token_size() -> Result<u64, Error> {
    metrics::get_token_size().await
}

#[query]
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

#[update(guard = "is_admin")]
pub async fn set_logger_filter(filter: String) {
    LoggerConfigService::default().set_logger_filter(&filter);
}

#[query]
pub async fn get_logs(time: Option<u64>, offset: usize, limit: usize) -> Vec<String> {
    let max_skip_timestamp = time.unwrap_or(0);
    StableLogWriter::get_logs(max_skip_timestamp, offset, limit)
}

#[query(hidden = true)]
fn http_request(req: HttpRequest) -> HttpResponse {
    StableLogWriter::http_request(req)
}

#[query]
fn get_events(args: GetEventsArg) -> Vec<Event> {
    event::events(args)
}

#[query]
pub async fn get_directive_size() -> Result<u64, Error> {
    metrics::get_directive_size().await
}
#[query]
pub async fn get_directives(offset: usize, limit: usize) -> Result<Vec<Directive>, Error> {
    metrics::get_directives(offset, limit).await
}

#[query]
pub async fn sync_ticket_size() -> Result<u64, Error> {
    with_metrics(|metrics| metrics.sync_ticket_size())
}

#[query]
pub async fn sync_tickets(offset: usize, limit: usize) -> Result<Vec<(u64, Ticket)>, Error> {
    with_metrics(|metrics| metrics.sync_tickets(offset, limit))
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
        ChainType, Factor, FeeTokenFactor, TargetChainFactor, Ticket,  TicketType,
        ToggleAction, ToggleState, TxAction,
    };

    // use env_logger;
    // use log::LevelFilter;
    use std::{
        collections::HashMap,
        time::{SystemTime, UNIX_EPOCH},
    };
    use uuid::Uuid;

    fn init_hub() {
        let arg = HubArg::Init(InitArgs {
            admin: PrincipalId::new_user_test_id(1).0,
        });
        init(arg)
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
            Topic::AddToken,
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
        init_hub();
        sub_dires().await;
        let result = query_subscribers(None).await;
        println!("query_subscribers result: {:?}", result);
        unsub_dires().await;
        let result = query_subscribers(None).await;
        println!("query_subscribers result: {:?}", result);
    }

    #[tokio::test]
    async fn test_add_chain() {
        init_hub();

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
    async fn test_add_token() {
        init_hub();
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

        // add chain
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
    async fn test_toggle_chain_state() {
        init_hub();
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
        init_hub();

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
    async fn test_a_b_tx_ticket() {
        init_hub();
        sub_dires().await;
        // add chain
        add_chains().await;
        // add token
        add_tokens().await;
        //
        // A->B: `transfer` ticket
        let src_chain = "Bitcoin";
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

        with_state(|hus_state| {
            hus_state.ticket_queue.iter().for_each(|(seq_key, ticket)| {
                println!(" seq key: {:?} ticket: {:?}", seq_key, ticket)
            })
        });

        assert!(result.is_ok());
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
            hus_state.ticket_queue.iter().for_each(|(seq_key, ticket)| {
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
        init_hub();
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
        // let ticket_len = state_bytes.len() as u128;
        let ticket_len = 1024 as u128;
        let total_storage = 500 * 1024 * 1024 * 1024 as u128;
        let daily_ticket_storage = 100000 as u128 * ticket_len;
        let days = total_storage / daily_ticket_storage;

        println!(
            "Ticket_len:{} \ndaily_ticket_storage:{} MB \nStorable Time: {} days,about {} years ",
            ticket_len,
            daily_ticket_storage / 1024 / 1024,
            days,
            days / 365,
        );
    }
}
