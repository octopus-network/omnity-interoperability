use ic_cdk_timers::TimerId;
use std::cell::RefCell;
use std::collections::HashMap;

use super::{directive, ticket};
use crate::constants::CREATE_ATA_INTERVAL;
use crate::constants::CREATE_MINT_INTERVAL;
use crate::{
    constants::{MINT_TOKEN_INTERVAL, QUERY_DERECTIVE_INTERVAL, QUERY_TICKET_INTERVAL},
    guard::{TaskType, TimerGuard},
};
thread_local! {
    static TIMER_GUARD: RefCell<HashMap<TaskType,TimerId>> = RefCell::new(HashMap::default());
}

pub fn start_schedule() {
    // query_directives task
    let directive_timer_id = ic_cdk_timers::set_timer_interval(QUERY_DERECTIVE_INTERVAL, || {
        ic_cdk::spawn(async {
            let _guard = match TimerGuard::new(TaskType::GetDirectives) {
                Ok(guard) => guard,
                Err(_) => return,
            };
            // ic_cdk::println!("directive::query_directives() at {:?}", ic_cdk::api::time());
            directive::query_directives().await;
        });
    });
    ic_cdk::println!(" started query_directives task : {:?}", directive_timer_id);
    TIMER_GUARD.with_borrow_mut(|guard| {
        guard.insert(TaskType::GetDirectives, directive_timer_id);
    });

    // handle to create mint token account
    let create_mint_timer_id = ic_cdk_timers::set_timer_interval(CREATE_MINT_INTERVAL, || {
        ic_cdk::spawn(async {
            let _guard = match TimerGuard::new(TaskType::CreateMint) {
                Ok(guard) => guard,
                Err(_) => return,
            };

            // ic_cdk::println!(
            //     "directive::create_token_mint() at {:?}",
            //     ic_cdk::api::time()
            // );
            directive::create_token_mint().await;
        });
    });
    ic_cdk::println!(
        "started create_token_mint task : {:?}",
        create_mint_timer_id
    );
    TIMER_GUARD.with_borrow_mut(|guard| {
        guard.insert(TaskType::CreateMint, create_mint_timer_id);
    });

    // query_tickets task
    let query_ticket_timer_id = ic_cdk_timers::set_timer_interval(QUERY_TICKET_INTERVAL, || {
        ic_cdk::spawn(async {
            let _guard = match TimerGuard::new(TaskType::GetTickets) {
                Ok(guard) => guard,
                Err(_) => return,
            };

            // ic_cdk::println!("ticket::query_tickets() at {:?}", ic_cdk::api::time());
            ticket::query_tickets().await;
        });
    });
    ic_cdk::println!("started query_tickets task : {:?}", query_ticket_timer_id);
    TIMER_GUARD.with_borrow_mut(|guard| {
        guard.insert(TaskType::GetTickets, query_ticket_timer_id);
    });

    // handle to create_associated_account
    let create_associated_account_timer_id =
        ic_cdk_timers::set_timer_interval(CREATE_ATA_INTERVAL, || {
            ic_cdk::spawn(async {
                let _guard = match TimerGuard::new(TaskType::CreateAssoicatedAccount) {
                    Ok(guard) => guard,
                    Err(_) => return,
                };

                // ic_cdk::println!(
                //     "ticket::create_associated_account() at {:?}",
                //     ic_cdk::api::time()
                // );
                ticket::create_associated_account().await;
            });
        });
    ic_cdk::println!(
        "started create_token_mint task : {:?}",
        create_associated_account_timer_id
    );
    TIMER_GUARD.with_borrow_mut(|guard| {
        guard.insert(
            TaskType::CreateAssoicatedAccount,
            create_associated_account_timer_id,
        );
    });

    // handle to mint_to
    let mint_token_timer_id = ic_cdk_timers::set_timer_interval(MINT_TOKEN_INTERVAL, || {
        ic_cdk::spawn(async {
            let _guard = match TimerGuard::new(TaskType::MintToken) {
                Ok(guard) => guard,
                Err(_) => return,
            };

            // ic_cdk::println!("ticket::handle_mint_token() at {:?}", ic_cdk::api::time());
            ticket::handle_mint_token().await;
        });
    });
    ic_cdk::println!("started handle_mint_token task : {:?}", mint_token_timer_id);
    TIMER_GUARD.with_borrow_mut(|guard| {
        guard.insert(TaskType::MintToken, mint_token_timer_id);
    });
}

// clear the running tasks
pub fn cannel_schedule() {
    TIMER_GUARD.with_borrow_mut(|guard| {
        guard
            .iter()
            .for_each(|(_task_type, task_id)| ic_cdk_timers::clear_timer(*task_id));
        guard.clear()
    });
}
