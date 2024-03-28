use ic_cdk::{init, post_upgrade, pre_upgrade, query, update};
use ic_log::writer::Logs;
use ic_stable_structures::writer::Writer;
use ic_stable_structures::Memory;
use log::{debug, info};

use omnity_hub::auth::{auth, is_owner};
use omnity_hub::memory;
use omnity_hub::metrics;
use omnity_hub::state::HubState;
use omnity_hub::state::{set_state, with_state, with_state_mut};
use omnity_hub::types::{ChainWithSeq, Proposal};
use omnity_hub::util::{init_log, LoggerConfigService};
use omnity_types::{
    Chain, ChainId, ChainState, ChainType, Directive, Error, Fee, Seq, Ticket, TicketId, Token,
    TokenId, TokenOnChain, Topic,
};

#[init]
fn init() {
    init_log();
    let caller = ic_cdk::api::caller();
    info!("canister init caller:{}", caller.to_string());
    with_state_mut(|hs| {
        hs.owner = Some(caller.to_string());
    })
}

#[pre_upgrade]
fn pre_upgrade() {
    info!("begin to handle pre_update state ...");

    // Serialize the state.
    let mut state_bytes = vec![];
    with_state(|state| ciborium::ser::into_writer(state, &mut state_bytes))
        .expect("failed to encode state");

    // Write the length of the serialized bytes to memory, followed by the
    // by the bytes themselves.
    let len = state_bytes.len() as u32;
    let mut memory = memory::get_upgrades_memory();
    let mut writer = Writer::new(&mut memory, 0);
    writer
        .write(&len.to_le_bytes())
        .expect("failed to save hub state len");
    writer
        .write(&state_bytes)
        .expect("failed to save hub state");
}

#[post_upgrade]
fn post_upgrade() {
    // init log
    init_log();
    let memory = memory::get_upgrades_memory();

    // Read the length of the state bytes.
    let mut state_len_bytes = [0; 4];
    memory.read(0, &mut state_len_bytes);
    let state_len = u32::from_le_bytes(state_len_bytes) as usize;

    // Read the bytes
    let mut state_bytes = vec![0; state_len];
    memory.read(4, &mut state_bytes);

    // Deserialize and set the state.
    let state: HubState = ciborium::de::from_reader(&*state_bytes).expect("failed to decode state");
    set_state(state);
}

