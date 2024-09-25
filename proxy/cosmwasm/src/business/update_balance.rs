use icrc_ledger_types::icrc1::account::Subaccount;
use itertools::Itertools;
use omnity_types::ic_log::WARNING;
use omnity_types::TicketId;
use crate::*;

use crate::{
    approve_ckbtc_for_icp_custom,
    external::{
        ckbtc,
        custom::generate_ticket,
    },
    state::{
        extend_ticket_records, get_settings, get_utxo_records, insert_utxo_records, pop_first_scheduled_osmosis_account_id, set_settings, MintedUtxo, TicketRecord, UtxoRecord
    },
    AddressData, Errors, UpdateBalanceArgs, UtxoStatus,
};

pub fn process_update_balance_jobs() {
    ic_cdk::spawn(async {
        let _guard = match crate::guard::TimerLogicGuard::new(
            "process_update_balance_jobs".to_string(),
        ) {
            Some(guard) => guard,
            None => return,
        };
        let mut settings = get_settings();
        let mut new_balance_jobs_list = vec![];
        for mut job in settings.update_balances_jobs {
            if !job.executable() {
                new_balance_jobs_list.push(job);
                continue;
            }

            match update_balance_and_generate_ticket(job.osmosis_account_id.clone()).await {
                Ok(_) => {}
                Err(e) => {
                    if job.handle_execute_failed_and_continue() {
                        new_balance_jobs_list.push(job.clone());
                    }
                    log!(WARNING, "Failed to execute update balance job : {:?},error: {:?}", job, e);
                }
            }
        }
        settings.update_balances_jobs = new_balance_jobs_list;
        set_settings(settings);
    })
}

pub fn read_osmosis_account_id_then_update_balance() {
    ic_cdk::spawn(async {
        match pop_first_scheduled_osmosis_account_id().and_then(|opt_account_id| {
            opt_account_id.ok_or(Errors::CustomError(
                "Failed to get osmosis account id".to_string(),
            ))
        }) {
            Ok(account_id) => {
                match update_balance_and_generate_ticket(account_id.clone()).await {
                    Ok(_) => {
                        log!(INFO, "Successfully update balance and generate ticket for osmosis account id: {}", account_id);
                    }
                    Err(_) => {
                        log!(ERROR, "Failed to update balance and generate ticket for osmosis account id: {}", account_id);
                    }
                }
            }
            Err(e) => {
                log!(ERROR, "Failed to get first scheduled osmosis account id: {:?}", e);
            }
        }
    });
}

pub async fn update_balance_and_generate_ticket(
    osmosis_account_id: String,
) -> std::result::Result<TicketId, String> {
    let address_data = AddressData::try_from(osmosis_account_id.as_str())
        .map_err(|e| Errors::AccountIdParseError(osmosis_account_id.clone(), e.to_string()))
        .map_err(|e| e.to_string())?;

    let subaccount: Subaccount = address_data.into();

    let result = ckbtc::update_balance(UpdateBalanceArgs {
        owner: None,
        subaccount: Some(subaccount.clone()),
    })
    .await
    .map_err(|e| e.to_string())?;

    log!(INFO, "osmosis account id: {} ,update_balance result: {:?}", osmosis_account_id, result);

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

    let mut utxo_record_list = get_utxo_records(osmosis_account_id.clone());
    utxo_record_list.extend(minted_success_utxo_status.clone());
    insert_utxo_records(osmosis_account_id.clone(), utxo_record_list);

    approve_ckbtc_for_icp_custom(Some(subaccount), minted_success_amount.into())
        .await
        .map_err(|e| e.to_string())?;

    let settings = get_settings();
    let ticket_amount = minted_success_amount.checked_sub(2 * CKBTC_FEE)
    .ok_or("overflow".to_string())?;
    let ticket_id = generate_ticket(
        settings.token_id.to_string(),
        settings.target_chain_id.to_string(),
        ticket_amount as u128,
        subaccount,
    )
    .await
    .map_err(|e| e.to_string())?;

    log!(INFO, "osmosis account id: {} ,generate_ticket result: {:?}", osmosis_account_id, ticket_id);

    let mut utxo_record_list = get_utxo_records(osmosis_account_id.clone());
    utxo_record_list.iter_mut().for_each(|e| {
        if block_index_set.contains(&e.minted_utxo.block_index) {
            e.ticket_id = Some(ticket_id.clone());
        }
    });
    insert_utxo_records(osmosis_account_id.clone(), utxo_record_list.clone());

    let belong_ticket_utxos = utxo_record_list
        .iter()
        .filter(|e| e.ticket_id.is_some())
        .map(|e| e.minted_utxo.clone())
        .collect_vec();

    extend_ticket_records(
        osmosis_account_id,
        vec![TicketRecord {
            ticket_id: ticket_id.clone(),
            minted_utxos: belong_ticket_utxos,
        }],
    );

    Ok(ticket_id)
}
