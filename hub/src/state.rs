use crate::event::{record_event, Event};
use crate::lifecycle::init::InitArgs;
use crate::lifecycle::upgrade::UpgradeArgs;
use crate::memory::{self, Memory};
use crate::types::{Amount, ChainMeta, ChainTokenFactor, Subscribers, TokenKey, TokenMeta};

use candid::Principal;
use ic_stable_structures::writer::Writer;
use ic_stable_structures::{Memory as _, StableBTreeMap};

use log::info;
use omnity_types::{
    ChainId, ChainState, Directive, Error, Factor, Seq, SeqKey, Ticket, TicketId, TicketType,
    ToggleAction, ToggleState, TokenId, Topic, TxAction,
};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashMap;

use std::num::ParseIntError;

const HOUR: u64 = 3600_000_000_000;

thread_local! {
    static STATE: RefCell<Option<HubState>> = RefCell::default();
}

#[derive(Deserialize, Serialize)]
pub struct HubState {
    #[serde(skip, default = "memory::init_chain")]
    pub chains: StableBTreeMap<ChainId, ChainMeta, Memory>,
    #[serde(skip, default = "memory::init_token")]
    pub tokens: StableBTreeMap<TokenId, TokenMeta, Memory>,
    #[serde(skip, default = "memory::init_chain_factor")]
    pub target_chain_factors: StableBTreeMap<ChainId, u128, Memory>,
    #[serde(skip, default = "memory::init_token_factor")]
    pub fee_token_factors: StableBTreeMap<TokenKey, ChainTokenFactor, Memory>,
    #[serde(skip, default = "memory::init_directive")]
    pub directives: StableBTreeMap<String, Directive, Memory>,
    #[serde(skip, default = "memory::init_dire_queue")]
    pub dire_queue: StableBTreeMap<SeqKey, Directive, Memory>,
    #[serde(skip, default = "memory::init_subs")]
    pub topic_subscribers: StableBTreeMap<Topic, Subscribers, Memory>,
    #[serde(skip, default = "memory::init_ticket_queue")]
    pub ticket_queue: StableBTreeMap<SeqKey, Ticket, Memory>,
    #[serde(skip, default = "memory::init_token_position")]
    pub token_position: StableBTreeMap<TokenKey, Amount, Memory>,
    #[serde(skip, default = "memory::init_ledger")]
    pub cross_ledger: StableBTreeMap<TicketId, Ticket, Memory>,

    pub directive_seq: HashMap<String, Seq>,
    pub ticket_seq: HashMap<String, Seq>,
    pub admin: Principal,
    pub authorized_caller: HashMap<String, ChainId>,
    pub last_resubmit_ticket_time: u64,
}

impl From<InitArgs> for HubState {
    fn from(args: InitArgs) -> Self {
        Self {
            chains: StableBTreeMap::init(memory::get_chain_memory()),
            tokens: StableBTreeMap::init(memory::get_token_memory()),
            target_chain_factors: StableBTreeMap::init(memory::get_chain_factor_memory()),
            fee_token_factors: StableBTreeMap::init(memory::get_token_factor_memory()),
            token_position: StableBTreeMap::init(memory::get_token_position_memory()),
            cross_ledger: StableBTreeMap::init(memory::get_ledger_memory()),
            directives: StableBTreeMap::init(memory::get_directive_memory()),
            dire_queue: StableBTreeMap::init(memory::get_dire_queue_memory()),
            topic_subscribers: StableBTreeMap::init(memory::get_subs_memory()),
            ticket_queue: StableBTreeMap::init(memory::get_ticket_queue_memory()),
            directive_seq: HashMap::default(),
            ticket_seq: HashMap::default(),
            admin: args.admin,
            authorized_caller: HashMap::default(),
            last_resubmit_ticket_time: 0,
        }
    }
}

/// A helper method to read the state.
///
/// Precondition: the state is already initialized.
pub fn with_state<R>(f: impl FnOnce(&HubState) -> R) -> R {
    STATE.with(|cell| f(cell.borrow().as_ref().expect("State not initialized!")))
}

