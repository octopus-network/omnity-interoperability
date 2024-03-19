use std::time::{SystemTime, UNIX_EPOCH};

use ic_base_types::PrincipalId;
use omnity_hub::types::{ChainMeta, Proposal, TokenMeta};
use omnity_types::{ChainState, ChainType, Fee, Ticket, TxAction};
use omnity_types::{ToggleAction, ToggleState, Topic};
mod common;

use common::OmnityHub;

use uuid::Uuid;
fn canister_ids() -> Vec<PrincipalId> {
    vec![
        PrincipalId::new_user_test_id(0),
        PrincipalId::new_user_test_id(1),
        PrincipalId::new_user_test_id(2),
        PrincipalId::new_user_test_id(3),
        PrincipalId::new_user_test_id(4),
        PrincipalId::new_user_test_id(5),
    ]
}
fn chain_ids() -> Vec<String> {
    vec![
        "Bitcoin".to_string(),
        "Ethereum".to_string(),
        "ICP".to_string(),
        "Arbitrum".to_string(),
        "Optimistic".to_string(),
        "Starknet".to_string(),
    ]
}
fn chains() -> Vec<Proposal> {
    let chains = vec![
        Proposal::AddChain(ChainMeta {
            chain_id: "Bitcoin".to_string(),
            chain_type: ChainType::SettlementChain,
            chain_state: ChainState::Active,
            canister_id: PrincipalId::new_user_test_id(0).to_string(),
            contract_address: None,
            counterparties: None,
        }),
        Proposal::AddChain(ChainMeta {
            chain_id: "Ethereum".to_string(),
            chain_type: ChainType::SettlementChain,
            chain_state: ChainState::Active,
            canister_id: PrincipalId::new_user_test_id(1).to_string(),
            contract_address: Some("Ethereum constract address".to_string()),
            counterparties: Some(vec!["Bitcoin".to_string()]),
        }),
        Proposal::AddChain(ChainMeta {
            chain_id: "ICP".to_string(),
            chain_type: ChainType::SettlementChain,
            chain_state: ChainState::Active,
            canister_id: PrincipalId::new_user_test_id(2).to_string(),
            contract_address: Some("bkyz2-fmaaa-aaafa-qadaab-cai".to_string()),
            counterparties: Some(vec!["Bitcoin".to_string(), "Ethereum".to_string()]),
        }),
        Proposal::AddChain(ChainMeta {
            chain_id: "Arbitrum".to_string(),
            chain_type: ChainType::ExecutionChain,
            chain_state: ChainState::Active,
            canister_id: PrincipalId::new_user_test_id(3).to_string(),
            contract_address: Some("Arbitrum constract address".to_string()),
            counterparties: Some(vec![
                "Bitcoin".to_string(),
                "Ethereum".to_string(),
                "ICP".to_string(),
            ]),
        }),
        Proposal::AddChain(ChainMeta {
            chain_id: "Optimistic".to_string(),
            chain_type: ChainType::ExecutionChain,
            chain_state: ChainState::Active,
            canister_id: PrincipalId::new_user_test_id(4).to_string(),
            contract_address: Some("Optimistic constract address".to_string()),
            counterparties: Some(vec![
                "Bitcoin".to_string(),
                "Ethereum".to_string(),
                "ICP".to_string(),
                "Arbitrum".to_string(),
            ]),
        }),
        Proposal::AddChain(ChainMeta {
            chain_id: "Starknet".to_string(),
            chain_type: ChainType::ExecutionChain,
            chain_state: ChainState::Active,
            canister_id: PrincipalId::new_user_test_id(5).to_string(),
            contract_address: Some("Starknet constract address".to_string()),
            counterparties: Some(vec![
                "Bitcoin".to_string(),
                "Ethereum".to_string(),
                "ICP".to_string(),
                "Arbitrum".to_string(),
                "Optimistic".to_string(),
            ]),
        }),
    ];

    chains
}

