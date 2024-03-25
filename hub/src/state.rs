use candid::CandidType;
use log::info;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::num::ParseIntError;
use std::{cell::RefCell, collections::BTreeMap};

use crate::types::{Amount, ChainWithSeq, DireQueue, TicketQueue, TokenKey, TokenMeta};
use omnity_types::{
    ChainId, ChainState, Directive, Error, Fee, Ticket, TicketId, ToggleAction, ToggleState,
    TokenId, TxAction,
};

thread_local! {
    static STATE: RefCell<HubState> = RefCell::new(HubState::default());
}

#[derive(CandidType, Deserialize, Serialize, Default, Debug)]
pub struct HubState {
    pub chains: HashMap<ChainId, ChainWithSeq>,
    pub tokens: HashMap<TokenId, TokenMeta>,
    pub fees: HashMap<TokenKey, Fee>,
    pub cross_ledger: HashMap<TicketId, Ticket>,
    pub token_position: HashMap<TokenKey, Amount>,
    pub dire_queue: DireQueue,
    pub ticket_queue: TicketQueue,
    pub owner: Option<String>,
    pub authorized_caller: HashMap<String, ChainId>,
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

    pub fn chain(&self, chain_id: &ChainId) -> Result<&ChainWithSeq, Error> {
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
            .cloned()
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
    //check the dst chain must exsiting and not deactive,return a mut chain
    pub fn available_mut_chain(&mut self, chain_id: &ChainId) -> Result<&mut ChainWithSeq, Error> {
        self.chains
            .get_mut(chain_id)
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

    pub fn update_chain_state(&mut self, toggle_state: &ToggleState) -> Result<(), Error> {
        self.chains
            .get_mut(&toggle_state.chain_id)
            .map(|dst_chain| match toggle_state.action {
                ToggleAction::Activate => dst_chain.chain_state = ChainState::Active,
                ToggleAction::Deactivate => dst_chain.chain_state = ChainState::Deactive,
            });
        Ok(())
    }

    pub fn save_token(&mut self, token_meata: TokenMeta) -> Result<(), Error> {
        self.tokens
            .insert(token_meata.token_id.to_string(), token_meata);
        Ok(())
    }

    pub fn token(&self, token_id: &TokenId) -> Result<&TokenMeta, Error> {
        self.tokens
            .get(token_id)
            .ok_or(Error::NotFoundToken(token_id.to_string()))
    }

    pub fn gen_dire(&mut self, chain_id: &ChainId, dire: Directive) -> Result<(), Error> {
        self.chains
            .get_mut(chain_id)
            .ok_or(Error::NotFoundChain(chain_id.to_string()))
            .map_or_else(
                |e| Err(e),
                |chain| {
                    if matches!(chain.chain_state, ChainState::Deactive) {
                        Err(Error::DeactiveChain(chain_id.to_string()))
                    } else {
                        self.dire_queue
                            .entry(chain.chain_id.to_string())
                            .and_modify(|dire_map| {
                                chain.latest_dire_seq += 1;
                                dire_map.insert(chain.latest_dire_seq, dire.clone());
                            })
                            .or_insert_with(|| BTreeMap::from([(0, dire)]));

                        Ok(())
                    }
                },
            )
    }

    pub fn update_fee(&mut self, fee: Fee) -> Result<(), Error> {
        self.chains
            .get_mut(&fee.dst_chain_id)
            .ok_or(Error::NotFoundChain(fee.dst_chain_id.to_string()))
            .map_or_else(
                |e| Err(e),
                |chain| {
                    if matches!(chain.chain_state, ChainState::Deactive) {
                        Err(Error::DeactiveChain(fee.dst_chain_id.to_string()))
                    } else {
                        self.fees
                            .entry(TokenKey::from(
                                chain.chain_id.to_string(),
                                fee.fee_token.to_string(),
                            ))
                            .and_modify(|f| *f = fee.clone())
                            .or_insert(fee.clone());

                        Ok(())
                    }
                },
            )
    }
    pub fn add_token_position(&mut self, position: TokenKey, amount: u128) -> Result<(), Error> {
        self.token_position
            .entry(position)
            .and_modify(|total_amount| *total_amount += amount)
            .or_insert(amount);
        Ok(())
    }

    pub fn update_token_position(
        &mut self,
        position: TokenKey,
        f: impl FnOnce(&mut u128) -> Result<(), Error>,
    ) -> Result<(), Error> {
        self.token_position
            .get_mut(&position)
            .ok_or(Error::NotFoundChainToken(
                position.token_id.to_string(),
                position.chain_id.to_string(),
            ))
            .map_or_else(|e| Err(e), |total_amount| f(total_amount))
        // Ok(())
    }

    // check the ticket availability
    pub fn check_and_count(&mut self, ticket: &Ticket) -> Result<(), Error> {
        // check ticket id repetitive
        if self.ticket_queue.contains_key(&ticket.ticket_id) {
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
                                return Err::<(), omnity_types::Error>(Error::NotSufficientTokens(
                                    ticket.token.to_string(),
                                    ticket.src_chain.to_string(),
                                ));
                            }
                            *total_amount -= ticket_amount;
                            Ok(())
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
                            return Err::<(), omnity_types::Error>(Error::NotSufficientTokens(
                                ticket.token.to_string(),
                                ticket.src_chain.to_string(),
                            ));
                        }
                        *total_amount -= ticket_amount;
                        Ok(())
                    },
                )?;

                //  if the dst chain is not issue chain,then update token amount on dst chain
                if !self.is_origin(&ticket.dst_chain, &ticket.token)? {
                    self.update_token_position(
                        TokenKey::from(ticket.dst_chain.to_string(), ticket.token.to_string()),
                        |total_amount| {
                            *total_amount += ticket_amount;
                            Ok(())
                        },
                    )?;
                }
            }
        }

        Ok(())
    }

    pub fn push_ticket(&mut self, ticket: Ticket) -> Result<(), Error> {
        self.chains
            .get_mut(&ticket.dst_chain)
            .ok_or(Error::NotFoundChain(ticket.dst_chain.to_string()))
            .map_or_else(
                |e| Err(e),
                |chain| {
                    self.ticket_queue
                        .entry(ticket.dst_chain.to_string())
                        .and_modify(|tickets| {
                            //increase seq
                            chain.latest_ticket_seq += 1;
                            tickets.insert(chain.latest_ticket_seq, ticket.clone());
                        })
                        .or_insert_with(|| {
                            BTreeMap::from([(chain.latest_ticket_seq, ticket.clone())])
                        });

                    //save ticket
                    self.cross_ledger
                        .insert(ticket.ticket_id.to_string(), ticket.clone());
                    Ok(())
                },
            )
    }
}
