use crate::types::*;
use candid::Principal;

/// this could be empty since even some errors occur, we can't do anything but waiting for the next timer
async fn query_pending_task(principal: Principal) -> Vec<GenTicketRequestV2> {
    let args = GetGenTicketReqsArgs {
        start_txid: None,
        max_count: 50,
    };
    let (v,): (Vec<GenTicketRequestV2>,) =
        ic_cdk::call(principal, "get_pending_gen_ticket_requests", (args,))
            .await
            .inspect_err(|e| log::error!("fetch error: {:?}", e))
            .unwrap_or_default();
    v
}

/// query rune utxos
async fn query_indexer(
    principal: Principal,
    rune_id: RuneId,
    txid: String,
    vout: u32,
) -> anyhow::Result<Option<RunesBalance>> {
    let (balances,): (Result<Vec<Balance>, OrdError>,) =
        ic_cdk::call(principal, "get_runes_by_utxo", (txid, vout))
            .await
            .inspect_err(|e| log::error!("query indexer error {:?}", e))
            .map_err(|e| anyhow::anyhow!("{:?}", e))?;
    let balances = balances
        .inspect_err(|e| log::error!("indexer returns err: {:?}", e))
        .map_err(|e| anyhow::anyhow!("{:?}", e))?;
    let rune = balances
        .into_iter()
        .filter(|b| b.id == rune_id)
        .map(|b| b.into_runes_balance(vout))
        .reduce(|a, b| RunesBalance {
            rune_id: a.rune_id,
            vout: a.vout,
            amount: a.amount + b.amount,
        });
    Ok(rune)
}

async fn update_runes_balance(
    principal: Principal,
    txid: Txid,
    balances: Vec<RunesBalance>,
) -> anyhow::Result<()> {
    let args = UpdateRunesBalanceArgs { txid, balances };
    let _: () = ic_cdk::call(principal, "update_runes_balance", (args,))
        .await
        .inspect_err(|e| log::error!("update error: {:?}", e))
        .map_err(|e| anyhow::anyhow!("{:?}", e))?;
    Ok(())
}

pub(crate) fn fetch_then_submit(secs: u64) {
    let customs = crate::customs_principal();
    let indexer = crate::indexer_principal();
    ic_cdk_timers::set_timer(std::time::Duration::from_secs(secs), move || {
        ic_cdk::spawn(async move {
            let pending = query_pending_task(customs).await;
            if pending.is_empty() {
                fetch_then_submit(30);
                return;
            }
            // for each task
            for task in pending.iter() {
                let mut balances = vec![];
                let mut error = false;
                for utxo in task.new_utxos.iter() {
                    match query_indexer(
                        indexer,
                        task.rune_id,
                        format!("{}", utxo.outpoint.txid),
                        utxo.outpoint.vout,
                    )
                    .await
                    {
                        Ok(Some(balance)) => balances.push(balance),
                        Ok(None) => log::info!("no rune found for utxo {:?}", utxo.outpoint),
                        Err(e) => {
                            log::error!("{:?}", e);
                            error = true;
                            break;
                        }
                    }
                }
                // ignore the task if any error occurs
                if error {
                    continue;
                }
                log::info!(
                    "prepare to submit {} rune balances for task {:?}",
                    balances.len(),
                    task.txid
                );
                if let Err(e) = update_runes_balance(customs, task.txid, balances).await {
                    log::error!("{:?}", e);
                }
            }
            if pending.len() < 50 {
                fetch_then_submit(30);
            } else {
                fetch_then_submit(1);
            }
        });
    });
}