fn tokens() -> Vec<Proposal> {
    let tokens = vec![
        Proposal::AddToken(TokenMeta {
            token_id: "BTC".to_string(),
            symbol: "BTC".to_owned(),
            issue_chain: "Bitcoin".to_string(),
            decimals: 18,
            icon: None,
            dst_chains: vec![
                "Ethereum".to_string(),
                "ICP".to_string(),
                "Arbitrum".to_string(),
                "Optimistic".to_string(),
                "Starknet".to_string(),
            ],
        }),
        Proposal::AddToken(TokenMeta {
            token_id: "ETH".to_string(),
            symbol: "ETH".to_owned(),
            issue_chain: "Ethereum".to_string(),
            decimals: 18,
            icon: None,
            dst_chains: vec![
                "Bitcoin".to_string(),
                "ICP".to_string(),
                "Arbitrum".to_string(),
                "Optimistic".to_string(),
                "Starknet".to_string(),
            ],
        }),
        Proposal::AddToken(TokenMeta {
            token_id: "ICP".to_string(),
            symbol: "ICP".to_owned(),
            issue_chain: "ICP".to_string(),
            decimals: 18,
            icon: None,
            dst_chains: vec![
                "Bitcoin".to_string(),
                "Ethereum".to_string(),
                "Arbitrum".to_string(),
                "Optimistic".to_string(),
                "Starknet".to_string(),
            ],
        }),
        Proposal::AddToken(TokenMeta {
            token_id: "ARB".to_string(),
            symbol: "ARB".to_owned(),
            issue_chain: "Arbitrum".to_string(),
            decimals: 18,
            icon: None,
            dst_chains: vec![
                "Bitcoin".to_string(),
                "Ethereum".to_string(),
                "ICP".to_string(),
                "Optimistic".to_string(),
                "Starknet".to_string(),
            ],
        }),
        Proposal::AddToken(TokenMeta {
            token_id: "OP".to_string(),
            symbol: "OP".to_owned(),
            issue_chain: "Optimistic".to_string(),
            decimals: 18,
            icon: None,
            dst_chains: vec![
                "Bitcoin".to_string(),
                "Ethereum".to_string(),
                "ICP".to_string(),
                "Arbitrum".to_string(),
                "Starknet".to_string(),
            ],
        }),
        Proposal::AddToken(TokenMeta {
            token_id: "StarkNet".to_string(),
            symbol: "StarkNet".to_owned(),
            issue_chain: "Starknet".to_string(),
            decimals: 18,
            icon: None,
            dst_chains: vec![
                "Bitcoin".to_string(),
                "Ethereum".to_string(),
                "ICP".to_string(),
                "Arbitrum".to_string(),
                "Optimistic".to_string(),
            ],
        }),
    ];
    tokens
}

fn get_timestamp() -> u64 {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    since_the_epoch.as_millis() as u64
}

#[test]
fn test_init_hub() {
    let hub = OmnityHub::new();
    println!("hub id:{}", hub.hub_id.to_string())
}

#[test]
fn test_upgrade() {}

#[test]
fn test_validate_proposal() {
    let hub = OmnityHub::new();
    let ret = hub.validate_proposal(&chains());
    println!("test_validate_proposal result: {:#?}", ret)
}

#[test]
fn test_add_chain() {
    let hub = OmnityHub::new();
    let chains = chains();
    let ret = hub.validate_proposal(&chains);
    println!("test_validate_proposal result: {:#?}", ret);
    let ret = hub.execute_proposal(&chains);
    assert!(ret.is_ok());

    chain_ids().iter().for_each(|chain_id| {
        let result = hub.query_directives(
            &None,
            &Some(chain_id.to_string()),
            &Some(Topic::AddChain(None)),
            &0,
            &5,
        );
        println!("query_directives for {:} dires: {:#?}", chain_id, result);
        assert!(result.is_ok());
    });

    let result = hub.get_chains(&None, &None, &0, &10);
    println!("get_chains result : {:#?}", result);
    assert!(result.is_ok());

    let result = hub.get_chains(&Some(ChainType::ExecutionChain), &None, &0, &10);
    println!("get_chains result by chain type: {:#?}", result);
    assert!(result.is_ok());
}

#[test]
fn test_add_token() {
    let hub = OmnityHub::new();
    // add chain
    let ret = hub.validate_proposal(&chains());
    println!("test_validate_proposal result: {:#?}", ret);
    let ret = hub.execute_proposal(&chains());
    assert!(ret.is_ok());
    // add token
    let ret = hub.validate_proposal(&tokens());
    println!("test_validate_proposal result: {:#?}", ret);
    let ret = hub.execute_proposal(&tokens());
    assert!(ret.is_ok());

    for chain_id in chain_ids() {
        let result = hub.query_directives(
            &None,
            &Some(chain_id.to_string()),
            &Some(Topic::AddToken(None)),
            &0,
            &5,
        );
        println!("query_directives for {:} dires: {:#?}", chain_id, result);
        assert!(result.is_ok());
    }

    for canister_id in canister_ids() {
        let result = hub.query_directives(
            &Some(canister_id),
            &None,
            &Some(Topic::AddToken(None)),
            &0,
            &5,
        );
        println!("query_directives for {:} dires: {:#?}", canister_id, result);
        assert!(result.is_ok());
    }

    let result = hub.get_tokens(&None, &None, &0, &10);
    assert!(result.is_ok());
    println!("get_tokens result : {:#?}", result);

    let result = hub.get_tokens(&Some("Bitcoin".to_string()), &None, &0, &10);
    assert!(result.is_ok());
    println!("get_tokens result by chain_id: {:#?}", result);
    let result = hub.get_tokens(&Some("ICP".to_string()), &Some("ICP".to_string()), &0, &10);
    assert!(result.is_ok());
    println!("get_tokens result by chain_id and token id: {:#?}", result);
}

