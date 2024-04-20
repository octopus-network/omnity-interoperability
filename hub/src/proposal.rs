use log::info;

use omnity_types::{ChainState, ChainType, Directive, Error, Factor};

use crate::{
    state::{with_state, with_state_mut},
    types::{ChainWithSeq, Proposal},
};

pub async fn validate_proposal(proposals: Vec<Proposal>) -> Result<Vec<String>, Error> {
    if proposals.len() == 0 {
        return Err(Error::ProposalError(
            "Proposal can not be empty".to_string(),
        ));
    }
    let mut proposal_msgs = Vec::new();
    for proposal in proposals.into_iter() {
        match proposal {
            Proposal::AddChain(chain_meta) => {
                if chain_meta.chain_id.is_empty() {
                    return Err(Error::ProposalError(
                        "Chain name can not be empty".to_string(),
                    ));
                }

                if matches!(chain_meta.chain_state, ChainState::Deactive) {
                    return Err(Error::ProposalError(
                        "The status of the new chain state must be active".to_string(),
                    ));
                }

                with_state(|hub_state| {
                    hub_state.chain(&chain_meta.chain_id).map_or(Ok(()), |_| {
                        Err(Error::ChainAlreadyExisting(chain_meta.chain_id.to_string()))
                    })
                })?;

                let result = format!("Tne AddChain proposal: {}", chain_meta);
                info!("validate_proposal result:{} ", result);
                proposal_msgs.push(result);
            }
            Proposal::AddToken(token_meta) => {
                if token_meta.token_id.is_empty()
                    || token_meta.symbol.is_empty()
                    || token_meta.issue_chain.is_empty()
                {
                    return Err(Error::ProposalError(
                        "Token id, token symbol or issue chain can not be empty".to_string(),
                    ));
                }
                with_state(|hub_state| {
                    // check token repetitive
                    hub_state.token(&token_meta.token_id).map_or(Ok(()), |_| {
                        Err(Error::TokenAlreadyExisting(token_meta.to_string()))
                    })?;

                    //ensure the dst chains must exsits!
                    if let Some(id) = token_meta
                        .dst_chains
                        .iter()
                        .find(|id| !hub_state.chains.contains_key(&**id))
                    {
                        return Err(Error::NotFoundChain(id.to_string()));
                    }

                    hub_state.available_chain(&token_meta.issue_chain)
                })?;
                let result = format!("The AddToken proposal: {}", token_meta);
                info!("validate_proposal result:{} ", result);
                proposal_msgs.push(result);
            }
            Proposal::ToggleChainState(toggle_state) => {
                if toggle_state.chain_id.is_empty() {
                    return Err(Error::ProposalError(
                        "Chain id can not be empty".to_string(),
                    ));
                }

                with_state(|hub_state| hub_state.available_state(&toggle_state))?;
                let result = format!("The ToggleChainStatus proposal: {}", toggle_state);
                info!("validate_proposal result:{} ", result);
                proposal_msgs.push(result);
            }
            Proposal::UpdateFee(factor) => {
                match factor {
                    Factor::UpdateTargetChainFactor(ref cf) => {
                        with_state(|hub_state| {
                            //check the issue chain must exsiting and not deactive!
                            hub_state.available_chain(&cf.target_chain_id)
                        })?;
                        let result = format!("The UpdateFee proposal: {}", factor);
                        info!("validate_proposal result:{} ", result);
                        proposal_msgs.push(result);
                    }
                    Factor::UpdateFeeTokenFactor(ref tf) => {
                        if tf.fee_token.is_empty() {
                            return Err(Error::ProposalError(
                                "The fee token can not be empty".to_string(),
                            ));
                        };

                        let result = format!("The UpdateFee proposal: {}", factor);
                        info!("validate_proposal result:{} ", result);
                        proposal_msgs.push(result);
                    }
                }
            }
        }
    }
    Ok(proposal_msgs)
}

pub async fn execute_proposal(proposals: Vec<Proposal>) -> Result<(), Error> {
    for proposal in proposals.into_iter() {
        match proposal {
            Proposal::AddChain(chain_meta) => {
                info!(
                    "build directive for `AddChain` proposal :{:?}",
                    chain_meta.to_string()
                );

                let new_chain = ChainWithSeq::from(chain_meta.clone());
                // save new chain
                with_state_mut(|hub_state| {
                    info!(" save new chain: {:?}", new_chain);
                    hub_state.add_chain(new_chain.clone())
                })?;
                // build directives
                match chain_meta.chain_type {
                    // nothing to do
                    ChainType::SettlementChain => {
                        info!("for settlement chain,  no need to build directive!");
                    }

                    ChainType::ExecutionChain => {
                        // publish directive for the new chain)
                        with_state_mut(|hub_state| {
                            //check and update counterparty of dst chain
                            hub_state.pub_directive(Directive::AddChain(new_chain.clone().into()))
                        })?;
                    }
                }
            }

            Proposal::AddToken(token_meata) => {
                info!("build directive for `AddToken` proposal :{:?}", token_meata);
                // save token info
                with_state_mut(|hub_state| hub_state.add_token(token_meata.clone()))?;
                // publish directive
                with_state_mut(|hub_state| {
                    hub_state.pub_directive(Directive::AddToken(token_meata.clone().into()))
                })?
            }

            Proposal::ToggleChainState(toggle_status) => {
                info!(
                    "build directive for `ToggleChainState` proposal :{:?}",
                    toggle_status
                );

                // publish directive
                with_state_mut(|hub_state| {
                    hub_state.pub_directive(Directive::ToggleChainState(toggle_status.clone()))
                })?;
                // update dst chain state
                with_state_mut(|hub_state| hub_state.update_chain_state(&toggle_status))?;
            }

            Proposal::UpdateFee(factor) => {
                info!("build directive for `UpdateFee` proposal :{:?}", factor);
                // save fee info
                with_state_mut(|hub_state| hub_state.update_fee(factor.clone()))?;

                with_state_mut(|hub_state| {
                    hub_state.pub_directive(Directive::UpdateFee(factor.clone()))
                })?;
            }
        }
    }

    Ok(())
}
