use crate::auth::Permission;
use crate::event::{record_event, Event};
use crate::lifecycle::init::{HubArg, InitArgs};
use crate::lifecycle::upgrade::UpgradeArgs;
use crate::memory::{self, Memory};
use crate::metrics::with_metrics_mut;
use crate::migration::{migrate, PreHubState};
use crate::types::{Amount, ChainMeta, ChainTokenFactor, Subscribers, TokenKey, TokenMeta};
use candid::Principal;
use ic_stable_structures::writer::Writer;
use ic_stable_structures::{Memory as _, StableBTreeMap};
use log::{debug, error};
use omnity_types::{
    ChainId, ChainState, Directive, Error, Factor, Seq, SeqKey, Ticket, TicketId, TicketType,
    ToggleAction, ToggleState, TokenId, Topic, TxAction,
};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::{BTreeSet, HashMap};
use std::num::ParseIntError;
const HOUR: u64 = 3_600_000_000_000;

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
    pub caller_chain_map: HashMap<String, ChainId>,
    pub caller_perms: HashMap<String, Vec<Permission>>,
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
            caller_chain_map: HashMap::default(),
            caller_perms: HashMap::from([(
                args.admin.to_string(),
                vec![Permission::Update, Permission::Query],
            )]),
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

    pub fn post_upgrade(args: Option<HubArg>) {
        let memory = memory::get_upgrades_memory();
        // Read the length of the state bytes.
        let mut state_len_bytes = [0; 4];
        memory.read(0, &mut state_len_bytes);
        let state_len = u32::from_le_bytes(state_len_bytes) as usize;

        // Read the bytes
        let mut state_bytes = vec![0; state_len];
        memory.read(4, &mut state_bytes);

        // Deserialize pre state
        let pre_state: PreHubState =
            ciborium::de::from_reader(&*state_bytes).expect("failed to decode state");

        // migrate pre state to current state
        let mut cur_state = migrate(pre_state);

        if let Some(args) = args {
            match args {
                HubArg::Upgrade(upgrade_args) => {
                    if let Some(args) = upgrade_args {
                        if let Some(admin) = args.admin {
                            cur_state.caller_perms.insert(
                                admin.to_string(),
                                vec![Permission::Update, Permission::Query],
                            );
                            cur_state.admin = admin;
                        }
                        record_event(&Event::Upgrade(args));
                    }
                }
                HubArg::Init(_) => panic!("expected Option<UpgradeArgs> got InitArgs."),
            };
        }

        set_state(cur_state);
    }

    pub fn upgrade(&mut self, args: UpgradeArgs) {
        if let Some(admin) = args.admin {
            self.caller_perms.insert(
                admin.to_string(),
                vec![Permission::Update, Permission::Query],
            );
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

    pub fn issue_chain(&self, token_id: &TokenId) -> Result<String, Error> {
        self.tokens
            .get(token_id)
            .map(|v| v.issue_chain)
            .ok_or(Error::NotFoundToken(token_id.to_string()))
    }

    pub fn update_chain(&mut self, chain: ChainMeta) -> Result<(), Error> {
        // save chain
        self.chains
            .insert(chain.chain_id.to_string(), chain.clone());
        // update auth
        self.caller_perms.insert(
            chain.canister_id.to_string(),
            vec![Permission::Update, Permission::Query],
        );
        self.caller_chain_map
            .insert(chain.canister_id.to_string(), chain.chain_id.to_string());

        record_event(&Event::UpdatedChain(chain.clone()));
        // update counterparties
        if let Some(counterparties) = chain.counterparties {
            counterparties.iter().try_for_each(|counterparty| {
                //check and update counterparty of dst chain
                self.update_chain_counterparties(counterparty, &chain.chain_id)
            })?;
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
                debug!(
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
                record_event(&Event::UpdatedChainCounterparties(chain.clone()));
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
                        chain: chain.clone(),
                        state: toggle_state.clone(),
                    });

                    Ok(())
                },
            )
    }

    pub fn update_token(&mut self, token_meata: TokenMeta) -> Result<(), Error> {
        self.tokens
            .insert(token_meata.token_id.to_string(), token_meata.clone());
        record_event(&Event::AddedToken(token_meata.clone()));

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
                            error!("The `{}` is deactive", cf.target_chain_id.to_string());
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
                        target_chain_id: chain_id.to_string(),
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

    pub fn sub_directives(&mut self, chain_id: &ChainId, topics: &[Topic]) -> Result<(), Error> {
        topics.iter().try_for_each(|topic| {
            let mut subscribers = self.topic_subscribers.get(topic).unwrap_or_default();
            // check: repeat subscription
            if subscribers.subs.contains(chain_id) {
                Err(Error::RepeatSubscription(topic.to_string()))
            } else {
                subscribers.subs.insert(chain_id.to_string());

                //update subscribers
                self.topic_subscribers
                    .insert(topic.clone(), subscribers.clone());
                record_event(&Event::SubDirectives {
                    topic: topic.clone(),
                    subs: subscribers.clone(),
                });
                Ok(())
            }
        })?;

        Ok(())
    }

    pub fn unsub_directives(&mut self, chain_id: &ChainId, topics: &[Topic]) -> Result<(), Error> {
        topics.iter().for_each(|topic| {
            if let Some(mut subscribers) = self.topic_subscribers.get(topic) {
                if subscribers.subs.remove(chain_id) {
                    self.topic_subscribers.insert(topic.clone(), subscribers);
                    record_event(&Event::UnSubDirectives {
                        topic: topic.clone(),
                        sub: chain_id.to_string(),
                    })
                }
            }
        });

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
            .collect::<Vec<_>>();
        Ok(ret)
    }

    /// Broadcast to the subscribers if `target_subs` is none,
    /// otherwise multicast to target_subs.
    pub fn pub_directive(
        &mut self,
        target_subs: Option<Vec<ChainId>>,
        dire: &Directive,
    ) -> Result<(), Error> {
        // save directive
        self.save_directive(dire)?;
        //publish directive to subscribers
        self.pub_2_subscribers(target_subs, dire.clone())
    }

    pub fn save_directive(&mut self, dire: &Directive) -> Result<(), Error> {
        self.directives.insert(dire.hash(), dire.clone());
        record_event(&&Event::SavedDirective(dire.clone()));

        Ok(())
    }

    pub fn pub_2_subscribers(
        &mut self,
        target_subs: Option<Vec<ChainId>>,
        dire: Directive,
    ) -> Result<(), Error> {
        let topic_subs = self
            .topic_subscribers
            .get(&dire.to_topic())
            .unwrap_or_default()
            .subs;

        let subs = if let Some(target_subs) = target_subs {
            let mut subs = BTreeSet::new();
            target_subs
                .iter()
                .filter(|sub| topic_subs.contains(*sub))
                .for_each(|sub| {
                    subs.insert(sub.clone());
                });
            subs
        } else {
            topic_subs
        };

        subs.iter().for_each(|sub| {
            if !self
                .dire_queue
                .iter()
                .any(|(seq_key, directive)| seq_key.chain_id.eq(sub) && directive == dire)
            {
                let latest_dire_seq = self
                    .directive_seq
                    .entry(sub.to_string())
                    .and_modify(|seq| *seq += 1)
                    .or_insert(0);

                let seq_key = SeqKey::from(sub.to_string(), *latest_dire_seq);
                //TODO: match! and exclude diretive for  target chain self
                self.dire_queue.insert(seq_key.clone(), dire.clone());
                debug!("pub_2_targets:{:?}, directive:{:?}", sub.to_string(), dire);
                record_event(&Event::PubedDirective {
                    seq_key,
                    dire: dire.clone(),
                });
            }
        });
        Ok(())
    }

    pub fn pull_directives(
        &self,
        chain_id: ChainId,
        topic: Option<Topic>,
        offset: usize,
        limit: usize,
    ) -> Result<Vec<(Seq, Directive)>, Error> {
        Ok(self
            .dire_queue
            .iter()
            .filter(|(seq_key, _)| seq_key.chain_id.eq(&chain_id))
            .filter(|(_, dire)| topic.clone().map_or(true, |t| dire.to_topic() == t))
            .skip(offset)
            .take(limit)
            .map(|(seq_key, dire)| (seq_key.seq, dire.clone()))
            .collect::<Vec<_>>())
    }

    pub fn add_token_position(&mut self, position: TokenKey, amount: u128) -> Result<(), Error> {
        let amount = if let Some(total_amount) = self.token_position.get(&position).as_mut() {
            *total_amount += amount;
            *total_amount
        } else {
            amount
        };
        self.token_position.insert(position.clone(), amount);
        record_event(&Event::AddedTokenPosition { position, amount });

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
            .ok_or({
                Error::NotFoundChainToken(
                    position.token_id.to_string(),
                    position.chain_id.to_string(),
                )
            })
            .map_or_else(
                |e| Err(e),
                |total_amount| {
                    let total_amount = f(total_amount)?;
                    self.token_position.insert(position.clone(), total_amount);
                    record_event(&Event::UpdatedTokenPosition {
                        position,
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
            error!(
                "The ticket id (`{}`) already exists!`",
                ticket.ticket_id.to_string()
            );
            return Err(Error::AlreadyExistingTicketId(ticket.ticket_id.to_string()));
        }

        // check chain and state
        self.available_chain(&ticket.src_chain)?;
        self.available_chain(&ticket.dst_chain)?;

        //parse ticket token amount to unsigned bigint
        let ticket_amount: u128 = ticket.amount.parse().map_err(|e: ParseIntError| {
            error!(
                "The ticket amount(`{}`) parse error: `{}`",
                ticket.amount.to_string(),
                e.to_string()
            );
            Error::TicketAmountParseError(ticket.amount.to_string(), e.to_string())
        })?;

        // check token on chain availability
        match ticket.action {
            TxAction::Transfer => {
                // ticket from issue chain
                if self.is_origin(&ticket.src_chain, &ticket.token)? {
                    debug!(
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
                    debug!(
                        "ticket token({}) from a not issue chain({}).",
                        ticket.token, ticket.src_chain,
                    );

                    // esure dst chain != token`s issue chain
                    if self.is_origin(&ticket.dst_chain, &ticket.token)? {
                        error!(
                            "For a transfer ticket, the dst chain cannot be the token`s issue chain",
                        );
                        return Err(Error::CustomError("For a transfer ticket, the dst chain cannot be the token`s issue chain".to_string()));
                    }

                    // update token amount on src chain
                    self.update_token_position(
                        TokenKey::from(ticket.src_chain.to_string(), ticket.token.to_string()),
                        |total_amount| {
                            // check src chain token balance
                            if *total_amount < ticket_amount {
                                error!(
                                    "Insufficient token (`{}`) on chain (`{}`) !)",
                                    ticket.token.to_string(),
                                    ticket.src_chain.to_string(),
                                );
                                return Err::<u128, Error>(Error::NotSufficientTokens(
                                    ticket.token.to_string(),
                                    ticket.src_chain.to_string(),
                                ));
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
                            error!(
                                "Insufficient token (`{}`) on chain (`{}`) !)",
                                ticket.token.to_string(),
                                ticket.src_chain.to_string(),
                            );
                            return Err::<u128, Error>(Error::NotSufficientTokens(
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
        let latest_ticket_seq = self
            .ticket_seq
            .entry(ticket.dst_chain.to_string())
            .and_modify(|seq| *seq += 1)
            .or_insert(0);

        // add new ticket
        let seq_key = SeqKey::from(ticket.dst_chain.to_string(), *latest_ticket_seq);
        self.ticket_queue.insert(seq_key.clone(), ticket.clone());
        //save ticket
        self.cross_ledger
            .insert(ticket.ticket_id.to_string(), ticket.clone());
        //update ticket metrice
        with_metrics_mut(|metrics| metrics.update_ticket_metric(ticket.clone()));
        record_event(&Event::ReceivedTicket {
            seq_key,
            ticket: ticket.clone(),
        });
        Ok(())
    }

    pub fn resubmit_ticket(&mut self, ticket: Ticket) -> Result<(), Error> {
        let now = ic_cdk::api::time();
        if now - self.last_resubmit_ticket_time < 6 * HOUR {
            error!("The resumit ticket sent too often");
            return Err(Error::ResubmitTicketSentTooOften);
        }
        match self.cross_ledger.get(&ticket.ticket_id) {
            Some(old_ticket) => {
                if ticket != old_ticket {
                    error!("The resubmit ticket must same as the old ticket!");
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
            None => {
                error!("The resubmit ticket id must exist!");
                Err(Error::ResubmitTicketIdMustExist)
            }
        }
    }

    pub fn pull_tickets(
        &self,
        chain_id: &ChainId,
        offset: usize,
        limit: usize,
    ) -> Result<Vec<(Seq, Ticket)>, Error> {
        debug!("pull_tickets: {:?},{offset},{limit}", chain_id);
        let tickets = self
            .ticket_queue
            .iter()
            .filter(|(seq_key, _)| seq_key.chain_id.eq(chain_id))
            .skip(offset)
            .take(limit)
            .map(|(tk, ticket)| (tk.seq, ticket.clone()))
            .collect();
        Ok(tickets)
    }

    pub fn repub_2_subscriber(
        &mut self,
        chain_id: &ChainId,
        topics: &Option<Vec<Topic>>,
    ) -> Result<(), Error> {
        // find the directives that need to repub
        let target_dires = if let Some(topics) = topics {
            let mut dires = Vec::new();
            topics.iter().for_each(|topic| {
                let mut found_dires = self
                    .directives
                    .iter()
                    .filter(|(_, d)| d.to_topic() == *topic)
                    .map(|(_, d)| d)
                    .collect::<Vec<_>>();
                dires.append(&mut found_dires);
            });
            dires
        } else {
            self.directives
                .iter()
                .map(|(_, d)| d)
                .collect::<Vec<Directive>>()
        };

        target_dires.into_iter().for_each(|d| {
            debug!(
                "republish directives({:?}) for subscriber: {}",
                d,
                chain_id.to_string()
            );
            let _ = self.pub_2_subscribers(Some(vec![chain_id.clone()]), d);
        });

        Ok(())
    }

    pub fn delete_directives(&mut self, chain_id: &ChainId, topics: &[Topic]) -> Result<(), Error> {
        debug!(
            "delete directives with topic ({:?}) for subscriber: {}",
            topics,
            chain_id.to_string()
        );
        for (seq_key, dir) in self
            .dire_queue
            .iter()
            .filter(|(seq_key, _)| seq_key.chain_id.eq(chain_id))
            .map(|(seq_key, dire)| (seq_key, dire))
            .collect::<Vec<_>>()
        {
            let topics = BTreeSet::from_iter(topics.iter());
            if topics.contains(&dir.to_topic()) {
                self.dire_queue.remove(&seq_key);
                record_event(&Event::DeletedDirective(seq_key.clone()));
            }
        }
        Ok(())
    }

    pub fn get_chains_by_counterparty(&self, counterparty: ChainId) -> Vec<ChainId> {
        self.chains
            .iter()
            .filter(|(_, c)| {
                c.counterparties.clone().map_or(false, |counterparties| {
                    counterparties.iter().any(|c| counterparty.eq(c))
                })
            })
            .map(|(chain_id, _)| chain_id)
            .collect()
    }

    pub fn get_chains_by_fee_token(&self, fee_token: TokenId) -> Vec<ChainId> {
        self.chains
            .iter()
            .filter(|(_, c)| c.fee_token.clone().map_or(false, |t| t == fee_token))
            .map(|(chain_id, _)| chain_id)
            .collect()
    }
}
