use bitcoin_customs::{
    queries::GetGenTicketReqsArgs,
    state::{GenTicketRequestV2, RunesBalance},
    updates::update_runes_balance::UpdateRunesBalanceArgs,
};
use candid::Principal;
pub use ic_btc_interface::Txid;
use ic_canister_log::log;
use omnity_hub::state::{AddRunesTokenReq, FinalizeAddRunesArgs};
use omnity_types::{ic_log::*, rune_id::RuneId};
use runes_indexer_interface::RuneEntry;

const MIN_CONFIRMATIONS: u32 = 4;

/// this could be empty since even some errors occur, we can't do anything but waiting for the next timer
async fn query_pending_task(principal: Principal) -> Vec<GenTicketRequestV2> {
    let args = GetGenTicketReqsArgs {
        start_txid: None,
        max_count: 50,
    };
    let (v,): (Vec<GenTicketRequestV2>,) =
        ic_cdk::call(principal, "get_pending_gen_ticket_requests", (args,))
            .await
            .inspect_err(|e| log!(WARNING, "fetch error: {:?}", e))
            .unwrap_or_default();
    v
}

async fn query_pending_add_token_task(principal: Principal) -> Vec<AddRunesTokenReq> {
    let (v,): (Vec<AddRunesTokenReq>,) =
        ic_cdk::call(principal, "get_add_runes_token_requests", ())
            .await
            .inspect_err(|e| log!(WARNING, "get_add_runes_token_requests error: {:?}", e))
            .unwrap_or_default();
    v
}

async fn query_rune_id_from_indexer(principal: Principal, rune_id: String) -> Option<RuneEntry> {
    let (v,): (Option<RuneEntry>,) = ic_cdk::call(principal, "get_rune_by_id", (rune_id,))
        .await
        .inspect_err(|e| log!(WARNING, "get_rune_by_id error: {:?}", e))
        .unwrap_or_default();
    v
}

async fn finalize_add_runes_token_req(
    principal: Principal,
    rune_id: String,
    spaced_rune: String,
    divisibility: u8,
) -> anyhow::Result<()> {
    let finalize_args = FinalizeAddRunesArgs {
        rune_id,
        name: spaced_rune,
        decimal: divisibility,
    };

    let _: () = ic_cdk::call(principal, "finalize_add_runes_token_req", (finalize_args,))
        .await
        .inspect_err(|e| log!(WARNING, "finalize_add_runes_token_req error: {:?}", e))
        .unwrap_or_default();
    Ok(())
}

/// query rune utxos
async fn query_indexer(
    principal: Principal,
    rune_id: RuneId,
    txid: String,
    vout: u32,
) -> anyhow::Result<Option<RunesBalance>> {
    let outpoint = format!("{}:{}", txid, vout);

    let (balances,): (
        Result<
            Vec<Option<Vec<runes_indexer_interface::RuneBalance>>>,
            runes_indexer_interface::Error,
        >,
    ) = ic_cdk::call(
        principal,
        "get_rune_balances_for_outputs",
        (vec![outpoint.clone()],),
    )
    .await
    .map_err(|e| {
        log!(WARNING, "query indexer error {:?}", e);
        anyhow::anyhow!("{:?}", e)
    })?;

    let balances = balances.map_err(|e| {
        log!(WARNING, "indexer returns err: {:?}", e);
        anyhow::anyhow!("{:?}", e)
    })?;

    let Some(balances) = balances.get(0).and_then(|b| b.as_ref()) else {
        return Ok(None);
    };

    let rune = balances
        .iter()
        .filter(|b| b.rune_id == rune_id.to_string() && b.confirmations >= MIN_CONFIRMATIONS)
        .fold(None, |acc, b| {
            let balance = RunesBalance {
                rune_id,
                vout,
                amount: b.amount + acc.map_or(0, |prev: RunesBalance| prev.amount),
            };
            Some(balance)
        });
    log!(
        INFO,
        "query indexer for outpoint: {}, rune_id: {}, result: rune: {:?}",
        outpoint,
        rune_id,
        rune
    );

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
        .inspect_err(|e| log!(WARNING, "update error: {:?}", e))
        .map_err(|e| anyhow::anyhow!("{:?}", e))?;
    Ok(())
}

async fn finalize_update_runes_balance() {
    let customs = crate::customs_principal();
    let indexer = crate::indexer_principal();
    let pending = query_pending_task(customs).await;
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
                Ok(None) => log!(INFO, "no rune found for utxo {:?}", utxo.outpoint),
                Err(e) => {
                    log!(ERROR, "{:?}", e);
                    error = true;
                    break;
                }
            }
        }
        // ignore the task if any error occurs
        if error {
            continue;
        }
        log!(
            INFO,
            "prepare to submit {:?} rune balances for task {}",
            balances,
            task.txid.to_string()
        );
        if let Err(e) = update_runes_balance(customs, task.txid, balances).await {
            log!(ERROR, "{:?}", e);
        }
    }
}
async fn finalize_add_rune() {
    let hub = crate::hub_principal();
    let indexer = crate::indexer_principal();
    let pending = query_pending_add_token_task(hub).await;
    for task in pending.iter() {
        log!(INFO, "finalize add rune for task {:?}", task);
        let rune_entry = query_rune_id_from_indexer(indexer, task.rune_id.clone()).await;
        if let Some(rune_entry) = rune_entry {
            finalize_add_runes_token_req(
                hub,
                task.rune_id.clone(),
                rune_entry.spaced_rune,
                rune_entry.divisibility,
            )
            .await;
        }
    }
}

pub(crate) fn fetch_then_submit(secs: u64) {
    ic_cdk_timers::set_timer(std::time::Duration::from_secs(secs), move || {
        ic_cdk::spawn(async move {
            finalize_update_runes_balance().await;
            finalize_add_rune().await;
            fetch_then_submit(5);
        });
    });
}
