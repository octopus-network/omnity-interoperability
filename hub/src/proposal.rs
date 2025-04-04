use ic_canister_log::log;
use omnity_types::{ic_log::ERROR, ChainState, Directive, Error, Factor};
use omnity_types::hub_types::Proposal;
use omnity_types::ic_log::WARNING;

use crate::{
    state::{with_state, with_state_mut},
};

pub async fn validate_proposal(proposals: &Vec<Proposal>) -> Result<Vec<String>, Error> {
    if proposals.is_empty() {
        log!(ERROR, "Proposal can not be empty");
        return Err(Error::ProposalError(
            "Proposal can not be empty".to_string(),
        ));
    }
    let mut proposal_msgs = Vec::new();
    for proposal in proposals.iter() {
        match proposal {
            Proposal::AddChain(chain_meta) => {
                if chain_meta.chain_id.is_empty() {
                    log!(ERROR, "Proposal can not be empty");
                    return Err(Error::ProposalError(
                        "Chain name can not be empty".to_string(),
                    ));
                }

                if matches!(chain_meta.chain_state, ChainState::Deactive) {
                    log!(ERROR, "The status of the new chain state must be active");
                    return Err(Error::ProposalError(
                        "The status of the new chain state must be active".to_string(),
                    ));
                }

                with_state(|hub_state| {
                    hub_state.chain(&chain_meta.chain_id).map_or(Ok(()), |_| {
                        log!(
                            WARNING,
                            "The chain(`{}`) already exists",
                            chain_meta.chain_id.to_string()
                        );
                        Err(Error::ChainAlreadyExisting(chain_meta.chain_id.to_string()))
                    })
                })?;

                proposal_msgs.push(format!("Tne AddChain proposal: {}", chain_meta));
            }

            Proposal::UpdateChain(chain_meta) => {
                if chain_meta.chain_id.is_empty() {
                    return Err(Error::ProposalError(
                        "Chain name can not be empty".to_string(),
                    ));
                }
                // check chain available
                let ori_chain =
                    with_state(|hub_state| hub_state.available_chain(&chain_meta.chain_id))?;

                if ori_chain.eq(chain_meta) {
                    return Err(Error::ProposalError(
                        "The updated chain must be different with origial chain".to_string(),
                    ));
                }

                proposal_msgs.push(format!("Tne UpdateChain proposal: {}", chain_meta));
            }

            Proposal::AddToken(token_meta) => {
                if token_meta.token_id.is_empty()
                    || token_meta.symbol.is_empty()
                    || token_meta.issue_chain.is_empty()
                {
                    log!(
                        ERROR,
                        "Token id, token symbol or issue chain can not be empty"
                    );
                    return Err(Error::ProposalError(
                        "Token id, token symbol or issue chain can not be empty".to_string(),
                    ));
                }
                if token_meta.decimals > 18 {
                    log!(ERROR, "Token decimals can not be more than 18",);
                    return Err(Error::ProposalError(
                        "Token decimals can not be more than 18".to_string(),
                    ));
                }
                with_state(|hub_state| {
                    // check token repetitive
                    hub_state.token(&token_meta.token_id).map_or(Ok(()), |_| {
                        log!(
                            WARNING,
                            "The token(`{}`) already exists",
                            token_meta.to_string()
                        );
                        Err(Error::TokenAlreadyExisting(token_meta.to_string()))
                    })?;

                    //ensure the dst chains must exsits!
                    if let Some(id) = token_meta
                        .dst_chains
                        .iter()
                        .find(|id| !hub_state.chains.contains_key(*id))
                    {
                        log!(ERROR, "not found chain: (`{}`)", id.to_string());
                        return Err(Error::NotFoundChain(id.to_string()));
                    }

                    hub_state.available_chain(&token_meta.issue_chain)
                })?;

                proposal_msgs.push(format!("The AddToken proposal: {}", token_meta));
            }
            Proposal::UpdateToken(token_meta) => {
                if token_meta.token_id.is_empty()
                    || token_meta.symbol.is_empty()
                    || token_meta.issue_chain.is_empty()
                {
                    return Err(Error::ProposalError(
                        "Token id, token symbol or issue chain can not be empty".to_string(),
                    ));
                }
                if token_meta.decimals > 18 {
                    log!(ERROR, "Token decimals can not be more than 18",);
                    return Err(Error::ProposalError(
                        "Token decimals can not be more than 18".to_string(),
                    ));
                }

                let ori_token = with_state(|hub_state| hub_state.token(&token_meta.token_id))?;

                if ori_token.eq(token_meta) {
                    return Err(Error::ProposalError(
                        "The updated token must be different with origial token".to_string(),
                    ));
                }

                with_state(|hub_state| {
                    //ensure the dst chains must exsits!
                    if let Some(id) = token_meta
                        .dst_chains
                        .iter()
                        .find(|id| !hub_state.chains.contains_key(*id))
                    {
                        return Err(Error::NotFoundChain(id.to_string()));
                    }

                    hub_state.available_chain(&token_meta.issue_chain)
                })?;

                proposal_msgs.push(format!("The UpdateToken proposal: {}", token_meta));
            }

            Proposal::ToggleChainState(toggle_state) => {
                if toggle_state.chain_id.is_empty() {
                    log!(ERROR, "Chain id can not be empty");
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
                            log!(ERROR, "The fee token can not be empty");
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
                with_state_mut(|hub_state| hub_state.update_chain(chain_meta.clone()))?;
                // publish directive for the new chain)
                with_state_mut(|hub_state| {
                    let target_subs = chain_meta.counterparties.clone().unwrap_or_default();
                    hub_state
                        .pub_directive(Some(target_subs), &Directive::AddChain(chain_meta.into()))
                })?;
            }
            Proposal::UpdateChain(chain_meta) => {
                // update chain meta
                with_state_mut(|hub_state| hub_state.update_chain(chain_meta.clone()))?;
                // publish directive for the new chain)
                with_state_mut(|hub_state| {
                    let target_subs = chain_meta.counterparties.clone().unwrap_or_default();
                    hub_state.pub_directive(
                        Some(target_subs),
                        &Directive::UpdateChain(chain_meta.into()),
                    )
                })?;
            }
            Proposal::AddToken(token_meata) => {
                with_state_mut(|hub_state| {
                    // save token info
                    hub_state.update_token(token_meata.clone())?;
                    // publish directive
                    hub_state.pub_directive(
                        Some(token_meata.dst_chains.clone()),
                        &Directive::AddToken(token_meata.into()),
                    )
                })?
            }
            Proposal::UpdateToken(token_meata) => {
                with_state_mut(|hub_state| {
                    // update token info
                    hub_state.update_token(token_meata.clone())?;
                    // publish directive
                    hub_state.pub_directive(
                        Some(token_meata.dst_chains.clone()),
                        &Directive::UpdateToken(token_meata.into()),
                    )
                })?
            }

            Proposal::ToggleChainState(toggle_status) => {
                with_state_mut(|hub_state| {
                    // publish directive
                    hub_state
                        .pub_directive(None, &Directive::ToggleChainState(toggle_status.clone()))?;
                    // update dst chain state
                    hub_state.update_chain_state(&toggle_status)
                })?;
            }

            Proposal::UpdateFee(factor) => {
                with_state_mut(|hub_state| {
                    hub_state.update_fee(factor.clone())?;
                    let target_subs = match &factor {
                        Factor::UpdateTargetChainFactor(factor) => hub_state
                            .chains
                            .iter()
                            .filter_map(|s| match s.0 != factor.target_chain_id {
                                true => Some(s.0),
                                false => None,
                            })
                            .collect(),
                        Factor::UpdateFeeTokenFactor(factor) => {
                            hub_state.get_chains_by_fee_token(factor.fee_token.clone())
                        }
                    };
                    hub_state
                        .pub_directive(Some(target_subs), &Directive::UpdateFee(factor.clone()))
                })?;
            }
        }
    }
    Ok(())
}
