use external::{ckbtc::{self, approve_ckbtc_for_icp_custom, UpdateBalanceArgs, UtxoStatus, CKBTC_FEE}, custom::generate_ticket};
use icrc_ledger_types::icrc1::account::Subaccount;
use itertools::Itertools;
use omnity_types::ic_log::WARNING;
use omnity_types::TicketId;
use state::{extend_ticket_records, get_state, get_utxo_records, insert_utxo_records, set_state};
use types::{MintedUtxo, OmnityAccount, TicketRecord, UtxoRecord};
use crate::*;

pub fn process_update_balance_jobs() {
    ic_cdk::spawn(async {
        let _guard = match crate::guard::TimerLogicGuard::new(
            "process_update_balance_jobs".to_string(),
        ) {
            Some(guard) => guard,
            None => return,
        };
        let mut state = get_state();
        let mut new_balance_jobs_list = vec![];
        for mut job in state.update_balances_jobs {
            if !job.executable() {
                new_balance_jobs_list.push(job);
                continue;
            }

            match update_balance_and_generate_ticket(job.omnity_account.clone()).await {
                Ok(_) => {}
                Err(e) => {
                    if job.handle_execute_failed_and_continue() {
                        new_balance_jobs_list.push(job.clone());
                    }
                    log!(WARNING, "Failed to execute update balance job : {:?},error: {:?}", job, e);
                }
            }
        }
        state.update_balances_jobs = new_balance_jobs_list;
        set_state(state);
    })
}

pub async fn update_balance_and_generate_ticket(
    omnity_account: OmnityAccount,
) -> std::result::Result<TicketId, String> {

    let subaccount: Subaccount = omnity_account.get_mapping_subaccount();

    let result = ckbtc::update_balance(UpdateBalanceArgs {
        owner: None,
        subaccount: Some(subaccount.clone()),
    })
    .await
    .map_err(|e| e.to_string())?;

    log!(INFO, "omnity_account: {:?} ,update_balance result: {:?}", omnity_account, result);

    let minted_success_utxo_status = result
        .iter()
        .filter_map(|e| match e {
            UtxoStatus::Minted {
                block_index,
                minted_amount,
                utxo,
            } => Some(UtxoRecord {
                minted_utxo: MintedUtxo {
                    block_index: block_index.clone(),
                    minted_amount: minted_amount.clone(),
                    utxo: utxo.clone(),
                },
                ticket_id: None,
            }),
            _ => None,
        })
        .collect_vec();

    let minted_success_amount = minted_success_utxo_status
        .iter()
        .map(|e| e.minted_utxo.minted_amount)
        .sum::<u64>();
    let block_index_set = minted_success_utxo_status
        .iter()
        .map(|e| e.minted_utxo.block_index)
        .collect::<std::collections::HashSet<u64>>();

    let mut utxo_record_list = get_utxo_records(omnity_account.clone());
    utxo_record_list.extend(minted_success_utxo_status.clone());
    insert_utxo_records(omnity_account.clone(), utxo_record_list);

    approve_ckbtc_for_icp_custom(Some(subaccount), minted_success_amount.into())
        .await
        .map_err(|e| e.to_string())?;

    let settings = get_state();
    let ticket_amount = minted_success_amount.checked_sub(2 * CKBTC_FEE)
    .ok_or("overflow".to_string())?;
    let ticket_id = generate_ticket(
        settings.token_id.to_string(),
        ticket_amount as u128,
        omnity_account.clone(),
    )
    .await
    .map_err(|e| e.to_string())?;

    log!(INFO, "Success to generate_ticket, omnity account: {:?} , ticket id: {:?}", omnity_account, ticket_id);

    let mut utxo_record_list = get_utxo_records(omnity_account.clone());
    utxo_record_list.iter_mut().for_each(|e| {
        if block_index_set.contains(&e.minted_utxo.block_index) {
            e.ticket_id = Some(ticket_id.clone());
        }
    });
    insert_utxo_records(omnity_account.clone(), utxo_record_list.clone());

    let belong_ticket_utxos = utxo_record_list
        .iter()
        .filter(|e| e.ticket_id.is_some())
        .map(|e| e.minted_utxo.clone())
        .collect_vec();

    extend_ticket_records(
        omnity_account,
        vec![TicketRecord {
            ticket_id: ticket_id.clone(),
            minted_utxos: belong_ticket_utxos,
        }],
    );

    Ok(ticket_id)
}