/// A helper method to mutate the state.
///
/// Precondition: the state is already initialized.
pub fn with_state_mut<R>(f: impl FnOnce(&mut HubState) -> R) -> R {
    STATE.with(|cell| f(cell.borrow_mut().as_mut().expect("State not initialized!")))
}

// A helper method to set the state.
//
// Precondition: the state is _not_ initialized.
pub fn set_state(state: HubState) {
    STATE.with(|cell| *cell.borrow_mut() = Some(state));
}

/// get settlement chain from token id
impl HubState {
    pub fn pre_upgrade(&self) {
        // Serialize the state.
        let mut state_bytes = vec![];

        let _ = ciborium::ser::into_writer(self, &mut state_bytes);

        // Write the length of the serialized bytes to memory, followed by the
        // by the bytes themselves.
        let len = state_bytes.len() as u32;
        let mut memory = memory::get_upgrades_memory();
        let mut writer = Writer::new(&mut memory, 0);
        writer
            .write(&len.to_le_bytes())
            .expect("failed to save hub state len");
        writer
            .write(&state_bytes)
            .expect("failed to save hub state");
    }

    pub fn post_upgrade(&mut self) {
        let memory = memory::get_upgrades_memory();
        // Read the length of the state bytes.
        let mut state_len_bytes = [0; 4];
        memory.read(0, &mut state_len_bytes);
        let state_len = u32::from_le_bytes(state_len_bytes) as usize;

        // Read the bytes
        let mut state_bytes = vec![0; state_len];
        memory.read(4, &mut state_bytes);

        // Deserialize and set the state.
        let state: HubState =
            ciborium::de::from_reader(&*state_bytes).expect("failed to decode state");
        *self = state;
    }

    pub fn upgrade(&mut self, args: UpgradeArgs) {
        if let Some(admin) = args.admin {
            self.admin = admin;
        }
    }

    pub fn settlement_chain(&self, token_id: &TokenId) -> Result<ChainId, Error> {
        self.tokens
            .get(token_id)
            .map(|v| v.issue_chain.to_string())
            .ok_or(Error::NotFoundToken(token_id.to_string()))
    }
    //Determine whether the token is from the issuing chain
    pub fn is_origin(&self, chain_id: &ChainId, token_id: &TokenId) -> Result<bool, Error> {
        self.tokens
            .get(token_id)
            .map(|v| v.issue_chain.eq(chain_id))
            .ok_or(Error::NotFoundChainToken(
                token_id.to_string(),
                chain_id.to_string(),
            ))
    }
    pub fn add_chain(&mut self, chain: ChainMeta) -> Result<(), Error> {
        // save chain
        self.chains
            .insert(chain.chain_id.to_string(), chain.clone());
        // update auth
        self.authorized_caller
            .insert(chain.canister_id.to_string(), chain.chain_id.to_string());
        record_event(&Event::AddedChain(chain.clone()));

        // update counterparties
        if let Some(counterparties) = chain.counterparties {
            counterparties
                .iter()
                .map(|counterparty| {
                    //check and update counterparty of dst chain
                    self.update_chain_counterparties(&counterparty, &chain.chain_id)
                })
                .collect::<Result<(), Error>>()?;
        }

        Ok(())
    }

    pub fn update_chain_counterparties(
        &mut self,
        dst_chain_id: &ChainId,
        counterparty: &ChainId,
    ) -> Result<(), Error> {
        self.chains.get(dst_chain_id).map(|mut chain| {
            // excluds the deactive state
            if matches!(chain.chain_state, ChainState::Deactive) {
                info!(
                    "dst chain {} is deactive, donn`t update counterparties for it! ",
                    chain.chain_id.to_string()
                );
            } else {
                if let Some(ref mut counterparties) = chain.counterparties {
                    if !counterparties.contains(counterparty) {
                        counterparties.push(counterparty.to_string());
                    }
                } else {
                    chain.counterparties = Some(vec![counterparty.to_string()])
                }
                //update chain info
                self.chains
                    .insert(chain.chain_id.to_string(), chain.clone());
                record_event(&Event::UpdatedChainCounterparties(chain))
            }
        });
        Ok(())
    }

    pub fn chain(&self, chain_id: &ChainId) -> Result<ChainMeta, Error> {
        self.chains
            .get(chain_id)
            .ok_or(Error::NotFoundChain(chain_id.to_string()))
    }

