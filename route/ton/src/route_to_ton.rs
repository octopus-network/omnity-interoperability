use crate::base::const_args::SEND_TON_TASK_NAME;
use crate::chainkey::minter_addr;
use crate::hub;
use crate::state::{mutate_state, read_state};
use crate::ton_transaction::build_jetton_mint;
use crate::toncenter::get_account_seqno;
use crate::types::PendingTicketStatus;
use anyhow::anyhow;
use ic_canister_log::log;
use omnity_types::ic_log::{INFO, WARNING};
use omnity_types::{Seq, Ticket};

pub fn to_ton_task() {
    ic_cdk::spawn(async {
        let _guard = match crate::guard::TimerLogicGuard::new(SEND_TON_TASK_NAME.to_string()) {
            Some(guard) => guard,
            None => return,
        };
        send_tickets_to_ton().await;
    });
}

pub async fn send_tickets_to_ton() {
    let from = read_state(|s| s.next_consume_ticket_seq);
    let to = read_state(|s| s.next_ticket_seq);
    for seq in from..to {
        match send_ticket(seq).await {
            Ok(h) => match h {
                None => {}
                Some(tx_hash) => {
                    let hub_principal = read_state(|s| s.hub_principal);
                    let ticket_id = read_state(|s| s.tickets_queue.get(&seq).unwrap().ticket_id);
                    match hub::update_tx_hash(hub_principal, ticket_id, tx_hash.clone()).await {
                        Err(err) => {
                            log!(
                                INFO,
                                "[rewrite tx_hash] failed to write mint tx hash, reason: {}",
                                err
                            );
                        }
                        _ => {
                            log!(
                                INFO,
                                "[rewrite tx_hash] sucess to write mint tx hash to hub"
                            );
                        }
                    }
                }
            },
            Err(e) => {
                if e.to_string() == "seqno_duplicate" {
                    log!(
                        WARNING,
                        "[evm_route] send ticket to ton temp error: {}",
                        e.to_string()
                    );
                    break;
                }
                log!(
                    WARNING,
                    "[evm_route] send ticket to evm error: {}",
                    e.to_string()
                );
            }
        }
        mutate_state(|s| s.next_consume_ticket_seq = seq + 1);
    }
}

pub async fn send_ticket(seq: Seq) -> anyhow::Result<Option<String>> {
    let ticket = read_state(|s| s.tickets_queue.get(&seq));
    match ticket {
        None => Ok(None),
        Some(t) => {
            if read_state(|s| s.finalized_mint_requests.contains_key(&t.ticket_id)) {
                return Ok(None);
            }
            if read_state(|s| s.pending_tickets_map.contains_key(&seq)) {
                return Ok(None);
            }
            inner_send_ticket(t, seq).await
        }
    }
}

pub async fn inner_send_ticket(t: Ticket, seq: Seq) -> anyhow::Result<Option<String>> {
    let jetton_master = read_state(|s| s.token_jetton_master_map.get(&t.token).cloned())
        .ok_or(anyhow!("token jetton master not set"))?;
    let minter = minter_addr();
    let seqno = get_account_seqno(&minter).await?;
    let boc = build_jetton_mint(&jetton_master, &t, seq, seqno).await?;
    let last_sucess_seqno = read_state(|s| s.last_success_seqno);
    if last_sucess_seqno >= seqno {
        return Err(anyhow!("seqno_duplicate"));
    }
    let msg_hash = crate::toncenter::send_boc(boc).await;
    match msg_hash {
        Ok(mh) => {
            mutate_state(|s| {
                s.finalized_mint_requests
                    .insert(t.ticket_id.clone(), mh.clone());
                s.last_success_seqno = seqno;
            });
            Ok(Some(mh))
        }
        Err(e) => {
            mutate_state(|s| {
                s.pending_tickets_map.insert(
                    seq,
                    PendingTicketStatus {
                        ton_tx_hash: None,
                        ticket_id: t.ticket_id.clone(),
                        seq,
                        error: Some(e.to_string()),
                        pending_time: ic_cdk::api::time(),
                    },
                )
            });
            Err(e)
        }
    }
}
