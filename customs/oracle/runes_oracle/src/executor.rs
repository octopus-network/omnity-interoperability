use crate::{client::Client, indexer::Indexer};
use bitcoin_customs::{
    state::{self, GenTicketRequest},
    updates::update_runes_balance::UpdateRunesBalanceError,
};
use candid::Principal;
use ic_btc_interface::Txid;
use log;
use omnity_hub::self_help::{AddRunesTokenReq, FinalizeAddRunesArgs};
use omnity_types::rune_id::RuneId;
use std::{
    collections::{BTreeSet, VecDeque},
    str::FromStr,
    time::Duration,
};
use ticker::Ticker;

pub struct Executor {
    client: Client,
    indexer: Indexer,
    customs: Principal,
    hub: Principal,

    invalid_gen_ticket_requests: BTreeSet<Txid>,
    pending_gen_ticket_requests: VecDeque<GenTicketRequest>,
    pending_add_runes_requests: VecDeque<AddRunesTokenReq>,
}

impl Executor {
    pub fn new(client: Client, indexer: Indexer, customs: Principal, hub: Principal) -> Self {
        Self {
            client,
            indexer,
            customs,
            hub,
            invalid_gen_ticket_requests: Default::default(),
            pending_gen_ticket_requests: Default::default(),
            pending_add_runes_requests: Default::default(),
        }
    }

    pub async fn start(&mut self) {
        let ticker = Ticker::new(0.., Duration::from_secs(60));
        for _ in ticker {
            self.update_runes_balance().await;
            self.finalize_add_runes_token().await;
        }
    }

    async fn finalize_add_runes_token(&mut self) {
        if self.pending_add_runes_requests.is_empty() {
            match self.client.get_add_runes_token_requests(&self.hub).await {
                Ok(requests) => requests
                    .iter()
                    .for_each(|r| self.pending_add_runes_requests.push_back(r.clone())),
                Err(err) => {
                    log::error!("failed to get pending add runes requests: {}", err);
                    return;
                }
            }
        }
        while !self.pending_add_runes_requests.is_empty() {
            let request = self.pending_add_runes_requests.front().unwrap();
            match self.indexer.get_runes(&request.rune_id).await {
                Ok(runes) => {
                    let finalize_args = FinalizeAddRunesArgs {
                        rune_id: request.rune_id.clone(),
                        name: runes.spaced_rune.clone(),
                        decimal: runes.divisibility,
                    };
                    match self
                        .client
                        .finalize_add_runes_token_req(&self.hub, finalize_args.clone())
                        .await
                    {
                        Ok(result) => match result {
                            Ok(()) => {
                                log::info!(
                                    "finalize add runes token success: {}",
                                    finalize_args.rune_id
                                );
                            }
                            Err(err) => {
                                log::error!(
                                    "failed to finalize add runes token: {}, {:?}",
                                    finalize_args.rune_id,
                                    err
                                );
                            }
                        },
                        Err(err) => {
                            log::error!("failed to send finalize add runes to hub: {}", err);
                            break;
                        }
                    }
                }
                Err(err) => {
                    log::error!("failed to get runes from indexer: {:?}", err);
                    break;
                }
            }
            self.pending_add_runes_requests.pop_front();
        }
    }

    async fn update_runes_balance(&mut self) {
        if self.pending_gen_ticket_requests.is_empty() {
            match self
                .client
                .get_pending_gen_ticket_requests(&self.customs, None, 50)
                .await
            {
                Ok(requests) => requests
                    .iter()
                    .for_each(|r| self.pending_gen_ticket_requests.push_back(r.clone())),
                Err(err) => {
                    log::error!("failed to get pending gen ticket requests: {}", err);
                    return;
                }
            }
        }
        while !self.pending_gen_ticket_requests.is_empty() {
            let request = self.pending_gen_ticket_requests.front().unwrap();
            if self.invalid_gen_ticket_requests.contains(&request.txid) {
                self.pending_gen_ticket_requests.pop_front();
                continue;
            }

            match self.indexer.get_transaction(request.txid).await {
                Ok(tx) => {
                    let mut balances = tx.get_runes_balances();
                    balances.retain(|b| {
                        b.address == request.address && b.rune_id == request.rune_id.to_string()
                    });

                    match self
                        .client
                        .update_runes_balance(
                            &self.customs,
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
                                self.invalid_gen_ticket_requests.insert(request.txid);
                                log::error!(
                                    "mismatch with ticket request for txid:{}",
                                    request.txid
                                );
                            }
                            Err(UpdateRunesBalanceError::UtxoNotFound)
                            | Err(UpdateRunesBalanceError::RequestNotConfirmed) => {
                                // Should never happen.
                                log::error!("utxo not found for txid:{}", request.txid);
                            }
                            Err(UpdateRunesBalanceError::BalancesIsEmpty) => {
                                // Should never happen.
                                log::error!("balances is empty for txid:{}", request.txid);
                            }
                            Err(UpdateRunesBalanceError::FinalizeTicketErr(err)) => {
                                log::error!("finalize ticket err({}) for txid:{}", err, request.txid);
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
            self.pending_gen_ticket_requests.pop_front();
        }
    }
}