    //check the dst chain must exsiting and not deactive!
    pub fn available_chain(&self, chain_id: &ChainId) -> Result<ChainMeta, Error> {
        self.chains
            .get(chain_id)
            .ok_or(Error::NotFoundChain(chain_id.to_string()))
            .map_or_else(
                |e| Err(e),
                |chain| {
                    if matches!(chain.chain_state, ChainState::Deactive) {
                        Err(Error::DeactiveChain(chain_id.to_string()))
                    } else {
                        Ok(chain)
                    }
                },
            )
    }
    //check the dst chain must exsiting and right state!
    pub fn available_state(&self, toggle_state: &ToggleState) -> Result<(), Error> {
        self.chains
            .get(&toggle_state.chain_id)
            .ok_or(Error::NotFoundChain(toggle_state.chain_id.to_string()))
            .map_or_else(
                |e| Err(e),
                |chain| {
                    //If the state and action are consistent, no need to switch
                    if (matches!(chain.chain_state, ChainState::Active)
                        && matches!(toggle_state.action, ToggleAction::Activate))
                        || (matches!(chain.chain_state, ChainState::Deactive)
                            && matches!(toggle_state.action, ToggleAction::Deactivate))
                    {
                        Err(Error::ProposalError(format!(
                            "The chain({}) don`nt need to switch",
                            toggle_state.chain_id
                        )))
                    } else {
                        Ok(())
                    }
                },
            )
    }

    pub fn update_chain_state(&mut self, toggle_state: &ToggleState) -> Result<(), Error> {
        self.chains
            .get(&toggle_state.chain_id)
            .ok_or(Error::NotFoundChain(toggle_state.chain_id.to_string()))
            .map_or_else(
                |e| Err(e),
                |mut chain| {
                    match toggle_state.action {
                        ToggleAction::Activate => {
                            chain.chain_state = ChainState::Active;
                        }
                        ToggleAction::Deactivate => chain.chain_state = ChainState::Deactive,
                    }
                    self.chains
                        .insert(toggle_state.chain_id.to_string(), chain.clone());
                    record_event(&Event::ToggledChainState {
                        chain,
                        state: toggle_state.clone(),
                    });
                    Ok(())
                },
            )
    }

    pub fn add_token(&mut self, token_meata: TokenMeta) -> Result<(), Error> {
        self.tokens
            .insert(token_meata.token_id.to_string(), token_meata.clone());
        record_event(&Event::AddedToken(token_meata));
        Ok(())
    }

    pub fn token(&self, token_id: &TokenId) -> Result<TokenMeta, Error> {
        self.tokens
            .get(token_id)
            .ok_or(Error::NotFoundToken(token_id.to_string()))
    }

    pub fn update_fee(&mut self, fee: Factor) -> Result<(), Error> {
        match fee {
            Factor::UpdateTargetChainFactor(ref cf) => self
                .chains
                .get(&cf.target_chain_id)
                .ok_or(Error::NotFoundChain(cf.target_chain_id.to_string()))
                .map_or_else(
                    |e| Err(e),
                    |chain| {
                        if matches!(chain.chain_state, ChainState::Deactive) {
                            Err(Error::DeactiveChain(cf.target_chain_id.to_string()))
                        } else {
                            self.target_chain_factors
                                .insert(cf.target_chain_id.to_string(), cf.target_chain_factor);
                            record_event(&Event::UpdatedFee(fee.clone()));

                            Ok(())
                        }
                    },
                ),
            Factor::UpdateFeeTokenFactor(ref tf) => {
                self.target_chain_factors.iter().for_each(|(chain_id, _)| {
                    let token_key = TokenKey::from(chain_id.to_string(), tf.fee_token.to_string());
                    let fee_factor = ChainTokenFactor {
                        dst_chain_id: chain_id.to_string(),
                        fee_token: tf.fee_token.to_string(),
                        fee_token_factor: tf.fee_token_factor,
                    };

                    self.fee_token_factors.insert(token_key, fee_factor);
                    record_event(&Event::UpdatedFee(fee.clone()));
                });
                Ok(())
            }
        }
    }

