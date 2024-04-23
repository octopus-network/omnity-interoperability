use log::info;

use omnity_types::{ChainState, ChainType, Directive, Error, Factor};

use crate::{
    state::{with_state, with_state_mut},
    types::Proposal,
};

pub async fn validate_proposal(proposals: &Vec<Proposal>) -> Result<Vec<String>, Error> {
    if proposals.is_empty() {
        return Err(Error::ProposalError(
            "Proposal can not be empty".to_string(),
        ));
    }
    let mut proposal_msgs = Vec::new();
    for proposal in proposals.iter() {
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

                proposal_msgs.push(format!("Tne AddChain proposal: {}", chain_meta));
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

                proposal_msgs.push(format!("The AddToken proposal: {}", token_meta));
            }
            Proposal::ToggleChainState(toggle_state) => {
                if toggle_state.chain_id.is_empty() {
                    return Err(Error::ProposalError(
                        "Chain id can not be empty".to_string(),
                    ));
                }

                with_state(|hub_state| hub_state.available_state(toggle_state))?;

                proposal_msgs.push(format!("The ToggleChainStatus proposal: {}", toggle_state));
            }
            Proposal::UpdateFee(factor) => {
                match factor {
                    Factor::UpdateTargetChainFactor(ref cf) => {
                        with_state(|hub_state| {
                            //check the issue chain must exsiting and not deactive!
                            hub_state.available_chain(&cf.target_chain_id)
                        })?;

                        proposal_msgs.push(format!("The UpdateFee proposal: {}", factor));
                    }
                    Factor::UpdateFeeTokenFactor(ref tf) => {
                        if tf.fee_token.is_empty() {
                            return Err(Error::ProposalError(
                                "The fee token can not be empty".to_string(),
                            ));
                        };
                        proposal_msgs.push(format!("The UpdateFee proposal: {}", factor));
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
                // save new chain
                with_state_mut(|hub_state| {
                    info!(" save new chain: {:?}", chain_meta);
                    hub_state.add_chain(chain_meta.clone())
                })?;
                // publish directive for the new chain)
                info!(
                    "publish directive for `AddChain` proposal :{:?}",
                    chain_meta.to_string()
                );
                with_state_mut(|hub_state| {
                    hub_state.pub_directive(&Directive::AddChain(chain_meta.into()))
                })?;
            }

            Proposal::AddToken(token_meata) => {
                info!(
                    "publish directive for `AddToken` proposal :{:?}",
                    token_meata
                );

                with_state_mut(|hub_state| {
                    // save token info
                    hub_state.add_token(token_meata.clone())?;
                    // publish directive
                    hub_state.pub_directive(&Directive::AddToken(token_meata.into()))
                })?
            }

            Proposal::ToggleChainState(toggle_status) => {
                info!(
                    "publish directive for `ToggleChainState` proposal :{:?}",
                    toggle_status
                );

                with_state_mut(|hub_state| {
                    // publish directive
                    hub_state.pub_directive(&Directive::ToggleChainState(toggle_status.clone()))?;
                    // update dst chain state
                    hub_state.update_chain_state(&toggle_status)
                })?;
            }

            Proposal::UpdateFee(factor) => {
                info!("publish directive for `UpdateFee` proposal :{:?}", factor);
                with_state_mut(|hub_state| {
                    hub_state.update_fee(factor.clone())?;
                    hub_state.pub_directive(&Directive::UpdateFee(factor.clone()))
                })?;
            }
        }
    }

    Ok(())
}
