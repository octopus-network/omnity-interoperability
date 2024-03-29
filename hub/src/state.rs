use crate::memory::{self, Memory};
use crate::types::{Amount, ChainWithSeq, TokenKey, TokenMeta};


use ic_stable_structures::StableBTreeMap;
use log::info;
use omnity_types::{
    ChainId, ChainState, Directive, Error, Fee, Seq, SeqKey, Ticket, TicketId,
    ToggleAction, ToggleState, TokenId, Topic, TxAction,
};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::{HashMap};

use std::num::ParseIntError;

thread_local! {
    static STATE: RefCell<HubState> = RefCell::new(HubState::default());
}

#[derive(Deserialize, Serialize)]
pub struct HubState {
    #[serde(skip, default = "init_chain")]
    pub chains: StableBTreeMap<ChainId, ChainWithSeq, Memory>,
    #[serde(skip, default = "init_token")]
    pub tokens: StableBTreeMap<TokenId, TokenMeta, Memory>,
    #[serde(skip, default = "init_fee")]
    pub fees: StableBTreeMap<TokenKey, Fee, Memory>,

    #[serde(skip, default = "init_dire_queue")]
    pub dire_queue: StableBTreeMap<SeqKey, Directive, Memory>,
    #[serde(skip, default = "init_ticket_queue")]
    pub ticket_queue: StableBTreeMap<SeqKey, Ticket, Memory>,
    #[serde(skip, default = "init_token_position")]
    pub token_position: StableBTreeMap<TokenKey, Amount, Memory>,

    #[serde(skip, default = "init_ledger")]
    pub cross_ledger: StableBTreeMap<TicketId, Ticket, Memory>,
    pub owner: Option<String>,
    pub authorized_caller: HashMap<String, ChainId>,
}

impl Default for HubState {
    fn default() -> Self {
        Self {
            chains: StableBTreeMap::init(memory::get_chain_memory()),
            tokens: StableBTreeMap::init(memory::get_token_memory()),
            fees: StableBTreeMap::init(memory::get_fee_memory()),
            token_position: StableBTreeMap::init(memory::get_token_position_memory()),
            cross_ledger: StableBTreeMap::init(memory::get_ledger_memory()),
            dire_queue: StableBTreeMap::init(memory::get_dire_queue_memory()),
            ticket_queue: StableBTreeMap::init(memory::get_ticket_queue_memory()),
            owner: None,
            authorized_caller: HashMap::default(),
        }
    }
}
fn init_chain() -> StableBTreeMap<ChainId, ChainWithSeq, Memory> {
    StableBTreeMap::init(memory::get_chain_memory())
}
fn init_token() -> StableBTreeMap<TokenId, TokenMeta, Memory> {
    StableBTreeMap::init(memory::get_token_memory())
}
fn init_fee() -> StableBTreeMap<TokenKey, Fee, Memory> {
    StableBTreeMap::init(memory::get_fee_memory())
}
fn init_token_position() -> StableBTreeMap<TokenKey, Amount, Memory> {
    StableBTreeMap::init(memory::get_token_position_memory())
}
fn init_ledger() -> StableBTreeMap<TicketId, Ticket, Memory> {
    StableBTreeMap::init(memory::get_ledger_memory())
}
fn init_dire_queue() -> StableBTreeMap<SeqKey, Directive, Memory> {
    StableBTreeMap::init(memory::get_dire_queue_memory())
}
fn init_ticket_queue() -> StableBTreeMap<SeqKey, Ticket, Memory> {
    StableBTreeMap::init(memory::get_ticket_queue_memory())
}

/// A helper method to read the state.
///
/// Precondition: the state is already initialized.
pub fn with_state<R>(f: impl FnOnce(&HubState) -> R) -> R {
    STATE.with(|cell| f(&cell.borrow()))
}

/// A helper method to mutate the state.
///
/// Precondition: the state is already initialized.
pub fn with_state_mut<R>(f: impl FnOnce(&mut HubState) -> R) -> R {
    STATE.with(|cell| f(&mut cell.borrow_mut()))
}

// A helper method to set the state.
//
// Precondition: the state is _not_ initialized.
pub fn set_state(state: HubState) {
    STATE.with(|cell| *cell.borrow_mut() = state);
}

/// get settlement chain from token id
impl HubState {
    pub fn settlement_chain(&self, token_id: &TokenId) -> Result<ChainId, Error> {
        self.tokens
            .get(token_id)
            .map(|v| v.settlement_chain.to_string())
            .ok_or(Error::NotFoundToken(token_id.to_string()))
    }
    //Determine whether the token is from the issuing chain
    pub fn is_origin(&self, chain_id: &ChainId, token_id: &TokenId) -> Result<bool, Error> {
        self.tokens
            .get(token_id)
            .map(|v| v.settlement_chain.eq(chain_id))
            .ok_or(Error::NotFoundChainToken(
                token_id.to_string(),
                chain_id.to_string(),
            ))
    }
    pub fn save_chain(&mut self, chain: ChainWithSeq) -> Result<(), Error> {
        self.chains
            .insert(chain.chain_id.to_string(), chain.clone());
        // update auth
        self.authorized_caller
            .insert(chain.canister_id.to_string(), chain.chain_id.to_string());
        Ok(())
    }