#[test]
fn test_toggle_chain_state() {
    let hub = OmnityHub::new();
    // add chain
    let ret = hub.validate_proposal(&chains());
    println!("test_validate_proposal result: {:#?}", ret);
    let ret = hub.execute_proposal(&chains());
    assert!(ret.is_ok());
    // add token
    let ret = hub.validate_proposal(&tokens());
    println!("test_validate_proposal result: {:#?}", ret);
    let ret = hub.execute_proposal(&tokens());
    assert!(ret.is_ok());

    // change chain state
    let chain_state = ToggleState {
        chain_id: "Optimistic".to_string(),
        action: ToggleAction::Deactivate,
    };

    // let toggle_state = Proposal::ToggleChainState(chain_state);
    let result = hub.validate_proposal(&vec![Proposal::ToggleChainState(chain_state.clone())]);
    assert!(result.is_ok());
    println!(
        "validate_proposal for Proposal::ToggleChainState(chain_state) result:{:#?}",
        result
    );
    let result = hub.execute_proposal(&vec![Proposal::ToggleChainState(chain_state)]);
    assert!(result.is_ok());

    // query directives for chain id
    let chaid_ids = [
        "Bitcoin",
        "Ethereum",
        "ICP",
        "Arbitrum",
        "Optimistic",
        "Starknet",
    ];

    for chain_id in chaid_ids {
        let result = hub.query_directives(
            &None,
            &Some(chain_id.to_string()),
            &Some(Topic::DeactivateChain),
            &0,
            &5,
        );
        println!("query_directives for {:} dires: {:#?}", chain_id, result);
        assert!(result.is_ok());
    }

    let result = hub.get_chains(
        &Some(ChainType::ExecutionChain),
        &Some(ChainState::Deactive),
        &0,
        &10,
    );
    assert!(result.is_ok());
    println!(
        "get_chains result by chain type and chain state: {:#?}",
        result
    );
}

#[test]
fn test_update_fee() {
    let hub = OmnityHub::new();
    // add chain
    let ret = hub.validate_proposal(&chains());
    println!("test_validate_proposal result: {:#?}", ret);
    let ret = hub.execute_proposal(&chains());
    assert!(ret.is_ok());
    // add token
    let ret = hub.validate_proposal(&tokens());
    println!("test_validate_proposal result: {:#?}", ret);
    let ret = hub.execute_proposal(&tokens());
    assert!(ret.is_ok());

    // change chain state
    let fee = Fee {
        dst_chain_id: "Arbitrum".to_string(),
        fee_token: "OP".to_string(),
        factor: 12,
    };

    let result = hub.update_fee(&vec![fee]);
    assert!(result.is_ok());
    println!("update_fee result:{:?}", result);

    // query directives for chain id
    let chaid_ids = [
        "Bitcoin",
        "Ethereum",
        "ICP",
        "Arbitrum",
        "Optimistic",
        "Starknet",
    ];

    for chain_id in chaid_ids {
        let result = hub.query_directives(
            &None,
            &Some(chain_id.to_string()),
            &Some(Topic::UpdateFee(None)),
            &0,
            &5,
        );
        println!("query_directives for {:} dires: {:#?}", chain_id, result);
        assert!(result.is_ok());
    }

    let result = hub.get_fees(&None, &None, &0, &10);
    assert!(result.is_ok());
    println!("get_chains result : {:#?}", result);

    let result = hub.get_fees(&None, &Some("OP".to_string()), &0, &10);
    assert!(result.is_ok());
    println!("get_chains result filter by token id : {:#?}", result);
}

