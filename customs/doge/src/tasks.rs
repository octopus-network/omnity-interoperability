use std::time::Duration;

use crate::doge::block::DogecoinHeader;
use crate::doge::header::BlockHeaderJsonResult;
use crate::doge::rpc::DogeRpc;
use crate::dogeoin_to_custom::finalize_lock_ticket_task;
use crate::errors::CustomsError;
use crate::state::{mutate_state, read_state};
use bitcoin::consensus::deserialize;
use ic_canister_log::log;
use ic_cdk_timers::set_timer_interval;
use omnity_types::ic_log::{ERROR, INFO, WARNING};

use crate::constants::*;
use crate::custom_to_dogecoin::{finalize_unlock_tickets_task, submit_unlock_tickets_task};
use crate::hub_to_custom::{fetch_hub_directive_task, fetch_hub_ticket_task};

pub fn start_tasks() {
    set_timer_interval(
        Duration::from_secs(FETCH_HUB_TICKET_INTERVAL),
        fetch_hub_ticket_task,
    );
    set_timer_interval(
        Duration::from_secs(FETCH_HUB_DIRECTIVE_INTERVAL),
        fetch_hub_directive_task,
    );
    set_timer_interval(
        Duration::from_secs(SUBMIT_UNLOCK_TICKETS_INTERVAL),
        submit_unlock_tickets_task,
    );
    set_timer_interval(
        Duration::from_secs(FINALIZE_LOCK_TICKET_INTERVAL),
        finalize_lock_ticket_task,
    );
    set_timer_interval(
        Duration::from_secs(FINALIZE_UNLOCK_TICKET_INTERVAL),
        finalize_unlock_tickets_task,
    );
    set_timer_interval(
        Duration::from_secs(SYNC_DOGE_BLOCK_HEADER_INTERVAL),
        sync_doge_block_header_task,
    );
}

fn sync_doge_block_header_task() {
    ic_cdk::spawn(async {
        let _guard =
            match crate::guard::TimerLogicGuard::new(SYNC_DOGE_BLOCK_HEADER_NAME.to_string()) {
                Some(guard) => guard,
                None => return,
            };

        match process_sync_doge_block_header().await {
            Ok(_) => {}
            Err(e) => {
                log!(ERROR, "sync doge block header error: {:?}", e);
            }
        }
    });
}

async fn process_sync_doge_block_header() -> Result<(), CustomsError> {
    let mut max_sync_times = 5;
    while max_sync_times > 0 {
        let current_block_header =
            read_state(|s| s.doge_block_headers.get(&s.sync_doge_block_header_height)).ok_or(
                CustomsError::CustomError("current block header not found".to_string()),
            )?;
        let next_block_hash = if let Some(next_block_hash) = current_block_header.nextblockhash {
            next_block_hash
        } else {
            let doge_rpc: DogeRpc = read_state(|s| s.default_doge_rpc_config.clone()).into();
            match doge_rpc
                .get_block_hash(current_block_header.height + 1)
                .await
            {
                Ok(next_block_hash) => next_block_hash,
                Err(e) => {
                    log!(WARNING, "get next block hash error: {:?}", e);
                    return Ok(())
                }
            }
        };
        let r = sync_doge_block_header(next_block_hash).await?;
        log!(
            INFO,
            "success to sync doge block header height: {:?}, hash: {:?}",
            r.height,
            r.hash
        );

        if r.confirmations < 2 {
            break;
        }
        max_sync_times -= 1;
    }

    Ok(())
}

async fn sync_doge_block_header(block_hash: String) -> Result<BlockHeaderJsonResult, CustomsError> {
    use hex::test_hex_unwrap as hex;

    // fetch block header
    let doge_rpc: DogeRpc = read_state(|s| s.default_doge_rpc_config.clone()).into();
    let mut block_header_json_result = doge_rpc.get_block_header(block_hash.as_str()).await?;
    let blocker_header_hex = doge_rpc.get_block_header_hex(block_hash.as_str()).await?;
    block_header_json_result.block_header_hex = Some(blocker_header_hex.clone());

    // verify pow
    let doge_header: DogecoinHeader =
        deserialize(&hex!(blocker_header_hex.as_str())).map_err(|e| {
            CustomsError::CustomError(format!("deserialize doge header error: {:?}", e))
        })?;
    let _ = doge_header.validate_doge_pow(true)?;

    // verify prev hash

    let last_block_hash = read_state(|s| {
        s.doge_block_headers
            .get(&(block_header_json_result.height - 1))
    })
    .ok_or(CustomsError::CustomError(
        "last block header not found".to_string(),
    ))?
    .hash;

    if last_block_hash != block_header_json_result.previousblockhash {
        return Err(CustomsError::CustomError(
            "prev block hash not match".to_string(),
        ));
    }

    // save to state
    mutate_state(|s| {
        s.doge_block_headers.insert(
            block_header_json_result.height,
            block_header_json_result.clone(),
        );
        s.sync_doge_block_header_height = block_header_json_result.height;
    });

    Ok(block_header_json_result)
}