    pub fn sub_directives(&mut self, chain_id: &ChainId, topics: &Vec<Topic>) -> Result<(), Error> {
        topics.iter().for_each(|topic| {
            if let Some(subscribers) = self.topic_subscribers.get(topic).as_mut() {
                if !subscribers.subs.contains(chain_id) {
                    subscribers.subs.push(chain_id.to_string());
                    //update subscribers
                    self.topic_subscribers
                        .insert(topic.clone(), subscribers.clone());
                    record_event(&Event::SubDirectives {
                        topic: topic.clone(),
                        subs: subscribers.clone(),
                    })
                }
            } else {
                let subs = Subscribers {
                    subs: vec![chain_id.to_string()],
                };
                //update subscribers
                self.topic_subscribers.insert(topic.clone(), subs.clone());
                record_event(&Event::SubDirectives {
                    topic: topic.clone(),
                    subs: subs,
                })
            }
        });
        //publish directives for new subscribers
        self.directives
            .iter()
            .map(|(_, d)| d.clone())
            .collect::<Vec<Directive>>()
            .into_iter()
            .for_each(|d| {
                info!(
                    "publish directives({:?}) for new subscribers: {}",
                    d,
                    chain_id.to_string()
                );
                let _ = self.pub_2_subscribers(&Some(chain_id.to_string()), d);
            });

        Ok(())
    }

    pub fn unsub_directives(
        &mut self,
        chain_id: &ChainId,
        topics: &Vec<Topic>,
    ) -> Result<(), Error> {
        topics.iter().for_each(|topic| {
            if let Some(mut subscribers) = self.topic_subscribers.get(topic) {
                if let Some(idx) = subscribers.subs.iter().position(|dst| dst.eq(chain_id)) {
                    subscribers.subs.remove(idx);
                    self.topic_subscribers.insert(topic.clone(), subscribers);
                    record_event(&Event::UnSubDirectives {
                        topic: topic.clone(),
                        sub: chain_id.to_string(),
                    })
                }
            }
        });

        self.delete_directives(chain_id, topics)
    }

