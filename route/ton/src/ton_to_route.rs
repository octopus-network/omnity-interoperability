use crate::base::const_args::SCAN_TON_TASK_NAME;
use crate::hub;
use crate::state::{mutate_state, read_state};
use crate::toncenter::{query_mint_message, Message};
use ic_canister_log::log;
use num_traits::ToPrimitive;
use omnity_types::ic_log::INFO;
use omnity_types::Seq;
use tonlib_core::cell::BagOfCells;

pub fn scan_mint_events_task() {
    ic_cdk::spawn(async {
        let _guard = match crate::guard::TimerLogicGuard::new(SCAN_TON_TASK_NAME.to_string()) {
            Some(guard) => guard,
            None => return,
        };
        scan_mint_events().await;
    });
}

pub async fn scan_mint_events() {
    let time = ic_cdk::api::time() - 300000000000;
    let pending_tickets = read_state(|s| {
        s.pending_tickets_map
            .iter()
            .filter_map(|p| {
                if p.1.pending_time > time {
                    Some(p.0)
                } else {
                    None
                }
            })
            .collect::<Vec<Seq>>()
    });
    if pending_tickets.is_empty() {
        return;
    }
    let r = query_mint_message().await;
    if let Ok(messages) = r {
        let ms = messages.messages;
        for msg in ms {
            if let Ok((seqno, hash)) = check_mint_message(msg) {
                if read_state(|s| s.pending_tickets_map.contains_key(&seqno)) {
                    let hub_principal = read_state(|s| s.hub_principal);
                    let ticket_id = read_state(|s| s.tickets_queue.get(&seqno).unwrap().ticket_id);
                    match hub::update_tx_hash(hub_principal, ticket_id.clone(), hash.clone()).await
                    {
                        Err(err) => {
                            log!(
                                INFO,
                                "[rewrite tx_hash] failed to write mint tx hash, reason: {}",
                                err
                            );
                        }
                        Ok(_) => {
                            log!(
                                INFO,
                                "[rewrite tx_hash] successed to write mint tx hash to hub"
                            );
                        }
                    }
                    mutate_state(|s| s.pending_tickets_map.remove(&seqno));
                    mutate_state(|s| s.finalized_mint_requests.insert(ticket_id, hash));
                }
            }
        }
    }
}

pub fn check_mint_message(message: Message) -> Result<(Seq, String), String> {
    let message_body = message.message_content.body;
    let r = BagOfCells::parse_base64(message_body.as_str()).unwrap();
    let mut r = r.single_root().map_err(|e| e.to_string())?.parser();
    r.skip_bits(32).unwrap(); //skip opcode
    let query_id = r.load_uint(64).unwrap().to_u64().unwrap_or_default();
    let hash = message.hash;
    Ok((query_id, hash))
}