#[test]
fn test_a_b_tx() {
    let hub = OmnityHub::new();
    // add chain
    let ret = hub.validate_proposal(&chains());
    println!("test_validate_proposal result: {:#?}", ret);
    let ret = hub.execute_proposal(&chains());
    assert!(ret.is_ok());
    // add token
    let ret = hub.validate_proposal(&tokens());
    println!("test_validate_proposal result: {:#?}", ret);
    let ret = hub.execute_proposal(&tokens());
    assert!(ret.is_ok());
    //
    // A->B: `transfer` ticket
    let src_chain = "Bitcoin";
    let dst_chain = "Arbitrum";
    let sender = "address_on_Bitcoin";
    let receiver = "address_on_Arbitrum";

    let transfer_ticket = Ticket {
        ticket_id: Uuid::new_v4().to_string(),
        ticket_time: get_timestamp(),
        src_chain: src_chain.to_string(),
        dst_chain: dst_chain.to_string(),
        action: TxAction::Transfer,
        token: "BTC".to_string(),
        amount: 88888.to_string(),
        sender: Some(sender.to_string()),
        receiver: receiver.to_string(),
        memo: None,
    };

    println!(
        " {} -> {} ticket:{:#?}",
        src_chain, dst_chain, transfer_ticket
    );
    let caller = Some(PrincipalId::new_user_test_id(1));
    let result = hub.send_ticket(&caller, &transfer_ticket);
    println!(
        "{} -> {} transfer result:{:?}",
        src_chain, dst_chain, result
    );
    assert!(result.is_ok());

    // query tickets for chain id
    let result = hub.query_tickets(&caller, &Some(dst_chain.to_string()), &0, &5);
    println!("query tickets for {:} tickets: {:#?}", dst_chain, result);
    assert!(result.is_ok());

    // query token on chain
    let result = hub.get_chain_tokens(&None, &None, &0, &5);
    println!("get_chain_tokens result: {:#?}", result);
    assert!(result.is_ok());

    // B->A: `redeem` ticket
    let src_chain = "Arbitrum";
    let dst_chain = "Bitcoin";
    let sender = "address_on_Arbitrum";
    let receiver = "address_on_Bitcoin";

    let redeem_ticket = Ticket {
        ticket_id: Uuid::new_v4().to_string(),
        ticket_time: get_timestamp(),
        src_chain: src_chain.to_string(),
        dst_chain: dst_chain.to_string(),
        action: TxAction::Redeem,
        token: "BTC".to_string(),
        amount: 88888.to_string(),
        sender: Some(sender.to_string()),
        receiver: receiver.to_string(),
        memo: None,
    };

    println!(
        " {} -> {} ticket:{:#?}",
        src_chain, dst_chain, redeem_ticket
    );
    let caller = Some(PrincipalId::new_user_test_id(2));
    let result = hub.send_ticket(&caller, &redeem_ticket);
    println!("{} -> {} redeem result:{:?}", src_chain, dst_chain, result);
    assert!(result.is_ok());

    // query tickets for chain id

    let result = hub.query_tickets(&caller, &Some(dst_chain.to_string()), &0, &5);
    println!("query tickets for {:} tickets: {:#?}", dst_chain, result);
    assert!(result.is_ok());

    // query token on chain
    let result = hub.get_chain_tokens(&None, &None, &0, &5);
    println!("get_chain_tokens result: {:#?}", result);
    assert!(result.is_ok());
}

