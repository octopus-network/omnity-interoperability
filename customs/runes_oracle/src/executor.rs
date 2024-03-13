use crate::{customs::Customs, indexer::Indexer};
use bitcoin_customs::{
    state::{self, GenTicketRequest, RuneId},
    updates::update_runes_balance::UpdateRunesBalanceError,
};
use log;
use std::{collections::VecDeque, str::FromStr, time::Duration};
use ticker::Ticker;

pub struct Executor {
    customs: Customs,
    indexer: Indexer,
    pending_requests: VecDeque<GenTicketRequest>,
}

impl Executor {
    pub fn new(customs: Customs, indexer: Indexer) -> Self {
        Self {
            customs,
            indexer,
            pending_requests: Default::default(),
        }
    }

    pub async fn start(&mut self) {
        let ticker = Ticker::new(0.., Duration::from_secs(60));
        for _ in ticker {
            if self.pending_requests.is_empty() {
                match self.customs.get_pending_gen_ticket_requests(None, 50).await {
                    Ok(requests) => requests
                        .iter()
                        .for_each(|r| self.pending_requests.push_back(r.clone())),
                    Err(err) => {
                        log::error!("failed to get pending requests: {}", err);
                        continue;
                    }
                }
            }
            while !self.pending_requests.is_empty() {
                let request = self.pending_requests.front().unwrap();

                match self.indexer.get_transaction(request.txid).await {
                    Ok(tx) => {
                        let mut balances = tx.get_runes_balances();
                        balances.retain(|b| {
                            b.address == request.address && b.rune_id == request.rune_id.to_string()
                        });

                        match self
                            .customs
                            .update_runes_balance(
                                request.txid,
                                balances
                                    .iter()
                                    .map(|b| {
                                        let rune_id = RuneId::from_str(&b.rune_id).unwrap();
                                        state::RunesBalance {
                                            rune_id,
                                            vout: b.vout,
                                            amount: b.amount,
                                        }
                                    })
                                    .collect(),
                            )
                            .await
                        {
                            Ok(result) => match result {
                                Ok(()) => {
                                    log::info!(
                                        "update runes balance success for txid:{}",
                                        request.txid
                                    );
                                }
                                Err(UpdateRunesBalanceError::AleardyProcessed) => {}
                                Err(UpdateRunesBalanceError::RequestNotFound) => {
                                    // Should never happen.
                                    log::error!("request not found for txid:{}", request.txid);
                                }
                                Err(UpdateRunesBalanceError::MismatchWithGenTicketReq) => {
                                    // Customs will remove the pending request.
                                    log::error!(
                                        "mismatch with ticket request for txid:{}",
                                        request.txid
                                    );
                                }
                                Err(UpdateRunesBalanceError::UtxoNotFound) => {
                                    // Should never happen.
                                    log::error!("utxo not found for txid:{}", request.txid);
                                }
                                Err(UpdateRunesBalanceError::SendTicketErr(err)) => {
                                    log::error!(
                                        "send ticket err({}) for txid:{}",
                                        err,
                                        request.txid
                                    );
                                }
                            },
                            Err(err) => {
                                log::error!("failed to update runes balance: {}", err);
                                break;
                            }
                        }
                    }
                    Err(err) => {
                        log::error!("failed to get transaction from indexer: {:?}", err);
                        break;
                    }
                }
                self.pending_requests.pop_front();
            }
        }
    }
}