/// validate directive ,this method will be called by sns
#[query(guard = "auth")]
pub async fn validate_proposal(proposals: Vec<Proposal>) -> Result<Vec<String>, Error> {
    if proposals.len() == 0 {
        return Err(Error::ProposalError(
            "Proposal can not be empty".to_string(),
        ));
    }
    let mut proposal_msgs = Vec::new();
    for proposal in proposals.into_iter() {
        match proposal {
            Proposal::AddChain(chain_meta) => {
                if chain_meta.chain_id.is_empty() {
                    return Err(Error::ProposalError(
                        "Chain name can not be empty".to_string(),
                    ));
                }

                if matches!(chain_meta.chain_state, ChainState::Deactive) {
                    return Err(Error::ProposalError(
                        "The status of the new chain state must be active".to_string(),
                    ));
                }

                with_state(|hub_state| {
                    hub_state.chain(&chain_meta.chain_id).map_or(Ok(()), |_| {
                        Err(Error::ChainAlreadyExisting(chain_meta.chain_id.to_string()))
                    })
                })?;

                let result = format!("Tne AddChain proposal: {}", chain_meta);
                info!("validate_proposal result:{} ", result);
                proposal_msgs.push(result);
            }
            Proposal::AddToken(token_meta) => {
                if token_meta.token_id.is_empty()
                    || token_meta.symbol.is_empty()
                    || token_meta.settlement_chain.is_empty()
                {
                    return Err(Error::ProposalError(
                        "Token id, token symbol or issue chain can not be empty".to_string(),
                    ));
                }
                with_state(|hub_state| {
                    // check token repetitive
                    hub_state.token(&token_meta.token_id).map_or(Ok(()), |_| {
                        Err(Error::TokenAlreadyExisting(token_meta.to_string()))
                    })?;

                    //ensure the dst chains must exsits!
                    if let Some(id) = token_meta
                        .dst_chains
                        .iter()
                        .find(|id| !hub_state.chains.contains_key(&**id))
                    {
                        return Err(Error::NotFoundChain(id.to_string()));
                    }

                    hub_state.available_chain(&token_meta.settlement_chain)

                    // Ok(())
                })?;
                let result = format!("The AddToken proposal: {}", token_meta);
                info!("validate_proposal result:{} ", result);
                proposal_msgs.push(result);
            }
            Proposal::ToggleChainState(toggle_state) => {
                if toggle_state.chain_id.is_empty() {
                    return Err(Error::ProposalError(
                        "Chain id can not be empty".to_string(),
                    ));
                }

                with_state(|hub_state| hub_state.available_state(&toggle_state))?;
                let result = format!("The ToggleChainStatus proposal: {}", toggle_state);
                info!("validate_proposal result:{} ", result);
                proposal_msgs.push(result);
            }
            Proposal::UpdateFee(fee) => {
                if fee.fee_token.is_empty() {
                    return Err(Error::ProposalError(
                        "The fee token can not be empty".to_string(),
                    ));
                };

                with_state(|hub_state| {
                    //check the issue chain must exsiting and not deactive!
                    hub_state.available_chain(&fee.dst_chain_id)?;
                    hub_state.token(&fee.fee_token)
                })?;
                let result = format!("The UpdateFee proposal: {}", fee);
                info!("validate_proposal result:{} ", result);
                proposal_msgs.push(result);
            }
        }
    }
    Ok(proposal_msgs)
}

/// build directive based on proposal, this method will be called by sns
#[update(guard = "auth")]
pub async fn execute_proposal(proposals: Vec<Proposal>) -> Result<(), Error> {
    for proposal in proposals.into_iter() {
        match proposal {
            Proposal::AddChain(chain_meta) => {
                info!(
                    "build directive for `AddChain` proposal :{:?}",
                    chain_meta.to_string()
                );

                let new_chain = ChainWithSeq::from(chain_meta.clone());
                // save new chain
                with_state_mut(|hub_state| {
                    info!(" save new chain: {:?}", new_chain);
                    hub_state.save_chain(new_chain.clone())
                })?;
                // build directives
                match chain_meta.chain_type {
                    // nothing to do
                    ChainType::SettlementChain => {
                        info!("for settlement chain,  no need to build directive!");
                    }

                    ChainType::ExecutionChain => {
                        if let Some(counterparties) = chain_meta.counterparties {
                            let result: Result<(), Error> = counterparties
                                .into_iter()
                                .map(|counterparty| {
                                    info!(
                                        " build directive for counterparty chain:{:?} !",
                                        counterparty.to_string()
                                    );
                                    let dst_chain = with_state(|hub_state| {
                                        hub_state.available_chain(&counterparty)
                                    })?;
                                    // build directive for counterparty chain
                                    with_state_mut(|hub_state| {
                                        //TODO: Consider whether this is necessary？
                                        hub_state.push_dire(
                                            &counterparty,
                                            Directive::AddChain(new_chain.clone().into()),
                                        )?;
                                        // generate directive for the new chain
                                        hub_state.push_dire(
                                            &new_chain.chain_id,
                                            Directive::AddChain(dst_chain.into()),
                                        )
                                    })
                                })
                                .collect();
                            result?;
                        }
                    }
                }
            }

            Proposal::AddToken(token_meata) => {
                info!("build directive for `AddToken` proposal :{:?}", token_meata);
                // save token info
                with_state_mut(|hub_state| hub_state.save_token(token_meata.clone()))?;
                // generate dirctive
                let result: Result<(), Error> = token_meata
                    .dst_chains
                    .iter()
                    .map(|dst_chain| {
                        info!(
                            " build directive for counterparty chain:{:?} !",
                            dst_chain.to_string()
                        );

                        with_state_mut(|hub_state| {
                            hub_state.push_dire(
                                &dst_chain,
                                Directive::AddToken(token_meata.clone().into()),
                            )
                        })
                    })
                    .collect();

                result?;
            }
            Proposal::ToggleChainState(toggle_status) => {
                info!(
                    "build directive for `ToggleChainState` proposal :{:?}",
                    toggle_status
                );

                // generate directive for counterparty chain
                if let Some(counterparties) =
                    with_state(|hs| hs.available_chain(&toggle_status.chain_id))?.counterparties
                {
                    let result: Result<(), Error> = counterparties
                        .into_iter()
                        .map(|counterparty| {
                            info!(
                                " build directive for counterparty chain:{:?} !",
                                counterparty.to_string()
                            );

                            // build directive for counterparty chain
                            with_state_mut(|hub_state| {
                                hub_state.push_dire(
                                    &counterparty,
                                    Directive::ToggleChainState(toggle_status.clone()),
                                )
                            })
                        })
                        .collect();

                    result?;
                }
                // update dst chain state
                with_state_mut(|hub_state| hub_state.update_chain_state(&toggle_status))?;
            }
            Proposal::UpdateFee(fee) => {
                info!("build directive for `UpdateFee` proposal :{:?}", fee);

                with_state_mut(|hub_state| {
                    // save fee info
                    hub_state.update_fee(fee.clone())?;
                    // generate directive
                    hub_state.push_dire(&fee.dst_chain_id, Directive::UpdateFee(fee.clone()))
                })?;
            }
        }
    }

    Ok(())
}

