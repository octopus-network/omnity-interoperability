use crate::types::*;
use candid::Principal;

/// this could be empty since even some errors occur, we can't do anything but waiting for the next timer
async fn query_pending_task(principal: Principal) -> Vec<OutPoint> {
    let (v,): (Vec<GenTicketRequestV2>,) = ic_cdk::call(
        principal,
        "get_pending_gen_ticket_requests",
        (None::<Txid>, 50),
    )
    .await
    .unwrap_or_default();
    v.into_iter()
        .flat_map(|r| r.new_utxos.into_iter())
        .map(|utxo| utxo.outpoint)
        .collect()
}

/// query rune utxos
async fn query_indexer(
    principal: Principal,
    txid: String,
    vout: u32,
) -> anyhow::Result<Vec<Balance>> {
    let (balances,): (Vec<Balance>,) = ic_cdk::call(principal, "get_runes_by_utxo", (txid, vout))
        .await
        .map_err(|e| anyhow::anyhow!("{:?}", e))?;
    Ok(balances)
}

async fn update_runes_balance(
    principal: Principal,
    txid: Txid,
    vout: u32,
    balances: Vec<Balance>,
) -> anyhow::Result<()> {
    let args = UpdateRunesBalanceArgs {
        txid,
        balances: balances
            .into_iter()
            .map(|b| RunesBalance::from((vout, b)))
            .collect(),
    };
    let _: () = ic_cdk::call(principal, "update_runes_balance", (args,))
        .await
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
                fetch_then_submit(300);
                return;
            }
            for outpoint in pending.iter() {
                match query_indexer(indexer, format!("{}", outpoint.txid), outpoint.vout).await {
                    Ok(balances) => {
                        match update_runes_balance(customs, outpoint.txid, outpoint.vout, balances)
                            .await
                        {
                            Ok(_) => {}
                            Err(e) => log::error!("error updating runes balance: {:?}", e),
                        }
                    }
                    Err(e) => log::error!("error querying indexer: {:?}", e),
                }
            }
            fetch_then_submit(300);
        });
    });
}