#[test]
fn test_a_b_c_tx() {
    let hub = OmnityHub::new();
    // add chain
    let ret = hub.validate_proposal(&chains());
    println!("test_validate_proposal result: {:#?}", ret);
    let ret = hub.execute_proposal(&chains());
    assert!(ret.is_ok());
    // add token
    let ret = hub.validate_proposal(&tokens());
    println!("test_validate_proposal result: {:#?}", ret);
    let ret = hub.execute_proposal(&tokens());
    assert!(ret.is_ok());

    // transfer
    // A->B: `transfer` ticket
    let src_chain = "Ethereum";
    let dst_chain = "Optimistic";
    let sender = "address_on_Ethereum";
    let receiver = "address_on_Optimistic";

    let a_2_b_ticket = Ticket {
        ticket_id: Uuid::new_v4().to_string(),
        ticket_time: get_timestamp(),
        src_chain: src_chain.to_string(),
        dst_chain: dst_chain.to_string(),
        action: TxAction::Transfer,
        token: "ETH".to_string(),
        amount: 6666.to_string(),
        sender: Some(sender.to_string()),
        receiver: receiver.to_string(),
        memo: None,
    };

    println!(" {} -> {} ticket:{:#?}", src_chain, dst_chain, a_2_b_ticket);
    let caller = Some(PrincipalId::new_user_test_id(3));
    let result = hub.send_ticket(&caller, &a_2_b_ticket);
    println!(
        "{} -> {} transfer result:{:?}",
        src_chain, dst_chain, result
    );
    assert!(result.is_ok());
    // query tickets for chain id
    let result = hub.query_tickets(&caller, &Some(dst_chain.to_string()), &0, &5);
    println!("query tickets for {:} tickets: {:#?}", dst_chain, result);
    assert!(result.is_ok());

    // query token on chain
    let result = hub.get_chain_tokens(&None, &None, &0, &5);
    println!("get_chain_tokens result: {:#?}", result);
    assert!(result.is_ok());

    // B->C: `transfer` ticket
    let sender = "address_on_Optimistic";
    let receiver = "address_on_Starknet";
    let src_chain = "Optimistic";
    let dst_chain = "Starknet";

    let b_2_c_ticket = Ticket {
        ticket_id: Uuid::new_v4().to_string(),
        ticket_time: get_timestamp(),
        src_chain: src_chain.to_string(),
        dst_chain: dst_chain.to_string(),
        action: TxAction::Transfer,
        token: "ETH".to_string(),
        amount: 6666.to_string(),
        sender: Some(sender.to_string()),
        receiver: receiver.to_string(),
        memo: None,
    };

    println!(" {} -> {} ticket:{:#?}", src_chain, dst_chain, b_2_c_ticket);
    assert!(result.is_ok());
    let caller = Some(PrincipalId::new_user_test_id(4));
    let result = hub.send_ticket(&caller, &b_2_c_ticket);
    println!(
        "{} -> {} transfer result:{:?}",
        src_chain, dst_chain, result
    );

    // query tickets for chain id
    let result = hub.query_tickets(&caller, &Some(dst_chain.to_string()), &0, &5);
    println!("query tickets for {:} tickets: {:#?}", dst_chain, result);
    assert!(result.is_ok());

    // query token on chain
    let result = hub.get_chain_tokens(&None, &None, &0, &5);
    println!("get_chain_tokens result: {:#?}", result);
    assert!(result.is_ok());

    // redeem
    // C->B: `redeem` ticket
    let src_chain = "Starknet";
    let dst_chain = "Optimistic";
    let sender = "address_on_Starknet";
    let receiver = "address_on_Optimistic";

    let c_2_b_ticket = Ticket {
        ticket_id: Uuid::new_v4().to_string(),
        ticket_time: get_timestamp(),
        src_chain: src_chain.to_string(),
        dst_chain: dst_chain.to_string(),
        action: TxAction::Redeem,
        token: "ETH".to_string(),
        amount: 6666.to_string(),
        sender: Some(sender.to_string()),
        receiver: receiver.to_string(),
        memo: None,
    };

    println!(" {} -> {} ticket:{:#?}", src_chain, dst_chain, c_2_b_ticket);
    let caller = Some(PrincipalId::new_user_test_id(5));
    let result = hub.send_ticket(&caller, &c_2_b_ticket);
    println!("{} -> {} redeem result:{:?}", src_chain, dst_chain, result);
    assert!(result.is_ok());
    // query tickets for chain id
    let result = hub.query_tickets(&caller, &Some(dst_chain.to_string()), &0, &5);
    println!("query tickets for {:} tickets: {:#?}", dst_chain, result);
    assert!(result.is_ok());
    // query token on chain
    let result = hub.get_chain_tokens(&None, &None, &0, &5);
    println!("get_chain_tokens result: {:#?}", result);
    assert!(result.is_ok());

    // B->A: `redeem` ticket
    let sender = "address_on_Optimistic";
    let receiver = "address_on_Ethereum";
    let src_chain = "Optimistic";
    let dst_chain = "Ethereum";

    let b_2_a_ticket = Ticket {
        ticket_id: Uuid::new_v4().to_string(),
        ticket_time: get_timestamp(),
        src_chain: src_chain.to_string(),
        dst_chain: dst_chain.to_string(),
        action: TxAction::Redeem,
        token: "ETH".to_string(),
        amount: 6666.to_string(),
        sender: Some(sender.to_string()),
        receiver: receiver.to_string(),
        memo: None,
    };

    println!(" {} -> {} ticket:{:#?}", src_chain, dst_chain, b_2_a_ticket);

    let result = hub.send_ticket(&caller, &b_2_a_ticket);
    println!("{} -> {} redeem result:{:?}", src_chain, dst_chain, result);
    assert!(result.is_ok());

    // query tickets for chain id
    let result = hub.query_tickets(&caller, &Some(dst_chain.to_string()), &0, &5);
    println!("query tickets for {:} tickets: {:#?}", dst_chain, result);
    assert!(result.is_ok());

    // query token on chain
    let result = hub.get_chain_tokens(&None, &None, &0, &5);
    println!("get_chain_tokens result: {:#?}", result);
    assert!(result.is_ok());

    // query txs
    let result = hub.get_txs(&None, &None, &None, &None, &0, &10);
    println!("get_txs result: {:#?}", result);
    assert!(result.is_ok());
}