/// check and build update fee directive and push it to the directive queue
#[update(guard = "auth")]
pub async fn update_fee(fees: Vec<Fee>) -> Result<(), Error> {
    let proposals: Vec<Proposal> = fees
        .into_iter()
        .map(|fee| Proposal::UpdateFee(fee))
        .collect();

    // validate proposal
    validate_proposal(proposals.clone()).await?;
    // exection proposal and generate directives
    execute_proposal(proposals).await?;

    Ok(())
}

/// query directives for chain id filter by topic,this method will be called by route and custom
#[query(guard = "auth")]
pub async fn query_dires(
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
    with_state(|hub_state| hub_state.pull_dires(dst_chain_id, topic, offset, limit))
}

/// check and push ticket into queue
#[update(guard = "auth")]
pub async fn send_ticket(ticket: Ticket) -> Result<(), Error> {
    info!("received ticket: {:?}", ticket);

    with_state_mut(|hub_state| {
        // checke ticket and update token on chain
        hub_state.check_and_update(&ticket)?;
        // push ticket into queue
        hub_state.push_ticket(ticket)
    })?;
    Ok(())
}

/// query tickets for chain id,this method will be called by route and custom
#[query(guard = "auth")]
pub async fn query_tickets(
    chain_id: Option<ChainId>,
    offset: usize,
    limit: usize,
) -> Result<Vec<(Seq, Ticket)>, Error> {
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
pub async fn get_chain(chain_id: String) -> Result<Chain, Error> {
    metrics::get_chain(chain_id).await
}

#[query]
pub async fn get_tokens(
    chain_id: Option<ChainId>,
    token_id: Option<TokenId>,
    offset: usize,
    limit: usize,
) -> Result<Vec<Token>, Error> {
    metrics::get_tokens(chain_id, token_id, offset, limit).await
}

#[query]
pub async fn get_fees(
    chain_id: Option<ChainId>,
    token_id: Option<TokenId>,
    offset: usize,
    limit: usize,
) -> Result<Vec<Fee>, Error> {
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
pub async fn get_txs(
    src_chain: Option<ChainId>,
    dst_chain: Option<ChainId>,
    token_id: Option<TokenId>,
    time_range: Option<(u64, u64)>,
    offset: usize,
    limit: usize,
) -> Result<Vec<Ticket>, Error> {
    metrics::get_txs(src_chain, dst_chain, token_id, time_range, offset, limit).await
}

#[query]
pub async fn get_total_tx() -> Result<u64, Error> {
    metrics::get_total_tx().await
}

#[query]
pub fn get_log_records(limit: usize, offset: usize) -> Logs {
    debug!("collecting {limit} log records");
    ic_log::take_memory_records(limit, offset)
}

#[update(guard = "is_owner")]
pub async fn set_logger_filter(filter: String) {
    LoggerConfigService::default().set_logger_filter(&filter);
    debug!("log filter set to {filter}");
}

fn main() {}

ic_cdk::export_candid!();

#[cfg(test)]
mod tests {

    use omnity_hub::types::{ChainMeta, TokenMeta};

    use super::*;
    use omnity_types::{ChainType, Fee, Ticket, ToggleAction, ToggleState, TxAction};

    use env_logger;
    use log::LevelFilter;
    use std::{
        collections::HashMap,
        time::{SystemTime, UNIX_EPOCH},
    };
    use uuid::Uuid;

    // init logger
    pub fn init_logger() {
        env_logger::builder().filter_level(LevelFilter::Info).init();
    }
    // #[before]
    // fn setup() {
    //     init_logger();
    // }

    fn get_timestamp() -> u64 {
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        since_the_epoch.as_millis() as u64
    }

    fn chain_ids() -> Vec<String> {
        vec![
            "Bitcoin".to_string(),
            "Ethereum".to_string(),
            "ICP".to_string(),
            "ICP-Exection".to_string(),
            "EVM-Arbitrum".to_string(),
            "EVM-Optimistic".to_string(),
            "EVM-Starknet".to_string(),
        ]
    }

    fn token_ids() -> Vec<String> {
        vec![
            "Bitcoin-RUNES-150:1".to_string(),
            "Bitcoin-RUNES-XXX".to_string(),
            "Bitcoin-RUNES-XXY".to_string(),
            "Ethereum-Native-ETH".to_string(),
            "Ethereum-ERC20-OCT".to_string(),
            "Ethereum-ERC20-XXX".to_string(),
            "Ethereum-ERC20-XXY".to_string(),
            "ICP-Native-ICP".to_string(),
            "ICP-ICRC2-XXX".to_string(),
            "ICP-ICRC2-XXY".to_string(),
        ]
    }

    async fn build_chains() {
        let btc = ChainMeta {
            chain_id: "Bitcoin".to_string(),
            chain_type: ChainType::SettlementChain,
            chain_state: ChainState::Active,
            canister_id: "bkyz2-fmaaa-aaaaa-qaaaq-cai".to_string(),
            contract_address: None,
            counterparties: None,
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
            chain_type: ChainType::SettlementChain,
            chain_state: ChainState::Active,
            canister_id: "bkyz2-fmaaa-aaaaa-qadaab-cai".to_string(),
            contract_address: Some("bkyz2-fmaaa-aaafa-qadaab-cai".to_string()),
            counterparties: Some(vec!["Bitcoin".to_string(), "Ethereum".to_string()]),
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

    async fn build_tokens() {
        let btc = TokenMeta {
            token_id: "Bitcoin-RUNES-150:1".to_string(),
            symbol: "BTC".to_owned(),
            settlement_chain: "Bitcoin".to_string(),
            decimals: 18,
            icon: None,
            metadata: Some(HashMap::from([(
                "rune_id".to_string(),
                "150:1".to_string(),
            )])),
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

        let btc = TokenMeta {
            token_id: "Bitcoin-RUNES-150:1".to_string(),
            symbol: "BTC".to_owned(),
            settlement_chain: "Bitcoin".to_string(),
            decimals: 18,
            icon: None,
            metadata: None,
            dst_chains: vec![
                "Ethereum".to_string(),
                "ICP".to_string(),
                "EVM-Arbitrum".to_string(),
                "EVM-Optimistic".to_string(),
                "EVM-Starknet".to_string(),
            ],
        };
        let result = validate_proposal(vec![Proposal::AddToken(btc.clone())]).await;
        println!(
            "validate_proposal for Proposal::AddToken(token) result:{:#?}",
            result
        );
        assert!(result.is_err());

        let btc = TokenMeta {
            token_id: "Bitcoin-RUNES-WTF".to_string(),
            symbol: "BTC".to_owned(),
            settlement_chain: "Bitcoin".to_string(),
            decimals: 18,
            icon: None,
            metadata: None,
            dst_chains: vec![
                "Ethereum".to_string(),
                "ICP_S".to_string(),
                "EVM-Arbitrum".to_string(),
                "EVM-Optimistic".to_string(),
                "EVM-Starknet".to_string(),
            ],
        };
        let result = validate_proposal(vec![Proposal::AddToken(btc.clone())]).await;
        println!(
            "validate_proposal for Proposal::AddToken(token) result:{:#?}",
            result
        );
        assert!(result.is_err());

        let eth = TokenMeta {
            token_id: "Ethereum-Native-ETH".to_string(),
            symbol: "ETH".to_owned(),
            settlement_chain: "Ethereum".to_string(),
            decimals: 18,
            icon: None,
            metadata: None,
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
            token_id: "ICP-Native-ICP".to_string(),
            symbol: "ICP".to_owned(),
            settlement_chain: "ICP".to_string(),
            decimals: 18,
            icon: None,
            metadata: None,
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
            symbol: "ARB".to_owned(),
            settlement_chain: "Ethereum".to_string(),
            decimals: 18,
            icon: None,
            metadata: None,
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
            symbol: "OP".to_owned(),
            settlement_chain: "Ethereum".to_string(),
            decimals: 18,
            icon: None,
            metadata: None,
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
            symbol: "StarkNet".to_owned(),
            settlement_chain: "Ethereum".to_string(),
            decimals: 18,
            icon: None,
            metadata: None,
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
    async fn test_add_chain() {
        // init_logger();
        // add chain
        build_chains().await;

        with_state(|hs| {
            for (chain_id, dires) in hs.dire_queue.iter() {
                println!("{},{:#?}\n", chain_id, dires)
            }
        });

        with_state(|hs| {
            for (chain_id, chain) in hs.chains.iter() {
                println!("{},{:#?}\n", chain_id, chain)
            }
        });

        for chain_id in chain_ids() {
            let result = query_dires(Some(chain_id.to_string()), None, 0, 10).await;
            println!("query_directives for {:} dires: {:#?}", chain_id, result);
            assert!(result.is_ok());
            let chain = get_chain(chain_id.to_string()).await;
            println!("get chain for {:} chain: {:#?}", chain_id, chain);
        }
        // let result = query_dires(None, None, 0, 10).await;
        // println!("query_dires dires: {:#?}", result);

        let result = get_chains(None, None, 0, 10).await;
        println!("get_chains result : {:#?}", result);
        assert!(result.is_ok());

        let result = get_chains(Some(ChainType::ExecutionChain), None, 0, 10).await;
        println!("get_chains result by chain type: {:#?}", result);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_add_token() {
        init_logger();
        // add chain
        build_chains().await;
        // add token
        build_tokens().await;

        with_state(|hs| {
            for (chain_id, dires) in hs.dire_queue.iter() {
                println!("{},{:#?}", chain_id, dires)
            }
        });

        with_state(|hs| {
            for (chain_id, chain) in hs.chains.iter() {
                println!("{},{:#?}\n", chain_id, chain)
            }
        });

        for chain_id in chain_ids() {
            let result = query_dires(
                Some(chain_id.to_string()),
                Some(Topic::AddToken(None)),
                0,
                5,
            )
            .await;
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
        let result = get_tokens(
            Some("ICP".to_string()),
            Some("ICP-Native-ICP".to_string()),
            0,
            10,
        )
        .await;
        assert!(result.is_ok());
        println!("get_tokens result by chain_id and token id: {:#?}", result);
    }

    #[tokio::test]
    async fn test_toggle_chain_state() {
        init_logger();
        // add chain
        build_chains().await;
        // add token
        build_tokens().await;

        // change chain state
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

        with_state(|hs| {
            for (chain_id, dires) in hs.dire_queue.iter() {
                println!("{},{:#?}", chain_id, dires)
            }
        });

        with_state(|hs| {
            for (chain_id, chain) in hs.chains.iter() {
                println!("{},{:#?}\n", chain_id, chain)
            }
        });

        // query directives for chain id

        for chain_id in chain_ids() {
            let result = query_dires(
                Some(chain_id.to_string()),
                // None,
                Some(Topic::DeactivateChain),
                0,
                5,
            )
            .await;
            println!("query_directives for {:} dires: {:#?}", chain_id, result);
            assert!(result.is_ok());
        }

        // let result = get_chains(None, Some(ChainState::Deactive), 0, 10).await;
        // let result = get_chains(None, None, 0, 10).await;
        assert!(result.is_ok());
        println!(
            "get_chains result by chain type and chain state: {:#?}",
            result
        );
    }

    #[tokio::test]
    async fn test_update_fee() {
        init_logger();
        // add chain
        build_chains().await;
        // add token
        build_tokens().await;

        // with_state(|hs| {
        //     for (chain_id, dires) in hs.dire_queue.iter() {
        //         println!("{},{:#?}", chain_id, dires)
        //     }
        // });

        // with_state(|hs| {
        //     for (chain_id, chain) in hs.chains.iter() {
        //         println!("{},{:#?}\n", chain_id, chain)
        //     }
        // });

        // change chain state
        let fee = Fee {
            dst_chain_id: "EVM-Arbitrum".to_string(),
            fee_token: "Ethereum-ERC20-OP".to_string(),
            factor: 12,
        };

        // let update_fee = Proposal::UpdateFee(fee);
        // let _ = build_directive(update_fee).await;
        let result = update_fee(vec![fee]).await;
        assert!(result.is_ok());
        println!("update_fee result:{:?}", result);

        with_state(|hs| {
            for (chain_id, dires) in hs.dire_queue.iter() {
                println!("{},{:#?}", chain_id, dires)
            }
        });

        with_state(|hs| {
            for (chain_id, chain) in hs.chains.iter() {
                println!("{},{:#?}\n", chain_id, chain)
            }
        });

        // query directives for chain id
        for chain_id in chain_ids() {
            let result = query_dires(
                Some(chain_id.to_string()),
                Some(Topic::UpdateFee(None)),
                0,
                5,
            )
            .await;
            println!("query_directives for {:} dires: {:#?}", chain_id, result);
            assert!(result.is_ok());
        }

        let result = get_fees(None, None, 0, 10).await;
        assert!(result.is_ok());
        println!("get_fees result : {:#?}", result);

        let result = get_fees(None, Some("Ethereum-ERC20-OP".to_string()), 0, 10).await;
        assert!(result.is_ok());
        println!("get_fees result filter by token id : {:#?}", result);
    }

    #[tokio::test]
    async fn test_a_b_tx_ticket() {
        // add chain
        build_chains().await;
        // add token
        build_tokens().await;
        //
        // A->B: `transfer` ticket
        let src_chain = "Bitcoin";
        let dst_chain = "EVM-Arbitrum";
        let sender = "address_on_Bitcoin";
        let receiver = "address_on_Arbitrum";
        let token = "Bitcoin-RUNES-150:1".to_string();

        let transfer_ticket = Ticket {
            ticket_id: Uuid::new_v4().to_string(),
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
        // query tickets for chain id
        let result = query_tickets(Some(dst_chain.to_string()), 0, 5).await;
        println!("query tickets for {:} tickets: {:#?}", dst_chain, result);
        assert!(result.is_ok());
        // query token on chain
        let result = get_chain_tokens(None, None, 0, 5).await;
        println!("get_chain_tokens result: {:#?}", result);
        assert!(result.is_ok());

        // query tx from get_txs
        let result = get_txs(Some(src_chain.to_string()), None, None, None, 0, 10).await;
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
            ticket_time: get_timestamp(),
            src_chain: src_chain.to_string(),
            dst_chain: dst_chain.to_string(),
            action: TxAction::Redeem,
            token: token.clone(),
            amount: 88888.to_string(),
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

        // query tickets for chain id
        let result = query_tickets(Some(dst_chain.to_string()), 0, 5).await;
        assert!(result.is_ok());
        println!("query tickets for {:} tickets: {:#?}", dst_chain, result);
        // query token on chain
        let result = get_chain_tokens(None, None, 0, 5).await;
        println!("get_chain_tokens result: {:#?}", result);
        assert!(result.is_ok());

        // query tx from get_txs
        let result = get_txs(None, Some(dst_chain.to_string()), None, None, 0, 10).await;
        println!(
            "get_txs by dst chain({}) result: {:#?}",
            dst_chain.to_string(),
            result
        );
        assert!(result.is_ok());

        // query tx from get_txs
        let result = get_txs(
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
    }

    #[tokio::test]
    async fn test_a_b_c_tx_ticket() {
        // add chain
        build_chains().await;
        // add token
        build_tokens().await;

        // transfer
        // A->B: `transfer` ticket
        let src_chain = "Ethereum";
        let dst_chain = "EVM-Optimistic";
        let sender = "address_on_Ethereum";
        let receiver = "address_on_Optimistic";
        let token = "Ethereum-Native-ETH".to_string();

        let a_2_b_ticket = Ticket {
            ticket_id: Uuid::new_v4().to_string(),
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

        // B->C: `transfer` ticket
        let sender = "address_on_Optimistic";
        let receiver = "address_on_Starknet";
        let src_chain = "EVM-Optimistic";
        let dst_chain = "EVM-Starknet";

        let b_2_c_ticket = Ticket {
            ticket_id: Uuid::new_v4().to_string(),
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

        // redeem
        // C->B: `redeem` ticket
        let src_chain = "EVM-Starknet";
        let dst_chain = "EVM-Optimistic";
        let sender = "address_on_Starknet";
        let receiver = "address_on_Optimistic";

        let c_2_b_ticket = Ticket {
            ticket_id: Uuid::new_v4().to_string(),
            ticket_time: get_timestamp(),
            src_chain: src_chain.to_string(),
            dst_chain: dst_chain.to_string(),
            action: TxAction::Redeem,
            token: token.clone(),
            amount: 6666.to_string(),
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

        // B->A: `redeem` ticket
        let sender = "address_on_Optimistic";
        let receiver = "address_on_Ethereum";
        let src_chain = "EVM-Optimistic";
        let dst_chain = "Ethereum";

        let b_2_a_ticket = Ticket {
            ticket_id: Uuid::new_v4().to_string(),
            ticket_time: get_timestamp(),
            src_chain: src_chain.to_string(),
            dst_chain: dst_chain.to_string(),
            action: TxAction::Redeem,
            token: token.clone(),
            amount: 6666.to_string(),
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

        // query txs
        let result = get_txs(None, None, None, None, 0, 10).await;
        println!("get_txs result: {:#?}", result);
        assert!(result.is_ok());
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