    pub fn chain(&self, chain_id: &ChainId) -> Result<ChainWithSeq, Error> {
        self.chains
            .get(chain_id)
            .ok_or(Error::NotFoundChain(chain_id.to_string()))
    }

    //check the dst chain must exsiting and not deactive!
    pub fn available_chain(&self, chain_id: &ChainId) -> Result<ChainWithSeq, Error> {
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
                    self.chains.insert(toggle_state.chain_id.to_string(), chain);
                    Ok(())
                },
            )
    }

    pub fn save_token(&mut self, token_meata: TokenMeta) -> Result<(), Error> {
        self.tokens
            .insert(token_meata.token_id.to_string(), token_meata);
        Ok(())
    }

    pub fn token(&self, token_id: &TokenId) -> Result<TokenMeta, Error> {
        self.tokens
            .get(token_id)
            .ok_or(Error::NotFoundToken(token_id.to_string()))
    }

    pub fn update_fee(&mut self, fee: Fee) -> Result<(), Error> {
        self.chains
            .get(&fee.dst_chain_id)
            .ok_or(Error::NotFoundChain(fee.dst_chain_id.to_string()))
            .map_or_else(
                |e| Err(e),
                |chain| {
                    if matches!(chain.chain_state, ChainState::Deactive) {
                        Err(Error::DeactiveChain(fee.dst_chain_id.to_string()))
                    } else {
                        let token_key =
                            TokenKey::from(chain.chain_id.to_string(), fee.fee_token.to_string());
                        self.fees.insert(token_key, fee);

                        Ok(())
                    }
                },
            )
    }

    pub fn push_dire(&mut self, chain_id: &ChainId, dire: Directive) -> Result<(), Error> {
        self.chains
            .get(chain_id)
            .ok_or(Error::NotFoundChain(chain_id.to_string()))
            .map_or_else(
                |e| Err(e),
                |mut chain| {
                    if matches!(chain.chain_state, ChainState::Deactive) {
                        Err(Error::DeactiveChain(chain_id.to_string()))
                    } else {
                        
                        if self
                            .dire_queue
                            .iter()
                            .find(|(seq_key, _)| seq_key.chain_id.eq(chain_id))
                            .is_some()
                        {
                            //increase seq
                            chain.latest_ticket_seq += 1;
                            //update chain info
                            self.chains
                                .insert(chain.chain_id.to_string(), chain.clone());
                        }
                        self.dire_queue.insert(
                            SeqKey::from(chain.chain_id.to_string(), chain.latest_ticket_seq),
                            dire.clone(),
                        );

                        Ok(())
                    }
                },
            )
    }

    pub fn pull_dires(
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
                Topic::UpdateFee(token_id) => {
                    filter_dires(&self.dire_queue, &chain_id, offset, limit, |dire| {
                        if let Some(dst_token_id) = &token_id {
                            matches!(dire, Directive::UpdateFee(fee) if fee.fee_token.eq(dst_token_id))
                        } else {
                            matches!(dire, Directive::UpdateFee(_))
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
        if let Some(total_amount) = self.token_position.get(&position).as_mut() {
            *total_amount += amount;
            self.token_position.insert(position, *total_amount);
        } else {
            self.token_position.insert(position, amount);
        }

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
                    self.token_position.insert(position, total_amount);
                    Ok(())
                },
            )
        // Ok(())
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
        self.chains
            .get(&ticket.dst_chain)
            .as_mut()
            .ok_or(Error::NotFoundChain(ticket.dst_chain.to_string()))
            .map_or_else(
                |e| Err(e),
                |chain| {
                    if matches!(chain.chain_state, ChainState::Deactive) {
                        return Err(Error::DeactiveChain(chain.chain_id.to_string()));
                    }

                    if self
                        .ticket_queue
                        .iter()
                        .find(|(seq_key, ticket)| seq_key.chain_id.eq(&ticket.dst_chain))
                        .is_some()
                    {
                        //increase seq
                        chain.latest_ticket_seq += 1;
                        //update chain info
                        self.chains
                            .insert(ticket.dst_chain.to_string(), chain.clone());
                    }

                    // add new ticket
                    self.ticket_queue.insert(
                        SeqKey::from(ticket.dst_chain.to_string(), chain.latest_ticket_seq),
                        ticket.clone(),
                    );
                    //save ticket
                    self.cross_ledger
                        .insert(ticket.ticket_id.to_string(), ticket.clone());
                    Ok(())
                },
            )
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
