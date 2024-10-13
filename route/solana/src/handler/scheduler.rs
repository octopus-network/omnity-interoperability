use super::{fecth_directive, mint_token};
use crate::constants::CREATE_ATA_INTERVAL;
use crate::constants::CREATE_MINT_INTERVAL;
use crate::handler::associated_account;
use crate::handler::fetch_ticket;
use crate::handler::token_account;
use crate::state::mutate_state;
use crate::{
    constants::{
        MINT_TOKEN_INTERVAL, QUERY_DERECTIVE_INTERVAL, QUERY_TICKET_INTERVAL, UPDATE_TOKEN_INTERVAL,
    },
    guard::{TaskType, TimerGuard},
};
use ic_canister_log::log;
use ic_cdk_timers::TimerId;
use ic_solana::ic_log::{DEBUG, WARNING};
use std::cell::RefCell;
use std::collections::HashMap;

thread_local! {
    static TIMER_GUARD: RefCell<HashMap<TaskType,TimerId>> = RefCell::new(HashMap::default());
}

pub fn start_schedule() {
    // query_directives task
    let directive_timer_id = ic_cdk_timers::set_timer_interval(QUERY_DERECTIVE_INTERVAL, || {
        ic_cdk::spawn(async {
            let _guard = match TimerGuard::new(TaskType::GetDirectives) {
                Ok(guard) => guard,
                Err(e) => {
                    log!(WARNING, "TaskType::GetDirectives error : {:?}", e);
                    return;
                }
            };
            fecth_directive::query_directives().await;
        });
    });
    log!(DEBUG, " GetDirectives task id : {:?}", directive_timer_id);
    TIMER_GUARD.with_borrow_mut(|guard| {
        guard.insert(TaskType::GetDirectives, directive_timer_id);
    });

    // handle to create mint token account
    let create_mint_timer_id = ic_cdk_timers::set_timer_interval(CREATE_MINT_INTERVAL, || {
        ic_cdk::spawn(async {
            let _guard = match TimerGuard::new(TaskType::CreateMint) {
                Ok(guard) => guard,
                Err(e) => {
                    log!(WARNING, "TaskType::CreateMint error : {:?}", e);
                    return;
                }
            };
            token_account::create_token_mint().await;
        });
    });
    log!(DEBUG, "CreateMint task id : {:?}", create_mint_timer_id);
    TIMER_GUARD.with_borrow_mut(|guard| {
        guard.insert(TaskType::CreateMint, create_mint_timer_id);
    });

    // handle to update token metadata
    let update_token_timer_id = ic_cdk_timers::set_timer_interval(UPDATE_TOKEN_INTERVAL, || {
        ic_cdk::spawn(async {
            let _guard = match TimerGuard::new(TaskType::UpdateToken) {
                Ok(guard) => guard,
                Err(e) => {
                    log!(WARNING, "TaskType::UpdateToken error : {:?}", e);
                    return;
                }
            };
            token_account::update_token().await;
        });
    });
    log!(DEBUG, "UpdateToken task id: {:?}", update_token_timer_id);
    TIMER_GUARD.with_borrow_mut(|guard| {
        guard.insert(TaskType::UpdateToken, update_token_timer_id);
    });

    // query_tickets task
    let query_ticket_timer_id = ic_cdk_timers::set_timer_interval(QUERY_TICKET_INTERVAL, || {
        ic_cdk::spawn(async {
            let _guard = match TimerGuard::new(TaskType::GetTickets) {
                Ok(guard) => guard,
                Err(e) => {
                    log!(WARNING, "TaskType::GetTickets error : {:?}", e);
                    return;
                }
            };

            fetch_ticket::query_tickets().await;
        });
    });
    log!(DEBUG, "GetTickets task id : {:?}", query_ticket_timer_id);
    TIMER_GUARD.with_borrow_mut(|guard| {
        guard.insert(TaskType::GetTickets, query_ticket_timer_id);
    });

    // handle to create_associated_account
    let create_ata_timer_id = ic_cdk_timers::set_timer_interval(CREATE_ATA_INTERVAL, || {
        ic_cdk::spawn(async {
            let _guard = match TimerGuard::new(TaskType::CreateATA) {
                Ok(guard) => guard,
                Err(e) => {
                    log!(WARNING, "TaskType::CreateATA error : {:?}", e);
                    return;
                }
            };

            associated_account::create_associated_account().await;
        });
    });
    log!(DEBUG, "CreateATA task id : {:?}", create_ata_timer_id);
    TIMER_GUARD.with_borrow_mut(|guard| {
        guard.insert(TaskType::CreateATA, create_ata_timer_id);
    });

    // handle to mint_to
    let mint_token_timer_id = ic_cdk_timers::set_timer_interval(MINT_TOKEN_INTERVAL, || {
        ic_cdk::spawn(async {
            let _guard = match TimerGuard::new(TaskType::MintToken) {
                Ok(guard) => guard,
                Err(e) => {
                    log!(WARNING, "TaskType::MintToken error : {:?}", e);
                    return;
                }
            };

            mint_token::mint_token().await;
        });
    });
    log!(DEBUG, "MintToken task id : {:?}", mint_token_timer_id);
    TIMER_GUARD.with_borrow_mut(|guard| {
        guard.insert(TaskType::MintToken, mint_token_timer_id);
    });
}

// clear the running tasks
pub fn cancel_schedule() {
    TIMER_GUARD.with_borrow_mut(|guard| {
        guard
            .iter()
            .for_each(|(_task_type, task_id)| ic_cdk_timers::clear_timer(*task_id));
        guard.clear()
    });
    mutate_state(|s| s.active_tasks.clear());
}
// get tasks
// pub fn get_tasks() -> Vec<(TaskType, TimerId)> {
//     TIMER_GUARD
//         .with_borrow(|guard|
//             guard.iter().map(|(k, v)| (k.to_owned(), v.to_owned())))
//         .collect()
// }