    pub fn delete_directives(
        &mut self,
        chain_id: &ChainId,
        topics: &Vec<Topic>,
    ) -> Result<(), Error> {
        for (seq_key, dir) in self
            .dire_queue
            .iter()
            .filter_map(|(seq_key, dir)| {
                if seq_key.chain_id.eq(chain_id) {
                    Some((seq_key.clone(), dir.clone()))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
        {
            topics.iter().for_each(|topic| match topic {
                Topic::AddChain(chain_type) => {
                    if let Some(chain_type) = chain_type {
                        if matches!(&dir,Directive::AddChain(chain) if chain.chain_type==*chain_type)
                        {
                            self.dire_queue.remove(&seq_key);
                            record_event(&Event::DeletedDirective(seq_key.clone()))
                        }
                        
                    }else {
                        if matches!(&dir,Directive::AddChain(_))
                        {
                            self.dire_queue.remove(&seq_key);
                            record_event(&Event::DeletedDirective(seq_key.clone()))
                        }
                    }
                }
                Topic::AddToken(dst_token_id) =>                 {
                    if let Some(dst_token_id) = dst_token_id {
                        
                        if matches!(&dir,Directive::AddToken(token) if token.token_id.eq(dst_token_id))
                        {
                            self.dire_queue.remove(&seq_key);
                            record_event(&Event::DeletedDirective(seq_key.clone()))
                        }
                        
                    }else {
                        if matches!(&dir,Directive::AddToken(_))
                        {
                            self.dire_queue.remove(&seq_key);
                            record_event(&Event::DeletedDirective(seq_key.clone()))
                        }
                    }
                },
                Topic::UpdateTargetChainFactor(targe_chain_id) =>{
                    if let Some(targe_chain_id) = targe_chain_id {
                        
                        if matches!(&dir,Directive::UpdateFee(factor) if matches!(factor,Factor::UpdateTargetChainFactor(cf) if cf.target_chain_id.eq(targe_chain_id)))
                        {
                            self.dire_queue.remove(&seq_key);
                            record_event(&Event::DeletedDirective(seq_key.clone()))
                        }
                        
                    }else {
                        if matches!(&dir,Directive::UpdateFee(factor) if matches!(factor,Factor::UpdateTargetChainFactor(_)))
                        {
                            self.dire_queue.remove(&seq_key);
                            record_event(&Event::DeletedDirective(seq_key.clone()))
                        }
                    }
                },
                Topic::UpdateFeeTokenFactor(dst_token_id) => {
                    if let Some(dst_token_id) = dst_token_id {
                        
                        if matches!(&dir,Directive::UpdateFee(factor) if matches!(factor,Factor::UpdateFeeTokenFactor(tf) if tf.fee_token.eq(dst_token_id)))
                        {
                            self.dire_queue.remove(&seq_key);
                            record_event(&Event::DeletedDirective(seq_key.clone()))
                        }
                        
                    }else {
                        if matches!(&dir,Directive::UpdateFee(factor) if matches!(factor,Factor::UpdateFeeTokenFactor(_)))
                        {
                            self.dire_queue.remove(&seq_key);
                            record_event(&Event::DeletedDirective(seq_key.clone()))
                        }
                    }
                },
                Topic::ActivateChain => {
                    if matches!(&dir,Directive::ToggleChainState(toggle_state) if matches!(toggle_state.action,ToggleAction::Activate))
                    {
                        self.dire_queue.remove(&seq_key);
                        record_event(&Event::DeletedDirective(seq_key.clone()))
                    }
                }
                Topic::DeactivateChain => {
                    if matches!(&dir,Directive::ToggleChainState(toggle_state) if matches!(toggle_state.action,ToggleAction::Deactivate))
                    {
                        self.dire_queue.remove(&seq_key);
                        record_event(&Event::DeletedDirective(seq_key.clone()))
                    }
                }
            });

        }

        Ok(())
    }

    pub fn query_subscribers(
        &self,
        dst_topic: Option<Topic>,
    ) -> Result<Vec<(Topic, Subscribers)>, Error> {
        let ret = self
            .topic_subscribers
            .iter()
            .filter(|(topic, _)| {
                dst_topic
                    .as_ref()
                    .map_or(true, |dst_topic| topic == dst_topic)
            })
            .map(|(topic, subs)| (topic, subs))
            .collect::<Vec<_>>();
        Ok(ret)
    }
    pub fn pub_directive(&mut self, dire: &Directive) -> Result<(), Error> {
        // save directive
        self.save_directive(&dire)?;
        //publish directive to subscribers
        self.pub_2_subscribers(&None, dire.clone())
    }

    pub fn save_directive(&mut self, dire: &Directive) -> Result<(), Error> {
        self.directives.insert(format!("{}", dire), dire.clone());
        record_event(&&Event::SavedDirective(dire.clone()));
        Ok(())
    }

    pub fn pub_2_subscribers(
        &mut self,
        target_sub: &Option<String>,
        dire: Directive,
    ) -> Result<(), Error> {
        fn pub_2_targets<'a>(
            hub_state: &'a mut HubState,
            target_sub: &Option<String>,
            condition: impl Fn(&Topic) -> bool,
            dire: &'a Directive,
        ) -> Result<(), Error> {
            hub_state
                .topic_subscribers
                .iter()
                .filter_map(|(topic, subs)| {
                    if condition(&topic) {
                        Some((topic, subs))
                    } else {
                        None
                    }
                })
                .flat_map(|(_, subs)| subs.subs)
                .map(|sub| {
                    let sub = target_sub.clone().map_or(sub, |targe_sub| targe_sub);

                    //repeatability detection
                    if hub_state
                        .dire_queue
                        .iter()
                        .find(|(seq_key, directive)| seq_key.chain_id.eq(&sub) && directive == dire)
                        .is_none()
                    {
                        let latest_seq = hub_state
                            .directive_seq
                            .entry(sub.to_string())
                            .and_modify(|seq| *seq += 1)
                            .or_insert(0);

                        let seq_key = SeqKey::from(sub.to_string(), *latest_seq);
                        hub_state.dire_queue.insert(seq_key.clone(), dire.clone());
                        info!("pub_2_targets:{:?}, directive:{:?}", sub.to_string(), dire);
                        record_event(&Event::PubedDirective {
                            seq_key,
                            dire: dire.clone(),
                        });
                    }

                    Ok(())
                })
                .collect::<Result<Vec<()>, _>>()?;
            Ok(())
        }

        match dire {
            Directive::AddChain(ref chain) => pub_2_targets(
                self,
                target_sub,
                |topic: &Topic| {
                    matches!(topic, Topic::AddChain(None))
                        || matches!(topic, Topic::AddChain(Some(chain_type)) if *chain_type == chain.chain_type)
                },
                &dire,
            ),
            Directive::AddToken(ref token) => pub_2_targets(
                self,
                target_sub,
                |topic: &Topic| {
                    matches!(topic, Topic::AddToken(None))
                        || matches!(topic, Topic::AddToken(Some(token_id)) if *token_id == token.token_id)
                },
                &dire,
            ),
            Directive::ToggleChainState(ref toggle_state) => pub_2_targets(
                self,
                target_sub,
                |topic: &Topic| match toggle_state.action {
                    ToggleAction::Activate => matches!(topic, Topic::ActivateChain),
                    ToggleAction::Deactivate => matches!(topic, Topic::DeactivateChain),
                },
                &dire,
            ),
            Directive::UpdateFee(ref factor) => pub_2_targets(
                self,
                target_sub,
                |topic: &Topic| match factor {
                    Factor::UpdateTargetChainFactor(cf) => {
                        matches!(topic, Topic::UpdateTargetChainFactor(None))
                            || matches!(topic, Topic::UpdateTargetChainFactor(Some(chain_id)) if cf.target_chain_id.eq(chain_id))
                    }
                    Factor::UpdateFeeTokenFactor(tf) => {
                        matches!(topic, Topic::UpdateFeeTokenFactor(None))
                            || matches!(topic, Topic::UpdateFeeTokenFactor(Some(token_id)) if tf.fee_token.eq(token_id))
                    }
                },
                &dire,
            ),
        }
    }

    pub fn pull_directives(
        &self,
        chain_id: ChainId,
        topic: Option<Topic>,
        offset: usize,
        limit: usize,
    ) -> Result<Vec<(Seq, Directive)>, Error> {
        fn filter_dires<F>(
            dire_queue: &StableBTreeMap<SeqKey, Directive, Memory>,
            chain_id: &ChainId,
            offset: usize,
            limit: usize,
            predicate: F,
        ) -> Result<Vec<(Seq, Directive)>, Error>
        where
            F: Fn(&Directive) -> bool,
        {
            Ok(dire_queue
                .iter()
                .filter(|(seq_key, _)| seq_key.chain_id.eq(chain_id))
                .filter(|(_, dire)| predicate(dire))
                .skip(offset)
                .take(limit)
                .map(|(seq_key, dire)| (seq_key.seq, dire.clone()))
                .collect::<Vec<_>>())
        }

        match topic {
            None => Ok(self
                .dire_queue
                .iter()
                .filter(|(seq_key, _)| seq_key.chain_id.eq(&chain_id))
                .skip(offset)
                .take(limit)
                .map(|(seq_key, dire)| (seq_key.seq, dire.clone()))
                .collect::<Vec<_>>()),
            Some(topic) => match topic {
                Topic::AddChain(chain_type) => {
                    filter_dires(&self.dire_queue, &chain_id, offset, limit, |dire| {
                        if let Some(dst_chain_type) = &chain_type {
                            matches!(dire, Directive::AddChain(chain_info) if chain_info.chain_type == *dst_chain_type)
                        } else {
                            matches!(dire, Directive::AddChain(_))
                        }
                    })
                }
                Topic::AddToken(token_id) => {
                    filter_dires(&self.dire_queue, &chain_id, offset, limit, |dire| {
                        if let Some(dst_token_id) = &token_id {
                            matches!(dire, Directive::AddToken(token_meta) if token_meta.token_id.eq(dst_token_id))
                        } else {
                            matches!(dire, Directive::AddToken(_))
                        }
                    })
                }
                Topic::UpdateTargetChainFactor(dst_chain_id) => {
                    filter_dires(&self.dire_queue, &chain_id, offset, limit, |dire| {
                        if let Some(dst_token_id) = &dst_chain_id {
                            matches!(dire, Directive::UpdateFee(factor) if  matches!(factor,Factor::UpdateTargetChainFactor(cf) if cf.target_chain_id.eq(dst_token_id)))
                        } else {
                            matches!(dire, Directive::UpdateFee(factor) if  matches!(factor,Factor::UpdateTargetChainFactor(_)))
                        }
                    })
                }
                Topic::UpdateFeeTokenFactor(dst_token_id) => {
                    filter_dires(&self.dire_queue, &chain_id, offset, limit, |dire| {
                        if let Some(dst_token_id) = &dst_token_id {
                            matches!(dire, Directive::UpdateFee(factor) if  matches!(factor,Factor::UpdateFeeTokenFactor(tf) if tf.fee_token.eq(dst_token_id)))
                        } else {
                            matches!(dire, Directive::UpdateFee(factor) if  matches!(factor,Factor::UpdateFeeTokenFactor(_)))
                        }
                    })
                }
                Topic::ActivateChain => filter_dires(
                    &self.dire_queue,
                    &chain_id,
                    offset,
                    limit,
                    |dire| matches!(dire, Directive::ToggleChainState(toggle_state) if toggle_state.action == ToggleAction::Activate),
                ),
                Topic::DeactivateChain => {
                    info!(
                        "query  'Topic::DeactivateChain' directives for chain: {}",
                        chain_id
                    );
                    filter_dires(
                        &self.dire_queue,
                        &chain_id,
                        offset,
                        limit,
                        |dire| matches!(dire, Directive::ToggleChainState(toggle_state) if toggle_state.action == ToggleAction::Deactivate),
                    )
                }
            },
        }
    }

    pub fn add_token_position(&mut self, position: TokenKey, amount: u128) -> Result<(), Error> {
        let amount = if let Some(total_amount) = self.token_position.get(&position).as_mut() {
            *total_amount += amount;
            *total_amount
        } else {
            amount
        };
        self.token_position.insert(position.clone(), amount);
        record_event(&Event::AddedTokenPosition {
            position: position,
            amount: amount,
        });

        Ok(())
    }

    pub fn update_token_position(
        &mut self,
        position: TokenKey,
        f: impl FnOnce(&mut u128) -> Result<u128, Error>,
    ) -> Result<(), Error> {
        self.token_position
            .get(&position)
            .as_mut()
            .ok_or(Error::NotFoundChainToken(
                position.token_id.to_string(),
                position.chain_id.to_string(),
            ))
            .map_or_else(
                |e| Err(e),
                |total_amount| {
                    let total_amount = f(total_amount)?;
                    self.token_position.insert(position.clone(), total_amount);
                    record_event(&Event::UpdatedTokenPosition {
                        position: position,
                        amount: total_amount,
                    });
                    Ok(())
                },
            )
    }

    // check the ticket availability
    pub fn check_and_update(&mut self, ticket: &Ticket) -> Result<(), Error> {
        // check ticket id repetitive
        if self.cross_ledger.contains_key(&ticket.ticket_id) {
            return Err(Error::AlreadyExistingTicketId(ticket.ticket_id.to_string()));
        }
        // check chain and state
        self.available_chain(&ticket.src_chain)?;
        self.available_chain(&ticket.dst_chain)?;

        //parse ticket token amount to unsigned bigint
        let ticket_amount: u128 = ticket.amount.parse().map_err(|e: ParseIntError| {
            Error::TicketAmountParseError(ticket.amount.to_string(), e.to_string())
        })?;

        // check token on chain availability

        match ticket.action {
            TxAction::Transfer => {
                // ticket from issue chain
                if self.is_origin(&ticket.src_chain, &ticket.token)? {
                    info!(
                        "ticket token({}) from issue chain({}).",
                        ticket.token, ticket.src_chain,
                    );

                    // just update token amount on dst chain
                    self.add_token_position(
                        TokenKey::from(ticket.dst_chain.to_string(), ticket.token.to_string()),
                        ticket_amount,
                    )?;

                // not from issue chain
                } else {
                    info!(
                        "ticket token({}) from a not issue chain({}).",
                        ticket.token, ticket.src_chain,
                    );

                    // update token amount on src chain
                    self.update_token_position(
                        TokenKey::from(ticket.src_chain.to_string(), ticket.token.to_string()),
                        |total_amount| {
                            // check src chain token balance
                            if *total_amount < ticket_amount {
                                return Err::<u128, omnity_types::Error>(
                                    Error::NotSufficientTokens(
                                        ticket.token.to_string(),
                                        ticket.src_chain.to_string(),
                                    ),
                                );
                            }
                            *total_amount -= ticket_amount;
                            Ok(*total_amount)
                        },
                    )?;
                    // update token amount on dst chain
                    self.add_token_position(
                        TokenKey::from(ticket.dst_chain.to_string(), ticket.token.to_string()),
                        ticket_amount,
                    )?;
                }
            }

            TxAction::Redeem => {
                // update token amount on src chain
                self.update_token_position(
                    TokenKey::from(ticket.src_chain.to_string(), ticket.token.to_string()),
                    |total_amount| {
                        // check src chain token balance
                        if *total_amount < ticket_amount {
                            return Err::<u128, omnity_types::Error>(Error::NotSufficientTokens(
                                ticket.token.to_string(),
                                ticket.src_chain.to_string(),
                            ));
                        }
                        *total_amount -= ticket_amount;
                        Ok(*total_amount)
                    },
                )?;

                //  if the dst chain is not issue chain,then update token amount on dst chain
                if !self.is_origin(&ticket.dst_chain, &ticket.token)? {
                    self.update_token_position(
                        TokenKey::from(ticket.dst_chain.to_string(), ticket.token.to_string()),
                        |total_amount| {
                            *total_amount += ticket_amount;
                            Ok(*total_amount)
                        },
                    )?;
                }
            }
        }

        Ok(())
    }

    pub fn push_ticket(&mut self, ticket: Ticket) -> Result<(), Error> {
        // get latest ticket seq
        let latest_seq = self
            .ticket_seq
            .entry(ticket.dst_chain.to_string())
            .and_modify(|seq| *seq += 1)
            .or_insert(0);

        // add new ticket
        let seq_key = SeqKey::from(ticket.dst_chain.to_string(), *latest_seq);
        self.ticket_queue.insert(seq_key.clone(), ticket.clone());
        //save ticket
        self.cross_ledger
            .insert(ticket.ticket_id.to_string(), ticket.clone());
        record_event(&Event::ReceivedTicket {
            seq_key,
            ticket: ticket.clone(),
        });
        Ok(())
    }

    pub fn resubmit_ticket(&mut self, ticket: Ticket) -> Result<(), Error> {
        let now = ic_cdk::api::time();
        if now - self.last_resubmit_ticket_time < 6 * HOUR {
            return Err(Error::ResubmitTicketSentTooOften);
        }
        match self.cross_ledger.get(&ticket.ticket_id) {
            Some(old_ticket) => {
                if ticket != old_ticket {
                    return Err(Error::ResubmitTicketMustSame);
                }
                let ticket_id = format!("{}_{}", ticket.ticket_id, now);
                let new_ticket = Ticket {
                    ticket_id: ticket_id.clone(),
                    ticket_type: TicketType::Resubmit,
                    ticket_time: now,
                    src_chain: ticket.src_chain,
                    dst_chain: ticket.dst_chain,
                    action: ticket.action,
                    token: ticket.token,
                    amount: ticket.amount,
                    sender: ticket.sender,
                    receiver: ticket.receiver,
                    memo: ticket.memo,
                };
                self.push_ticket(new_ticket)?;
                self.last_resubmit_ticket_time = now;

                record_event(&Event::ResubmitTicket {
                    ticket_id,
                    timestamp: now,
                });
                Ok(())
            }
            None => Err(Error::ResubmitTicketIdMustExist),
        }
    }

    pub fn pull_tickets(
        &self,
        chain_id: &ChainId,
        offset: usize,
        limit: usize,
    ) -> Result<Vec<(Seq, Ticket)>, Error> {
        // let end = from + num;

        let tickets = self
            .ticket_queue
            .iter()
            .filter(|(seq_key, _)| seq_key.chain_id.eq(chain_id))
            .skip(offset)
            .take(limit)
            .map(|(tk, ticket)| (tk.seq, ticket.clone()))
            .collect();
        info!("query_tickets result : {:?}", tickets);
        Ok(tickets)
    }
}
